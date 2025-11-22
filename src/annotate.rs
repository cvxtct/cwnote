use anyhow::{Context, Result};
use aws_sdk_cloudwatch::types::DashboardEntry;
use aws_sdk_cloudwatch::Client;
use chrono::Utc;
use serde_json::{Map, Value};

/// Controlls which widget we annotate.
#[derive(Debug, Clone)]
pub struct WidgetSelector {
    pub title_contains: Option<String>,
}

impl WidgetSelector {
    pub fn matches(&self, widget_obj: &Map<String, Value>) -> bool {
        // If we have a title filter, go check it.
        if let Some(ref title_filter) = self.title_contains {
            let title = widget_obj
                .get("properties")
                .and_then(|p| p.get("title"))
                .and_then(|t| t.as_str())
                .unwrap_or("");
            if !title.contains(title_filter) {
                return false;
            }
        }
        true
    }
}

/// Internal helper: apply a single annotation object to all matching widgets.
/// Returns the number of widgets annotated.
fn apply_annotation_to_body(
    body: &mut Value,
    ann_obj: &Map<String, Value>,
    selector: &WidgetSelector,
) -> usize {
    let mut widgets_annotated = 0usize;

    if let Some(widgets) = body.get_mut("widgets").and_then(|w| w.as_array_mut()) {
        for widget in widgets.iter_mut() {
            if let Some(widget_obj) = widget.as_object_mut() {
                // Only metric widgets.
                let is_metric = widget_obj.get("type").and_then(|t| t.as_str()) == Some("metric");
                if !is_metric {
                    continue;
                }

                // Apply selector (e.g. title contains substring).
                if !selector.matches(widget_obj) {
                    continue;
                }

                let props_val = widget_obj
                    .entry("properties")
                    .or_insert_with(|| Value::Object(Map::new()));
                let props_obj = props_val
                    .as_object_mut()
                    .expect("properties should be object");

                let anns_val = props_obj
                    .entry("annotations")
                    .or_insert_with(|| Value::Object(Map::new()));
                let anns_obj = anns_val
                    .as_object_mut()
                    .expect("annotations should be object");

                let vertical_val = anns_obj
                    .entry("vertical")
                    .or_insert_with(|| Value::Array(Vec::new()));
                let vertical_arr = vertical_val
                    .as_array_mut()
                    .expect("vertical should be array");

                vertical_arr.push(Value::Object(ann_obj.clone()));
                widgets_annotated += 1;
            }
        }
    }

    widgets_annotated
}

/// Annotate a single dashboard by name.
pub async fn annotate_single_dashboard(
    client: &Client,
    dashboard_name: &str,
    label: &str,
    value: &str,
    time_override: Option<&str>,
    dry_run: bool,
    selector: &WidgetSelector,
) -> Result<()> {
    // 1) Get current dashboard.
    let resp = client
        .get_dashboard()
        .dashboard_name(dashboard_name)
        .send()
        .await
        .with_context(|| format!("failed to get dashboard {dashboard_name}"))?;

    let body_str = resp
        .dashboard_body()
        .with_context(|| format!("dashboard {dashboard_name} has no body"))?;

    let mut body: Value =
        serde_json::from_str(body_str).context("failed to parse dashboard body JSON")?;

    // 2) Determine annotation timestamp.
    let ts = match time_override {
        Some(s) => s.to_string(),
        None => Utc::now().to_rfc3339(),
    };

    // 3) Build annotation object
    let mut ann_obj = Map::new();
    ann_obj.insert(
        "label".to_string(),
        Value::String(format!("{label}: {value}")),
    );
    ann_obj.insert("value".to_string(), Value::String(ts));

    // Optional: color, visible, etc.
    // ann_obj.insert("color".into(), Value::String("#ff9900".into()));

    // 4) Insert annotation into selected metric widgets.
    let widgets_annotated = apply_annotation_to_body(&mut body, &ann_obj, selector);

    if widgets_annotated == 0 {
        println!(
            "{}: no matching metric widgets found (nothing to annotate)",
            dashboard_name
        );
        return Ok(());
    }

    if dry_run {
        println!(
            "[dry-run] {}: would annotate {} metric widget(s) with version '{}'",
            dashboard_name, widgets_annotated, value
        );
        return Ok(());
    }

    // 5) Serialize back and put dashboard.
    let updated_body =
        serde_json::to_string(&body).context("failed to serialize updated dashboard body")?;

    client
        .put_dashboard()
        .dashboard_name(dashboard_name)
        .dashboard_body(updated_body)
        .send()
        .await
        .with_context(|| format!("failed to put updated dashboard {dashboard_name}"))?;

    println!(
        "Annotated {} metric widget(s) on dashboard '{}' with value '{}'",
        widgets_annotated, dashboard_name, value
    );

    Ok(())
}

/// Annotate all dashboards whose name starts with the given prefix.
pub async fn annotate_dashboards_by_prefix(
    client: &Client,
    prefix: &str,
    label: &str,
    value: &str,
    time_override: Option<&str>,
    dry_run: bool,
    selector: &WidgetSelector,
) -> Result<()> {
    let dashboards = list_dashboards_with_prefix(client, prefix).await?;

    if dashboards.is_empty() {
        println!("No dashboards found with prefix '{}'", prefix);
        return Ok(());
    }

    println!(
        "{} dashboard(s) match prefix '{}':",
        dashboards.len(),
        prefix
    );
    for d in &dashboards {
        println!("  - {}", d);
    }

    for name in dashboards {
        annotate_single_dashboard(
            client,
            &name,
            label,
            value,
            time_override,
            dry_run,
            selector,
        )
        .await?;
    }

    Ok(())
}

/// List dashboards whose names start with the given prefix.
async fn list_dashboards_with_prefix(client: &Client, prefix: &str) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut next_token: Option<String> = None;

    loop {
        let mut req = client.list_dashboards();
        if let Some(ref token) = next_token {
            req = req.next_token(token);
        }

        let resp = req.send().await.context("failed to list dashboards")?;

        let entries: &[DashboardEntry] = resp.dashboard_entries();

        for entry in entries {
            if let Some(name) = entry.dashboard_name() {
                if name.starts_with(prefix) {
                    result.push(name.to_string());
                }
            }
        }

        match resp.next_token() {
            Some(t) if !t.is_empty() => {
                next_token = Some(t.to_string());
            }
            _ => break,
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn widget_selector_matches_without_filter() {
        let selector = WidgetSelector {
            title_contains: None,
        };

        // Widget without title, but since no filter, it should match.
        let widget = json!({
            "type": "metric",
            "properties": {
                "metrics": []
            }
        });

        let widget_obj = widget.as_object().unwrap();
        assert!(selector.matches(widget_obj));
    }

    #[test]
    fn widget_selector_matches_when_title_contains_substring() {
        let selector = WidgetSelector {
            title_contains: Some("Latency".to_string()),
        };

        let widget = json!({
            "type": "metric",
            "properties": {
                "title": "Overall Latency P95",
                "metrics": []
            }
        });

        let widget_obj = widget.as_object().unwrap();
        assert!(selector.matches(widget_obj));
    }

    #[test]
    fn widget_selector_does_not_match_when_title_does_not_contain_substring() {
        let selector = WidgetSelector {
            title_contains: Some("Latency".to_string()),
        };

        let widget = json!({
            "type": "metric",
            "properties": {
                "title": "Error Rate",
                "metrics": []
            }
        });

        let widget_obj = widget.as_object().unwrap();
        assert!(!selector.matches(widget_obj));
    }

    #[test]
    fn apply_annotation_only_hits_metric_widgets_that_match_selector() {
        // Dashboard body with:
        // - metric widget "Overall Latency"
        // - metric widget "Error Rate"
        // - text widget
        let mut body = json!({
            "widgets": [
                {
                    "type": "metric",
                    "properties": {
                        "title": "Overall Latency",
                        "metrics": []
                    }
                },
                {
                    "type": "metric",
                    "properties": {
                        "title": "Error Rate",
                        "metrics": []
                    }
                },
                {
                    "type": "text",
                    "properties": {
                        "markdown": "# Hello"
                    }
                }
            ]
        });

        // Only annotate widgets whose title contains "Latency"
        let selector = WidgetSelector {
            title_contains: Some("Latency".to_string()),
        };

        // Build a fake annotation object.
        let mut ann_obj = Map::new();
        ann_obj.insert(
            "label".to_string(),
            Value::String("version: 1.2.3".to_string()),
        );
        ann_obj.insert(
            "value".to_string(),
            Value::String("2025-01-20T12:00:00Z".to_string()),
        );

        let count = apply_annotation_to_body(&mut body, &ann_obj, &selector);
        assert_eq!(
            count, 1,
            "only one matching metric widget should be annotated"
        );

        // Check that only the "Overall Latency" widget has vertical annotations.
        let widgets = body.get("widgets").unwrap().as_array().unwrap();

        // First widget: "Overall Latency" → should have the annotation
        let w0 = widgets[0].as_object().unwrap();
        let props0 = w0.get("properties").unwrap().as_object().unwrap();
        let anns0 = props0.get("annotations").unwrap().as_object().unwrap();
        let vertical0 = anns0.get("vertical").unwrap().as_array().unwrap();
        assert_eq!(vertical0.len(), 1);
        let ann0 = vertical0[0].as_object().unwrap();
        assert_eq!(
            ann0.get("label").unwrap(),
            &Value::String("version: 1.2.3".to_string())
        );
        assert_eq!(
            ann0.get("value").unwrap(),
            &Value::String("2025-01-20T12:00:00Z".to_string())
        );

        // Second widget: "Error Rate" → should NOT have annotations
        let w1 = widgets[1].as_object().unwrap();
        let props1 = w1.get("properties").unwrap().as_object().unwrap();
        assert!(
            !props1.contains_key("annotations"),
            "non-matching metric widget should not have annotations"
        );

        // Third widget: type "text" → should not be touched
        let w2 = widgets[2].as_object().unwrap();
        assert_eq!(w2.get("type").unwrap().as_str(), Some("text"));
        let props2 = w2.get("properties").unwrap().as_object().unwrap();
        assert!(
            !props2.contains_key("annotations"),
            "non-metric widget should not have annotations"
        );
    }

    #[test]
    fn apply_annotation_with_no_matching_widgets_returns_zero() {
        let mut body = json!({
            "widgets": [
                {
                    "type": "metric",
                    "properties": {
                        "title": "Error Rate",
                        "metrics": []
                    }
                }
            ]
        });

        let selector = WidgetSelector {
            title_contains: Some("Latency".to_string()),
        };

        let mut ann_obj = Map::new();
        ann_obj.insert(
            "label".to_string(),
            Value::String("version: 1.2.3".to_string()),
        );
        ann_obj.insert(
            "value".to_string(),
            Value::String("2025-01-20T12:00:00Z".to_string()),
        );

        let count = apply_annotation_to_body(&mut body, &ann_obj, &selector);
        assert_eq!(count, 0);

        let widgets = body.get("widgets").unwrap().as_array().unwrap();
        let w0 = widgets[0].as_object().unwrap();
        let props0 = w0.get("properties").unwrap().as_object().unwrap();
        assert!(
            !props0.contains_key("annotations"),
            "widget should remain unannotated when selector doesn't match"
        );
    }
}
