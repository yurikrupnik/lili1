use crate::crd::{KclConfig, KclSource, TerranResource};
use eyre::{Result, WrapErr};
use k8s_openapi::api::core::v1::{ConfigMap, Secret};
use kube::{api::Api, client::Client, ResourceExt};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
#[allow(unused_imports)]
use std::process::{Command, Stdio};
use tempfile::TempDir;
use tracing::{info, warn};

#[derive(Clone)]
pub struct KclRunner {
    client: Client,
}

#[derive(Debug)]
pub struct KclExecutionResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
    pub generated_resources: Vec<GeneratedK8sResource>,
}

#[derive(Debug, Clone)]
pub struct GeneratedK8sResource {
    pub api_version: String,
    pub kind: String,
    pub metadata: BTreeMap<String, Value>,
    pub spec: Option<Value>,
    pub data: Option<BTreeMap<String, String>>,
}

impl KclRunner {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn execute_kcl(
        &self,
        resource: &TerranResource,
        _parameters: Option<&BTreeMap<String, String>>,
    ) -> Result<KclExecutionResult> {
        let kcl_config = &resource.spec.kcl_config;

        // Create temporary directory for KCL execution
        let temp_dir = TempDir::new().wrap_err("Failed to create temporary directory")?;

        // Get KCL script content
        let script_content = self.get_kcl_script_content(kcl_config, resource).await?;

        // Write script to temporary file
        let script_path = temp_dir.path().join("main.k");
        fs::write(&script_path, script_content).wrap_err("Failed to write KCL script to file")?;

        // For now, simulate KCL execution since the actual KCL binary might not be available
        // In production, you would execute the actual KCL command
        info!(
            "Simulating KCL execution for resource {}",
            resource.name_any()
        );

        // Generate mock Kubernetes resources based on the script content
        let generated_resources = self.generate_mock_resources(resource)?;

        Ok(KclExecutionResult {
            success: true,
            output: serde_json::json!({
                "resources": generated_resources.len(),
                "status": "success"
            }),
            error: None,
            generated_resources,
        })
    }

    async fn get_kcl_script_content(
        &self,
        kcl_config: &KclConfig,
        resource: &TerranResource,
    ) -> Result<String> {
        match &kcl_config.source {
            KclSource::Inline { script } => Ok(script.clone()),

            KclSource::ConfigMap { name, key } => {
                let namespace = resource.namespace().unwrap_or("default".to_string());
                let configmaps: Api<ConfigMap> = Api::namespaced(self.client.clone(), &namespace);

                let configmap = configmaps
                    .get(name)
                    .await
                    .wrap_err(format!("Failed to get ConfigMap {}/{}", namespace, name))?;

                configmap
                    .data
                    .and_then(|data| data.get(key).cloned())
                    .ok_or_else(|| {
                        eyre::eyre!(
                            "Key '{}' not found in ConfigMap {}/{}",
                            key,
                            namespace,
                            name
                        )
                    })
            }

            KclSource::Secret { name, key } => {
                let namespace = resource.namespace().unwrap_or("default".to_string());
                let secrets: Api<Secret> = Api::namespaced(self.client.clone(), &namespace);

                let secret = secrets
                    .get(name)
                    .await
                    .wrap_err(format!("Failed to get Secret {}/{}", namespace, name))?;

                secret
                    .data
                    .and_then(|data| data.get(key).cloned())
                    .and_then(|bytes| String::from_utf8(bytes.0).ok())
                    .ok_or_else(|| {
                        eyre::eyre!("Key '{}' not found in Secret {}/{}", key, namespace, name)
                    })
            }

            KclSource::Git {
                repository: _,
                path: _,
                branch: _,
            } => {
                // For now, return an error as Git source requires more complex implementation
                Err(eyre::eyre!("Git source not yet implemented"))
            }
        }
    }

    fn generate_mock_resources(
        &self,
        resource: &TerranResource,
    ) -> Result<Vec<GeneratedK8sResource>> {
        // Generate mock resources for demonstration
        // In a real implementation, this would be based on actual KCL execution
        let resource_name = format!("{}-generated", resource.name_any());
        let namespace = resource.namespace().unwrap_or("default".to_string());

        let mut resources = Vec::new();

        // Generate a ConfigMap
        let mut cm_metadata = BTreeMap::new();
        cm_metadata.insert(
            "name".to_string(),
            Value::String(format!("{}-config", resource_name)),
        );
        cm_metadata.insert("namespace".to_string(), Value::String(namespace.clone()));

        let mut cm_data = BTreeMap::new();
        cm_data.insert(
            "config.yaml".to_string(),
            "# Generated configuration".to_string(),
        );

        resources.push(GeneratedK8sResource {
            api_version: "v1".to_string(),
            kind: "ConfigMap".to_string(),
            metadata: cm_metadata,
            spec: None,
            data: Some(cm_data),
        });

        // Generate a Deployment
        let mut deploy_metadata = BTreeMap::new();
        deploy_metadata.insert(
            "name".to_string(),
            Value::String(format!("{}-deployment", resource_name)),
        );
        deploy_metadata.insert("namespace".to_string(), Value::String(namespace));

        let spec = serde_json::json!({
            "replicas": 1,
            "selector": {
                "matchLabels": {
                    "app": resource_name
                }
            },
            "template": {
                "metadata": {
                    "labels": {
                        "app": resource_name
                    }
                },
                "spec": {
                    "containers": [{
                        "name": "app",
                        "image": "nginx:latest",
                        "ports": [{
                            "containerPort": 80
                        }]
                    }]
                }
            }
        });

        resources.push(GeneratedK8sResource {
            api_version: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            metadata: deploy_metadata,
            spec: Some(spec),
            data: None,
        });

        Ok(resources)
    }

    fn extract_k8s_resources(&self, value: &Value) -> Result<Vec<GeneratedK8sResource>> {
        let mut resources = Vec::new();

        match value {
            Value::Object(obj) => {
                // Single resource
                if let Some(resource) = self.parse_k8s_resource(obj)? {
                    resources.push(resource);
                }
            }
            Value::Array(arr) => {
                // Multiple resources
                for item in arr {
                    if let Value::Object(obj) = item {
                        if let Some(resource) = self.parse_k8s_resource(obj)? {
                            resources.push(resource);
                        }
                    }
                }
            }
            _ => {
                warn!("KCL output is not a valid Kubernetes resource format");
            }
        }

        Ok(resources)
    }

    fn parse_k8s_resource(
        &self,
        obj: &serde_json::Map<String, Value>,
    ) -> Result<Option<GeneratedK8sResource>> {
        // Check if this looks like a Kubernetes resource
        let api_version = match obj.get("apiVersion") {
            Some(Value::String(v)) => v.clone(),
            _ => return Ok(None),
        };

        let kind = match obj.get("kind") {
            Some(Value::String(k)) => k.clone(),
            _ => return Ok(None),
        };

        let metadata = obj
            .get("metadata")
            .and_then(|m| m.as_object())
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        let spec = obj.get("spec").cloned();

        let data = obj.get("data").and_then(|d| d.as_object()).map(|d| {
            d.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        });

        Ok(Some(GeneratedK8sResource {
            api_version,
            kind,
            metadata,
            spec,
            data,
        }))
    }
}
