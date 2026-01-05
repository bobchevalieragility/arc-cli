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
    async fn execute(&self, args: &Option<Args>, _state: &HashMap<Goal, TaskResult>, is_terminal_goal: bool) -> GoalStatus {
        if let ArcCommand::Switch{ use_current: true, .. } = &args.as_ref().expect("Args is None").command {
            // User wants to use current KUBECONFIG, if it's already set
            let current_kubeconfig = env::var("KUBECONFIG");
            if current_kubeconfig.is_ok() {
                return GoalStatus::Completed(TaskResult::KubeContext(None))
            }
        }

        // Read the master kubeconfig file
        let mut config = Kubeconfig::read_from(default_kube_path())
            .expect("Could not read kube config from default path.");

        // Prompt user to select a kubernetes context
        intro("Kube Context Selector").unwrap();
        let selected_kube_context = prompt_for_kube_context(&config);

        // Find the cluster associated with the selected context
        let named_context = config.contexts.iter()
            .find(|ctx| ctx.name == selected_kube_context)
            .expect("Selected context not found in kubeconfig.");
        let cluster_name = match &named_context.context {
            Some(ctx) => ctx.cluster.clone(),
            None => panic!("No context data found for selected context."),
        };
        let eks_cluster = EksCluster::from(cluster_name.as_str());

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
        let info = Some(KubeContextInfo::new(eks_cluster, tmp_kube_path));
        GoalStatus::Completed(TaskResult::KubeContext(info))
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