mod tasks;

use std::collections::{HashMap, HashSet};
use clap::{Parser, Subcommand, ValueEnum};
use topo_sort::{SortResults, TopoSort};
use crate::tasks::{Executor, Goal, State, Task, TaskResult, ALL_TASKS};
use crate::tasks::select_aws_profile::SelectAwsProfileExecutor;

#[derive(Parser, Debug)]
#[command(author, version, about = "CLI Tool for Arc Backend")]
pub struct Args {
    //TODO should this be optional?
    #[command(subcommand)]
    command: Option<ArcCommand>,
}

#[derive(Subcommand, Debug)]
enum ArcCommand {
    Switch {
        #[arg(short, long)]
        aws_profile: bool,

        #[arg(short, long)]
        kube_context: bool,
    },
    Vault {
        #[arg(short, long)]
        secret: String,
    }
}

// pub fn run(args: &Args) {
//     match &args.command {
//         Some(Command::Switch { aws_profile, kube_context }) => {
//             if *list {
//                 println!("Printing testing lists...");
//             } else {
//                 println!("Not printing testing lists...");
//             }
//         }
//         // None => {}
//         _ => {}
//     }
// }


pub fn run(args: &Args, args_os: &Vec<std::ffi::OsString>) {
    // print!("Args: {:?}", args);
    // Create a map indexed by the Goal each available Task provides
    let goal_providers: HashMap<Goal, &Task> = ALL_TASKS
        .iter()
        .map(|task| (task.provides(), task))
        .collect();

    //TODO recursively determine needed tasks based on args and dependencies
    let needed_tasks = vec![
        // Task::AwsProfile(AwsProfileExecutor),
        // Task::SpawnShell(SpawnShellExecutor),
        // Task::ConfigureEnvironment(ConfigureEnvironmentExecutor),
        Task::SelectAwsProfile(SelectAwsProfileExecutor),
    ];

    // Create nodes to be sorted, one for each needed task, specifying dependencies
    let needed_nodes: HashMap<Goal, HashSet<Goal>> = needed_tasks
        .iter()
        .map(|task| (task.provides(), task.needs()))
        .collect();

    // Topologically sort the needed tasks based on dependencies
    let topo_sort = TopoSort::from_map(needed_nodes.clone());
    match topo_sort.into_vec_nodes() {
        SortResults::Full(sorted_nodes) => {
            execute_tasks(
                args,
                args_os,
                // sorted_nodes.iter().map(|g| *goal_providers.get(g).unwrap()).collect()
                sorted_nodes
                    .iter()
                    .map(|g| *goal_providers.get(g).expect(&format!("No task provides goal: {:?}", g)))
                    .collect()
            )
        },
        SortResults::Partial(_) => {
            panic!("There's a cycle in the dependency graph!: {:?}", needed_nodes)
        },
    }
}

fn execute_tasks(args: &Args, args_os: &Vec<std::ffi::OsString>, tasks: Vec<&Task>) {
    let mut eval_string = String::new();
    let mut results: HashMap<Goal, TaskResult> = HashMap::new();

    for task in tasks {
        let state = State::new(args, args_os, &results);
        let result = task.execute(&state);
        if let Some(s) = result.eval_string() {
            eval_string.push_str(&s);
        }
        results.insert(task.provides(), result);
    }

    print!("{eval_string}");
}