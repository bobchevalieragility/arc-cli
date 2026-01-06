use cliclack::{intro, outro, select};
use async_trait::async_trait;
use std::collections::HashMap;
use std::{env, fs};
use std::path::PathBuf;
use kube::config::Kubeconfig;
use crate::{ArcCommand, Args, Goal, GoalStatus};
use crate::aws::eks_cluster::EksCluster;
use crate::tasks::{color_output, Task, TaskResult};

#[derive(Debug)]
pub struct SelectKubeContextTask;

#[async_trait]
impl Task for SelectKubeContextTask {
    fn print_intro(&self) {
        let _ = intro("Select Kube Context");
    }

    async fn execute(&self, args: &Option<Args>, _state: &HashMap<Goal, TaskResult>, is_terminal_goal: bool) -> GoalStatus {
        if let ArcCommand::Switch{ use_current: true, .. } = &args.as_ref().expect("Args is None").command {
            if let Ok(current_kubeconfig) = env::var("KUBECONFIG") {
                let kube_path = PathBuf::from(current_kubeconfig);
                let config = Kubeconfig::read_from(&kube_path)
                    .expect("Could not read kube config from KUBECONFIG path.");
                let current_context = config.current_context.as_ref()
                    .expect("No current context set in kubeconfig.");

                // Find the cluster associated with the current context
                let eks_cluster = get_cluster(current_context, &config);
                let info = KubeContextInfo::new(eks_cluster, kube_path);
                let task_result = TaskResult::KubeContext{ existing: Some(info), updated: None };
                return GoalStatus::Completed(task_result)
            }
        }

        // Read the master kubeconfig file
        let mut config = Kubeconfig::read_from(default_kube_path())
            .expect("Could not read kube config from default path.");

        // Prompt user to select a kubernetes context
        let selected_kube_context = prompt_for_kube_context(&config);

        // Find the cluster associated with the selected context
        let eks_cluster = get_cluster(&selected_kube_context, &config);

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

        // Export the KUBECONFIG environment variable so that it can be used by dependent tasks
        unsafe { env::set_var("KUBECONFIG", &tmp_kube_path); }

        outro(format!("Kube context: {}", color_output(&selected_kube_context, is_terminal_goal))).unwrap();
        let info = KubeContextInfo::new(eks_cluster, tmp_kube_path);
        let task_result = TaskResult::KubeContext{ existing: None, updated: Some(info) };
        GoalStatus::Completed(task_result)
    }
}

pub struct KubeContextInfo {
    pub cluster: EksCluster,
    pub kubeconfig: PathBuf,
}

impl KubeContextInfo {
    pub fn new(cluster: EksCluster, kubeconfig: PathBuf) -> KubeContextInfo {
        KubeContextInfo { cluster, kubeconfig }
    }
}

fn default_kube_path() -> PathBuf {
    home::home_dir().expect("Unable to find HOME dir.").join(".kube").join("config")
}

fn get_cluster(context_name: &str, config: &Kubeconfig) -> EksCluster {
    let named_context = config.contexts.iter()
        .find(|ctx| ctx.name == context_name)
        .expect("Provided context not found in kubeconfig.");
    let cluster_name = match &named_context.context {
        Some(ctx) => ctx.cluster.clone(),
        None => panic!("No context data found for provided context."),
    };
    EksCluster::from(cluster_name.as_str())
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