use clap::{Parser, Subcommand};
use std;
use std::convert::From;
use std::path::PathBuf;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use crate::tasks::set_log_level::Level;
use crate::goals::Goal;

#[derive(Parser, Clone, Debug, PartialEq, Eq, Hash)]
#[command(author, version, about = "CLI Tool for Arc Backend")]
pub struct CliArgs {
    #[command(subcommand)]
    pub(crate) command: CliCommand,
}

#[derive(Subcommand, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CliCommand {
    #[command(about = "View or set the log level for a Java Spring Boot service")]
    LogLevel {
        #[arg(short, long, help = "Service name, e.g. 'metrics' (if omitted, will prompt)")]
        service: Option<String>,

        #[arg(short, long, default_value = "ROOT", help = "Package, e.g. 'com.agilityrobotics.metrics' (defaults to ROOT)")]
        package: String,

        #[arg(short, long, help = "Desired log level (if omitted, will prompt)")]
        level: Option<Level>,

        #[arg(short, long, help = "Just print the current log level")]
        display_only: bool,
    },
    #[command(about = "Retrieve a secret value from AWS Secrets Manager")]
    AwsSecret {
        #[arg(short, long, help = "Name of the secret to retrieve (if omitted, will prompt)")]
        name: Option<String>,
    },
    #[command(about = "Retrieve a secret value from Vault")]
    Vault {
        #[arg(short, long, help = "Path to secret to retrieve (if omitted, will prompt)")]
        path: Option<String>,

        #[arg(short, long, help = "Field within secret to retrieve (defaults to entire secret)")]
        field: Option<String>,
    },
    #[command(about = "Launch pgcli to interact with a Postgres RDS instance")]
    Pgcli,
    #[command(about = "Launch the InfluxDB UI")]
    Influx,
    #[command(about = "Query InfluxDB and save results to a CSV file")]
    InfluxQuery {
        #[arg(short, long, help = "Query for all records on this day (e.g., '2026-01-19')", conflicts_with = "start")]
        day: Option<NaiveDate>,

        #[arg(short, long, help = "Start time as either RFC3339 or ms since epoch (e.g. '2026-01-01T00:00:00Z')", conflicts_with = "day", value_parser = parse_datetime)]
        start: Option<DateTime<Utc>>,

        #[arg(short, long, help = "End time as either RFC3339 or ms since epoch. Defaults to NOW. (e.g. '2025-01-19T00:00:00Z')", requires = "start", conflicts_with = "day", value_parser = parse_datetime)]
        end: Option<DateTime<Utc>>,

        #[arg(short, long, help = "Path to output file")]
        output: PathBuf,
    },
    #[command(about = "Start port-forwarding to a Kubernetes service")]
    PortForward {
        #[arg(short, long, help = "Service name, e.g. 'metrics' (if omitted, will prompt)")]
        service: Option<String>,

        #[arg(short, long, help = "Local port (defaults to random, unused port)")]
        port: Option<u16>,
    },
    #[command(about = "Switch AWS profile and/or Kubernetes context")]
    Switch {
        #[arg(short, long, help = "Switch AWS profile (if false and kube_context is false, will switch both)")]
        aws_profile: bool,

        #[arg(short, long, help = "Switch kube context (if false and kube_context is false, will switch both)")]
        kube_context: bool,
    },
    #[command(about = "Generate a shell completion script")]
    Completions,
}

impl CliCommand {
    pub(crate) fn to_goals(self) -> Vec<Goal> {
        match self {
            CliCommand::AwsSecret { name } => vec![
                Goal::terminal_aws_secret_known(name)
            ],
            CliCommand::Completions => vec![Goal::terminal_tab_completions()],
            CliCommand::LogLevel { service, package, level, display_only } => vec![
                Goal::terminal_log_level_set(service, package, level, display_only)
            ],
            CliCommand::Pgcli => vec![Goal::terminal_pgcli_running()],
            CliCommand::PortForward { service, port } => vec![
                Goal::terminal_port_forward_established(service, port)
            ],
            CliCommand::Influx => vec![Goal::terminal_influx_launched()],
            CliCommand::InfluxQuery { day, start, end, output } => vec![
                Goal::terminal_influx_queried(day, start, end, output)
            ],
            CliCommand::Switch { aws_profile: true, kube_context: true } => vec![
                Goal::terminal_aws_profile_selected(),
                Goal::terminal_kube_context_selected(),
            ],
            CliCommand::Switch { aws_profile: false, kube_context: false } => vec![
                Goal::terminal_aws_profile_selected(),
                Goal::terminal_kube_context_selected(),
            ],
            CliCommand::Switch { aws_profile: true, kube_context: false } => vec![
                Goal::terminal_aws_profile_selected(),
            ],
            CliCommand::Switch { aws_profile: false, kube_context: true } => vec![
                Goal::terminal_kube_context_selected(),
            ],
            CliCommand::Vault { path, field } => vec![
                Goal::terminal_vault_secret_known(path, field)
            ],
        }
    }
}

fn parse_datetime(input: &str) -> Result<DateTime<Utc>, String> {
    // Try parsing as milliseconds since epoch
    if let Ok(millis) = input.parse::<i64>() {
        // Convert to seconds and nanoseconds
        let seconds = millis / 1000;
        let nanoseconds = (millis % 1000) * 1_000_000;

        return Utc.timestamp_opt(seconds, nanoseconds as u32)
            .single()
            .ok_or_else(|| format!("Milliseconds since epoch '{}' is out of range", input));
    }

    // Try parsing as RFC3339 string
    DateTime::parse_from_rfc3339(input)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| format!("Invalid datetime format '{}': {}", input, e))
}