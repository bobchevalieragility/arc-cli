mod args;
mod aws;
mod errors;
mod goals;
mod state;
mod tasks;

// Re-export Args for use in main.rs
pub use args::CliArgs;

use std::collections::HashSet;
use cliclack::{outro, outro_note};
use console::style;
use crate::errors::ArcError;
use std;
use crate::goals::{Goal, GoalStatus, OutroText};
use crate::state::State;

pub async fn run(args: &CliArgs) -> Result<(), ArcError> {
    // A given Args with a single ArcCommand may map to multiple goals
    // (e.g., Switch may require both AWS profile and Kube context selection)
    let terminal_goals = CliArgs::to_goals(args);

    // Execute each goal, including any dependent goals
    execute_goals(terminal_goals).await
}

async fn execute_goals(terminal_goals: Vec<Goal>) -> Result<(), ArcError> {
    let mut goals = terminal_goals.clone();
    let mut eval_string = String::new();
    // let mut state: HashMap<Goal, TaskResult> = HashMap::new();
    let mut state = State::new();
    let mut intros: HashSet<Goal> = HashSet::new();

    // Process goals until there are none left, peeking and processing before popping
    while let Some(next_goal) = goals.last() {
        let Goal { goal_type: task_type, args, is_terminal_goal } = next_goal;

        // Check to see if the goal has already been completed. While unlikely,
        // it's possible if multiple goals depend on the same sub-goal.
        if state.contains(next_goal) {
            goals.pop();
            continue;
        }

        // Instantiate a task for the current goal
        let task = task_type.to_task();

        // Determine if this is one of the original, user-requested goals
        if *is_terminal_goal && !intros.contains(next_goal) {
            task.print_intro()?;
            intros.insert(next_goal.clone());
        }

        // Attempt to complete the next goal on the stack
        let goal_result = task.execute(args, &state).await;

        // If next goal indicates that it needs the result of a dependent goal, then add the
        // dependent goal onto the stack, leaving the original goal to be executed at a later time.
        // Otherwise, pop the goal from the stack and store its result in the state.
        match goal_result? {
            GoalStatus::Needs(dependent_goal) => goals.push(dependent_goal),
            GoalStatus::Completed(result, outro_text) => {
                if *is_terminal_goal {
                    // Print outro message
                    if let OutroText::SingleLine{ key, value } = outro_text {
                        let text = format!("{}: {}", style(key).green(), style(value).dim());
                        outro(text)?;
                    } else if let OutroText::MultiLine{ key, value } = outro_text {
                        let prompt = style(key).green();
                        let message = style(value).dim();
                        outro_note(prompt, message)?;
                    }
                }

                // Collect any text that needs to be eval'd in the parent shell
                if let Some(s) = result.eval_string() {
                    eval_string.push_str(&s);
                }

                // Pop the completed goal and store its result in state
                let goal = goals.pop().unwrap();
                state.insert(goal, result);
            },
        }
    }

    // This is the final output that the parent shell should eval.
    // All other program outputs are sent to stderr (i.e. clickack interactive menus, outros, etc).
    Ok(println!("{eval_string}"))
}

fn config_path() -> Result<std::path::PathBuf, ArcError> {
    //TODO .arc-cli path should be configurable
    let mut config_path = home::home_dir().ok_or_else(|| ArcError::HomeDirError)?;
    config_path.push(".arc-cli");

    // Create the config directory if it doesn't already exist
    std::fs::create_dir_all(&config_path)?;
    Ok(config_path)
}
