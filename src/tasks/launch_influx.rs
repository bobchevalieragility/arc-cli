use async_trait::async_trait;
use cliclack::intro;
use crate::errors::ArcError;
use crate::goals::{Goal, GoalParams, GoalType};
use crate::{GoalStatus, OutroText};
use crate::state::State;
use crate::tasks::{Task, TaskResult};

#[derive(Debug)]
pub struct LaunchInfluxTask;

#[async_trait]
impl Task for LaunchInfluxTask {
    fn print_intro(&self) -> Result<(), ArcError> {
        intro("Launch Influx UI")?;
        Ok(())
    }

    async fn execute(&self, _params: &GoalParams, state: &State) -> Result<GoalStatus, ArcError> {
        // Ensure that SSO token has not expired
        let sso_goal = GoalType::SsoTokenValid.into();
        if !state.contains(&sso_goal) {
            return Ok(GoalStatus::Needs(sso_goal));
        }

        // If an Influx instance has not yet been selected, we need to wait for that goal to complete
        let influx_selection_goal = GoalType::InfluxInstanceSelected.into();
        if !state.contains(&influx_selection_goal) {
            return Ok(GoalStatus::Needs(influx_selection_goal));
        }

        // Retrieve selected Influx instance from state
        let influx_instance = state.get_influx_instance(&influx_selection_goal)?;

        // If the password for this Influx instance has not yet been retrieved, we need to wait for that goal to complete
        let influx_secret_name = influx_instance.secret_id().to_string();
        let secret_goal = Goal::new(
            GoalType::AwsSecretKnown,
            GoalParams::AwsSecretKnown { name: Some(influx_secret_name) }
        );
        if !state.contains(&secret_goal) {
            return Ok(GoalStatus::Needs(secret_goal));
        }

        // Retrieve secret value as JSON Value from state
        let secret_value = state.get_aws_secret(&secret_goal)?;

        // Set outro text content
        let username = secret_value["username"]
            .as_str()
            .ok_or_else(|| ArcError::invalid_secret("username"))?;
        let password = secret_value["password"]
            .as_str()
            .ok_or_else(|| ArcError::invalid_secret("password"))?;

        let outro_text = OutroText::multi(
            "Influx Credentials".to_string(),
            format!("username: {}\npassword: {}", username, password),
        );

        // Open the user's default web browser to the auth URL
        webbrowser::open(influx_instance.url())?;

        Ok(GoalStatus::Completed(TaskResult::InfluxCommand, outro_text))
    }
}