use std;
use std::convert::From;
use crate::args::{CliCommand, CliArgs};
use crate::tasks::Task;
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
            GoalType::VaultTokenValid => Goal::new(GoalType::VaultTokenValid, None),
            GoalType::SsoTokenValid => Goal::new(GoalType::SsoTokenValid, None),
            GoalType::ActuatorServiceSelected => Goal::new(GoalType::ActuatorServiceSelected, None),
            GoalType::AwsProfileSelected => Goal::new(GoalType::AwsProfileSelected, Some(CliArgs {
                command: CliCommand::Switch {
                    aws_profile: true,
                    kube_context: false,
                    use_current: true,
                }
            })),
            GoalType::InfluxInstanceSelected => Goal::new(GoalType::InfluxInstanceSelected, None),
            GoalType::KubeContextSelected => Goal::new(GoalType::KubeContextSelected, Some(CliArgs {
                command: CliCommand::Switch {
                    aws_profile: false,
                    kube_context: true,
                    use_current: true,
                }
            })),
            GoalType::RdsInstanceSelected => Goal::new(GoalType::RdsInstanceSelected, None),
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
    ActuatorServiceSelected,
    AwsProfileSelected,
    AwsSecretKnown,
    InfluxInstanceSelected,
    InfluxLaunched,
    KubeContextSelected,
    LogLevelSet,
    PgcliRunning,
    PortForwardEstablished,
    RdsInstanceSelected,
    SsoTokenValid,
    TabCompletionsExist,
    VaultSecretKnown,
    VaultTokenValid,
}

impl GoalType {
    pub fn to_task(&self) -> Box<dyn Task> {
        match self {
            GoalType::TabCompletionsExist => Box::new(CreateTabCompletionsTask),
            GoalType::AwsSecretKnown => Box::new(GetAwsSecretTask),
            GoalType::VaultSecretKnown => Box::new(GetVaultSecretTask),
            GoalType::InfluxLaunched => Box::new(LaunchInfluxTask),
            GoalType::VaultTokenValid => Box::new(LoginToVaultTask),
            GoalType::SsoTokenValid => Box::new(PerformSsoTask),
            GoalType::PortForwardEstablished => Box::new(PortForwardTask),
            GoalType::PgcliRunning => Box::new(RunPgcliTask),
            GoalType::ActuatorServiceSelected => Box::new(SelectActuatorServiceTask),
            GoalType::AwsProfileSelected => Box::new(SelectAwsProfileTask),
            GoalType::InfluxInstanceSelected => Box::new(SelectInfluxInstanceTask),
            GoalType::KubeContextSelected => Box::new(SelectKubeContextTask),
            GoalType::RdsInstanceSelected => Box::new(SelectRdsInstanceTask),
            GoalType::LogLevelSet => Box::new(SetLogLevelTask),
        }
    }
}
