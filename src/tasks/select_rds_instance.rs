use cliclack::{intro, select};
use async_trait::async_trait;
use std::collections::HashMap;
use crate::{Args, Goal, GoalStatus, OutroMessage};
use crate::aws::rds::RdsInstance;
use crate::tasks::{Task, TaskResult, TaskType};

#[derive(Debug)]
pub struct SelectRdsInstanceTask;

#[async_trait]
impl Task for SelectRdsInstanceTask {
    fn print_intro(&self) {
        let _ = intro("Select RDS Instance");
    }

    async fn execute(&self, _args: &Option<Args>, state: &HashMap<Goal, TaskResult>, _is_terminal_goal: bool) -> GoalStatus {
        // If AWS profile info is not available, we need to wait for that goal to complete
        let profile_goal = Goal::from(TaskType::SelectAwsProfile);
        if !state.contains_key(&profile_goal) {
            return GoalStatus::Needs(profile_goal);
        }

        // Retrieve the desired AWS account ID from state
        let aws_profile_result = state.get(&profile_goal)
            .expect("TaskResult for SelectAwsProfile not found");
        let profile_info = match aws_profile_result {
            TaskResult::AwsProfile { existing, updated } => {
                updated.as_ref().or(existing.as_ref())
                    .expect("No AWS profile available (both existing and updated are None)")
            },
            _ => panic!("Expected TaskResult::AwsProfile"),
        };

        // Get a list of all available RDS instances for this account
        let available_rds_instances = profile_info.account.rds_instances();

        // Prompt user to select RDS instance only if there are multiple options
        let (rds_instance, msg) = match available_rds_instances.len() {
            1 => (available_rds_instances[0], Some(format!("Inferred RDS instance: {}", available_rds_instances[0].name()))),
            _ => (prompt_for_rds_instance(available_rds_instances).await, None)
        };

        // If there's a message to display, wrap it in an OutroMessage
        let outro_msg = match msg {
            Some(m) => Some(OutroMessage::new(None, m)),
            None => None,
        };

        GoalStatus::Completed(TaskResult::RdsInstance(rds_instance), outro_msg)
    }
}

async fn prompt_for_rds_instance(available_rds_instances: Vec<RdsInstance>) -> RdsInstance {
    let mut menu = select("Which RDS instance would you like to connect to?");
    for rds in &available_rds_instances {
        menu = menu.item(rds.name(), rds.name(), "");
    }

    let rds_name = menu.interact().unwrap().to_string();
    RdsInstance::from(rds_name.as_str())
}