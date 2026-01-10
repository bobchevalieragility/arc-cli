use aws_runtime::env_config::error::EnvConfigFileLoadError;
use aws_sdk_secretsmanager::config::http::HttpResponse;
use aws_sdk_secretsmanager::error::SdkError;
use aws_sdk_secretsmanager::operation::get_secret_value::GetSecretValueError;
use aws_sdk_secretsmanager::operation::list_secrets::ListSecretsError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ArcError {
    #[error("AWS Config Error: {0}")]
    AwsEnvConfigError(#[from] EnvConfigFileLoadError),

    #[error("AWS SDK error: {0}")]
    AwsGetSecretError(#[from] SdkError<GetSecretValueError, HttpResponse>),

    #[error("AWS SDK error: {0}")]
    AwsListSecretError(#[from] SdkError<ListSecretsError, HttpResponse>),

    #[error("Aws Profile Error: {0}")]
    AwsProfileError(String),

    #[error("Error: {0}")]
    Error(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Missing TaskResult for goal: {0}")]
    InsufficientState(String),

    #[error("Expected: {0}, actual: {1}")]
    InvalidArcCommand(String, String),

    #[error("Invalid secret: {0}")]
    InvalidSecret(String),

    #[error("Invalid TaskResult for goal: {0}. Expected: {1}, Actual: {2}")]
    InvalidState(String, String, String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Kubernetes Config error: {0}")]
    KubeconfigError(#[from] kube::config::KubeconfigError),

    #[error("Kube Context Error: {0}")]
    KubeContextError(String),

    #[error("Kubernetes error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("Unable to find any pods matching service selector: {0}")]
    KubePodError(String),

    #[error("Unable to lookup Kube Service spec: {0}")]
    KubeServiceSpecError(String),

    #[error("Path error: {0}")]
    PathError(String),

    #[error("Tokio Join error: {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),

    #[error("URL Parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Unable to extract query param: {1}, from URL: {0}")]
    UrlQueryParamError(Url, String),

    #[error("Vault error: {0}")]
    VaultError(#[from] vaultrs::error::ClientError),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}