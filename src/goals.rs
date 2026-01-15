use std;
use std::convert::From;
use crate::args::{CliCommand, CliArgs};
use crate::tasks::{TaskResult, Task};
use crate::tasks::create_tab_completions::CreateTabCompletionsTask;
use crate::tasks::get_aws_secret::GetAwsSecretTask;
use crate::tasks::get_vault_secret::GetVaultSecretTask;
use crate::tasks::launch_influx::LaunchInfluxTask;
use crate::tasks::login_to_vault::LoginToVaultTask;
use crate::tasks::perform_sso::PerformSsoTask;
use crate::tasks::port_forward::PortForwardTask;
use crate::tasks::run_pgcli::RunPgcliTask;
use crate::tasks::select_actuator_service::SelectActuatorServiceTask;
use crate::tasks::select_aws_profile::SelectAwsProfileTask;
use crate::tasks::select_influx_instance::SelectInfluxInstanceTask;
use crate::tasks::select_kube_context::SelectKubeContextTask;
use crate::tasks::select_rds_instance::SelectRdsInstanceTask;
use crate::tasks::set_log_level::SetLogLevelTask;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Goal {
    pub(crate) goal_type: GoalType,
    pub(crate) args: Option<CliArgs>,
    pub(crate) is_terminal_goal: bool,
}

impl Goal {
    pub fn new(goal_type: GoalType, args: Option<CliArgs>) -> Self {
        Goal { goal_type, args, is_terminal_goal: false }
    }
    pub fn new_terminal(goal_type: GoalType, args: Option<CliArgs>) -> Self {
        Goal { goal_type, args, is_terminal_goal: true }
    }
}

impl From<GoalType> for Goal {
    fn from(task_type: GoalType) -> Self {
        match task_type {
            GoalType::LoginToVault => Goal::new(GoalType::LoginToVault, None),
            GoalType::PerformSso => Goal::new(GoalType::PerformSso, None),
            GoalType::SelectActuatorService => Goal::new(GoalType::SelectActuatorService, None),
            GoalType::SelectAwsProfile => Goal::new(GoalType::SelectAwsProfile, Some(CliArgs {
                command: CliCommand::Switch {
                    aws_profile: true,
                    kube_context: false,
                    use_current: true,
                }
            })),
            GoalType::SelectInfluxInstance => Goal::new(GoalType::SelectInfluxInstance, None),
            GoalType::SelectKubeContext => Goal::new(GoalType::SelectKubeContext, Some(CliArgs {
                command: CliCommand::Switch {
                    aws_profile: false,
                    kube_context: true,
                    use_current: true,
                }
            })),
            GoalType::SelectRdsInstance => Goal::new(GoalType::SelectRdsInstance, None),
            _ => panic!("GoalType=>Goal conversion is missing."),
        }
    }
}

impl From<&Goal> for String {
    fn from(goal: &Goal) -> Self {
        format!("{:?}", goal)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GoalType {
    CreateTabCompletions,
    GetAwsSecret,
    GetVaultSecret,
    LaunchInflux,
    LoginToVault,
    PerformSso,
    PortForward,
    RunPgcli,
    SelectActuatorService,
    SelectAwsProfile,
    SelectInfluxInstance,
    SelectKubeContext,
    SelectRdsInstance,
    SetLogLevel,
}

impl GoalType {
    pub fn to_task(&self) -> Box<dyn Task> {
        match self {
            GoalType::CreateTabCompletions => Box::new(CreateTabCompletionsTask),
            GoalType::GetAwsSecret => Box::new(GetAwsSecretTask),
            GoalType::GetVaultSecret => Box::new(GetVaultSecretTask),
            GoalType::LaunchInflux => Box::new(LaunchInfluxTask),
            GoalType::LoginToVault => Box::new(LoginToVaultTask),
            GoalType::PerformSso => Box::new(PerformSsoTask),
            GoalType::PortForward => Box::new(PortForwardTask),
            GoalType::RunPgcli => Box::new(RunPgcliTask),
            GoalType::SelectActuatorService => Box::new(SelectActuatorServiceTask),
            GoalType::SelectAwsProfile => Box::new(SelectAwsProfileTask),
            GoalType::SelectInfluxInstance => Box::new(SelectInfluxInstanceTask),
            GoalType::SelectKubeContext => Box::new(SelectKubeContextTask),
            GoalType::SelectRdsInstance => Box::new(SelectRdsInstanceTask),
            GoalType::SetLogLevel => Box::new(SetLogLevelTask),
        }
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
