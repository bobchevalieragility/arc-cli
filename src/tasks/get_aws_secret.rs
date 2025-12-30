use cliclack::{intro, outro, select};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use std::collections::HashSet;
use aws_sdk_secretsmanager::Client;
use aws_types::region::Region;
use crate::tasks::{Executor, Task, State, TaskResult};

#[derive(Debug)]
pub struct GetAwsSecretExecutor;

#[async_trait]
impl Executor for GetAwsSecretExecutor {
    fn needs(&self) -> HashSet<Task> {
        HashSet::from([Task::SelectAwsProfile])
    }

    async fn execute(&self, state: &State) -> TaskResult{
        intro("AWS Secret Selector").unwrap();

        // Get the desired profile name from the result of the SelectAwsProfile task
        let aws_profile_result = state.results.get(&Task::SelectAwsProfile)
            .expect("TaskResult for SelectAwsProfile not found");
        let profile_name = match aws_profile_result {
            TaskResult::AwsProfile { old, new } => {
                new.as_ref().or(old.as_ref())
                    .expect("No AWS profile available (both old and new are None)")
            },
            _ => panic!("Expected TaskResult::AwsProfile"),
        };

        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-west-2"))
            .profile_name(profile_name)
            .load()
            .await;
        let client = Client::new(&aws_config);

        // List secrets asynchronously
        let paginator = client.list_secrets().into_paginator();
        let pages: Vec<_> = paginator.send().collect::<Vec<_>>().await;

        // Process the results
        let mut all_secrets: Vec<String> = Vec::new();
        for page_result in pages {
            let page = page_result.unwrap();
            let secrets: Vec<String> = page.secret_list()
                .iter()
                .filter_map(|e| e.name.clone())
                .collect();
            all_secrets.extend(secrets);
        }
        print!("Available secrets: {:?}\n", all_secrets);

        // // If the AWS_PROFILE environment variable is already set, then we'll keep it,
        // // unless the user specifically requested to switch it
        // if let Ok(current_profile) = env::var("AWS_PROFILE") {
        //     match state.args.command {
        //         ArcCommand::Switch{ aws_profile: true, .. } |
        //         ArcCommand::Switch{ aws_profile: false, kube_context: false } => {
        //             // All of these cases are interpreted as the user wanting to switch AWS profile
        //         },
        //         _ => {
        //             // Remaining Switch case and all other commands result in keeping current profile
        //             outro(format!("Using existing AWS profile: {}", current_profile)).unwrap();
        //             return TaskResult::AwsProfile(None)
        //         }
        //     }
        // }
        //
        // // Prompt user to select an AWS profile
        // let selected_aws_profile = prompt_for_aws_profile();
        // outro(format!("AWS profile will be set to: {}", selected_aws_profile)).unwrap();
        //
        // TaskResult::AwsProfile(Some(selected_aws_profile))

        //TODO fix me
        TaskResult::AwsSecret(None)
    }
}

// fn prompt_for_aws_secret() -> String {
//     let mut menu = select("Which AWS profile would you like to use?");
//
//     let available_profiles = get_available_aws_profiles();
//     for profile in &available_profiles {
//         menu = menu.item(profile, profile, "");
//     }
//
//     menu.interact().unwrap().to_string()
// }

// fn get_available_aws_profiles() -> Vec<String> {
//     // Use real filesystem and environment access
//     let fs = Fs::real();
//     let env = Env::real();
//
//     // Load default profile files (~/.aws/config and ~/.aws/credentials)
//     let config_files = EnvConfigFiles::default();
//
//     // Block on the async call to load env config sections
//     let config_sections = profile::load(&fs, &env, &config_files, None)
//         .block_on()
//         .unwrap();
//
//     // Extract profile names
//     let mut profile_names: Vec<String> = config_sections
//         .profiles()
//         .map(|s| s.to_string())
//         .collect();
//
//     if profile_names.is_empty() {
//         panic!("No AWS profiles found");
//     }
//
//     profile_names.sort();
//     profile_names
// }