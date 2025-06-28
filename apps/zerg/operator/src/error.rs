use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Command execution error: {0}")]
    CommandError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Finalizer error: {0}")]
    FinalizerError(Box<dyn std::error::Error + Send + Sync>),
    
    #[error("Dependency error: {0}")]
    DependencyError(String),
    
    #[error("GitOps error: {0}")]
    GitOpsError(String),
    
    #[error("CI/CD error: {0}")]
    CiCdError(String),
}