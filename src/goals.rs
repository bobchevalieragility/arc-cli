use std;
use std::convert::From;
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
use crate::tasks::set_log_level::{Level, SetLogLevelTask};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Goal {
    pub goal_type: GoalType,
    pub params: GoalParams,
    pub is_terminal_goal: bool,
}

impl Goal {
    pub fn new(goal_type: GoalType, params: GoalParams) -> Self {
        Goal { goal_type, params, is_terminal_goal: false }
    }
    pub fn new_terminal(goal_type: GoalType, params: GoalParams) -> Self {
        Goal { goal_type, params, is_terminal_goal: true }
    }
}

impl From<GoalType> for Goal {
    fn from(goal_type: GoalType) -> Self {
        match goal_type {
            GoalType::ActuatorServiceSelected => Goal::new(
                GoalType::ActuatorServiceSelected,
                GoalParams::None
            ),
            GoalType::AwsProfileSelected => Goal::new(
                GoalType::AwsProfileSelected,
                GoalParams::AwsProfileSelected { use_current: true },
            ),
            GoalType::InfluxInstanceSelected => Goal::new(
                GoalType::InfluxInstanceSelected,
                GoalParams::None
            ),
            GoalType::KubeContextSelected => Goal::new(
                GoalType::KubeContextSelected,
                GoalParams::KubeContextSelected { use_current: true},
            ),
            GoalType::RdsInstanceSelected => Goal::new(
                GoalType::RdsInstanceSelected,
                GoalParams::None
            ),
            GoalType::SsoTokenValid => Goal::new(
                GoalType::SsoTokenValid,
                GoalParams::None
            ),
            GoalType::VaultTokenValid => Goal::new(
                GoalType::VaultTokenValid,
                GoalParams::None
            ),
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

impl From<GoalType> for String {
    fn from(goal_type: GoalType) -> Self {
        format!("{:?}", goal_type)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum GoalParams {
    AwsProfileSelected {
        use_current: bool,
    },
    AwsSecretKnown {
        name: Option<String>,
    },
    KubeContextSelected {
        use_current: bool,
    },
    LogLevelSet {
        service: Option<String>,
        package: String,
        level: Option<Level>,
        display_only: bool,
    },
    None,
    PortForwardEstablished {
        service: Option<String>,
        port: Option<u16>,
        tear_down: bool,
    },
    VaultSecretKnown {
        path: Option<String>,
        field: Option<String>,
    },
}

impl From<&GoalParams> for String {
    fn from(params: &GoalParams) -> Self {
        format!("{:?}", params)
    }
}
