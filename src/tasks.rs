pub mod get_aws_secret;
pub mod get_vault_secret;
pub mod launch_influx;
pub mod login_to_vault;
pub mod port_forward;
pub mod run_pgcli;
pub mod select_aws_profile;
pub mod select_influx_instance;
pub mod select_kube_context;
pub mod select_rds_instance;
pub mod set_log_level;

use async_trait::async_trait;
use std::collections::HashMap;
use cliclack::progress_bar;
use console::{style, StyledObject};
use crate::{Args, Goal, GoalStatus};
use crate::aws::influx::InfluxInstance;
use crate::aws::rds::RdsInstance;
use crate::tasks::get_aws_secret::GetAwsSecretTask;
use crate::tasks::get_vault_secret::GetVaultSecretTask;
use crate::tasks::launch_influx::LaunchInfluxTask;
use crate::tasks::login_to_vault::LoginToVaultTask;
use crate::tasks::port_forward::{PortForwardInfo, PortForwardTask};
use crate::tasks::run_pgcli::RunPgcliTask;
use crate::tasks::select_aws_profile::{AwsProfileInfo, SelectAwsProfileTask};
use crate::tasks::select_influx_instance::SelectInfluxInstanceTask;
use crate::tasks::select_kube_context::{KubeContextInfo, SelectKubeContextTask};
use crate::tasks::select_rds_instance::SelectRdsInstanceTask;
use crate::tasks::set_log_level::SetLogLevelTask;
use crate::tasks::TaskType::SetLogLevel;

#[async_trait]
pub trait Task: Send + Sync {
    async fn execute(&self, args: &Option<Args>, state: &HashMap<Goal, TaskResult>, is_terminal_goal: bool) -> GoalStatus;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskType {
    GetAwsSecret,
    GetVaultSecret,
    LaunchInflux,
    LoginToVault,
    PortForward,
    RunPgcli,
    SelectAwsProfile,
    SelectInfluxInstance,
    SelectKubeContext,
    SelectRdsInstance,
    SetLogLevel,
}

impl TaskType {
    pub fn to_task(&self) -> Box<dyn Task> {
        match self {
            TaskType::GetAwsSecret => Box::new(GetAwsSecretTask),
            TaskType::GetVaultSecret => Box::new(GetVaultSecretTask),
            TaskType::LaunchInflux => Box::new(LaunchInfluxTask),
            TaskType::LoginToVault => Box::new(LoginToVaultTask),
            TaskType::PortForward => Box::new(PortForwardTask),
            TaskType::RunPgcli => Box::new(RunPgcliTask),
            TaskType::SelectAwsProfile => Box::new(SelectAwsProfileTask),
            TaskType::SelectInfluxInstance => Box::new(SelectInfluxInstanceTask),
            TaskType::SelectKubeContext => Box::new(SelectKubeContextTask),
            TaskType::SelectRdsInstance => Box::new(SelectRdsInstanceTask),
            TaskType::SetLogLevel => Box::new(SetLogLevelTask),
        }
    }
}

pub enum TaskResult {
    AwsProfile{ existing: Option<AwsProfileInfo>, updated: Option<AwsProfileInfo> },
    AwsSecret(String),
    InfluxCommand,
    InfluxInstance(InfluxInstance),
    KubeContext{ existing: Option<KubeContextInfo>, updated: Option<KubeContextInfo> },
    LogLevel,
    PgcliCommand(String),
    PortForward(PortForwardInfo),
    RdsInstance(RdsInstance),
    VaultSecret(String),
    VaultToken(String),
}

impl TaskResult {
    pub fn eval_string(&self) -> Option<String> {
        match self {
            TaskResult::AwsProfile{ existing: _, updated: Some(AwsProfileInfo { name, .. }) } => {
                Some(String::from(format!("export AWS_PROFILE={name}\n")))
            },
            TaskResult::KubeContext{ existing: _, updated: Some(KubeContextInfo { kubeconfig, .. }) } => {
                let path = kubeconfig.to_string_lossy();
                Some(String::from(format!("export KUBECONFIG={path}\n")))
            },
            TaskResult::PgcliCommand(cmd) => {
                Some(String::from(format!("{cmd}\n")))
            },
            _ => None,
        }
    }
}

pub fn color_output(output: &str, is_terminal_goal: bool) -> StyledObject<&str> {
    if is_terminal_goal {
        style(output).green()
    } else {
        style(output).blue()
    }
}

pub async fn sleep_indicator(seconds: u64, start_msg: &str, end_msg: &str) {
    let progress = progress_bar(seconds).with_spinner_template();
    progress.start(start_msg);

    let sleep_duration = tokio::time::Duration::from_secs(2);
    let steps = 100;
    let step_duration = sleep_duration / steps;

    for i in 0..=steps {
        progress.inc(1);
        if i < steps {
            tokio::time::sleep(step_duration).await;
        }
    }

    progress.stop(end_msg);
}
