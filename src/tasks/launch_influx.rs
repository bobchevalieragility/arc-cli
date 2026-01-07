use async_trait::async_trait;
use cliclack::intro;
use serde_json::Value;
use std::collections::HashMap;
use crate::{ArcCommand, Args, Goal, GoalStatus, OutroText};
use crate::tasks::{Task, TaskResult, TaskType};
use crate::tasks::TaskType::GetAwsSecret;

#[derive(Debug)]
pub struct LaunchInfluxTask;

#[async_trait]
impl Task for LaunchInfluxTask {
    fn print_intro(&self) {
        let _ = intro("Launch Influx UI");
    }

    async fn execute(&self, _args: &Option<Args>, state: &HashMap<Goal, TaskResult>) -> GoalStatus {
        // If an Influx instance has not yet been selected, we need to wait for that goal to complete
        let influx_selection_goal = Goal::from(TaskType::SelectInfluxInstance);
        if !state.contains_key(&influx_selection_goal) {
            return GoalStatus::Needs(influx_selection_goal);
        }

        // Retrieve selected Influx instance from state
        let influx_selection_result = state.get(&influx_selection_goal)
            .expect("TaskResult for SelectInfluxInstance not found");
        let influx_instance = match influx_selection_result {
            TaskResult::InfluxInstance(value) => value,
            _ => panic!("Expected TaskResult::InfluxInstance"),
        };

        // If the password for this Influx instance has not yet been retrieved, we need to wait for that goal to complete
        let secret_goal = Goal::new(GetAwsSecret, Some(Args {
            command: ArcCommand::AwsSecret { name: Some(influx_instance.secret_id().to_string()) }
        }));
        if !state.contains_key(&secret_goal) {
            return GoalStatus::Needs(secret_goal);
        }

        // Retrieve secret value from state
        let secret_result = state.get(&secret_goal)
            .expect("TaskResult for GetAwsSecret not found");
        let secret_value = match secret_result {
            TaskResult::AwsSecret(value) => value,
            _ => panic!("Expected TaskResult::AwsSecret"),
        };

        // Parse the secret value into  JSON
        let secret_json: Value = serde_json::from_str(secret_value)
            .expect("Failed to parse JSON");

        // Set outro text content
        let username= secret_json["username"].as_str().unwrap();
        let password = secret_json["password"].as_str().unwrap();
        let outro_text = OutroText::multi(
            "Influx Credentials".to_string(),
            format!("username: {}\npassword: {}", username, password),
        );

        // Open the user's default web browser to the auth URL
        let _ = webbrowser::open(influx_instance.url());

        GoalStatus::Completed(TaskResult::InfluxCommand, outro_text)
    }
}