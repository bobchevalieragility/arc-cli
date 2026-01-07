use async_trait::async_trait;
use cliclack::{intro, select};
use std::collections::HashMap;
use vaultrs::client::VaultClient;
use vaultrs::kv2;

use crate::{ArcCommand, Args, Goal, GoalStatus, OutroMessage};
use crate::tasks::{Task, TaskResult, TaskType};
use crate::aws::vault;

#[derive(Debug)]
pub struct GetVaultSecretTask;

#[async_trait]
impl Task for GetVaultSecretTask {
    fn print_intro(&self) {
        let _ = intro("Get Vault Secret");
    }

    async fn execute(&self, args: &Option<Args>, state: &HashMap<Goal, TaskResult>) -> GoalStatus {
        // If AWS profile info is not available, we need to wait for that goal to complete
        let profile_goal = Goal::from(TaskType::SelectAwsProfile);
        if !state.contains_key(&profile_goal) {
            return GoalStatus::Needs(profile_goal);
        }

        // If we haven't obtained a valid Vault token yet, we need to wait for that goal to complete
        //TODO create a "wait_for_goal(TaskType) -> Goal" function in tasks.rs?
        // (that might not work for complex goals with args)
        let login_goal = Goal::from(TaskType::LoginToVault);
        if !state.contains_key(&login_goal) {
            return GoalStatus::Needs(login_goal);
        }

        // Retrieve info about the desired AWS profile from state
        let aws_profile_result = state.get(&profile_goal)
            .expect("TaskResult for SelectAwsProfile not found");
        // TODO Add a TaskResult::extract_value<T>() -> T method?
        let profile_info = match aws_profile_result {
            TaskResult::AwsProfile { existing, updated } => {
                updated.as_ref().or(existing.as_ref())
                    .expect("No AWS profile available (both existing and updated are None)")
            },
            _ => panic!("Expected TaskResult::AwsProfile"),
        };
        let aws_account = &profile_info.account;
        let vault_instance = aws_account.vault_instance();

        // Retrieve validated Vault token from state
        let login_result = state.get(&login_goal)
            .expect("TaskResult for LoginToVault not found");
        let token = match login_result {
            TaskResult::VaultToken(value) => value,
            _ => panic!("Expected TaskResult::VaultToken"),
        };

        // Create Vault client using the token
        let client = vault::create_client(
            vault_instance.address(),
            vault_instance.secrets_namespace(aws_account),
            Some(token.to_string())
        );

        let args = args.as_ref().expect("Args is None");

        // Determine which secret to retrieve, prompting user if necessary
        let secret_path = match &args.command {
            ArcCommand::Vault{ path: Some(p), .. } => p.clone(),
            _ => prompt_for_secret_path(&client).await.expect("Failed to select secret path"),
        };

        // Retrieve the secret key-value pairs from Vault
        let secrets: HashMap<String, String> = kv2::read(&client, "kv-v2", &secret_path)
            .await.expect("Unable to read Vault secret");

        // Optionally extract a specific field from the secret and format for display
        let (_, outro_msg) = match &args.command {
            ArcCommand::Vault{ field: Some(f), .. } => {
                // Extract specific field
                let secret_field = match secrets.get(f) {
                    Some(value) => value.to_string(),
                    None => {
                        panic!("Field '{}' not found in secret at path '{}'", f, secret_path);
                    }
                };
                let outro_msg = OutroMessage::new(Some(f.clone()), secret_field.clone());
                (secret_field, outro_msg)
            },
            _ => {
                // Concatenate k: v pairs into a single, newline-delimited string
                let full_secret = secrets.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<String>>()
                    .join("\n");
                let prompt = "Secret Value".to_string();
                let outro_msg = OutroMessage::new(Some(prompt), full_secret.clone());
                (full_secret, outro_msg)
            },
        };

        GoalStatus::Completed(TaskResult::VaultSecret, Some(outro_msg))
    }
}

async fn prompt_for_secret_path(client: &VaultClient) -> Result<String, Box<dyn std::error::Error>> {
    let mut current_path = String::new();

    while current_path.is_empty() || current_path.ends_with('/') {
        let items = kv2::list(client, "kv-v2", &current_path).await?;

        // Collect all available sub-paths
        let available_paths: Vec<String> = items
            .iter()
            .map(|i|  format!("{}{}", current_path, i))
            .collect();

        // Prompt user to select a path
        let mut menu = select("Select a secret path");
        for path in &available_paths {
            menu = menu.item(path, path, "");
        }
        current_path = menu.interact()?.to_string();
    }

    Ok(current_path.to_string())
}