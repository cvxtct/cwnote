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
            if !title_contains(title_filter) {
                return false;
            }
        }
        true
    }
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

    let mut ann_obj = Map::new();
    ann_obj.insert("label".to_string(), Value::String(format!("{label}: {value}")));
    ann_obj.insert("value".to_string(), Value::String(ts));

    // Optional: color, visible, etc.
    // ann_obj.insert("color".into(), Value::String("#ff9900".into()));



}



 // let dashboards = list_dashboards_with_prefix(client, prefix).await?;

    // if dashboards.is_empty() {
    //     println!("No dashboards found with prefix '{}'", prefix);
    //     return Ok(());
    // }

    // println!(
    //     "{} dashboard(s) match prefix '{}':",
    //     dashboards.len(),
    //     prefix
    // );

    // for d in &dashboards {
    //     println!("  - {}", d);
    // }