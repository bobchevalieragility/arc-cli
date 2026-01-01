use cliclack::{intro, outro, select};
use async_trait::async_trait;
use std::collections::HashMap;
use crate::{Args, Goal, GoalStatus};
use crate::aws_account::AwsAccount;
use crate::rds::RdsInstance;
use crate::tasks::{Task, TaskResult, TaskType};

#[derive(Debug)]
pub struct SelectRdsInstanceTask;

#[async_trait]
impl Task for SelectRdsInstanceTask {
    async fn execute(&self, _args: &Option<Args>, state: &HashMap<Goal, TaskResult>) -> GoalStatus {
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
        let rds_instance = prompt_for_rds_instance(&profile_info.account).await;

        outro(format!("RDS instance selected: {}", rds_instance.name())).unwrap();
        GoalStatus::Completed(TaskResult::RdsInstance(Some(rds_instance)))
    }
}

async fn prompt_for_rds_instance(account: &AwsAccount) -> RdsInstance {
    let available_rds_instances = get_available_rds_names(account);

    let mut menu = select("Which RDS instance would you like to connect to?");
    for secret in &available_rds_instances {
        menu = menu.item(secret, secret, "");
    }

    let rds_name = menu.interact().unwrap().to_string();
    RdsInstance::from(rds_name.as_str())
}

fn get_available_rds_names(account: &AwsAccount) -> Vec<&'static str> {
    match account {
        AwsAccount::Dev => vec![RdsInstance::WorkcellDev.name(), RdsInstance::EventLogDev.name()],
        AwsAccount::Stage => vec![RdsInstance::WorkcellStage.name(), RdsInstance::EventLogStage.name()],
        AwsAccount::Prod => vec![RdsInstance::WorkcellProd.name(), RdsInstance::EventLogProd.name()],
    }
}