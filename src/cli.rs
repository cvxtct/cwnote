use clap::{Parser, Subcommand};

/// CloudWatch dashoard vertical annotator.
#[derive(Debug, Parser)]
#[command(name = "cwnote")]
#[command(version, about = "Add annotation to CloudWatch dashboards.", long_about = None)]
pub struct Cli {
    /// AWS region (fails back to AWS_REGION / profile if omitted).
    #[arg(long)]
    pub region: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Parser)]
pub enum Commands {
    /// Add vertical annotation to dasboard(s) / widget(s).
    Annotate(AnnotateOpts),
}

#[derive(Debug, Parser)]
pub struct AnnotateOpts {
    /// Single dashboard name to update.
    #[arg(long, required_unless_present = "dashboard_prefix")]
    pub dashboard: Option<String>,

    /// Prefx of dashboard names to update.
    #[arg(long, required_unless_present = "dashboard")]
    pub dashboard_prefix: Option<String>,

    /// Annotation label, e.g.: "version", "incident", "deploy", "alarm".
    #[arg(long, default_value = "version")]
    pub label: String,

    /// Annotation value e.g.: "0.0.0-49u4ref" or "INC-1234", or "SOME-EVENT".
    #[arg(long)]
    pub value: String,

    /// Annotation time (ISO8601 / RFC3339). If omitted, uses current UTC time.
    #[arg(long)]
    pub time: Option<String>,

    /// Dry run: don’t actually update dashboards, just show what would change.
    #[arg(long)]
    pub dry_run: bool,

    /// Only annotate widgets whose title contains this substring.
    #[arg(long)]
    pub widget_title_contains: Option<String>,
}
