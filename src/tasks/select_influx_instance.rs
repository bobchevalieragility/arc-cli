use cliclack::{intro, outro, select};
use async_trait::async_trait;
use std::collections::HashMap;
use crate::{Args, Goal, GoalStatus};
use crate::aws::influx::InfluxInstance;
use crate::tasks::{color_output, Task, TaskResult};

#[derive(Debug)]
pub struct SelectInfluxInstanceTask;

#[async_trait]
impl Task for SelectInfluxInstanceTask {
    async fn execute(&self, _args: &Option<Args>, state: &HashMap<Goal, TaskResult>, is_terminal_goal: bool) -> GoalStatus {
        intro("Influx instance selector").unwrap();

        // Prompt the user to select an Influx instance
        let influx_instance = prompt_for_influx_instance().await;

        outro(format!("RDS instance: {}", color_output(influx_instance.name(), is_terminal_goal))).unwrap();
        GoalStatus::Completed(TaskResult::InfluxInstance(influx_instance))
    }
}

async fn prompt_for_influx_instance() -> InfluxInstance {
    let available_influx_instances = InfluxInstance::all();

    let mut menu = select("Which InfluxDB instance would you like to connect to?");
    for influx in &available_influx_instances {
        menu = menu.item(influx.name(), influx.name(), "");
    }

    let influx_name = menu.interact().unwrap().to_string();
    InfluxInstance::from(influx_name.as_str())
}