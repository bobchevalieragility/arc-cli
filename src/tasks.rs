pub mod get_aws_secret;
pub mod select_aws_profile;
pub mod select_kube_context;

use async_trait::async_trait;
use std::collections::{HashSet, HashMap};
use crate::{ArcCommand, Args};
use crate::tasks::get_aws_secret::GetAwsSecretExecutor;
use crate::tasks::select_aws_profile::SelectAwsProfileExecutor;
use crate::tasks::select_kube_context::SelectKubeContextExecutor;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Task {
    GetAwsSecret,
    SelectAwsProfile,
    SelectKubeContext,
}

impl Task {
    pub fn command_tasks(command: &ArcCommand) -> Vec<Task> {
        let mut tasks = Vec::new();
        match command {
            ArcCommand::AwsSecret { .. } => {
                tasks.push(Task::GetAwsSecret);
            },
            ArcCommand::Switch { aws_profile: true, .. } => {
                tasks.push(Task::SelectAwsProfile);
            },
            ArcCommand::Switch { kube_context: true, .. } => {
                tasks.push(Task::SelectKubeContext);
            },
            ArcCommand::Switch { aws_profile: false, kube_context: false } => {
                tasks.push(Task::SelectAwsProfile);
                tasks.push(Task::SelectKubeContext);
            },
        }
        tasks
    }
}

#[async_trait]
impl Executor for Task {
    fn needs(&self) -> HashSet<Task> {
        match self {
            Task::GetAwsSecret => GetAwsSecretExecutor.needs(),
            Task::SelectAwsProfile => SelectAwsProfileExecutor.needs(),
            Task::SelectKubeContext => SelectKubeContextExecutor.needs(),
        }
    }

    async fn execute(&self, state: &State) -> TaskResult {
        match self {
            Task::GetAwsSecret => GetAwsSecretExecutor.execute(state).await,
            Task::SelectAwsProfile => SelectAwsProfileExecutor.execute(state).await,
            Task::SelectKubeContext => SelectKubeContextExecutor.execute(state).await,
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

pub struct State<'a> {
    args: &'a Args,
    results: &'a HashMap<Task, TaskResult>,
}

impl<'a> State<'a> {
    pub fn new(args: &'a Args, results: &'a HashMap<Task, TaskResult>) -> State<'a> {
        State { args, results }
    }
}

#[async_trait]
pub trait Executor {
    fn needs(&self) -> HashSet<Task>;

    async fn execute(&self, state: &State) -> TaskResult;
}