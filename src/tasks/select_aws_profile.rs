use cliclack::{intro, outro, select};
use async_trait::async_trait;
use aws_runtime::env_config::file::EnvConfigFiles;
use aws_config::profile;
use aws_types::os_shim_internal::{Env, Fs};
use std::collections::HashMap;
use std::env;
use crate::{ArcCommand, Args, Goal, GoalStatus};
use crate::tasks::{Executor, TaskResult};

#[derive(Debug)]
pub struct SelectAwsProfileExecutor;

#[async_trait]
impl Executor for SelectAwsProfileExecutor {
    async fn execute(&self, args: &Args, state: &HashMap<Goal, TaskResult>) -> GoalStatus {
        intro("AWS Profile Selector").unwrap();

        // If the AWS_PROFILE environment variable is already set, then we'll keep it,
        // unless the user specifically requested to switch it
        if let Ok(current_profile) = env::var("AWS_PROFILE") {
            match args.command {
                ArcCommand::Switch{ aws_profile: true, .. } |
                ArcCommand::Switch{ aws_profile: false, kube_context: false } => {
                    // All of these cases are interpreted as the user wanting to switch AWS profile
                },
                _ => {
                    // Remaining Switch case and all other commands result in keeping current profile
                    outro(format!("Using existing AWS profile: {}", current_profile)).unwrap();
                    let task_result = TaskResult::AwsProfile{ old: Some(current_profile), new: None };
                    return GoalStatus::Completed(task_result);
                }
            }
        }

        // Prompt user to select an AWS profile
        let selected_aws_profile = prompt_for_aws_profile().await;
        outro(format!("AWS profile will be set to: {}", selected_aws_profile)).unwrap();

        let task_result = TaskResult::AwsProfile{ old: None, new: Some(selected_aws_profile) };
        GoalStatus::Completed(task_result)
    }
}

async fn prompt_for_aws_profile() -> String {
    let available_profiles = get_available_aws_profiles().await;

    let mut menu = select("Which AWS profile would you like to use?");
    for profile in &available_profiles {
        menu = menu.item(profile, profile, "");
    }

    menu.interact().unwrap().to_string()
}

async fn get_available_aws_profiles() -> Vec<String> {
    // Use real filesystem and environment access
    let fs = Fs::real();
    let env = Env::real();

    // Load default profile files (~/.aws/config and ~/.aws/credentials)
    let config_files = EnvConfigFiles::default();

    // Load env config sections asynchronously
    let config_sections = profile::load(&fs, &env, &config_files, None)
        .await
        .unwrap();

    // Extract profile names
    let mut profile_names: Vec<String> = config_sections
        .profiles()
        .map(|s| s.to_string())
        .collect();

    if profile_names.is_empty() {
        panic!("No AWS profiles found");
    }

    profile_names.sort();
    profile_names
}