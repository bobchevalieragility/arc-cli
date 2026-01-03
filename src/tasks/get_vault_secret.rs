use cliclack::{intro, select};
use async_trait::async_trait;
use std::collections::HashMap;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use crate::{Args, Goal, GoalStatus};
use crate::tasks::{Task, TaskResult, TaskType};
use vaultrs::kv2;
use crate::aws::vault;

#[derive(Debug)]
pub struct GetVaultSecretTask;

#[async_trait]
impl Task for GetVaultSecretTask {
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

        intro("Vault Secret Retriever").unwrap();

        // Retrieve info about the desired AWS profile from state
        let aws_profile_result = state.get(&profile_goal)
            .expect("TaskResult for SelectAwsProfile not found");
        let profile_info = match aws_profile_result {
            TaskResult::AwsProfile { old, new } => {
                new.as_ref().or(old.as_ref())
                    .expect("No AWS profile available (both old and new are None)")
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
        // let settings = VaultClientSettingsBuilder::default()
        //     .address(vault_instance.address())
        //     .namespace(vault_instance.secrets_namespace(aws_account))
        //     .token(token)
        //     .build()
        //     .expect("Unable to build VaultClient");
        // let client = VaultClient::new(settings).expect("Vault Client creation failed");
        let client = vault::create_client(
            vault_instance.address(),
            vault_instance.secrets_namespace(aws_account),
            Some(token.to_string())
        );

        // Login to Vault using OIDC
        //TODO once we change signature to return Result, propagate error instead of unwrapping it
        // let token = vault_login(&mut client).await.expect("Vault login failed");
        // client.set_token(&token);
        // client.settings.namespace = Some("admin/dev".to_string());

        // let list_response = kv2::list(&client, "kv-v2", "mp/metrics")
        //     .await.expect("Unable to list Vault secrets"); // returns []
        // let list_response = kv2::list(&client, "kv-v2", "/v1/admin/dev/kv-v2/data/mp/metrics")
        //     .await.expect("Unable to list Vault secrets"); // bad
        let list_response = kv2::list(&client, "kv-v2", "mp")
            .await.expect("Unable to list Vault secrets"); // returns ["metrics", "artifactory-credentials", ...]
        println!("{:?}", list_response);

        // let secret: HashMap<String, serde_json::Value> = kv2::read(&client, "kv-v2", "mp/metrics")
        //     .await.expect("Unable to read Vault secret");
        // println!("secret: {:?}", secret);


        // outro(format!("Secret value: {}", secret_value)).unwrap();
        // GoalStatus::Completed(TaskResult::VaultSecret(Some(secret_value)))
        GoalStatus::Completed(TaskResult::VaultSecret("foo".to_string()))
    }
}

async fn prompt_for_vault_secret(client: &VaultClient) -> String {
    let available_secrets = get_available_secrets(client).await;

    let mut menu = select("Which secret would you like to retrieve?");
    for secret in &available_secrets {
        menu = menu.item(secret, secret, "");
    }

    menu.interact().unwrap().to_string()
}

async fn get_available_secrets(client: &VaultClient) -> Vec<String> {
    // // List secrets asynchronously
    // let paginator = client.list_secrets().into_paginator();
    // let pages: Vec<_> = paginator.send().collect::<Vec<_>>().await;

    // Process the results
    let mut all_secrets: Vec<String> = Vec::new();
    // for page_result in pages {
    //     let page = page_result.unwrap();
    //     let secrets: Vec<String> = page.secret_list()
    //         .iter()
    //         .filter_map(|e| e.name.clone())
    //         .collect();
    //     all_secrets.extend(secrets);
    // }
    //
    // if all_secrets.is_empty() {
    //     panic!("No AWS secrets found");
    // }

    all_secrets.sort();
    all_secrets
}

// fn lookup_address(account: &AwsAccount) -> &'static str {
// fn get_vault_address(account: &AwsAccount) -> &str {
//     match account {
//         AwsAccount::Dev => "https://nonprod-public-vault-b4ed83ad.91d9045d.z1.hashicorp.cloud:8200",
//         AwsAccount::Stage => "https://nonprod-public-vault-b4ed83ad.91d9045d.z1.hashicorp.cloud:8200",
//         AwsAccount::Prod => "https://prod-public-vault-752e7a3c.c39279c9.z1.hashicorp.cloud:8200",
//     }
// }

// async fn vault_login(client: &mut VaultClient) -> Result<String, Box<dyn std::error::Error>> {
//     let redirect_host = "localhost:8250";
//     let redirect_uri = format!("http://{}/oidc/callback", redirect_host);
//     let server = tiny_http::Server::http(redirect_host)
//         .map_err(|e| e.to_string())?;
//         // .expect("Could not start server");
//
//     let auth_response = oidc::auth(
//         client,
//         "oidc", // mount path (defaults to "oidc")
//         &redirect_uri,
//         Some(VAULT_OIDC_ROLE.to_string()),
//     // ).await.expect("Vault OIDC auth failed");
//     ).await?;
//
//     let auth_resp_url = Url::parse(&auth_response.auth_url)?;
//         // .expect("Could not parse URL");
//
//     let nonce = auth_resp_url.query_pairs()
//         .find(|(key, _)| key == "nonce")
//         .map(|(_, value)| value.into_owned())
//         .ok_or("No nonce found in redirect")?;
//         // .expect("No nonce found in redirect");
//
//     println!("Opening browser for Okta login");
//     let _ = webbrowser::open(&auth_response.auth_url);
//
//     let request = server.recv().expect("Server receive failed");
//     let url = Url::parse(&format!("http://localhost{}", request.url()))?;
//         // .expect("Could not parse URL");
//
//     // Extract query parameters
//     let code = url.query_pairs()
//         .find(|(key, _)| key == "code")
//         .map(|(_, value)| value.into_owned())
//         .ok_or("No code found in redirect")?;
//         // .expect("No code found in redirect");
//
//     let state = url.query_pairs()
//         .find(|(key, _)| key == "state")
//         .map(|(_, value)| value.into_owned())
//         .ok_or("No state found in redirect")?;
//         // .expect("No state found in redirect");
//
//     // 5. Respond to the user in the browser
//     let response = tiny_http::Response::from_string("Authentication successful! You can close this tab.");
//     // request.respond(response).expect("Could not send response");
//     request.respond(response)?;
//
//     // 6. Complete the login with Vault using the captured code
//     let token_auth = oidc::callback(client, "oidc", state.as_str(), nonce.as_str(), code.as_str()).await.expect("Vault OIDC callback failed");
//     println!("token_auth: {:?}", token_auth);
//     Ok(token_auth.client_token)
// }

// fn extract_query_param(url: &Url, param: &str) -> Option<String> {
//     url.query_pairs()
//         .find(|(key, _)| key == param)
//         .map(|(_, value)| value.into_owned())
// }
