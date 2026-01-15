use cliclack::{intro, select};
use async_trait::async_trait;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use crate::args::Args;
use crate::config_path;
use crate::errors::ArcError;
use crate::goals::{GoalStatus, OutroText};
use crate::state::State;
use crate::tasks::{Task, TaskResult};

#[derive(Debug)]
pub struct CreateTabCompletionsTask;

#[async_trait]
impl Task for CreateTabCompletionsTask {
    fn print_intro(&self) -> Result<(), ArcError> {
        intro("Creating tab completions file")?;
        Ok(())
    }

    async fn execute(&self, _args: &Option<Args>, _state: &State) -> Result<GoalStatus, ArcError> {
        // Get a list of all available RDS instances for this account
        // let available_rds_instances = profile_info.account.rds_instances();
        let shell = prompt_for_shell()?;

        // Create a file to store the completions
        let mut path = config_path()?;
        path.push(format!("arc-completions-{}", shell.to_string().to_lowercase()));
        let mut file = std::fs::File::create(&path)?;

        // Generate the completion file
        let mut cmd = Args::command();
        generate(shell, &mut cmd, "arc", &mut file);

        let prompt = format!("Tab completions file generated to {}", path.display());
        let msg = "Source this file from your startup script (i.e. ~/.zshrc) to enable.";
        let outro_text = OutroText::multi(prompt, msg.to_string());

        Ok(GoalStatus::Completed(TaskResult::TabCompletionsCreated, outro_text))
    }
}

fn prompt_for_shell() -> Result<Shell, ArcError> {
    let available_shells = vec!["bash", "zsh", "fish", "powershell", "elvish"];
    let mut menu = select("Select shell");
    for shell in &available_shells {
        menu = menu.item(shell, shell, "");
    }

    let shell_name = menu.interact()?.to_string();

    match shell_name.as_str() {
        "bash" => Ok(Shell::Bash),
        "zsh" => Ok(Shell::Zsh),
        "fish" => Ok(Shell::Fish),
        "powershell" => Ok(Shell::PowerShell),
        "elvish" => Ok(Shell::Elvish),
        _ => Err(ArcError::UserInputError(format!("Unsupported shell: {shell_name}"))),
    }
}