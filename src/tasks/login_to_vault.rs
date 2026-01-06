use async_trait::async_trait;
use cliclack::{intro, outro};
use std::collections::HashMap;
use std::fs;
use url::Url;
use vaultrs::auth::oidc;
use vaultrs::token;

use crate::{Args, Goal, GoalStatus};
use crate::aws::vault;
use crate::aws::vault::VaultInstance;
use crate::tasks::{color_output, Task, TaskResult, TaskType};

#[derive(Debug)]
pub struct LoginToVaultTask;

#[async_trait]
impl Task for LoginToVaultTask {
    fn print_intro(&self) {
        let _ = intro("Login to Vault");
    }

    async fn execute(&self, _args: &Option<Args>, state: &HashMap<Goal, TaskResult>, is_terminal_goal: bool) -> GoalStatus {
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
        let vault_instance = &profile_info.account.vault_instance();

        // Check for existing local Vault token
        if let Some(token) = read_token_file() {
            // A local Vault token already exists, let's add it to a client to see if it is expired
            let client = vault::create_client(
                vault_instance.address(),
                None,
                Some(token.clone())
            );

            // We use the lookup_self endpoint to check token validity
            if let Ok(token_info) = token::lookup_self(&client).await {
                if token_info.ttl > 0 {
                    // Existing token is still valid, so let's use it
                    outro(color_output("Using existing Vault token", is_terminal_goal)).unwrap();
                    return GoalStatus::Completed(TaskResult::VaultToken(token));
                }
            }
        }

        // If we made it this far, then we need to re-login to Vault via OIDC
        let token = vault_login(&vault_instance).await.expect("Vault login failed");
        save_token_file(&token).expect("Failed to save token file");

        outro(color_output("Successfully logged into Vault", is_terminal_goal)).unwrap();
        GoalStatus::Completed(TaskResult::VaultToken(token))
    }
}

fn vault_token_path() -> Option<std::path::PathBuf> {
    //TODO .arc-cli path should be configurable
    let mut path = home::home_dir()?;
    path.push(".arc-cli");
    path.push("vault_token");
    Some(path)
}

fn read_token_file() -> Option<String> {
    let token_path = vault_token_path()?;
    match fs::read_to_string(token_path) {
        Ok(token) => Some(token.trim().to_string()),
        Err(_) => None,
    }
}

fn save_token_file(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut token_path = vault_token_path().ok_or("Unable to determine home directory")?;
    token_path.pop(); // Remove "vault_token" to get the directory
    fs::create_dir_all(&token_path)?;
    token_path.push("vault_token");

    fs::write(token_path, token)?;
    Ok(())
}

async fn vault_login(vault_instance: &VaultInstance) -> Result<String, Box<dyn std::error::Error>> {
    // Start a local HTTP server to listen for the OIDC callback
    let redirect_host = "localhost:8250";
    let redirect_uri = format!("http://{}/oidc/callback", redirect_host);
    let server = tiny_http::Server::http(redirect_host)
        .map_err(|e| e.to_string())?;

    // Retrieve the OIDC auth URL from Vault
    let client = vault::create_client(
        vault_instance.address(),
        vault_instance.oidc_namespace(),
        None
    );
    let auth_response = oidc::auth(
        &client,
        "oidc", // mount path
        &redirect_uri,
        vault_instance.oidc_role(),
    ).await?;

    // Extract nonce from auth URL
    let url = Url::parse(&auth_response.auth_url)?;
    let nonce = extract_query_param(&url, "nonce")?;

    // Open the user's default web browser to the auth URL
    let _ = webbrowser::open(&auth_response.auth_url);

    // Wait for the OIDC callback request
    let request = server.recv()?;

    // The request URL is relative, so we need to construct the absolute URL
    let base_url_str = format!("http://{}/", redirect_host);
    let base_url = Url::parse(&base_url_str)?;
    let absolute_request_url = base_url.join(request.url())?;

    // Extract code and state from the request URL
    let code = extract_query_param(&absolute_request_url, "code")?;
    let state = extract_query_param(&absolute_request_url, "state")?;

    // Respond to the user in the browser
    let response = tiny_http::Response::from_string(
        "Authentication successful! You can close this tab."
    );
    request.respond(response)?;

    // Complete the login with Vault using the captured parameters
    let token_auth = oidc::callback(
        &client,
        "oidc",
        state.as_str(),
        nonce.as_str(),
        code.as_str()
    ).await?;

    Ok(token_auth.client_token)
}

fn extract_query_param(url: &Url, key: &str) -> Result<String, Box<dyn std::error::Error>> {
    url.query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, value)| value.into_owned())
        .ok_or_else(|| format!("Query parameter not found: {}", key).into())
}
