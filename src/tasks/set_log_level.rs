use async_trait::async_trait;
use cliclack::{intro, outro_note};
use std::collections::HashMap;
use clap::ValueEnum;
use crate::{ArcCommand, Args, Goal, GoalStatus};
use crate::tasks::{color_output, Task, TaskResult, TaskType};
use crate::tasks::port_forward::PortForwardInfo;
use crate::tasks::TaskType::PortForward;

#[derive(Debug)]
pub struct SetLogLevelTask;

#[async_trait]
impl Task for SetLogLevelTask {
    async fn execute(&self, args: &Option<Args>, state: &HashMap<Goal, TaskResult>, is_terminal_goal: bool) -> GoalStatus {
        // If a service has not yet been selected, we need to wait for that goal to complete
        let svc_selection_goal = Goal::from(TaskType::SelectActuatorService);
        if !state.contains_key(&svc_selection_goal) {
            return GoalStatus::Needs(svc_selection_goal);
        }

        // Retrieve selected service from state
        let svc_selection_result = state.get(&svc_selection_goal)
            .expect("TaskResult for SelectActuatorService not found");
        let service = match svc_selection_result {
            TaskResult::ActuatorService(value) => value,
            _ => panic!("Expected TaskResult::ActuatorService"),
        };

        // If a port-forwarding session doesn't exist, we need to wait for that goal to complete
        let port_fwd_goal = Goal::new(PortForward, Some(Args {
            command: ArcCommand::PortForward { service: Some(service.name().to_string()), port: None }
        }));
        if !state.contains_key(&port_fwd_goal) {
            return GoalStatus::Needs(port_fwd_goal);
        }

        // Retrieve port-forwarding info from state
        let port_fwd_result = state.get(&port_fwd_goal)
            .expect("TaskResult for PortForward not found");
        let port_fwd_info = match port_fwd_result {
            TaskResult::PortForward(info) => info,
            _ => panic!("Expected TaskResult::PortForward"),
        };

        intro("Log Level Selector").unwrap();

        // Extract parameters from args
        let (package, display_only) = match &args.as_ref().expect("Args is None").command {
            ArcCommand::LogLevel{ package, display_only, .. } => (package, display_only),
            _ => panic!("Expected ArcCommand::LogLevel"),
        };

        if *display_only {
            // We only want to display the current log level
            display_log_level(package, port_fwd_info, is_terminal_goal).await;
        } else {
            // We want to change the log level
            println!("Changing log level...");
            //TODO: Implement changing log level
        }

        GoalStatus::Completed(TaskResult::LogLevel)
    }
}

async fn display_log_level(package: &str, port_fwd_info: &PortForwardInfo, is_terminal_goal: bool) {
    // Make HTTP GET request to the actuator/loggers endpoint
    let url = format!(
        "http://localhost:{}/actuator/loggers/{}",
        port_fwd_info.local_port,
        package
    );

    let http_client = reqwest::Client::new();
    match http_client.get(&url).send().await {
        Ok(response) => {
            match response.text().await {
                Ok(body) => {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                        let prompt = format!("{} log level", package);
                        let _ = outro_note(
                            color_output(&prompt, is_terminal_goal),
                            &serde_json::to_string_pretty(&json).unwrap()
                        );
                    }
                },
                Err(e) => eprintln!("Failed to read response body: {}", e),
            }
        },
        Err(e) => eprintln!("HTTP request failed: {}", e),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, ValueEnum)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
    Inherit,
}