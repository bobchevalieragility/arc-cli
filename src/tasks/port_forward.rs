use async_trait::async_trait;
use kube::{Api, Client};
use kube::api::ListParams;
use cliclack::{intro, outro_note, select, spinner};
use std::collections::HashMap;
use console::style;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use k8s_openapi::api::core::v1::{Pod, Service};
use tokio::task::AbortHandle;
use crate::{Args, Goal, GoalStatus};
use crate::aws::eks_cluster::EksCluster;
use crate::tasks::{sleep_indicator, Task, TaskResult, TaskType};

#[derive(Debug)]
pub struct PortForwardTask;

#[async_trait]
impl Task for PortForwardTask {
    async fn execute(&self, args: &Option<Args>, state: &HashMap<Goal, TaskResult>, is_terminal_goal: bool) -> GoalStatus {
        // If Kube context has not been selected, we need to wait for that goal to complete
        let context_goal = Goal::from(TaskType::SelectKubeContext);
        if !state.contains_key(&context_goal) {
            return GoalStatus::Needs(context_goal);
        }

        //TODO add service name to intro
        intro("Port Forward").unwrap();

        // Retrieve info about the desired Kube context from state
        let context_result = state.get(&context_goal)
            .expect("TaskResult for SelectKubeContext not found");
        let context_info = match context_result {
            TaskResult::KubeContext { existing, updated } => {
                updated.as_ref().or(existing.as_ref())
                    .expect("No Kube context available (both existing and updated are None)")
            },
            _ => panic!("Expected TaskResult::KubeContext"),
        };

        // Get the cluster that corresponds to the selected context
        let cluster = &context_info.cluster;

        // If a Kube context has been selected, then the KUBECONFIG env
        // var will be set so we can proceed to create a Kube client
        let spinner = spinner();
        spinner.start("Creating Kubernetes client...");
        let client = Client::try_default().await
            .expect("Could not get default client");
        spinner.stop("Kubernetes client created");

        // let namespace = "development";
        let service_name = "metrics";
        let service_port = 8080u16;
        let local_port = 8082u16;

        // Find one of the given service's pods
        let pod = get_service_pod(service_name, cluster, &client).await;

        // Clone values that need to be moved into the async block
        let namespace = cluster.namespace();
        let pod_name = pod.clone();

        // Start port forwarding using Kubernetes API
        let port_forward_handle = tokio::spawn(async move {
            if let Err(e) = port_forward(client, &namespace, &pod_name, local_port, service_port).await {
                eprintln!("Port-forward error: {}", e);
            }
        });
        let handle = port_forward_handle.abort_handle();

        // Give port-forward time to establish with a progress indicator
        sleep_indicator(
            2,
            "Establishing port-forward...",
            "Port-Forward established"
        ).await;

        // Display summary message to user
        let prompt = format!("Port-Forwarding to {} service", service_name);
        let mut summary = format!("Listening on 127.0.0.1:{}", local_port);
        if is_terminal_goal {
            // Assume user wants to keep port-forward open until manually closed
            summary.push_str("\nPress Ctrl+X to terminate");
            let _ = outro_note(style(prompt).green(), summary);
            let _ = port_forward_handle.await;
        } else {
            // Port-forward session will be cleaned up when PortForwardInfo is dropped
            let _ = outro_note(style(prompt).blue(), summary);
        }

        GoalStatus::Completed(TaskResult::PortForward(PortForwardInfo::new(local_port, handle)))
    }
}

pub struct PortForwardInfo {
    pub local_port: u16,
    pub handle: AbortHandle,
}

impl PortForwardInfo {
    pub fn new(local_port: u16, handle: AbortHandle) -> PortForwardInfo {
        PortForwardInfo { local_port, handle }
    }
}

impl Drop for PortForwardInfo {
    // Ensure graceful cleanup of the spawned port-forward task
    fn drop(&mut self) {
        self.handle.abort();
    }
}

async fn get_service_pod(service_name: &str, cluster: &EksCluster, client: &Client) -> String {
    // Get the selector label for the given service so that we can find its pods
    let selector_label = get_selector_label(service_name, cluster, &client).await;

    // List pods matching the service selector
    let pod_api: Api<Pod> = Api::namespaced(client.clone(), &cluster.namespace());
    let list_params = ListParams::default().labels(&selector_label);
    let pod_list = pod_api.list(&list_params).await
        .expect("Failed to list pods");

    // Return the name of the first pod found
    pod_list.items.first()
        .and_then(|pod| pod.metadata.name.as_ref())
        .expect("No pods found matching service selector")
        .clone()
}

async fn get_selector_label(service_name: &str, cluster: &EksCluster, client: &Client) -> String {
    let service_api: Api<Service> = Api::namespaced(client.clone(), &cluster.namespace());
    let service = service_api.get(service_name).await
        .unwrap_or_else(|_| panic!("Failed to get service: {service_name}"));

    // Extract selector labels from the service
    let selector = service.spec
        .and_then(|spec| spec.selector)
        .expect("Service has no selector");

    // Return label selector string (e.g., "app=metrics,tier=backend")
    selector
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",")
}

async fn port_forward(
    client: Client,
    namespace: &str,
    pod_name: &str,
    local_port: u16,
    pod_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let pod_api: Api<Pod> = Api::namespaced(client, namespace);

    // Bind local TCP listener
    let listener = TcpListener::bind(("127.0.0.1", local_port)).await?;
    // println!("Listening on 127.0.0.1:{}", local_port);

    loop {
        let (mut local_stream, _) = listener.accept().await?;
        let pod_api = pod_api.clone();
        let pod_name = pod_name.to_string();

        tokio::spawn(async move {
            // Create port-forward connection to the pod
            let mut port_forward_stream = match pod_api
                .portforward(&pod_name, &[pod_port])
                .await
            {
                Ok(mut pf) => match pf.take_stream(pod_port) {
                    Some(stream) => stream,
                    None => {
                        eprintln!("Port {} not available", pod_port);
                        return;
                    }
                },
                Err(e) => {
                    eprintln!("Failed to establish port-forward: {}", e);
                    return;
                }
            };

            // Bidirectional copy between local connection and port-forward stream
            let (mut local_read, mut local_write) = tokio::io::split(local_stream);
            let (mut remote_read, mut remote_write) = tokio::io::split(port_forward_stream);

            let client_to_server = async {
                let mut buf = vec![0u8; 8192];
                loop {
                    match local_read.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            if remote_write.write_all(&buf[..n]).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            };

            let server_to_client = async {
                let mut buf = vec![0u8; 8192];
                loop {
                    match remote_read.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            if local_write.write_all(&buf[..n]).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            };

            // Run both directions concurrently
            tokio::select! {
                _ = client_to_server => {},
                _ = server_to_client => {},
            }
        });
    }
}

