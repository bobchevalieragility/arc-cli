use std::collections::HashMap;
use serde_json::Value;
use crate::aws::influx::InfluxInstance;
use crate::aws::rds::RdsInstance;
use crate::errors::ArcError;
use crate::tasks::TaskResult;
use crate::tasks::port_forward::PortForwardInfo;
use crate::tasks::select_actuator_service::ActuatorService;
use crate::tasks::select_aws_profile::AwsProfileInfo;
use crate::tasks::select_kube_context::KubeContextInfo;
use std;
use crate::goals::Goal;

pub struct State {
    results: HashMap<Goal, TaskResult>,
}

impl State {
    pub(crate) fn new() -> Self {
        State { results: HashMap::new() }
    }

    pub(crate) fn contains(&self, goal: &Goal) -> bool {
        self.results.contains_key(goal)
    }

    pub(crate) fn insert(&mut self, goal: Goal, result: TaskResult) {
        self.results.insert(goal, result);
    }

    fn get(&self, goal: &Goal) -> Result<&TaskResult, ArcError> {
        self.results.get(goal).ok_or_else(|| ArcError::insufficient_state(goal))
    }

    pub(crate) fn get_actuator_service(&self, goal: &Goal) -> Result<&ActuatorService, ArcError> {
        match self.get(goal)? {
            TaskResult::ActuatorService(x) => Ok(x),
            result => Err(ArcError::invalid_state(goal, "ActuatorService", result)),
        }
    }

    pub(crate) fn get_aws_profile_info(&self, goal: &Goal) -> Result<&AwsProfileInfo, ArcError> {
        match self.get(goal)? {
            TaskResult::AwsProfile { profile, .. } => Ok(profile),
            result => Err(ArcError::invalid_state(goal, "AwsProfile", result)),
        }
    }

    pub(crate) fn get_aws_secret(&self, goal: &Goal) -> Result<Value, ArcError> {
        match self.get(goal)? {
            TaskResult::AwsSecret(x) => {
                let secret_json: Value = serde_json::from_str(x)?;
                Ok(secret_json)
            },
            result => Err(ArcError::invalid_state(goal, "AwsSecret", result)),
        }
    }

    pub(crate) fn get_influx_instance(&self, goal: &Goal) -> Result<&InfluxInstance, ArcError> {
        match self.get(goal)? {
            TaskResult::InfluxInstance(x) => Ok(x),
            result => Err(ArcError::invalid_state(goal, "InfluxInstance", result)),
        }
    }

    pub(crate) fn get_kube_context_info(&self, goal: &Goal) -> Result<&KubeContextInfo, ArcError> {
        match self.get(goal)? {
            TaskResult::KubeContext { context, .. } => Ok(context),
            result => Err(ArcError::invalid_state(goal, "KubeContext", result)),
        }
    }

    pub(crate) fn get_port_forward_info(&self, goal: &Goal) -> Result<&PortForwardInfo, ArcError> {
        match self.get(goal)? {
            TaskResult::PortForward(info) => Ok(info),
            result => Err(ArcError::invalid_state(goal, "PortForward", result)),
        }
    }

    pub(crate) fn get_rds_instance(&self, goal: &Goal) -> Result<&RdsInstance, ArcError> {
        match self.get(goal)? {
            TaskResult::RdsInstance(x) => Ok(x),
            result => Err(ArcError::invalid_state(goal, "RdsInstance", result)),
        }
    }

    pub(crate) fn get_vault_token(&self, goal: &Goal) -> Result<String, ArcError> {
        match self.get(goal)? {
            TaskResult::VaultToken(x) => Ok(x.clone()),
            result => Err(ArcError::invalid_state(goal, "VaultToken", result)),
        }
    }
}