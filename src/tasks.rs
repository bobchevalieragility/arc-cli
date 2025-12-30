pub mod select_aws_profile;
pub mod select_kube_context;

use std::collections::{HashSet, HashMap};
use crate::{ArcCommand, Args};
use crate::tasks::select_aws_profile::SelectAwsProfileExecutor;
use crate::tasks::select_kube_context::SelectKubeContextExecutor;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Task {
    SelectAwsProfile,
    SelectKubeContext,
}

impl Task {
    pub fn command_tasks(command: &ArcCommand) -> Vec<Task> {
        let mut tasks = Vec::new();
        match command {
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

impl Executor for Task {
    fn needs(&self) -> HashSet<Task> {
        match self {
            Task::SelectAwsProfile => SelectAwsProfileExecutor.needs(),
            Task::SelectKubeContext => SelectKubeContextExecutor.needs(),
        }
    }

    fn execute(&self, state: &State) -> TaskResult {
        match self {
            Task::SelectAwsProfile => SelectAwsProfileExecutor.execute(state),
            Task::SelectKubeContext => SelectKubeContextExecutor.execute(state),
        }
    }
}

pub enum TaskResult {
    AwsProfile(Option<String>),
    KubeContext(Option<String>), //TODO should this contain a Path instead of a String?
}

impl TaskResult {
    pub fn eval_string(&self) -> Option<String> {
        match self {
            TaskResult::AwsProfile(Some(aws_profile)) => {
                Some(String::from(format!("export AWS_PROFILE={aws_profile}\n")))
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

pub trait Executor {
    fn needs(&self) -> HashSet<Task>;

    fn execute(&self, state: &State) -> TaskResult;
}