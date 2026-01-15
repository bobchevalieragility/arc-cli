use std::convert::From;
use clap::{Parser, Subcommand};
use crate::tasks::TaskType;
use crate::tasks::set_log_level::Level;
use std;
use crate::goals::Goal;

#[derive(Parser, Clone, Debug, PartialEq, Eq, Hash)]
#[command(author, version, about = "CLI Tool for Arc Backend")]
pub struct Args {
    #[command(subcommand)]
    pub(crate) command: ArcCommand,
}

#[derive(Subcommand, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArcCommand {
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

impl Args {
    pub(crate) fn to_goals(&self) -> Vec<Goal> {
        match self.command {
            ArcCommand::AwsSecret { .. } => vec![
                Goal::new_terminal(TaskType::GetAwsSecret, Some(self.clone()))
            ],
            ArcCommand::Completions => vec![
                Goal::new_terminal(TaskType::CreateTabCompletions, Some(self.clone()))
            ],
            ArcCommand::LogLevel { .. } => vec![
                Goal::new_terminal(TaskType::SetLogLevel, Some(self.clone()))
            ],
            ArcCommand::Pgcli => vec![
                Goal::new_terminal(TaskType::RunPgcli, Some(self.clone()))
            ],
            ArcCommand::PortForward { .. } => vec![
                Goal::new_terminal(TaskType::PortForward, Some(self.clone()))
            ],
            ArcCommand::Influx => vec![
                Goal::new_terminal(TaskType::LaunchInflux, Some(self.clone()))
            ],
            ArcCommand::Switch { aws_profile: true, .. } => vec![
                Goal::new_terminal(TaskType::SelectAwsProfile, Some(self.clone()))
            ],
            ArcCommand::Switch { kube_context: true, .. } => vec![
                Goal::new_terminal(TaskType::SelectKubeContext, Some(self.clone()))
            ],
            ArcCommand::Switch { aws_profile: false, kube_context: false, .. } => vec![
                Goal::new_terminal(TaskType::SelectKubeContext, Some(self.clone())),
                Goal::new_terminal(TaskType::SelectAwsProfile, Some(self.clone()))
            ],
            ArcCommand::Vault { .. } => vec![
                Goal::new_terminal(TaskType::GetVaultSecret, Some(self.clone()))
            ],
            ArcCommand::Sso => vec![
                Goal::new_terminal(TaskType::PerformSso, Some(self.clone()))
            ],
        }
    }
}