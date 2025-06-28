use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Operator configuration
    pub operator: OperatorConfig,
    
    /// Default dependency templates
    pub dependency_templates: HashMap<String, DependencyTemplate>,
    
    /// GitOps configuration templates
    pub gitops_templates: HashMap<String, GitOpsTemplate>,
    
    /// CI/CD configuration templates
    pub cicd_templates: HashMap<String, CiCdTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorConfig {
    /// Default namespace for operator operations
    pub default_namespace: String,
    
    /// Reconciliation interval in seconds
    pub reconciliation_interval: u64,
    
    /// Maximum concurrent reconciliations
    pub max_concurrent_reconciles: usize,
    
    /// Enable metrics
    pub metrics_enabled: bool,
    
    /// Metrics port
    pub metrics_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTemplate {
    /// Template name
    pub name: String,
    
    /// Template description
    pub description: String,
    
    /// Dependency type
    pub type_: String,
    
    /// Source repository
    pub repo: String,
    
    /// Default chart name (for Helm)
    pub chart: Option<String>,
    
    /// Default version
    pub version: Option<String>,
    
    /// Default namespace
    pub namespace: Option<String>,
    
    /// Default values
    pub values: Option<HashMap<String, serde_json::Value>>,
    
    /// Prerequisites
    pub prerequisites: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitOpsTemplate {
    /// Template name
    pub name: String,
    
    /// GitOps provider
    pub provider: String,
    
    /// Default configuration
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiCdTemplate {
    /// Template name
    pub name: String,
    
    /// CI/CD provider
    pub provider: String,
    
    /// Default pipeline configuration
    pub config: HashMap<String, serde_json::Value>,
}

impl Default for Config {
    fn default() -> Self {
        let mut dependency_templates = HashMap::new();
        
        // External Secrets template
        dependency_templates.insert(
            "external-secrets".to_string(),
            DependencyTemplate {
                name: "external-secrets".to_string(),
                description: "External Secrets Operator for managing secrets from external systems".to_string(),
                type_: "operator".to_string(),
                repo: "https://charts.external-secrets.io".to_string(),
                chart: Some("external-secrets".to_string()),
                version: Some("0.9.0".to_string()),
                namespace: Some("external-secrets-system".to_string()),
                values: None,
                prerequisites: None,
            },
        );
        
        // Crossplane template
        dependency_templates.insert(
            "crossplane".to_string(),
            DependencyTemplate {
                name: "crossplane".to_string(),
                description: "Crossplane for infrastructure as code".to_string(),
                type_: "operator".to_string(),
                repo: "https://charts.crossplane.io/stable".to_string(),
                chart: Some("crossplane".to_string()),
                version: Some("1.14.0".to_string()),
                namespace: Some("crossplane-system".to_string()),
                values: None,
                prerequisites: None,
            },
        );
        
        // Loki template
        dependency_templates.insert(
            "loki".to_string(),
            DependencyTemplate {
                name: "loki".to_string(),
                description: "Loki logging stack".to_string(),
                type_: "helm".to_string(),
                repo: "https://grafana.github.io/helm-charts".to_string(),
                chart: Some("loki-stack".to_string()),
                version: Some("2.9.0".to_string()),
                namespace: Some("loki-system".to_string()),
                values: Some({
                    let mut values = HashMap::new();
                    values.insert("grafana.enabled".to_string(), serde_json::Value::Bool(true));
                    values.insert("prometheus.enabled".to_string(), serde_json::Value::Bool(true));
                    values
                }),
                prerequisites: None,
            },
        );
        
        // Prometheus template
        dependency_templates.insert(
            "prometheus".to_string(),
            DependencyTemplate {
                name: "prometheus".to_string(),
                description: "Prometheus monitoring stack".to_string(),
                type_: "helm".to_string(),
                repo: "https://prometheus-community.github.io/helm-charts".to_string(),
                chart: Some("kube-prometheus-stack".to_string()),
                version: Some("55.0.0".to_string()),
                namespace: Some("monitoring".to_string()),
                values: None,
                prerequisites: None,
            },
        );
        
        // Cert-Manager template
        dependency_templates.insert(
            "cert-manager".to_string(),
            DependencyTemplate {
                name: "cert-manager".to_string(),
                description: "Certificate management for Kubernetes".to_string(),
                type_: "helm".to_string(),
                repo: "https://charts.jetstack.io".to_string(),
                chart: Some("cert-manager".to_string()),
                version: Some("v1.13.0".to_string()),
                namespace: Some("cert-manager".to_string()),
                values: Some({
                    let mut values = HashMap::new();
                    values.insert("installCRDs".to_string(), serde_json::Value::Bool(true));
                    values
                }),
                prerequisites: None,
            },
        );
        
        let mut gitops_templates = HashMap::new();
        
        // Flux template
        gitops_templates.insert(
            "flux".to_string(),
            GitOpsTemplate {
                name: "flux".to_string(),
                provider: "flux".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert("interval".to_string(), serde_json::Value::String("5m".to_string()));
                    config.insert("prune".to_string(), serde_json::Value::Bool(true));
                    config
                },
            },
        );
        
        // ArgoCD template
        gitops_templates.insert(
            "argocd".to_string(),
            GitOpsTemplate {
                name: "argocd".to_string(),
                provider: "argocd".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert("automated".to_string(), serde_json::Value::Bool(true));
                    config.insert("prune".to_string(), serde_json::Value::Bool(true));
                    config.insert("selfHeal".to_string(), serde_json::Value::Bool(true));
                    config
                },
            },
        );
        
        let mut cicd_templates = HashMap::new();
        
        // Tekton template
        cicd_templates.insert(
            "tekton".to_string(),
            CiCdTemplate {
                name: "tekton".to_string(),
                provider: "tekton".to_string(),
                config: HashMap::new(),
            },
        );
        
        // Argo Workflows template
        cicd_templates.insert(
            "argo-workflows".to_string(),
            CiCdTemplate {
                name: "argo-workflows".to_string(),
                provider: "argo-workflows".to_string(),
                config: HashMap::new(),
            },
        );
        
        Self {
            operator: OperatorConfig {
                default_namespace: "zerg-system".to_string(),
                reconciliation_interval: 300,
                max_concurrent_reconciles: 5,
                metrics_enabled: true,
                metrics_port: 8080,
            },
            dependency_templates,
            gitops_templates,
            cicd_templates,
        }
    }
}

pub async fn load_config(path: &str) -> Result<Config, Error> {
    info!("Loading configuration from: {}", path);
    
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let config: Config = serde_yaml::from_str(&content)
                .map_err(|e| Error::ConfigError(format!("Failed to parse config file: {}", e)))?;
            Ok(config)
        }
        Err(_) => {
            info!("Config file not found, using default configuration");
            Ok(Config::default())
        }
    }
}