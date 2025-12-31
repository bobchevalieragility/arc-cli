pub mod get_aws_secret;
pub mod select_aws_profile;
pub mod select_kube_context;

use async_trait::async_trait;
use std::collections::HashMap;
use crate::{Args, Goal, GoalStatus};
use crate::tasks::get_aws_secret::GetAwsSecretExecutor;
use crate::tasks::select_aws_profile::SelectAwsProfileExecutor;
use crate::tasks::select_kube_context::SelectKubeContextExecutor;

//TODO Should we just rename the Executor trait to Task and use dynamic dispatch?
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Task {
    GetAwsSecret,
    SelectAwsProfile,
    SelectKubeContext,
}

#[async_trait]
impl Executor for Task {
    // async fn execute(&self, state: &State) -> TaskResult {
    async fn execute(&self, args: &Args, state: &HashMap<Goal, TaskResult>) -> GoalStatus {
        match self {
            Task::GetAwsSecret => GetAwsSecretExecutor.execute(args, state).await,
            Task::SelectAwsProfile => SelectAwsProfileExecutor.execute(args, state).await,
            Task::SelectKubeContext => SelectKubeContextExecutor.execute(args, state).await,
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

#[async_trait]
pub trait Executor {
    async fn execute(&self, args: &Args, state: &HashMap<Goal, TaskResult>) -> GoalStatus;
}