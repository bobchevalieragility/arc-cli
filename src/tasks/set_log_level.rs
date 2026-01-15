use async_trait::async_trait;
use cliclack::{intro, select};
use clap::ValueEnum;
use serde_json::Value;
use crate::args::{CliCommand, CliArgs};
use crate::errors::ArcError;
use crate::goals::{Goal, GoalType};
use crate::{GoalStatus, OutroText};
use crate::state::State;
use crate::tasks::{Task, TaskResult};

#[derive(Debug)]
pub struct SetLogLevelTask;

#[async_trait]
impl Task for SetLogLevelTask {
    fn print_intro(&self) -> Result<(), ArcError> {
        intro("Log Level")?;
        Ok(())
    }

    async fn execute(&self, args: &Option<CliArgs>, state: &State) -> Result<GoalStatus, ArcError> {
        // Validate that args are present
        let args = args.as_ref()
            .ok_or_else(|| ArcError::invalid_arc_command("LogLevel", "None"))?;

        // Ensure that SSO token has not expired
        let sso_goal = GoalType::SsoTokenValid.into();
        if !state.contains(&sso_goal) {
            return Ok(GoalStatus::Needs(sso_goal));
        }

        // Extract the optional service name from args
        let service_arg = match &args.command {
            CliCommand::LogLevel{ service, .. } => service,
            _ => return Err(ArcError::invalid_arc_command("LogLevel", format!("{:?}", args.command))),
        };

        let svc_selection_goal = GoalType::ActuatorServiceSelected.into();
        if let None = service_arg && !state.contains(&svc_selection_goal) {
            // Since service name not provided in args, we need to wait for service selection goal
            return Ok(GoalStatus::Needs(svc_selection_goal));
        }

        // Identify service name either from args or the service selection task result
        let service = match service_arg {
            Some(x) => x.to_string(),
            None => state.get_actuator_service(&svc_selection_goal)?.name().to_string()
        };

        // If a port-forwarding session doesn't exist, we need to wait for that goal to complete
        let port_fwd_goal = Goal::new(GoalType::PortForwardEstablished, Some(CliArgs {
            command: CliCommand::PortForward { service: Some(service), port: None, tear_down: true }
        }));
        if !state.contains(&port_fwd_goal) {
            return Ok(GoalStatus::Needs(port_fwd_goal));
        }

        // Retrieve port-forwarding info from state
        let port_fwd_info = state.get_port_forward_info(&port_fwd_goal)?;

        // Extract parameters from args
        let (package, display_only) = match &args.command {
            CliCommand::LogLevel{ package, display_only, .. } => (package, display_only),
            _ => return Err(ArcError::invalid_arc_command("LogLevel", format!("{:?}", args.command))),
        };

        let outro_text = if *display_only {
            // We only want to display the current log level
            display_log_level(package, port_fwd_info.local_port).await
        } else {
            // We want to change the log level
            let level = match &args.command {
                CliCommand::LogLevel{ level: Some(level), .. } => level.clone(),
                CliCommand::LogLevel{ level: None, .. } => prompt_for_log_level()?,
                _ => return Err(ArcError::invalid_arc_command("LogLevel", format!("{:?}", args.command))),
            };

            set_log_level(package, port_fwd_info.local_port, &level).await
        };

        Ok(GoalStatus::Completed(TaskResult::LogLevel, outro_text))
    }
}

async fn display_log_level(package: &str, local_port: u16) -> OutroText {
    // Make HTTP GET request to the actuator/loggers endpoint
    let url = format!("http://localhost:{}/actuator/loggers/{}", local_port, package);

    let http_client = reqwest::Client::new();
    match http_client.get(&url).send().await {
        Ok(response) => {
            match response.text().await {
                Ok(body) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&body) {
                        let msg = serde_json::to_string_pretty(&json).unwrap();
                        return OutroText::multi(format!("{} log level", package), msg)
                    }
                },
                Err(e) => eprintln!("Failed to read response body: {}", e),
            }
        },
        Err(e) => eprintln!("HTTP request failed: {}", e),
    }
    OutroText::None
}

async fn set_log_level(package: &str, local_port: u16, level: &Level) -> OutroText {
    // Make HTTP POST request to the actuator/loggers endpoint
    let url = format!("http://localhost:{}/actuator/loggers/{}", local_port, package);

    // Create JSON body with configuredLevel
    let body = serde_json::json!({ "configuredLevel": level.value() });

    let http_client = reqwest::Client::new();
    match http_client.post(&url).json(&body).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let key = format!("{} log level", package);
                let value = format!("Set to {}", level.name());
                return OutroText::multi(key, value)
            } else {
                eprintln!("Failed to set log level: HTTP {}", response.status());
            }
        },
        Err(e) => eprintln!("HTTP request failed: {}", e),
    }
    OutroText::None
}

fn prompt_for_log_level() -> Result<Level, ArcError> {
    let available_levels = Level::all();

    let mut menu = select("Select desired log level");
    for level in &available_levels {
        menu = menu.item(level.name(), level.name(), "");
    }

    let selected_level = menu.interact()?;
    Ok(Level::from(selected_level))
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
