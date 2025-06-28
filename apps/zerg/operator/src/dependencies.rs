use anyhow::Result;
use kube::Client;
use std::process::Command;
use tracing::{info, instrument, warn};

use crate::crd::{Dependency, DependencyStatus, DependencyInstallStatus, DependencyType};
use crate::error::Error;

pub struct DependencyInstaller {
    client: Client,
}

impl DependencyInstaller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
    
    #[instrument(skip(self))]
    pub async fn install_dependency(
        &self,
        dependency: &Dependency,
        namespace: &str,
    ) -> Result<DependencyStatus, Error> {
        info!("Installing dependency: {} of type: {:?}", dependency.name, dependency.type_);
        
        let result = match dependency.type_ {
            DependencyType::Helm => self.install_helm_chart(dependency, namespace).await,
            DependencyType::Kustomize => self.install_kustomize(dependency, namespace).await,
            DependencyType::Yaml => self.install_yaml_manifests(dependency, namespace).await,
            DependencyType::Operator => self.install_operator(dependency, namespace).await,
        };
        
        match result {
            Ok(version) => Ok(DependencyStatus {
                name: dependency.name.clone(),
                status: DependencyInstallStatus::Installed,
                version: Some(version),
                last_updated: Some(chrono::Utc::now().to_rfc3339()),
                error: None,
            }),
            Err(e) => Ok(DependencyStatus {
                name: dependency.name.clone(),
                status: DependencyInstallStatus::Failed,
                version: None,
                last_updated: Some(chrono::Utc::now().to_rfc3339()),
                error: Some(e.to_string()),
            }),
        }
    }
    
    #[instrument(skip(self))]
    async fn install_helm_chart(
        &self,
        dependency: &Dependency,
        namespace: &str,
    ) -> Result<String, Error> {
        info!("Installing Helm chart: {}", dependency.name);
        
        let chart_name = dependency.source.chart
            .as_ref()
            .ok_or_else(|| Error::ConfigError("Chart name required for Helm dependency".to_string()))?;
        
        let target_namespace = dependency.namespace.as_deref().unwrap_or(namespace);
        
        // Add Helm repository
        let mut add_repo_cmd = Command::new("helm");
        add_repo_cmd
            .args(&["repo", "add", &dependency.name, &dependency.source.repo])
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to add Helm repo: {}", e)))?;
        
        // Update repositories
        let mut update_cmd = Command::new("helm");
        update_cmd
            .args(&["repo", "update"])
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to update Helm repos: {}", e)))?;
        
        // Build install command
        let mut install_cmd = Command::new("helm");
        install_cmd.args(&[
            "install",
            &dependency.name,
            &format!("{}/{}", dependency.name, chart_name),
            "--namespace",
            target_namespace,
            "--create-namespace",
        ]);
        
        // Add version if specified
        if let Some(version) = &dependency.version {
            install_cmd.args(&["--version", version]);
        }
        
        // Add values if specified
        if let Some(values) = &dependency.values {
            let values_yaml = serde_yaml::to_string(values)
                .map_err(|e| Error::SerializationError(format!("Failed to serialize values: {}", e)))?;
            
            // Write values to temporary file
            let values_file = format!("/tmp/values-{}.yaml", dependency.name);
            std::fs::write(&values_file, values_yaml)
                .map_err(|e| Error::IoError(format!("Failed to write values file: {}", e)))?;
            
            install_cmd.args(&["--values", &values_file]);
        }
        
        // Execute install command
        let output = install_cmd
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to execute helm install: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandError(format!("Helm install failed: {}", stderr)));
        }
        
        Ok(dependency.version.clone().unwrap_or_else(|| "latest".to_string()))
    }
    
    #[instrument(skip(self))]
    async fn install_kustomize(
        &self,
        dependency: &Dependency,
        namespace: &str,
    ) -> Result<String, Error> {
        info!("Installing Kustomize resources: {}", dependency.name);
        
        let path = dependency.source.path
            .as_ref()
            .unwrap_or(&dependency.source.repo);
        
        let mut cmd = Command::new("kubectl");
        cmd.args(&["apply", "-k", path]);
        
        if let Some(ns) = &dependency.namespace {
            cmd.args(&["--namespace", ns]);
        } else {
            cmd.args(&["--namespace", namespace]);
        }
        
        let output = cmd
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to execute kubectl apply: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandError(format!("Kustomize apply failed: {}", stderr)));
        }
        
        Ok("applied".to_string())
    }
    
    #[instrument(skip(self))]
    async fn install_yaml_manifests(
        &self,
        dependency: &Dependency,
        namespace: &str,
    ) -> Result<String, Error> {
        info!("Installing YAML manifests: {}", dependency.name);
        
        let mut cmd = Command::new("kubectl");
        cmd.args(&["apply", "-f", &dependency.source.repo]);
        
        if let Some(ns) = &dependency.namespace {
            cmd.args(&["--namespace", ns]);
        } else {
            cmd.args(&["--namespace", namespace]);
        }
        
        let output = cmd
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to execute kubectl apply: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandError(format!("YAML apply failed: {}", stderr)));
        }
        
        Ok("applied".to_string())
    }
    
    #[instrument(skip(self))]
    async fn install_operator(
        &self,
        dependency: &Dependency,
        namespace: &str,
    ) -> Result<String, Error> {
        info!("Installing Operator: {}", dependency.name);
        
        // For operators, we typically use OLM (Operator Lifecycle Manager)
        // or install via Helm charts
        
        match dependency.name.as_str() {
            "external-secrets" => self.install_external_secrets(namespace).await,
            "crossplane" => self.install_crossplane(namespace).await,
            "loki" => self.install_loki_operator(namespace).await,
            _ => {
                warn!("Unknown operator: {}, attempting generic installation", dependency.name);
                self.install_generic_operator(dependency, namespace).await
            }
        }
    }
    
    #[instrument(skip(self))]
    async fn install_external_secrets(&self, namespace: &str) -> Result<String, Error> {
        info!("Installing External Secrets Operator");
        
        let mut cmd = Command::new("helm");
        cmd.args(&[
            "repo", "add", "external-secrets", "https://charts.external-secrets.io"
        ]);
        cmd.output().map_err(|e| Error::CommandError(format!("Failed to add external-secrets repo: {}", e)))?;
        
        let mut install_cmd = Command::new("helm");
        install_cmd.args(&[
            "install", "external-secrets", "external-secrets/external-secrets",
            "--namespace", "external-secrets-system",
            "--create-namespace"
        ]);
        
        let output = install_cmd
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to install external-secrets: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandError(format!("External Secrets install failed: {}", stderr)));
        }
        
        Ok("installed".to_string())
    }
    
    #[instrument(skip(self))]
    async fn install_crossplane(&self, namespace: &str) -> Result<String, Error> {
        info!("Installing Crossplane");
        
        let mut cmd = Command::new("helm");
        cmd.args(&[
            "repo", "add", "crossplane-stable", "https://charts.crossplane.io/stable"
        ]);
        cmd.output().map_err(|e| Error::CommandError(format!("Failed to add crossplane repo: {}", e)))?;
        
        let mut install_cmd = Command::new("helm");
        install_cmd.args(&[
            "install", "crossplane", "crossplane-stable/crossplane",
            "--namespace", "crossplane-system",
            "--create-namespace"
        ]);
        
        let output = install_cmd
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to install crossplane: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandError(format!("Crossplane install failed: {}", stderr)));
        }
        
        Ok("installed".to_string())
    }
    
    #[instrument(skip(self))]
    async fn install_loki_operator(&self, namespace: &str) -> Result<String, Error> {
        info!("Installing Loki Operator");
        
        let mut cmd = Command::new("helm");
        cmd.args(&[
            "repo", "add", "grafana", "https://grafana.github.io/helm-charts"
        ]);
        cmd.output().map_err(|e| Error::CommandError(format!("Failed to add grafana repo: {}", e)))?;
        
        let mut install_cmd = Command::new("helm");
        install_cmd.args(&[
            "install", "loki", "grafana/loki-stack",
            "--namespace", "loki-system",
            "--create-namespace"
        ]);
        
        let output = install_cmd
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to install loki: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandError(format!("Loki install failed: {}", stderr)));
        }
        
        Ok("installed".to_string())
    }
    
    #[instrument(skip(self))]
    async fn install_generic_operator(
        &self,
        dependency: &Dependency,
        namespace: &str,
    ) -> Result<String, Error> {
        // Attempt to install as Helm chart first
        if dependency.source.chart.is_some() {
            self.install_helm_chart(dependency, namespace).await
        } else {
            // Fall back to YAML manifests
            self.install_yaml_manifests(dependency, namespace).await
        }
    }
}