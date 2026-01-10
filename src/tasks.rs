pub mod get_aws_secret;
pub mod get_vault_secret;
pub mod launch_influx;
pub mod login_to_vault;
pub mod port_forward;
pub mod run_pgcli;
pub mod select_actuator_service;
pub mod select_aws_profile;
pub mod select_influx_instance;
pub mod select_kube_context;
pub mod select_rds_instance;
pub mod set_log_level;

use async_trait::async_trait;
use cliclack::progress_bar;
use crate::{Args, GoalStatus, State};
use crate::aws::influx::InfluxInstance;
use crate::aws::rds::RdsInstance;
use crate::errors::ArcError;
use crate::tasks::get_aws_secret::GetAwsSecretTask;
use crate::tasks::get_vault_secret::GetVaultSecretTask;
use crate::tasks::launch_influx::LaunchInfluxTask;
use crate::tasks::login_to_vault::LoginToVaultTask;
use crate::tasks::port_forward::{PortForwardInfo, PortForwardTask};
use crate::tasks::run_pgcli::RunPgcliTask;
use crate::tasks::select_actuator_service::{ActuatorService, SelectActuatorServiceTask};
use crate::tasks::select_aws_profile::{AwsProfileInfo, SelectAwsProfileTask};
use crate::tasks::select_influx_instance::SelectInfluxInstanceTask;
use crate::tasks::select_kube_context::{KubeContextInfo, SelectKubeContextTask};
use crate::tasks::select_rds_instance::SelectRdsInstanceTask;
use crate::tasks::set_log_level::SetLogLevelTask;

#[async_trait]
pub trait Task: Send + Sync {
    fn print_intro(&self) -> Result<(), ArcError>;
    async fn execute(&self, args: &Option<Args>, state: &State) -> Result<GoalStatus, ArcError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskType {
    GetAwsSecret,
    GetVaultSecret,
    LaunchInflux,
    LoginToVault,
    PortForward,
    RunPgcli,
    SelectActuatorService,
    SelectAwsProfile,
    SelectInfluxInstance,
    SelectKubeContext,
    SelectRdsInstance,
    SetLogLevel,
}

impl TaskType {
    pub fn to_task(&self) -> Box<dyn Task> {
        match self {
            TaskType::GetAwsSecret => Box::new(GetAwsSecretTask),
            TaskType::GetVaultSecret => Box::new(GetVaultSecretTask),
            TaskType::LaunchInflux => Box::new(LaunchInfluxTask),
            TaskType::LoginToVault => Box::new(LoginToVaultTask),
            TaskType::PortForward => Box::new(PortForwardTask),
            TaskType::RunPgcli => Box::new(RunPgcliTask),
            TaskType::SelectActuatorService => Box::new(SelectActuatorServiceTask),
            TaskType::SelectAwsProfile => Box::new(SelectAwsProfileTask),
            TaskType::SelectInfluxInstance => Box::new(SelectInfluxInstanceTask),
            TaskType::SelectKubeContext => Box::new(SelectKubeContextTask),
            TaskType::SelectRdsInstance => Box::new(SelectRdsInstanceTask),
            TaskType::SetLogLevel => Box::new(SetLogLevelTask),
        }
    }
}

#[derive(Debug)]
pub enum TaskResult {
    ActuatorService(ActuatorService),
    AwsProfile{ profile: AwsProfileInfo, updated: bool },
    AwsSecret(String),
    InfluxCommand,
    InfluxInstance(InfluxInstance),
    KubeContext{ context: KubeContextInfo, updated: bool },
    LogLevel,
    PgcliCommand(String),
    PortForward(PortForwardInfo),
    RdsInstance(RdsInstance),
    VaultSecret,
    VaultToken(String),
}

impl TaskResult {
    pub fn eval_string(&self) -> Option<String> {
        match self {
            TaskResult::AwsProfile{ profile: AwsProfileInfo { name, .. }, updated: true } => {
                Some(format!("export AWS_PROFILE={name}\n"))
            },
            TaskResult::KubeContext{ context: KubeContextInfo { kubeconfig, .. }, updated: true } => {
                let path = kubeconfig.to_string_lossy();
                Some(format!("export KUBECONFIG={path}\n"))
            },
            TaskResult::PgcliCommand(cmd) => {
                Some(format!("{cmd}\n"))
            },
            _ => None,
        }
    }
}

impl From<&TaskResult> for String {
    fn from(result: &TaskResult) -> Self {
        format!("{:?}", result)
    }
}

pub async fn sleep_indicator(seconds: u64, start_msg: &str, end_msg: &str) {
    let progress = progress_bar(seconds).with_spinner_template();
    progress.start(start_msg);

    let sleep_duration = tokio::time::Duration::from_secs(2);
    let steps = 100;
    let step_duration = sleep_duration / steps;

    for i in 0..=steps {
        progress.inc(1);
        if i < steps {
            tokio::time::sleep(step_duration).await;
        }
    }

    progress.stop(end_msg);
}
