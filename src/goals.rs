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
    fn new(goal_type: GoalType, params: GoalParams) -> Self {
        Goal { goal_type, params, is_terminal_goal: false }
    }

    fn new_terminal(goal_type: GoalType, params: GoalParams) -> Self {
        Goal { goal_type, params, is_terminal_goal: true }
    }

    pub fn actuator_service_selected() -> Self {
        Goal::new(GoalType::ActuatorServiceSelected, GoalParams::None)
    }

    //TODO is use_current always true for non-terminal goals?
    pub fn aws_profile_selected(use_current: bool) -> Self {
        let params = GoalParams::AwsProfileSelected { use_current };
        Goal::new(GoalType::AwsProfileSelected, params)
    }

    pub fn terminal_aws_profile_selected() -> Self {
        let params = GoalParams::AwsProfileSelected { use_current: false };
        Goal::new_terminal(GoalType::AwsProfileSelected, params)
    }

    pub fn aws_secret_known(secret_name: String) -> Self {
        let params = GoalParams::AwsSecretKnown { name: Some(secret_name) };
        Goal::new(GoalType::AwsSecretKnown, params)
    }

    pub fn terminal_aws_secret_known(name: Option<String>) -> Self {
        let params = GoalParams::AwsSecretKnown { name };
        Goal::new_terminal(GoalType::AwsSecretKnown, params)
    }

    pub fn influx_instance_selected() -> Self {
        Goal::new(GoalType::InfluxInstanceSelected, GoalParams::None)
    }

    pub fn terminal_influx_launched() -> Self {
        Goal::new_terminal(GoalType::InfluxLaunched, GoalParams::None)
    }

    //TODO is use_current always true for non-terminal goals?
    pub fn kube_context_selected(use_current: bool) -> Self {
        let params = GoalParams::KubeContextSelected { use_current };
        Goal::new(GoalType::KubeContextSelected, params)
    }

    pub fn terminal_kube_context_selected() -> Self {
        let params = GoalParams::KubeContextSelected { use_current: false };
        Goal::new_terminal(GoalType::KubeContextSelected, params)
    }

    pub fn terminal_log_level_set(
        service: Option<String>,
        package: String,
        level: Option<Level>,
        display_only: bool
    ) -> Self {
        let params = GoalParams::LogLevelSet { service, package, level, display_only };
        Goal::new_terminal(GoalType::LogLevelSet, params)
    }

    pub fn terminal_pgcli_running() -> Self {
        Goal::new_terminal(GoalType::PgcliRunning, GoalParams::None)
    }

    pub fn port_forward_established(service: String) -> Self {
        let params = GoalParams::PortForwardEstablished {
            service: Some(service),
            port: None,
            tear_down: true
        };
        Goal::new(GoalType::PortForwardEstablished, params)
    }

    pub fn terminal_port_forward_established(service: Option<String>, port: Option<u16>) -> Self {
        let params = GoalParams::PortForwardEstablished { service, port, tear_down: false };
        Goal::new_terminal(GoalType::PortForwardEstablished, params)
    }

    pub fn rds_instance_selected() -> Self {
        Goal::new(GoalType::RdsInstanceSelected, GoalParams::None)
    }

    pub fn sso_token_valid() -> Self {
        Goal::new(GoalType::SsoTokenValid, GoalParams::None)
    }

    pub fn terminal_tab_completions() -> Self {
        Goal::new_terminal(GoalType::TabCompletionsExist, GoalParams::None)
    }

    pub fn vault_token_valid() -> Self {
        Goal::new(GoalType::VaultTokenValid, GoalParams::None)
    }

    pub fn terminal_vault_secret_known(path: Option<String>, field: Option<String>) -> Self {
        let params = GoalParams::VaultSecretKnown { path, field };
        Goal::new_terminal(GoalType::VaultSecretKnown, params)
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
