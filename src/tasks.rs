pub mod get_aws_secret;
pub mod select_aws_profile;
pub mod select_kube_context;

use async_trait::async_trait;
use std::collections::HashMap;
use crate::{Args, Goal, GoalStatus};
use crate::tasks::get_aws_secret::GetAwsSecretTask;
use crate::tasks::select_aws_profile::SelectAwsProfileTask;
use crate::tasks::select_kube_context::SelectKubeContextTask;

#[async_trait]
pub trait Task: Send + Sync {
    async fn execute(&self, args: &Args, state: &HashMap<Goal, TaskResult>) -> GoalStatus;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskType {
    GetAwsSecret,
    SelectAwsProfile,
    SelectKubeContext,
}

impl TaskType {
    pub fn to_task(&self) -> Box<dyn Task> {
        match self {
            TaskType::GetAwsSecret => Box::new(GetAwsSecretTask),
            TaskType::SelectAwsProfile => Box::new(SelectAwsProfileTask),
            TaskType::SelectKubeContext => Box::new(SelectKubeContextTask),
        }
    }
}

pub enum TaskResult {
    AwsSecret(Option<String>),
    AwsProfile{ old: Option<String>, new: Option<String> },
    KubeContext(Option<String>),
}

impl TaskResult {
    pub fn eval_string(&self) -> Option<String> {
        match self {
            TaskResult::AwsProfile{ old: _, new: Some(aws_profile) } => {
                Some(String::from(format!("export AWS_PROFILE={aws_profile}\n")))
            },
            TaskResult::KubeContext(Some(kubeconfig_path)) => {
                Some(String::from(format!("export KUBECONFIG={kubeconfig_path}\n")))
            },
            _ => None,
        }
    }
}
