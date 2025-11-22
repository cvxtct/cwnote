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
                        &client,
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
                        &client,
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
