mod tasks;

use std::collections::HashMap;
use clap::{Parser, Subcommand};
use crate::tasks::{Executor, Task, TaskResult};

#[derive(Parser, Clone, Debug, PartialEq, Eq, Hash)]
#[command(author, version, about = "CLI Tool for Arc Backend")]
pub struct Args {
    #[command(subcommand)]
    command: ArcCommand,
}

#[derive(Subcommand, Clone, Debug, PartialEq, Eq, Hash)]
enum ArcCommand {
    Switch {
        #[arg(short, long)]
        aws_profile: bool,

        #[arg(short, long)]
        kube_context: bool,
    },
    AwsSecret {
        #[arg(short, long)]
        name: Option<String>,
    },
    // Vault {
    //     #[arg(short, long)]
    //     secret: String,
    // }
}

impl Args {
    fn to_goals(&self) -> Vec<Goal> {
        match self.command {
            ArcCommand::AwsSecret { .. } => vec![
                Goal::new(Task::GetAwsSecret, self.clone())
            ],
            ArcCommand::Switch { aws_profile: true, .. } => vec![
                Goal::new(Task::SelectAwsProfile, self.clone())
            ],
            ArcCommand::Switch { kube_context: true, .. } => vec![
                Goal::new(Task::SelectKubeContext, self.clone())
            ],
            ArcCommand::Switch { aws_profile: false, kube_context: false } => vec![
                Goal::new(Task::SelectKubeContext, self.clone()),
                Goal::new(Task::SelectAwsProfile, self.clone())
            ],
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Goal {
    task: Task,
    args: Args,
}

impl Goal {
    fn new(task: Task, args: Args) -> Self {
        Goal { task, args }
    }
}

pub async fn run(args: &Args) {
    // A given Args with a single ArcCommand may map to multiple goals
    // (e.g., Switch may require both AWS profile and Kube context selection)
    let cmd_goals = Args::to_goals(args);

    // Execute each goal, including any dependent goals
    execute_goals(args, cmd_goals).await;
}

enum GoalStatus {
    Completed(TaskResult),
    Needs(Goal),
}

async fn execute_goals(args: &Args, mut goals: Vec<Goal>) {
    let mut eval_string = String::new();
    let mut state: HashMap<Goal, TaskResult> = HashMap::new();

    // Process goals until there are none left, peeking and processing before popping
    while let Some(Goal { task, args }) = goals.last() {
        // Check to see if the goal has already been completed. While unlikely,
        // it's possible if multiple goals depend on the same sub-goal.
        if state.contains_key(&goals.last().unwrap()) {
            goals.pop();
            continue;
        }

        // Attempt to complete the next goal on the stack
        let goal_result = task.execute(args, &state).await;

        // If next goal indicates that it needs the result of a dependent goal, then add the
        // dependent goal onto the stack, leaving the original goal to be executed at a later time.
        // Otherwise, pop the goal from the stack and store its result in the state.
        match goal_result {
            GoalStatus::Needs(dependent_goal) => goals.push(dependent_goal),
            GoalStatus::Completed(result) => {
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

    // This is the final output that the parent shell should eval
    println!("{eval_string}");
}