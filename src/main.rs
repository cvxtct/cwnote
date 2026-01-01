mod annotate;
mod aws_client;
mod cli;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli::{Cli, Commands};


#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let args = Cli::parse();

    let client = aws_client::make_client(args.region.as_deref()).await?;

    run_with_client(&client, args).await
}

// Extracted so we can unit test decision logic without going through Clap/#[tokio::main].
async fn run_with_client(client: &aws_sdk_cloudwatch::Client, args: Cli) -> Result<()> {
    match args.command {
        Commands::Annotate(opts) => {
            let time_override = opts.time.as_deref();

            // Build widget selector from CLI flags.
            let selector = annotate::WidgetSelector {
                title_contains: opts.widget_title_contains.clone(),
            };

            match (opts.dashboard.as_deref(), opts.dashboard_suffix.as_deref()) {
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
                (None, Some(suffix)) => {
                    // All dashboards matching suffix.
                    annotate::annotate_dashboards_by_suffix(
                        client,
                        suffix,
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
                        "Please specify either --dashboard OR --dashboard-suffix, not both"
                    ));
                }
                (None, None) => {
                    return Err(anyhow!(
                        "Either --dashboard or --dashboard-suffix is required"
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
    use crate::cli::{AnnotateOpts, Cli, Commands};

    // Helper: build a dummy client once for these tests.
    // It won't actually talk to AWS as long as we only hit the error paths
    // (we return before calling annotate::*).
    async fn make_dummy_client() -> aws_sdk_cloudwatch::Client {
        aws_client::make_client(Some("eu-central-1"))
            .await
            .expect("failed to create dummy client")
    }

    #[tokio::test]
    async fn run_with_client_errors_when_both_dashboard_and_suffix_are_set() {
        let client = make_dummy_client().await;

        let opts = AnnotateOpts {
            dashboard: Some("DashA".to_string()),
            dashboard_suffix: Some("suffixB".to_string()),
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
            "expected error when both dashboard and dashboard_suffix are set"
        );

        let msg = format!("{result:?}");
        assert!(
            msg.contains("Please specify either --dashboard OR --dashboard-suffix"),
            "unexpected error message: {msg}"
        );
    }

    #[tokio::test]
    async fn run_with_client_errors_when_neither_dashboard_nor_suffix_is_set() {
        let client = make_dummy_client().await;

        let opts = AnnotateOpts {
            dashboard: None,
            dashboard_suffix: None,
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
            "expected error when neither dashboard nor dashboard_suffix is set"
        );

        let msg = format!("{result:?}");
        assert!(
            msg.contains("Either --dashboard or --dashboard-suffix is required"),
            "unexpected error message: {msg}"
        );
    }
}
