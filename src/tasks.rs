pub mod select_aws_profile;

use std::collections::{HashSet, HashMap};
use crate::{ArcCommand, Args};
use crate::tasks::select_aws_profile::SelectAwsProfileExecutor;

pub const ALL_TASKS: [Task; 1] = [
    Task::SelectAwsProfile(SelectAwsProfileExecutor),
];

#[derive(Debug)]
pub enum Task {
    SelectAwsProfile(SelectAwsProfileExecutor),
}

impl Executor for Task {
    fn needs(&self) -> HashSet<Goal> {
        //TODO can this just return Task?
        match self {
            Task::SelectAwsProfile(e) => e.needs(),
        }
    }

    fn provides(&self) -> Goal {
        //TODO if we just return Task above can we remove this method?
        match self {
            Task::SelectAwsProfile(e) => e.provides(),
        }
    }

    fn execute(&self, state: &State) -> TaskResult {
        match self {
            Task::SelectAwsProfile(e) => e.execute(state),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Goal {
    AwsProfileSelected,
    KubeContextSelected,
}

impl Goal {
    pub fn command_goals(command: &ArcCommand) -> HashSet<Goal> {
        let mut goals = HashSet::new();
        match command {
            ArcCommand::Switch { aws_profile: true, .. } => {
                goals.insert(Goal::AwsProfileSelected);
            },
            ArcCommand::Switch { kube_context: true, .. } => {
                goals.insert(Goal::KubeContextSelected);
            },
            ArcCommand::Switch { aws_profile: false, kube_context: false } => {
                goals.insert(Goal::AwsProfileSelected);
                goals.insert(Goal::KubeContextSelected);
            },
        }
        goals
    }
}

pub enum TaskResult {
    AwsProfile(Option<String>),
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
    results: &'a HashMap<Goal, TaskResult>,
}

impl<'a> State<'a> {
    pub fn new(args: &'a Args, results: &'a HashMap<Goal, TaskResult>) -> State<'a> {
        State { args, results }
    }
}

pub trait Executor {
    fn needs(&self) -> HashSet<Goal>;
    fn provides(&self) -> Goal;
    fn execute(&self, state: &State) -> TaskResult;
}