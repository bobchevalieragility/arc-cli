use std;
use std::convert::From;
use crate::args::{ArcCommand, Args};
use crate::tasks::{TaskResult, TaskType};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Goal {
    pub(crate) task_type: TaskType,
    pub(crate) args: Option<Args>,
    pub(crate) is_terminal_goal: bool,
}

impl Goal {
    pub fn new(task_type: TaskType, args: Option<Args>) -> Self {
        Goal { task_type, args, is_terminal_goal: false }
    }
    pub fn new_terminal(task_type: TaskType, args: Option<Args>) -> Self {
        Goal { task_type, args, is_terminal_goal: true }
    }
}

impl From<TaskType> for Goal {
    fn from(task_type: TaskType) -> Self {
        match task_type {
            TaskType::LoginToVault => Goal::new(TaskType::LoginToVault, None),
            TaskType::PerformSso => Goal::new(TaskType::PerformSso, None),
            TaskType::SelectActuatorService => Goal::new(TaskType::SelectActuatorService, None),
            TaskType::SelectAwsProfile => Goal::new(TaskType::SelectAwsProfile, Some(Args {
                command: ArcCommand::Switch {
                    aws_profile: true,
                    kube_context: false,
                    use_current: true,
                }
            })),
            TaskType::SelectInfluxInstance => Goal::new(TaskType::SelectInfluxInstance, None),
            TaskType::SelectKubeContext => Goal::new(TaskType::SelectKubeContext, Some(Args {
                command: ArcCommand::Switch {
                    aws_profile: false,
                    kube_context: true,
                    use_current: true,
                }
            })),
            TaskType::SelectRdsInstance => Goal::new(TaskType::SelectRdsInstance, None),
            _ => panic!("TaskType=>Goal conversion is missing."),
        }
    }
}

impl From<&Goal> for String {
    fn from(goal: &Goal) -> Self {
        format!("{:?}", goal)
    }
}

pub enum GoalStatus {
    Completed(TaskResult, OutroText),
    Needs(Goal),
}

pub enum OutroText {
    SingleLine{ key: String, value: String },
    MultiLine{ key: String, value: String },
    None,
}

impl OutroText {
    pub fn single(key: String, value: String) -> OutroText {
        OutroText::SingleLine { key, value }
    }
    pub fn multi(key: String, value: String) -> OutroText {
        OutroText::MultiLine { key, value }
    }
}
