use async_trait::async_trait;
use cliclack::intro;
use crate::{ArcCommand, Args, Goal, GoalStatus, OutroText, State};
use crate::errors::ArcError;
use crate::tasks::{Task, TaskResult, TaskType};
use crate::tasks::TaskType::GetAwsSecret;

#[derive(Debug)]
pub struct LaunchInfluxTask;

#[async_trait]
impl Task for LaunchInfluxTask {
    fn print_intro(&self) {
        let _ = intro("Launch Influx UI");
    }

    async fn execute(&self, _args: &Option<Args>, state: &State) -> Result<GoalStatus, ArcError> {
        // If an Influx instance has not yet been selected, we need to wait for that goal to complete
        let influx_selection_goal = Goal::from(TaskType::SelectInfluxInstance);
        if !state.contains(&influx_selection_goal) {
            return Ok(GoalStatus::Needs(influx_selection_goal));
        }

        // Retrieve selected Influx instance from state
        let influx_instance = state.get_influx_instance(&influx_selection_goal)?;

        // If the password for this Influx instance has not yet been retrieved, we need to wait for that goal to complete
        let secret_goal = Goal::new(GetAwsSecret, Some(Args {
            command: ArcCommand::AwsSecret { name: Some(influx_instance.secret_id().to_string()) }
        }));
        if !state.contains(&secret_goal) {
            return Ok(GoalStatus::Needs(secret_goal));
        }

        // Retrieve secret value as JSON Value from state
        let secret_value = state.get_aws_secret(&secret_goal)?;

        // Set outro text content
        let username = secret_value["username"]
            .as_str()
            .ok_or_else(|| ArcError::InvalidSecret(
                "username field missing or not a string".to_string()
            ))?;
        let password = secret_value["password"]
            .as_str()
            .ok_or_else(|| ArcError::InvalidSecret(
                "password field missing or not a string".to_string()
            ))?;

        let outro_text = OutroText::multi(
            "Influx Credentials".to_string(),
            format!("username: {}\npassword: {}", username, password),
        );

        // Open the user's default web browser to the auth URL
        let _ = webbrowser::open(influx_instance.url());

        Ok(GoalStatus::Completed(TaskResult::InfluxCommand, outro_text))
    }
}