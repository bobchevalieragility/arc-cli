use cliclack::{intro, outro, select};
use async_trait::async_trait;
use std::collections::HashMap;
use crate::{Args, Goal, GoalStatus};
use crate::tasks::{color_output, Task, TaskResult};

#[derive(Debug)]
pub struct SelectActuatorServiceTask;

#[async_trait]
impl Task for SelectActuatorServiceTask {
    async fn execute(&self, _args: &Option<Args>, state: &HashMap<Goal, TaskResult>, is_terminal_goal: bool) -> GoalStatus {
        intro("Service selector").unwrap();

        // Prompt user to select a service that supports actuator functionality
        let service = prompt_for_service();

        outro(format!("Service: {}", color_output(service.name(), is_terminal_goal))).unwrap();
        GoalStatus::Completed(TaskResult::ActuatorService(service))
    }
}

fn prompt_for_service() -> ActuatorService {
    let services = ActuatorService::all();

    let mut menu = select("Select a service");
    for svc in &services {
        let name = svc.name();
        menu = menu.item(name, name, "");
    }

    let svc_name = menu.interact().unwrap();
    ActuatorService::from(svc_name)
}

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