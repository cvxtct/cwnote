mod annotate;
mod aws_client;
mod cli;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let client = aws_client::make_client(args.region.as_deref()).await?;

    run_with_client(&client, args).await
}

// Extracted so we can unit test decision logic without going through Clap/#[tokio::main].
async fn run_with_client(
    client: &aws_sdk_cloudwatch::Client,
    args: Cli,
) -> Result<()> {
    match args.command {
        Commands::Annotate(opts) => {
            let time_override = opts.time.as_deref();

            // Build widget selector from CLI flags.
            let selector = annotate::WidgetSelector {
                title_contains: opts.widget_title_contains.clone(),
            };

            match (opts.dashboard.as_deref(), opts.dashboard_prefix.as_deref()) {
                (Some(dashboard), None) => {
                    // Single dashboard.
                    annotate::annotate_single_dashboard(
                        client,
                        dashboard,
                        &opts.label,
                        &opts.value,
                        time_override,
                        opts.dry_run,
                        &selector,
                    )
                    .await?;
                }
                (None, Some(prefix)) => {
                    // All dashboards matching prefix.
                    annotate::annotate_dashboards_by_prefix(
                        client,
                        prefix,
                        &opts.label,
                        &opts.value,
                        time_override,
                        opts.dry_run,
                        &selector,
                    )
                    .await?;
                }
                (Some(_), Some(_)) => {
                    return Err(anyhow!(
                        "Please specify either --dashboard OR --dashboard-prefix, not both"
                    ));
                }
                (None, None) => {
                    return Err(anyhow!(
                        "Either --dashboard or --dashboard-prefix is required"
                    ));
                }
            }
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::aws_client;
    use crate::cli::{AnnotateOpts, Commands, Cli};

    // Helper: build a dummy client once for these tests.
    // It won't actually talk to AWS as long as we only hit the error paths
    // (we return before calling annotate::*).
    async fn make_dummy_client() -> aws_sdk_cloudwatch::Client {
        aws_client::make_client(Some("eu-central-1"))
            .await
            .expect("failed to create dummy client")
    }

    #[tokio::test]
    async fn run_with_client_errors_when_both_dashboard_and_prefix_are_set() {
        let client = make_dummy_client().await;

        let opts = AnnotateOpts {
            dashboard: Some("DashA".to_string()),
            dashboard_prefix: Some("PrefixB".to_string()),
            label: "version".to_string(),
            value: "1.2.3".to_string(),
            time: None,
            dry_run: false,
            widget_title_contains: None,
        };

        let args = Cli {
            region: None,
            command: Commands::Annotate(opts),
        };

        let result = run_with_client(&client, args).await;

        assert!(
            result.is_err(),
            "expected error when both dashboard and dashboard_prefix are set"
        );

        let msg = format!("{result:?}");
        assert!(
            msg.contains("Please specify either --dashboard OR --dashboard-prefix"),
            "unexpected error message: {msg}"
        );
    }

    #[tokio::test]
    async fn run_with_client_errors_when_neither_dashboard_nor_prefix_is_set() {
        let client = make_dummy_client().await;

        let opts = AnnotateOpts {
            dashboard: None,
            dashboard_prefix: None,
            label: "version".to_string(),
            value: "1.2.3".to_string(),
            time: None,
            dry_run: false,
            widget_title_contains: None,
        };

        let args = Cli {
            region: None,
            command: Commands::Annotate(opts),
        };

        let result = run_with_client(&client, args).await;

        assert!(
            result.is_err(),
            "expected error when neither dashboard nor dashboard_prefix is set"
        );

        let msg = format!("{result:?}");
        assert!(
            msg.contains("Either --dashboard or --dashboard-prefix is required"),
            "unexpected error message: {msg}"
        );
    }
}