use clap::{Parser, Subcommand};
use std;
use std::convert::From;
use crate::goals::GoalType;
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
    #[command(about = "Start port-forwarding to a Kubernetes service")]
    PortForward {
        #[arg(short, long, help = "Service name, e.g. 'metrics' (if omitted, will prompt)")]
        service: Option<String>,

        #[arg(short, long, help = "Local port (defaults to random, unused port)")]
        port: Option<u16>,

        #[arg(short, long, help = "Tear down port-forwarding when command exits")]
        tear_down: bool,
    },
    #[command(about = "Switch AWS profile and/or Kubernetes context")]
    Switch {
        #[arg(short, long, help = "Switch AWS profile (if false and kube_context is false, will switch both)")]
        aws_profile: bool,

        #[arg(short, long, help = "Switch kube context (if false and kube_context is false, will switch both)")]
        kube_context: bool,

        #[arg(short, long, help = "Whether to skip if already set (defaults to false)")]
        use_current: bool,
    },
    #[command(about = "Generate a shell completion script")]
    Completions,
    #[command(about = "Temporary command to test SSO")]
    Sso,
}

impl CliArgs {
    pub(crate) fn to_goals(&self) -> Vec<Goal> {
        match self.command {
            CliCommand::AwsSecret { .. } => vec![
                Goal::new_terminal(GoalType::AwsSecretKnown, Some(self.clone()))
            ],
            CliCommand::Completions => vec![
                Goal::new_terminal(GoalType::TabCompletionsExist, Some(self.clone()))
            ],
            CliCommand::LogLevel { .. } => vec![
                Goal::new_terminal(GoalType::LogLevelSet, Some(self.clone()))
            ],
            CliCommand::Pgcli => vec![
                Goal::new_terminal(GoalType::PgcliRunning, Some(self.clone()))
            ],
            CliCommand::PortForward { .. } => vec![
                Goal::new_terminal(GoalType::PortForwardEstablished, Some(self.clone()))
            ],
            CliCommand::Influx => vec![
                Goal::new_terminal(GoalType::InfluxLaunched, Some(self.clone()))
            ],
            CliCommand::Switch { aws_profile: true, .. } => vec![
                Goal::new_terminal(GoalType::AwsProfileSelected, Some(self.clone()))
            ],
            CliCommand::Switch { kube_context: true, .. } => vec![
                Goal::new_terminal(GoalType::KubeContextSelected, Some(self.clone()))
            ],
            CliCommand::Switch { aws_profile: false, kube_context: false, .. } => vec![
                Goal::new_terminal(GoalType::KubeContextSelected, Some(self.clone())),
                Goal::new_terminal(GoalType::AwsProfileSelected, Some(self.clone()))
            ],
            CliCommand::Vault { .. } => vec![
                Goal::new_terminal(GoalType::VaultSecretKnown, Some(self.clone()))
            ],
            CliCommand::Sso => vec![
                Goal::new_terminal(GoalType::SsoTokenValid, Some(self.clone()))
            ],
        }
    }
}