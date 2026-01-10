use cliclack::{intro, select};
use async_trait::async_trait;
use aws_runtime::env_config::file::EnvConfigFiles;
use aws_config::profile;
use aws_types::os_shim_internal::{Env, Fs};
use std::env;
use aws_runtime::env_config::section::EnvConfigSections;
use crate::{ArcCommand, Args, GoalStatus, OutroText, State};
use crate::aws::aws_account::AwsAccount;
use crate::errors::ArcError;
use crate::tasks::{Task, TaskResult};

#[derive(Debug)]
pub struct SelectAwsProfileTask;

#[async_trait]
impl Task for SelectAwsProfileTask {
    fn print_intro(&self) -> Result<(), ArcError> {
        intro("Switch AWS Profile")?;
        Ok(())
    }

    async fn execute(&self, args: &Option<Args>, _state: &State) -> Result<GoalStatus, ArcError> {
        //TODO Cannot switch to sandbox profile
        // Validate that args are present
        let args = args.as_ref()
            .ok_or_else(|| ArcError::invalid_arc_command("Switch", "None"))?;

        if let ArcCommand::Switch{ use_current: true, .. } = &args.command {
            // User wants to use current AWS_PROFILE, if it's already set
            if let Ok(current_profile) = env::var("AWS_PROFILE") {
                let account = get_aws_account(&current_profile).await?;
                let info = AwsProfileInfo::new(current_profile, account);
                let key = "Using current AWS profile".to_string();
                let outro_text = OutroText::single(key, info.name.clone());
                let task_result = TaskResult::AwsProfile{ profile: info, updated: false };
                return Ok(GoalStatus::Completed(task_result, outro_text));
            }
        }

        // Prompt user to select an AWS profile
        let selected_aws_profile = prompt_for_aws_profile().await?;

        // Set outro content
        let key = "Switched to AWS profile".to_string();
        let outro_text = OutroText::single(key, selected_aws_profile.clone());

        // Create task result
        let account_id = get_aws_account(&selected_aws_profile).await?;
        let info = AwsProfileInfo::new(selected_aws_profile, account_id);
        let task_result = TaskResult::AwsProfile{ profile: info, updated: true };

        Ok(GoalStatus::Completed(task_result, outro_text))
    }
}

#[derive(Debug)]
pub struct AwsProfileInfo {
    pub name: String,
    pub account: AwsAccount,
}

impl AwsProfileInfo {
    pub fn new(name: String, account: AwsAccount) -> AwsProfileInfo {
        AwsProfileInfo { name, account }
    }
}

async fn prompt_for_aws_profile() -> Result<String, ArcError> {
    let available_profiles = get_available_aws_profiles().await?;

    let mut menu = select("Select an AWS Profile");
    for profile in &available_profiles {
        menu = menu.item(profile, profile, "");
    }

    Ok(menu.interact()?.to_string())
}

async fn get_available_aws_profiles() -> Result<Vec<String>, ArcError> {
    let config_sections = get_env_configs().await?;

    // Extract profile names
    let mut profile_names: Vec<String> = config_sections
        .profiles()
        .map(|s| s.to_string())
        .filter(|s| s != "default")
        .collect();

    if profile_names.is_empty() {
        panic!("No AWS profiles found");
    }

    profile_names.sort();
    Ok(profile_names)
}

async fn get_aws_account(profile_name: &str) -> Result<AwsAccount, ArcError> {
    let config_sections = get_env_configs().await?;

    // Extract SSO account ID
    let profile = config_sections.get_profile(profile_name)
        .ok_or_else(|| ArcError::AwsProfileError(format!("Profile '{}' not found", profile_name)))?;
    let account_id = profile.get("sso_account_id")
        .ok_or_else(|| ArcError::AwsProfileError("sso_account_id not found in profile".to_string()))?;

    Ok(AwsAccount::from(account_id))
}

async fn get_env_configs() -> Result<EnvConfigSections, ArcError> {
    // Use real filesystem and environment access
    let fs = Fs::real();
    let env = Env::real();

    // Load default profile files (~/.aws/config and ~/.aws/credentials)
    let config_files = EnvConfigFiles::default();

    // Load env config sections asynchronously
    let env_config_sections = profile::load(&fs, &env, &config_files, None).await?;
    Ok(env_config_sections)
}
