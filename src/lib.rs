mod aws;
mod tasks;

use std::convert::From;
use std::collections::{HashMap, HashSet};
use clap::{Parser, Subcommand};
use cliclack::{outro, outro_note};
use console::{style, StyledObject};
use crate::OutroText::{MultiLine, SingleLine};
use crate::tasks::{TaskResult, TaskType};
use crate::tasks::set_log_level::Level;

#[derive(Parser, Clone, Debug, PartialEq, Eq, Hash)]
#[command(author, version, about = "CLI Tool for Arc Backend")]
pub struct Args {
    #[command(subcommand)]
    command: ArcCommand,
}

#[derive(Subcommand, Clone, Debug, PartialEq, Eq, Hash)]
enum ArcCommand {
    #[command(about = "View or set the log level for a Java Spring Boot service")]
    LogLevel {
        #[arg(short, long, help = "Service name, e.g. 'metrics' (if omitted, will prompt)")]
        service: Option<String>,

        #[arg(short, long, default_value = "ROOT", help = "Package, e.g. 'com.agilityrobotics.metrics' (defaults to ROOT)")]
        package: String,

        #[arg(short, long, help = "Desired log level (if omitted, will prompt)")]
        level: Option<Level>,

        #[arg(short, long, help = "Just print the current log level")]
        display_only: bool,
    },
    #[command(about = "Retrieve a secret value from AWS Secrets Manager")]
    AwsSecret {
        #[arg(short, long, help = "Name of the secret to retrieve (if omitted, will prompt)")]
        name: Option<String>,
    },
    #[command(about = "Retrieve a secret value from Vault")]
    Vault {
        #[arg(short, long, help = "Path to secret to retrieve (if omitted, will prompt)")]
        path: Option<String>,

        #[arg(short, long, help = "Field within secret to retrieve (defaults to entire secret)")]
        field: Option<String>,
    },
    #[command(about = "Launch pgcli to interact with a Postgres RDS instance")]
    Pgcli,
    #[command(about = "Launch the InfluxDB UI")]
    Influx,
    #[command(about = "Start port-forwarding to a Kubernetes service")]
    PortForward {
        #[arg(short, long, help = "Service name, e.g. 'metrics' (if omitted, will prompt)")]
        service: Option<String>,

        #[arg(short, long, help = "Local port (defaults to random, unused port)")]
        port: Option<u16>,

        #[arg(short, long, help = "Tear down port-forwarding when command exits")]
        tear_down: bool,
    },
    #[command(about = "Switch AWS profile and/or Kubernetes context")]
    Switch {
        #[arg(short, long, help = "Switch AWS profile (if false and kube_context is false, will switch both)")]
        aws_profile: bool,

        #[arg(short, long, help = "Switch kube context (if false and kube_context is false, will switch both)")]
        kube_context: bool,

        #[arg(short, long, help = "Whether to skip if already set (defaults to false)")]
        use_current: bool,
    },
}

impl Args {
    fn to_goals(&self) -> Vec<Goal> {
        match self.command {
            ArcCommand::AwsSecret { .. } => vec![
                Goal::new_terminal(TaskType::GetAwsSecret, Some(self.clone()))
            ],
            ArcCommand::LogLevel { .. } => vec![
                Goal::new_terminal(TaskType::SetLogLevel, Some(self.clone()))
            ],
            ArcCommand::Pgcli => vec![
                Goal::new_terminal(TaskType::RunPgcli, Some(self.clone()))
            ],
            ArcCommand::PortForward { .. } => vec![
                Goal::new_terminal(TaskType::PortForward, Some(self.clone()))
            ],
            ArcCommand::Influx => vec![
                Goal::new_terminal(TaskType::LaunchInflux, Some(self.clone()))
            ],
            ArcCommand::Switch { aws_profile: true, .. } => vec![
                Goal::new_terminal(TaskType::SelectAwsProfile, Some(self.clone()))
            ],
            ArcCommand::Switch { kube_context: true, .. } => vec![
                Goal::new_terminal(TaskType::SelectKubeContext, Some(self.clone()))
            ],
            ArcCommand::Switch { aws_profile: false, kube_context: false, .. } => vec![
                Goal::new_terminal(TaskType::SelectKubeContext, Some(self.clone())),
                Goal::new_terminal(TaskType::SelectAwsProfile, Some(self.clone()))
            ],
            ArcCommand::Vault { .. } => vec![
                Goal::new_terminal(TaskType::GetVaultSecret, Some(self.clone()))
            ],
        }
    }
}

//TODO move Goal into it's own module to force callers to use the Goal::new or Goal::new_terminal constructors
#[derive(Clone, PartialEq, Eq, Hash)]
struct Goal {
    task_type: TaskType,
    args: Option<Args>,
    is_terminal_goal: bool,
}

impl Goal {
    fn new(task_type: TaskType, args: Option<Args>) -> Self {
        Goal { task_type, args, is_terminal_goal: false }
    }
    fn new_terminal(task_type: TaskType, args: Option<Args>) -> Self {
        Goal { task_type, args, is_terminal_goal: true }
    }
}

impl From<TaskType> for Goal {
    fn from(task_type: TaskType) -> Self {
        match task_type {
            TaskType::LoginToVault => Goal::new(TaskType::LoginToVault, None),
            TaskType::SelectActuatorService => Goal::new(TaskType::SelectActuatorService, None),
            TaskType::SelectAwsProfile => Goal::new(TaskType::SelectAwsProfile, Some(Args {
                command: ArcCommand::Switch {
                    aws_profile: true,
                    kube_context: false,
                    use_current: true,
                }
            })),
            TaskType::SelectInfluxInstance => Goal::new(TaskType::SelectInfluxInstance, None),
            TaskType::SelectKubeContext => Goal::new(TaskType::SelectKubeContext, Some(Args {
                command: ArcCommand::Switch {
                    aws_profile: false,
                    kube_context: true,
                    use_current: true,
                }
            })),
            TaskType::SelectRdsInstance => Goal::new(TaskType::SelectRdsInstance, None),
            _ => panic!("TaskType=>Goal conversion is missing."),
        }
    }
}

pub async fn run(args: &Args) {
    // A given Args with a single ArcCommand may map to multiple goals
    // (e.g., Switch may require both AWS profile and Kube context selection)
    let terminal_goals = Args::to_goals(args);

    // Execute each goal, including any dependent goals
    execute_goals(terminal_goals).await;
}

enum GoalStatus {
    Completed(TaskResult, OutroText),
    Needs(Goal),
}

enum OutroText {
    SingleLine{ key: String, value: String },
    MultiLine{ key: String, value: String },
    None,
}

impl OutroText {
    pub fn single(key: String, value: String) -> OutroText {
        OutroText::SingleLine { key, value }
    }
    pub fn multi(key: String, value: String) -> OutroText {
        OutroText::MultiLine { key, value }
    }
}

async fn execute_goals(terminal_goals: Vec<Goal>) {
    let mut goals = terminal_goals.clone();
    let mut eval_string = String::new();
    let mut state: HashMap<Goal, TaskResult> = HashMap::new();
    let mut intros: HashSet<Goal> = HashSet::new();

    // Process goals until there are none left, peeking and processing before popping
    while let Some(Goal { task_type, args, is_terminal_goal }) = goals.last() {
        // Check to see if the goal has already been completed. While unlikely,
        // it's possible if multiple goals depend on the same sub-goal.
        if state.contains_key(&goals.last().unwrap()) {
            goals.pop();
            continue;
        }

        // Instantiate a task for the current goal
        let task = task_type.to_task();

        // Determine if this is one of the original, user-requested goals
        if *is_terminal_goal && !intros.contains(&goals.last().unwrap()) {
            task.print_intro();
            intros.insert(goals.last().unwrap().clone());
        }

        // Attempt to complete the next goal on the stack
        let goal_result = task_type.to_task().execute(args, &state).await;

        // If next goal indicates that it needs the result of a dependent goal, then add the
        // dependent goal onto the stack, leaving the original goal to be executed at a later time.
        // Otherwise, pop the goal from the stack and store its result in the state.
        match goal_result {
            GoalStatus::Needs(dependent_goal) => goals.push(dependent_goal),
            GoalStatus::Completed(result, outro_text) => {
                if *is_terminal_goal {
                    // Print outro message
                    if let SingleLine{ key, value } = outro_text {
                        let text = format!("{}: {}", style(key).green(), style(value).dim());
                        let _ = outro(text);
                    } else if let MultiLine{ key, value } = outro_text {
                        let prompt = style(key).green();
                        let message = style(value).dim();
                        let _ = outro_note(prompt, message);

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
    println!("{eval_string}");
}

pub fn color_output(output: &str, is_terminal_goal: bool) -> StyledObject<&str> {
    if is_terminal_goal {
        style(output).green()
    } else {
        style(output).blue()
    }
}

