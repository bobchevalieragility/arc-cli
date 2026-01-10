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

    #[error("Secret field missing or not a string: {0}")]
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

    #[error("Could not determine home directory")]
    HomeDirError,

    #[error("Tokio Join error: {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),

    #[error("Unable to parse secret as string: {0}")]
    UnparseableSecret(String),

    #[error("URL Parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Unable to extract query param: {1}, from URL: {0}")]
    UrlQueryParamError(Url, String),

    #[error("Vault error: {0}")]
    VaultError(#[from] vaultrs::error::ClientError),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

impl ArcError {
    pub fn insufficient_state(goal: impl Into<String>) -> Self {
        ArcError::InsufficientState(goal.into())
    }

    pub fn invalid_arc_command(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        ArcError::InvalidArcCommand(expected.into(), actual.into())
    }

    pub fn invalid_secret(field: impl Into<String>) -> Self {
        ArcError::InvalidSecret(field.into())
    }

    pub fn invalid_state(goal: impl Into<String>, expected: impl Into<String>, actual: impl Into<String>) -> Self {
        ArcError::InvalidState(goal.into(), expected.into(), actual.into())
    }

    pub fn kube_context_error(msg: impl Into<String>) -> Self {
        ArcError::KubeContextError(msg.into())
    }
}