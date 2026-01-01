use clap::{ArgGroup, Parser};

const APP_NAME: &str = "cwnote";
const ABOUT_TEXT: &str = "Add annotation to CloudWatch dashboards.";
const DEFAULT_LABEL: &str = "version";
const ARG_GROUP_TARGET: &str = "target";
const ARG_DASHBOARD: &str = "dashboard";
const ARG_DASHBOARD_SUFFIX: &str = "dashboard_suffix";

/**
CloudWatch dashoard vertical annotator.
*/
#[derive(Debug, Parser)]
#[command(name = APP_NAME)]
#[command(version, about = ABOUT_TEXT, long_about = None)]
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
#[command(
    group(
        ArgGroup::new(ARG_GROUP_TARGET)
            .required(true)
            .args(&[ARG_DASHBOARD, ARG_DASHBOARD_SUFFIX]),
    )
)]
pub struct AnnotateOpts {
    /// Single dashboard name to update.
    #[arg(long)]
    pub dashboard: Option<String>,

    /// Prefx of dashboard names to update.
    #[arg(long)]
    pub dashboard_suffix: Option<String>,

    /// Annotation label, e.g.: "version", "incident", "deploy", "alarm".
    #[arg(long, default_value = DEFAULT_LABEL)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    const CMD_ANNOTATE: &str = "annotate";

    #[test]
    fn parse_minimal_annotate_with_dashboard() {
        // cwnote annotate --dashboard TestDash --value 1.2.3
        let cli = Cli::try_parse_from([
            APP_NAME,
            CMD_ANNOTATE,
            "--dashboard",
            "TestDash",
            "--value",
            "1.2.3",
        ])
        .expect("failed to parse args");

        assert!(cli.region.is_none());

        match cli.command {
            Commands::Annotate(opts) => {
                assert_eq!(opts.dashboard.as_deref(), Some("TestDash"));
                assert!(opts.dashboard_suffix.is_none());
                assert_eq!(opts.label, DEFAULT_LABEL); // default
                assert_eq!(opts.value, "1.2.3");
                assert!(opts.time.is_none());
                assert!(!opts.dry_run);
                assert!(opts.widget_title_contains.is_none());
            }
        }
    }

    #[test]
    fn parse_with_dashboard_suffix() {
        // cwnote annotate --dashboard-suffix TestService- --value foo
        let cli = Cli::try_parse_from([
            APP_NAME,
            CMD_ANNOTATE,
            "--dashboard-suffix",
            "TestService-",
            "--value",
            "foo",
        ])
        .expect("failed to parse args");

        match cli.command {
            Commands::Annotate(opts) => {
                assert!(opts.dashboard.is_none());
                assert_eq!(opts.dashboard_suffix.as_deref(), Some("TestService-"));
                assert_eq!(opts.label, DEFAULT_LABEL);
                assert_eq!(opts.value, "foo");
            }
        }
    }

    #[test]
    fn parse_with_all_optional_extras() {
        // cwnote annotate --dashboard TestDash --value v \
        //   --time 2025-01-01T00:00:00Z --dry-run --widget-title-contains Latency
        let cli = Cli::try_parse_from([
            APP_NAME,
            CMD_ANNOTATE,
            "--dashboard",
            "TestDash",
            "--value",
            "v",
            "--time",
            "2025-01-01T00:00:00Z",
            "--dry-run",
            "--widget-title-contains",
            "Latency",
        ])
        .expect("failed to parse args");

        match cli.command {
            Commands::Annotate(opts) => {
                assert_eq!(opts.dashboard.as_deref(), Some("TestDash"));
                assert_eq!(opts.value, "v");
                assert_eq!(opts.time.as_deref(), Some("2025-01-01T00:00:00Z"));
                assert!(opts.dry_run);
                assert_eq!(opts.widget_title_contains.as_deref(), Some("Latency"));
            }
        }
    }

    #[test]
    fn error_when_neither_dashboard_nor_suffix_is_provided() {
        // cwnote annotate --value v
        let res = Cli::try_parse_from([APP_NAME, CMD_ANNOTATE, "--value", "v"]);
        assert!(
            res.is_err(),
            "expected clap error when missing dashboard and suffix"
        );
    }

    #[test]
    fn error_when_both_dashboard_and_suffix_are_provided() {
        // cwnote annotate --dashboard A --dashboard-suffix B --value v
        let res = Cli::try_parse_from([
            APP_NAME,
            CMD_ANNOTATE,
            "--dashboard",
            "A",
            "--dashboard-suffix",
            "B",
            "--value",
            "v",
        ]);
        assert!(
            res.is_err(),
            "expected clap error when both dashboard and suffix are set"
        );
    }
}
