pub mod get_aws_secret;
pub mod get_vault_secret;
pub mod login_to_vault;
pub mod run_pgcli;
pub mod select_aws_profile;
pub mod select_kube_context;
pub mod select_rds_instance;

use async_trait::async_trait;
use std::collections::HashMap;
use console::{style, StyledObject};
use crate::{Args, Goal, GoalStatus};
use crate::aws::rds::RdsInstance;
use crate::tasks::get_aws_secret::GetAwsSecretTask;
use crate::tasks::get_vault_secret::GetVaultSecretTask;
use crate::tasks::login_to_vault::LoginToVaultTask;
use crate::tasks::run_pgcli::RunPgcliTask;
use crate::tasks::select_aws_profile::{AwsProfileInfo, SelectAwsProfileTask};
use crate::tasks::select_kube_context::SelectKubeContextTask;
use crate::tasks::select_rds_instance::SelectRdsInstanceTask;

#[async_trait]
pub trait Task: Send + Sync {
    async fn execute(&self, args: &Option<Args>, state: &HashMap<Goal, TaskResult>, is_terminal_goal: bool) -> GoalStatus;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskType {
    GetAwsSecret,
    GetVaultSecret,
    LoginToVault,
    RunPgcli,
    SelectAwsProfile,
    SelectKubeContext,
    SelectRdsInstance,
}

impl TaskType {
    pub fn to_task(&self) -> Box<dyn Task> {
        match self {
            TaskType::GetAwsSecret => Box::new(GetAwsSecretTask),
            TaskType::GetVaultSecret => Box::new(GetVaultSecretTask),
            TaskType::LoginToVault => Box::new(LoginToVaultTask),
            TaskType::RunPgcli => Box::new(RunPgcliTask),
            TaskType::SelectAwsProfile => Box::new(SelectAwsProfileTask),
            TaskType::SelectKubeContext => Box::new(SelectKubeContextTask),
            TaskType::SelectRdsInstance => Box::new(SelectRdsInstanceTask),
        }
    }
}

//TODO Should some of these result variants NOT be Option types?
pub enum TaskResult {
    AwsSecret(Option<String>),
    VaultSecret(String),
    VaultToken(String),
    AwsProfile{ old: Option<AwsProfileInfo>, new: Option<AwsProfileInfo> },
    KubeContext(Option<String>),
    PgcliCommand(String),
    RdsInstance(Option<RdsInstance>),
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
