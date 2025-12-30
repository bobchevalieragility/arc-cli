pub mod select_aws_profile;

use std::collections::{HashSet, HashMap};
use crate::Args;
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
        match self {
            Task::SelectAwsProfile(e) => e.needs(),
        }
    }

    fn provides(&self) -> Goal {
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
    // TODO map goals to subcommands
    AwsProfileSelected,
}

// impl Goal {
//     pub fn satisfies_command(&self, command: &Command) -> bool {
//         match (self, command) {
//             (Goal::AwsProfileSelected, Command::Switch { aws_profile: true, .. }) => true,
//             (Goal::ShellSpawned, Some(crate::Command::Switch { .. })) => true,
//             _ => false,
//         }
//     }
// }

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