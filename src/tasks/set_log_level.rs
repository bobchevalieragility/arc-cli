use async_trait::async_trait;
use cliclack::{intro, select};
use std::collections::HashMap;
use clap::ValueEnum;
use serde_json::Value;
use crate::{ArcCommand, Args, Goal, GoalStatus, OutroMessage};
use crate::tasks::{Task, TaskResult, TaskType};
use crate::tasks::TaskType::PortForward;

#[derive(Debug)]
pub struct SetLogLevelTask;

#[async_trait]
impl Task for SetLogLevelTask {
    fn print_intro(&self) {
        let _ = intro("Log Level");
    }

    async fn execute(&self, args: &Option<Args>, state: &HashMap<Goal, TaskResult>) -> GoalStatus {
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
        let service = service.name().to_string();
        let port_fwd_goal = Goal::new(PortForward, Some(Args {
            command: ArcCommand::PortForward { service: Some(service), port: None, tear_down: true }
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

        // Extract parameters from args
        let (package, display_only) = match &args.as_ref().expect("Args is None").command {
            ArcCommand::LogLevel{ package, display_only, .. } => (package, display_only),
            _ => panic!("Expected ArcCommand::LogLevel"),
        };

        let outro_msg = if *display_only {
            // We only want to display the current log level
            display_log_level(package, port_fwd_info.local_port).await
        } else {
            // We want to change the log level
            let level = match &args.as_ref().expect("Args is None").command {
                ArcCommand::LogLevel{ level: Some(level), .. } => level.clone(),
                _ => prompt_for_log_level(),
            };

            set_log_level(package, port_fwd_info.local_port, &level).await
        };

        GoalStatus::Completed(TaskResult::LogLevel, outro_msg)
    }
}

async fn display_log_level(package: &str, local_port: u16) -> Option<OutroMessage> {
    // Make HTTP GET request to the actuator/loggers endpoint
    let url = format!("http://localhost:{}/actuator/loggers/{}", local_port, package);

    let http_client = reqwest::Client::new();
    match http_client.get(&url).send().await {
        Ok(response) => {
            match response.text().await {
                Ok(body) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&body) {
                        let msg = serde_json::to_string_pretty(&json).unwrap();
                        return Some(OutroMessage::new(Some(format!("{} log level", package)), msg))
                    }
                },
                Err(e) => eprintln!("Failed to read response body: {}", e),
            }
        },
        Err(e) => eprintln!("HTTP request failed: {}", e),
    }
    None
}

async fn set_log_level(package: &str, local_port: u16, level: &Level) -> Option<OutroMessage> {
    // Make HTTP POST request to the actuator/loggers endpoint
    let url = format!("http://localhost:{}/actuator/loggers/{}", local_port, package);

    // Create JSON body with configuredLevel
    let body = serde_json::json!({ "configuredLevel": level.value() });

    let http_client = reqwest::Client::new();
    match http_client.post(&url).json(&body).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let prompt = format!("{} log level", package);
                let msg = format!("Set to {}", level.name());
                return Some(OutroMessage::new(Some(prompt), msg))
            } else {
                eprintln!("Failed to set log level: HTTP {}", response.status());
            }
        },
        Err(e) => eprintln!("HTTP request failed: {}", e),
    }
    None
}

fn prompt_for_log_level() -> Level {
    let available_levels = Level::all();

    let mut menu = select("Select desired log level");
    for level in &available_levels {
        menu = menu.item(level.name(), level.name(), "");
    }

    let selected_level = menu.interact().unwrap();
    Level::from(selected_level)
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

impl Level {
    pub fn name(&self) -> &str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
            Level::Off => "OFF",
            Level::Inherit => "INHERIT",
        }
    }

    pub fn value(&self) -> Value {
        match self {
            Level::Trace => Value::String("trace".to_string()),
            Level::Debug => Value::String("debug".to_string()),
            Level::Info => Value::String("info".to_string()),
            Level::Warn => Value::String("warn".to_string()),
            Level::Error => Value::String("error".to_string()),
            Level::Off => Value::String("off".to_string()),
            Level::Inherit => Value::Null,
        }
    }

    fn all() -> Vec<Level> {
        vec![
            Level::Trace,
            Level::Debug,
            Level::Info,
            Level::Warn,
            Level::Error,
            Level::Off,
            Level::Inherit,
        ]
    }
}

impl From<&str> for Level {
    fn from(level_name: &str) -> Self {
        match level_name {
            "TRACE" => Level::Trace,
            "DEBUG" => Level::Debug,
            "INFO" => Level::Info,
            "WARN" => Level::Warn,
            "ERROR" => Level::Error,
            "OFF" => Level::Off,
            "INHERIT" => Level::Inherit,
            _ => panic!("Unknown log Level: {level_name}"),
        }
    }
}
