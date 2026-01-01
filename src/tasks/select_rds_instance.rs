use cliclack::{intro, outro, select};
use async_trait::async_trait;
use std::collections::HashMap;
use crate::{aws_accounts, Args, Goal, GoalStatus};
use crate::rds::RdsInstance;
use crate::tasks::{Task, TaskResult, TaskType};
use aws_accounts::DEV_ACCT;
use aws_accounts::STAGE_ACCT;
use aws_accounts::PROD_ACCT;

#[derive(Debug)]
pub struct SelectRdsInstanceTask;

#[async_trait]
impl Task for SelectRdsInstanceTask {
    async fn execute(&self, args: &Option<Args>, state: &HashMap<Goal, TaskResult>) -> GoalStatus {
        // If AWS profile info is not available, we need to wait for that goal to complete
        let profile_goal = Goal::from(TaskType::SelectAwsProfile);
        if !state.contains_key(&profile_goal) {
            return GoalStatus::Needs(profile_goal);
        }

        intro("RDS instance selector").unwrap();

        // Retrieve the desired AWS account ID from state
        let aws_profile_result = state.get(&profile_goal)
            .expect("TaskResult for SelectAwsProfile not found");
        let profile_info = match aws_profile_result {
            TaskResult::AwsProfile { old, new } => {
                new.as_ref().or(old.as_ref())
                    .expect("No AWS profile available (both old and new are None)")
            },
            _ => panic!("Expected TaskResult::AwsProfile"),
        };

        // Prompt the user to select an RDS instance
        let rds_instance = prompt_for_rds_instance(&profile_info.account_id).await;

        outro(format!("RDS instance selected: {}", rds_instance.name())).unwrap();
        GoalStatus::Completed(TaskResult::RdsInstance(Some(rds_instance)))
    }
}

async fn prompt_for_rds_instance(account_id: &str) -> RdsInstance {
    let available_rds_instances = get_available_rds_names(account_id);

    let mut menu = select("Which RDS instance would you like to connect to?");
    for secret in &available_rds_instances {
        menu = menu.item(secret, secret, "");
    }

    let rds_name = menu.interact().unwrap().to_string();
    RdsInstance::from(rds_name.as_str())
}

fn get_available_rds_names(account_id: &str) -> Vec<&'static str> {
    // fn available_rds_names(account_id: &str) -> Vec<String> {
    match account_id {
        DEV_ACCT => vec![RdsInstance::WorkcellDev.name(), RdsInstance::EventLogDev.name()],
        STAGE_ACCT => vec![RdsInstance::WorkcellStage.name(), RdsInstance::EventLogStage.name()],
        PROD_ACCT => vec![RdsInstance::WorkcellProd.name(), RdsInstance::EventLogProd.name()],
        _ => panic!("Unknown AWS account ID: {account_id}"),
    }
}