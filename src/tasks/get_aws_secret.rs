use cliclack::{intro, outro, select};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use std::collections::HashMap;
use aws_sdk_secretsmanager::Client;
use aws_types::region::Region;
use crate::{ArcCommand, Args, Goal, GoalStatus};
use crate::tasks::{Task, TaskResult};
use crate::tasks::TaskType::SelectAwsProfile;

#[derive(Debug)]
pub struct GetAwsSecretTask;

#[async_trait]
impl Task for GetAwsSecretTask {
    async fn execute(&self, args: &Args, state: &HashMap<Goal, TaskResult>) -> GoalStatus {
        // If AWS profile info is not available, we need to wait for that goal to complete
        let profile_goal = aws_profile_goal();
        if !state.contains_key(&profile_goal) {
            return GoalStatus::Needs(profile_goal);
        }

        intro("AWS Secret Retriever").unwrap();

        // Retrieve the desired AWS profile name from state
        let aws_profile_result = state.get(&profile_goal)
            .expect("TaskResult for SelectAwsProfile not found");
        let profile_name = match aws_profile_result {
            TaskResult::AwsProfile { old, new } => {
                new.as_ref().or(old.as_ref())
                    .expect("No AWS profile available (both old and new are None)")
            },
            _ => panic!("Expected TaskResult::AwsProfile"),
        };

        // Create AWS Secrets Manager client with the selected profile
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-west-2"))
            .profile_name(profile_name)
            .load()
            .await;
        let client = Client::new(&aws_config);

        // Determine which secret to retrieve, prompting user if necessary
        let secret_name = match &args.command {
            ArcCommand::AwsSecret{ name: Some(x) } => x.clone(),
            _ => prompt_for_aws_secret(&client).await,
        };

        // Retrieve the secret value
        let resp = client.get_secret_value()
            .secret_id(secret_name)
            .send()
            .await;
        let secret_value = resp.expect("Failed to get secret value")
            .secret_string.expect("Secret may be binary or not found");

        outro(format!("Secret value: {}", secret_value)).unwrap();
        GoalStatus::Completed(TaskResult::AwsSecret(Some(secret_value)))
    }
}

fn aws_profile_goal() -> Goal {
    Goal::new(
        SelectAwsProfile,
        Args {
            command: ArcCommand::Switch {
                aws_profile: true,
                kube_context: false,
                use_current: true,
            }
        },
    )
}

async fn prompt_for_aws_secret(client: &Client) -> String {
    let available_secrets = get_available_secrets(client).await;

    let mut menu = select("Which secret would you like to retrieve?");
    for secret in &available_secrets {
        menu = menu.item(secret, secret, "");
    }

    menu.interact().unwrap().to_string()
}

async fn get_available_secrets(client: &Client) -> Vec<String> {
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

    if all_secrets.is_empty() {
        panic!("No AWS secrets found");
    }

    all_secrets.sort();
    all_secrets
}