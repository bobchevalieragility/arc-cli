use async_trait::async_trait;
use kube::{Api, Client};
use kube::api::ListParams;
use cliclack::intro;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use k8s_openapi::api::core::v1::{Pod, Service};
use crate::{Args, Goal, GoalStatus};
use crate::tasks::{Task, TaskResult, TaskType};

#[derive(Debug)]
pub struct SetLogLevelTask;

#[async_trait]
impl Task for SetLogLevelTask {
    async fn execute(&self, args: &Option<Args>, state: &HashMap<Goal, TaskResult>, is_terminal_goal: bool) -> GoalStatus {
        // If Kube context has not been selected, we need to wait for that goal to complete
        let context_goal = Goal::from(TaskType::SelectKubeContext);
        if !state.contains_key(&context_goal) {
            return GoalStatus::Needs(context_goal);
        }

        intro("Log Level Selector").unwrap();

        // If a Kube context has been selected, then the KUBECONFIG env
        // var will be set so we can proceed to create a Kube client
        let client = Client::try_default().await
            .expect("Could not get default client");

        let namespace = "development";
        let service_name = "metrics";
        let service_port = 8080u16;
        let local_port = 8082u16;

        // Get the service to find its selector labels
        let services: Api<Service> = Api::namespaced(client.clone(), namespace);
        let service = services.get(service_name).await
            .expect("Failed to get service");

        // Extract selector labels from the service
        let selector = service.spec
            .and_then(|spec| spec.selector)
            .expect("Service has no selector");

        // Build label selector string (e.g., "app=metrics,tier=backend")
        let label_selector = selector
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",");

        println!("Using label selector: {}", label_selector);

        // List pods matching the service selector
        let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
        let list_params = ListParams::default().labels(&label_selector);
        let pod_list = pods.list(&list_params).await
            .expect("Failed to list pods");

        let pod_name = pod_list.items.first()
            .and_then(|pod| pod.metadata.name.as_ref())
            .expect("No pods found matching service selector")
            .clone();

        println!("Setting up port-forward: localhost:{} -> {}/{}:{}",
                 local_port, namespace, pod_name, service_port);

        // Start port forwarding using Kubernetes API
        let port_forward_handle = tokio::spawn(async move {
            if let Err(e) = port_forward(client, namespace, &pod_name, local_port, service_port).await {
                eprintln!("Port-forward error: {}", e);
            }
        });

        // Give port-forward time to establish
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        println!("Port-forward established on localhost:{}", local_port);

        // Make HTTP GET request to the actuator/loggers endpoint
        let url = format!("http://localhost:{}/actuator/loggers/com.agilityrobotics.metrics", local_port);
        println!("Fetching log level from: {}", url);

        let http_client = reqwest::Client::new();
        match http_client.get(&url).send().await {
            Ok(response) => {
                println!("Response status: {}", response.status());
                println!("Response headers:\n{:#?}", response.headers());

                match response.text().await {
                    Ok(body) => {
                        println!("\nResponse body:\n{}", body);

                        // Try to parse as JSON for better formatting
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                            println!("\nFormatted JSON:\n{}", serde_json::to_string_pretty(&json).unwrap());
                        }
                    },
                    Err(e) => eprintln!("Failed to read response body: {}", e),
                }
            },
            Err(e) => eprintln!("HTTP request failed: {}", e),
        }

        let handle = port_forward_handle.abort_handle();
        handle.abort();

        GoalStatus::Completed(TaskResult::LogLevel)
    }
}

async fn port_forward(
    client: Client,
    namespace: &str,
    pod_name: &str,
    local_port: u16,
    pod_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let pods: Api<Pod> = Api::namespaced(client, namespace);

    // Bind local TCP listener
    let listener = TcpListener::bind(("127.0.0.1", local_port)).await?;
    println!("Listening on 127.0.0.1:{}", local_port);

    loop {
        let (mut local_stream, _) = listener.accept().await?;
        let pods = pods.clone();
        let pod_name = pod_name.to_string();

        tokio::spawn(async move {
            // Create port-forward connection to the pod
            let mut port_forward_stream = match pods
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

