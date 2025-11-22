use clap::{ArgGroup, Parser};

/**
CloudWatch dashoard vertical annotator.
*/
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
#[command(
    group(
        ArgGroup::new("target")
            .required(true)
            .args(&["dashboard", "dashboard_prefix"]),
    )
)]
pub struct AnnotateOpts {
    /// Single dashboard name to update.
    #[arg(long)]
    pub dashboard: Option<String>,

    /// Prefx of dashboard names to update.
    #[arg(long)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_minimal_annotate_with_dashboard() {
        // cwnote annotate --dashboard TestDash --value 1.2.3
        let cli = Cli::try_parse_from([
            "cwnote",
            "annotate",
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
                assert!(opts.dashboard_prefix.is_none());
                assert_eq!(opts.label, "version"); // default
                assert_eq!(opts.value, "1.2.3");
                assert!(opts.time.is_none());
                assert!(!opts.dry_run);
                assert!(opts.widget_title_contains.is_none());
            }
        }
    }

    #[test]
    fn parse_with_dashboard_prefix() {
        // cwnote annotate --dashboard-prefix TestService- --value foo
        let cli = Cli::try_parse_from([
            "cwnote",
            "annotate",
            "--dashboard-prefix",
            "TestService-",
            "--value",
            "foo",
        ])
        .expect("failed to parse args");

        match cli.command {
            Commands::Annotate(opts) => {
                assert!(opts.dashboard.is_none());
                assert_eq!(opts.dashboard_prefix.as_deref(), Some("TestService-"));
                assert_eq!(opts.label, "version");
                assert_eq!(opts.value, "foo");
            }
        }
    }

    #[test]
    fn parse_with_all_optional_extras() {
        // cwnote annotate --dashboard TestDash --value v \
        //   --time 2025-01-01T00:00:00Z --dry-run --widget-title-contains Latency
        let cli = Cli::try_parse_from([
            "cwnote",
            "annotate",
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
    fn error_when_neither_dashboard_nor_prefix_is_provided() {
        // cwnote annotate --value v
        let res = Cli::try_parse_from(["cwnote", "annotate", "--value", "v"]);
        assert!(
            res.is_err(),
            "expected clap error when missing dashboard and prefix"
        );
    }

    #[test]
    fn error_when_both_dashboard_and_prefix_are_provided() {
        // cwnote annotate --dashboard A --dashboard-prefix B --value v
        let res = Cli::try_parse_from([
            "cwnote",
            "annotate",
            "--dashboard",
            "A",
            "--dashboard-prefix",
            "B",
            "--value",
            "v",
        ]);
        assert!(
            res.is_err(),
            "expected clap error when both dashboard and prefix are set"
        );
    }
}
