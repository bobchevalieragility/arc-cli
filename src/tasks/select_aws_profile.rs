use cliclack::{intro, outro, select};
use aws_runtime::env_config::file::EnvConfigFiles;
use aws_config::profile;
use aws_types::os_shim_internal::{Env, Fs};
use std::collections::HashSet;
use std::env;
use pollster::FutureExt as _;
use crate::ArcCommand;
use crate::tasks::{Executor, Task, State, TaskResult};

#[derive(Debug)]
pub struct SelectAwsProfileExecutor;

impl Executor for SelectAwsProfileExecutor {
    fn needs(&self) -> HashSet<Task> {
        HashSet::new()
    }

    fn execute(&self, state: &State) -> TaskResult{
        intro("AWS Profile Selector").unwrap();

        // If the AWS_PROFILE environment variable is already set, then we'll keep it,
        // unless the user specifically requested to switch it
        if let Ok(current_profile) = env::var("AWS_PROFILE") {
            match state.args.command {
                ArcCommand::Switch{ aws_profile: true, .. } |
                ArcCommand::Switch{ aws_profile: false, kube_context: false } => {
                    // All of these cases are interpreted as the user wanting to switch AWS profile
                },
                _ => {
                    // Remaining Switch case and all other commands result in keeping current profile
                    outro(format!("Using existing AWS profile: {}", current_profile)).unwrap();
                    return TaskResult::AwsProfile(None)
                }
            }
        }

        // Prompt user to select an AWS profile
        let selected_aws_profile = prompt_for_aws_profile();
        outro(format!("AWS profile will be set to: {}", selected_aws_profile)).unwrap();

        TaskResult::AwsProfile(Some(selected_aws_profile))
    }
}

fn prompt_for_aws_profile() -> String {
    let mut menu = select("Which AWS profile would you like to use?");

    let available_profiles = get_available_aws_profiles();
    for profile in &available_profiles {
        menu = menu.item(profile, profile, "");
    }

    menu.interact().unwrap().to_string()
}

fn get_available_aws_profiles() -> Vec<String> {
    // Use real filesystem and environment access
    let fs = Fs::real();
    let env = Env::real();

    // Load default profile files (~/.aws/config and ~/.aws/credentials)
    let config_files = EnvConfigFiles::default();

    // Block on the async call to load env config sections
    let config_sections = profile::load(&fs, &env, &config_files, None)
        .block_on()
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