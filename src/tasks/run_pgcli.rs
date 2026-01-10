use async_trait::async_trait;
use cliclack::intro;
use crate::{ArcCommand, Args, Goal, GoalStatus, OutroText, State};
use crate::errors::ArcError;
use crate::tasks::{Task, TaskResult, TaskType};
use crate::tasks::TaskType::GetAwsSecret;

#[derive(Debug)]
pub struct RunPgcliTask;

#[async_trait]
impl Task for RunPgcliTask {
    fn print_intro(&self) {
        let _ = intro("Run pgcli");
    }

    async fn execute(&self, _args: &Option<Args>, state: &State) -> Result<GoalStatus, ArcError> {
        // If an RDS instance has not yet been selected, we need to wait for that goal to complete
        let rds_selection_goal = Goal::from(TaskType::SelectRdsInstance);
        if !state.contains(&rds_selection_goal) {
            return Ok(GoalStatus::Needs(rds_selection_goal));
        }

        // Retrieve selected RDS instance from state
        let rds_instance = state.get_rds_instance(&rds_selection_goal)?;

        // If the password for this RDS instance has not yet been retrieved, we need to wait for that goal to complete
        let secret_goal = Goal::new(GetAwsSecret, Some(Args {
            command: ArcCommand::AwsSecret { name: Some(rds_instance.secret_id().to_string()) }
        }));
        if !state.contains(&secret_goal) {
            return Ok(GoalStatus::Needs(secret_goal));
        }

        // Retrieve secret value as JSON from state
        let secret_value = state.get_aws_secret(&secret_goal)?;

        let username = secret_value["username"]
            .as_str()
            .ok_or_else(|| ArcError::InvalidSecret(
                "username field missing or not a string".to_string()
            ))?;

        let cmd = format!(
            "export PGPASSWORD={}\npgcli -h {} -U {}",
            secret_value["password"], // Don't unwrap to string because we want to retain the quotes
            rds_instance.host(),
            username,
        );

        let outro_text = OutroText::single("Launching pgcli".to_string(), String::new());
        Ok(GoalStatus::Completed(TaskResult::PgcliCommand(cmd), outro_text))
    }
}