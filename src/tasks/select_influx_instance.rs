use cliclack::{intro, select};
use async_trait::async_trait;
use std::collections::HashMap;
use crate::{Args, Goal, GoalStatus, OutroMessage};
use crate::aws::influx::InfluxInstance;
use crate::tasks::{Task, TaskResult, TaskType};

#[derive(Debug)]
pub struct SelectInfluxInstanceTask;

#[async_trait]
impl Task for SelectInfluxInstanceTask {
    fn print_intro(&self) {
        let _ = intro("Select InfluxDB Instance");
    }

    async fn execute(&self, _args: &Option<Args>, state: &HashMap<Goal, TaskResult>) -> GoalStatus {
        // If AWS profile info is not available, we need to wait for that goal to complete
        let profile_goal = Goal::from(TaskType::SelectAwsProfile);
        if !state.contains_key(&profile_goal) {
            return GoalStatus::Needs(profile_goal);
        }

        // Retrieve info about the desired AWS profile from state
        let aws_profile_result = state.get(&profile_goal)
            .expect("TaskResult for SelectAwsProfile not found");
        let profile_info = match aws_profile_result {
            TaskResult::AwsProfile { existing, updated } => {
                updated.as_ref().or(existing.as_ref())
                    .expect("No AWS profile available (both existing and updated are None)")
            },
            _ => panic!("Expected TaskResult::AwsProfile"),
        };

        // Get a list of all available Influx instances for this account
        let available_influx_instances = profile_info.account.influx_instances();

        // Prompt user to select an Influx instance only if there are multiple options
        let (influx_instance, msg) = match available_influx_instances.len() {
            1 => (available_influx_instances[0], Some(format!("Inferred Influx instance: {}", available_influx_instances[0].name()))),
            _ => (prompt_for_influx_instance(available_influx_instances).await, None)
        };

        // If there's a message to display, wrap it in an OutroMessage
        let outro_msg = match msg {
            Some(m) => Some(OutroMessage::new(None, m)),
            None => None,
        };

        GoalStatus::Completed(TaskResult::InfluxInstance(influx_instance), outro_msg)
    }
}

async fn prompt_for_influx_instance(available_influx_instances: Vec<InfluxInstance>) -> InfluxInstance {
    let mut menu = select("Which InfluxDB instance would you like to connect to?");
    for influx in &available_influx_instances {
        menu = menu.item(influx.name(), influx.name(), "");
    }

    let influx_name = menu.interact().unwrap().to_string();
    InfluxInstance::from(influx_name.as_str())
}