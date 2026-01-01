use async_trait::async_trait;
use std::collections::HashMap;
use serde_json::Value;
use crate::{ArcCommand, Args, Goal, GoalStatus};
use crate::tasks::{Task, TaskResult, TaskType};
use crate::tasks::TaskType::GetAwsSecret;

#[derive(Debug)]
pub struct RunPgcliTask;

#[async_trait]
impl Task for RunPgcliTask {
    async fn execute(&self, args: &Option<Args>, state: &HashMap<Goal, TaskResult>) -> GoalStatus {
        // If an RDS instance has not yet been selected, we need to wait for that goal to complete
        let rds_selection_goal = Goal::from(TaskType::SelectRdsInstance);
        if !state.contains_key(&rds_selection_goal) {
            return GoalStatus::Needs(rds_selection_goal);
        }

        // Retrieve selected RDS instance from state
        let rds_selection_result = state.get(&rds_selection_goal)
            .expect("TaskResult for SelectRdsInstance not found");
        let rds_instance = match rds_selection_result {
            TaskResult::RdsInstance(value) => {
               &value.as_ref().expect("No RDS instance available")
            },
            _ => panic!("Expected TaskResult::RdsInstance"),
        };

        // If the password for this RDS instance has not yet been retrieved, we need to wait for that goal to complete
        let secret_goal = Goal::new(GetAwsSecret, Some(Args {
            command: ArcCommand::AwsSecret { name: Some(rds_instance.secret_id().to_string()) }
        }));
        if !state.contains_key(&secret_goal) {
            return GoalStatus::Needs(secret_goal);
        }

        // Retrieve secret value from state
        let secret_result = state.get(&secret_goal)
            .expect("TaskResult for GetAwsSecret not found");
        let secret_value = match secret_result {
            TaskResult::AwsSecret(value) => {
                &value.as_ref().expect("No AWS secret available")
            },
            _ => panic!("Expected TaskResult::AwsSecret"),
        };

        // Parse the secret value into  JSON
        let secret_json: Value = serde_json::from_str(secret_value)
            .expect("Failed to parse JSON");

        let cmd = format!(
            "export PGPASSWORD={}\npgcli -h {} -U {}",
            secret_json["password"].as_str().expect("Password field in AWS secret is missing"),
            rds_instance.host(),
            secret_json["username"].as_str().expect("Username field in AWS secret is missing"),
        );
        GoalStatus::Completed(TaskResult::PgcliCommand(cmd))
    }
}