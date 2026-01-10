use cliclack::{intro, select};
use async_trait::async_trait;
use crate::{Args, GoalStatus, OutroText, State};
use crate::errors::ArcError;
use crate::tasks::{Task, TaskResult};

#[derive(Debug)]
pub struct SelectActuatorServiceTask;

#[async_trait]
impl Task for SelectActuatorServiceTask {
    fn print_intro(&self) {
        let _ = intro("Select Actuator Service");
    }

    async fn execute(&self, _args: &Option<Args>, _state: &State) -> Result<GoalStatus, ArcError> {
        let services = ActuatorService::all();

        // Prompt user to select a service that supports actuator functionality
        let mut menu = select("Select a service");
        for svc in &services {
            let name = svc.name();
            menu = menu.item(name, name, "");
        }

        // Convert selected service name to an ActuatorService
        let svc_name = menu.interact()?;
        let service = ActuatorService::from(svc_name);

        Ok(GoalStatus::Completed(TaskResult::ActuatorService(service), OutroText::None))
    }
}

#[derive(Debug)]
pub enum ActuatorService {
    Metrics,
}

impl ActuatorService {
    pub fn name(&self) -> &str {
        match self {
            ActuatorService::Metrics => "metrics",
        }
    }

    fn all() -> Vec<ActuatorService> {
        vec![
            ActuatorService::Metrics,
        ]
    }
}

impl From<&str> for ActuatorService {
    fn from(svc_name: &str) -> Self {
        match svc_name {
            "metrics" => ActuatorService::Metrics,
            _ => panic!("Unknown service name: {svc_name}"),
        }
    }
}