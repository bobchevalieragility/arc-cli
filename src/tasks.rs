pub mod get_aws_secret;
pub mod get_vault_secret;
pub mod launch_influx;
pub mod login_to_vault;
pub mod run_pgcli;
pub mod select_aws_profile;
pub mod select_influx_instance;
pub mod select_kube_context;
pub mod select_rds_instance;
pub mod set_log_level;

use async_trait::async_trait;
use std::collections::HashMap;
use console::{style, StyledObject};
use crate::{Args, Goal, GoalStatus};
use crate::aws::influx::InfluxInstance;
use crate::aws::rds::RdsInstance;
use crate::tasks::get_aws_secret::GetAwsSecretTask;
use crate::tasks::get_vault_secret::GetVaultSecretTask;
use crate::tasks::launch_influx::LaunchInfluxTask;
use crate::tasks::login_to_vault::LoginToVaultTask;
use crate::tasks::run_pgcli::RunPgcliTask;
use crate::tasks::select_aws_profile::{AwsProfileInfo, SelectAwsProfileTask};
use crate::tasks::select_influx_instance::SelectInfluxInstanceTask;
use crate::tasks::select_kube_context::SelectKubeContextTask;
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
    AwsProfile{ old: Option<AwsProfileInfo>, new: Option<AwsProfileInfo> },
    AwsSecret(String),
    InfluxCommand,
    InfluxInstance(InfluxInstance),
    KubeContext(Option<String>),
    LogLevel,
    PgcliCommand(String),
    RdsInstance(RdsInstance),
    VaultSecret(String),
    VaultToken(String),
}

impl TaskResult {
    pub fn eval_string(&self) -> Option<String> {
        match self {
            TaskResult::AwsProfile{ old: _, new: Some(AwsProfileInfo { name, .. }) } => {
                Some(String::from(format!("export AWS_PROFILE={name}\n")))
            },
            TaskResult::KubeContext(Some(kubeconfig_path)) => {
                Some(String::from(format!("export KUBECONFIG={kubeconfig_path}\n")))
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
