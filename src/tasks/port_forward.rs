use async_trait::async_trait;
use kube::{Api, Client};
use kube::api::ListParams;
use cliclack::{intro, outro_note, select, spinner};
use console::style;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use k8s_openapi::api::core::v1::{Pod, Service, ServiceSpec};
use kube::config::Kubeconfig;
use tokio::task::AbortHandle;
use crate::{ArcCommand, Args, Goal, GoalStatus, OutroText, State};
use crate::aws::kube_service::KubeService;
use crate::errors::ArcError;
use crate::tasks::{sleep_indicator, Task, TaskResult, TaskType};

#[derive(Debug)]
pub struct PortForwardTask;

#[async_trait]
impl Task for PortForwardTask {
    fn print_intro(&self) -> Result<(), ArcError> {
        intro("Port Forward")?;
        Ok(())
    }

    async fn execute(&self, args: &Option<Args>, state: &State) -> Result<GoalStatus, ArcError> {
        // Validate that args are present
        let args = args.as_ref()
            .ok_or_else(|| ArcError::invalid_arc_command("PortForward", "None"))?;

        // Ensure that SSO token has not expired
        let sso_goal = Goal::from(TaskType::PerformSso);
        if !state.contains(&sso_goal) {
            return Ok(GoalStatus::Needs(sso_goal));
        }

        // If Kube context has not been selected, we need to wait for that goal to complete
        let context_goal = Goal::from(TaskType::SelectKubeContext);
        if !state.contains(&context_goal) {
            return Ok(GoalStatus::Needs(context_goal));
        }

        // Retrieve info about the desired Kube context from state
        let context_info = state.get_kube_context_info(&context_goal)?;

        // Get the cluster that corresponds to the selected context
        let cluster = &context_info.cluster;

        // Create a Kubernetes client using the KUBECONFIG path from state
        let spinner = spinner();
        spinner.start("Creating Kubernetes client...");
        let kubeconfig = Kubeconfig::read_from(&context_info.kubeconfig)?;
        let client = Client::try_from(kubeconfig)?;
        spinner.stop("Kubernetes client created");

        let service_api: Api<Service> = Api::namespaced(client.clone(), &cluster.namespace());

        // Determine which service to port-forward to, prompting user if necessary
        let service = match &args.command {
            ArcCommand::PortForward{ service: Some(x), .. } => {
                KubeService::new(x.clone(), get_service_port(&service_api, x).await)
            },
            ArcCommand::PortForward{ service: None, .. } => prompt_for_service(&service_api).await?,
            _ => return Err(ArcError::InvalidArcCommand(
                "PortForward".to_string(),
                format!("{:?}", args.command)
            )),
        };

        // Determine which local port will be used for port-forwarding
        let local_port: u16 = match &args.command {
            ArcCommand::PortForward{ port: Some(p), .. } => *p,
            ArcCommand::PortForward{ port: None, .. } => find_available_port().await?,
            _ => return Err(ArcError::invalid_arc_command(
                "PortForward",
                format!("{:?}", args.command)
            )),
        };

        // Find one of the given service's pods
        let pod_api: Api<Pod> = Api::namespaced(client.clone(), &cluster.namespace());
        let pod = get_service_pod(&service.name, &service_api, &pod_api).await?;

        // Start port forwarding using Kubernetes API
        let port_forward_handle = tokio::spawn(async move {
            if let Err(e) = port_forward(&pod, local_port, service.port, &pod_api).await {
                eprintln!("Port-forward error: {}", e);
            }
        });
        let handle = port_forward_handle.abort_handle();

        // Give port-forward time to establish with a progress indicator
        let end_msg = format!(
            "Service({}) listening on 127.0.0.1:{}",
            style(&service.name).dim(),
            style(local_port).dim()
        );
        sleep_indicator(2, "Establishing port-forward...", &end_msg).await;

        // Determine which local port will be used for port-forwarding
        if let ArcCommand::PortForward{ tear_down: false, .. } = &args.command {
            let prompt = format!("Port-Forwarding to {} service", service.name);
            let msg = format!("Listening on 127.0.0.1:{}\nPress Ctrl+X to terminate", local_port);
            outro_note(style(prompt).green(), msg)?;

            // Assume user wants to keep port-forward open until manually closed
            port_forward_handle.await?;
        }

        let info = PortForwardInfo::new(local_port, handle.clone());
        Ok(GoalStatus::Completed(TaskResult::PortForward(info), OutroText::None))
    }
}

#[derive(Debug)]
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

async fn get_app_services(service_api: &Api<Service>) -> Result<Vec<KubeService>, ArcError> {
    // Retrieve ALL services for the given namespace
    let list_params = ListParams::default();
    let svc_list = service_api.list(&list_params).await?;

    // Filter out services that don't contain "app" in their selector
    let kube_services = svc_list.items.into_iter()
        .filter(|svc| {
            svc.spec.as_ref()
                .and_then(|spec| spec.selector.as_ref())
                .map_or(false, |selector| selector.contains_key("app"))
        }).map(|svc| {
            let name = svc.metadata.name.unwrap();
            let port = extract_port(svc.spec);
            KubeService::new(name, port)
        }).collect();

    Ok(kube_services)
}

async fn prompt_for_service(service_api: &Api<Service>) -> Result<KubeService, ArcError> {
    let available_services = get_app_services(&service_api).await?;

    let mut menu = select("Select a service for port-forwarding");
    for svc in &available_services {
        menu = menu.item(&svc.name, &svc.name, "");
    }

    let selected_name = menu.interact()?;

    // Find the KubeService that matches the selected name
    let kube_service = available_services
        .iter()
        .find(|svc| &svc.name == selected_name)
        .expect("Selected service not found in available services")
        .clone();

    Ok(kube_service)
}

async fn get_service_port(service_api: &Api<Service>, service_name: &str) -> u16 {
    let svc = service_api.get(service_name).await
        .unwrap_or_else(|_| panic!("Failed to get service: {service_name}"));
    extract_port(svc.spec)
}

fn extract_port(spec: Option<ServiceSpec>) -> u16 {
    spec.as_ref()
        .and_then(|spec| spec.ports.as_ref())
        .and_then(|ports| ports.first())
        .map_or(0, |port| port.port as u16)
}

async fn get_service_pod(service_name: &str, service_api: &Api<Service>, pod_api: &Api<Pod>) -> Result<String, ArcError> {
    // Get the selector label for the given service so that we can find its pods
    let selector_label = get_selector_label(service_name, service_api).await?;

    // List pods matching the service selector
    //TODO return Selector from get_selector_label and then call labels_from(Selector)
    let list_params = ListParams::default().labels(&selector_label);
    let pod_list = pod_api.list(&list_params).await?;

    // Return the name of the first pod found
    pod_list.items.first()
        .and_then(|pod| pod.metadata.name.clone())
        .ok_or_else(|| ArcError::KubePodError(selector_label))
}

async fn get_selector_label(service_name: &str, service_api: &Api<Service>) -> Result<String, ArcError> {
    let service = service_api.get(service_name).await?;

    // Extract selector labels from the service
    let selector = service.spec
        .and_then(|spec| spec.selector)
        .ok_or_else(|| ArcError::KubeServiceSpecError(service_name.to_string()))?;

    // Return label selector string (e.g., "app=metrics")
    let selector_label = selector
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    Ok(selector_label)
}

async fn find_available_port() -> Result<u16, ArcError> {
    // Bind to port 0, which lets the OS assign an available port
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();

    // Drop the listener to free the port
    drop(listener);
    Ok(port)
}

async fn port_forward(
    pod_name: &str,
    local_port: u16,
    pod_port: u16,
    pod_api: &Api<Pod>,
) -> Result<(), ArcError> {
    // Bind local TCP listener
    let listener = TcpListener::bind(("127.0.0.1", local_port)).await?;

    loop {
        let (local_stream, _) = listener.accept().await?;
        let pod_api = pod_api.clone();
        let pod_name = pod_name.to_string();

        tokio::spawn(async move {
            // Create port-forward connection to the pod
            let port_forward_stream = match pod_api
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

            //TODO pull this duplicate code into a reusable function
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

