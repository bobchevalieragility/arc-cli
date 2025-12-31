use cliclack::{intro, outro, select};
use async_trait::async_trait;
use std::collections::HashMap;
use std::{env, fs};
use std::path::PathBuf;
use kube::config::Kubeconfig;
use crate::{ArcCommand, Args, Goal, GoalStatus};
use crate::tasks::{Executor, TaskResult};

#[derive(Debug)]
pub struct SelectKubeContextExecutor;

#[async_trait]
impl Executor for SelectKubeContextExecutor {
    async fn execute(&self, args: &Args, _state: &HashMap<Goal, TaskResult>) -> GoalStatus {
        intro("Kube Context Selector").unwrap();

        // If the KUBECONFIG environment variable is already set, then we'll keep it,
        // unless the user specifically requested to switch it
        if let Ok(current_config) = env::var("KUBECONFIG") {
            match args.command {
                ArcCommand::Switch{ kube_context: true, .. } |
                ArcCommand::Switch{ aws_profile: false, kube_context: false } => {
                    // All of these cases are interpreted as the user wanting to switch context
                },
                _ => {
                    // All other commands result in keeping the current kube context
                    outro(format!("Using context from existing Kube config: {}", current_config)).unwrap();
                    return GoalStatus::Completed(TaskResult::KubeContext(None))
                }
            }
        }

        // Read the master kubeconfig file
        let mut config = Kubeconfig::read_from(default_kube_path())
            .expect("Could not read kube config from default path.");

        // Prompt user to select a kubernetes context
        let selected_kube_context = prompt_for_kube_context(&config);

        // Modify the current context in the in-memory config
        config.current_context = Some(selected_kube_context.clone());

        // Create a unique, terminal-specific kubeconfig file in the tmp dir
        let timestamp = chrono::Local::now().format("%Y%m%dT%H%M%S");
        let tmp_kube_path = env::temp_dir()
            .join(format!("arc_kubeconfig_{}", timestamp));

        // Save the in-memory config to the new kubeconfig file
        let yaml_data = serde_yaml::to_string(&config)
            .expect("Failed to serialize kubeconfig to YAML");
        fs::write(&tmp_kube_path, yaml_data).expect("Failed to write kubeconfig to temp file");

        outro(format!("Kube context will be set to: {}", selected_kube_context)).unwrap();
        let path_str = Some(tmp_kube_path.to_string_lossy().to_string());
        GoalStatus::Completed(TaskResult::KubeContext(path_str))
    }
}

fn default_kube_path() -> PathBuf {
    home::home_dir().expect("Unable to find HOME dir.").join(".kube").join("config")
}

fn prompt_for_kube_context(config: &Kubeconfig) -> String {
    let mut menu = select("Which Kubernetes context would you like to use?");

    let available_contexts: Vec<String> = config.contexts
        .iter()
        .map(|ctx| ctx.name.clone())
        .collect();

    for ctx in &available_contexts {
        menu = menu.item(ctx, ctx, "");
    }

    menu.interact().unwrap().to_string()
}