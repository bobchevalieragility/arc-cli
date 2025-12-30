mod tasks;

use std::collections::{HashMap, HashSet};
use clap::{Parser, Subcommand};
use topo_sort::{SortResults, TopoSort};
use crate::tasks::{Executor, State, Task, TaskResult};

#[derive(Parser, Debug)]
#[command(author, version, about = "CLI Tool for Arc Backend")]
pub struct Args {
    #[command(subcommand)]
    command: ArcCommand,
}

#[derive(Subcommand, Debug)]
enum ArcCommand {
    Switch {
        #[arg(short, long)]
        aws_profile: bool,

        #[arg(short, long)]
        kube_context: bool,
    },
    // Vault {
    //     #[arg(short, long)]
    //     secret: String,
    // }
}

pub fn run(args: &Args) {
    // Recursively determine which tasks are needed for the given command
    let needed_tasks = get_tasks_for_command(&args.command);

    // Convert the tasks to nodes that can be topologically sorted
    let needed_nodes: HashMap<Task, HashSet<Task>> = needed_tasks
        .iter()
        .map(|task| (task.clone(), task.needs()))
        .collect();

    // Topologically sort the nodes so that dependent tasks are executed after their dependencies
    let topo_sort = TopoSort::from_map(needed_nodes);
    match topo_sort.into_vec_nodes() {
        SortResults::Full(sorted_tasks) => execute_tasks(args, sorted_tasks),
        SortResults::Partial(_) => panic!("There's a cycle in the dependency graph!: {:?}", needed_tasks),
    }
}

fn get_tasks_for_command(command: &ArcCommand) -> HashSet<Task> {
    // Start with the tasks that directly correspond to the given command
    let mut tasks_to_process = Task::command_tasks(command);

    // Recursively add the tasks and their dependencies
    let mut needed_tasks = HashSet::new();
    while let Some(task) = tasks_to_process.pop() {
        if !needed_tasks.contains(&task) {
            needed_tasks.insert(task.clone());
            for dep in task.needs() {
                tasks_to_process.push(dep);
            }
        }
    }

    needed_tasks
}

fn execute_tasks(args: &Args, tasks: Vec<Task>) {
    let mut eval_string = String::new();
    let mut results: HashMap<Task, TaskResult> = HashMap::new();

    for task in tasks {
        let state = State::new(args, &results);
        let result = task.execute(&state);
        if let Some(s) = result.eval_string() {
            eval_string.push_str(&s);
        }
        results.insert(task, result);
    }

    print!("{eval_string}");
}