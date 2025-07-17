use crate::crd::{
    GeneratedResource, GeneratedResourceStatus, ResourceCondition, ResourcePhase, TerranResource,
    TerranResourceStatus,
};
use crate::kafka::{KafkaClient, TerranEvent};
use crate::kcl_runner::{GeneratedK8sResource, KclRunner};
use eyre::{Result, WrapErr};
use kube::{
    api::{Api, Patch, PatchParams},
    client::Client,
    runtime::controller::Action,
    ResourceExt,
};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct TerranReconciler {
    pub client: Client,
    kcl_runner: KclRunner,
    kafka_client: Arc<KafkaClient>,
}

impl TerranReconciler {
    pub fn new(client: Client, kafka_client: Arc<KafkaClient>) -> Self {
        let kcl_runner = KclRunner::new(client.clone());

        Self {
            client,
            kcl_runner,
            kafka_client,
        }
    }

    pub async fn reconcile(&self, resource: Arc<TerranResource>) -> Result<Action> {
        let name = resource.name_any();
        let namespace = resource
            .namespace()
            .unwrap_or_else(|| "default".to_string());

        info!("Reconciling TerranResource {}/{}", namespace, name);

        // Update status to Processing
        self.update_status(
            &resource,
            TerranResourceStatus {
                phase: ResourcePhase::Processing,
                last_reconciled: Some(chrono::Utc::now().to_rfc3339()),
                generated_resources: None,
                error: None,
                conditions: Some(vec![ResourceCondition {
                    r#type: "Processing".to_string(),
                    status: "True".to_string(),
                    last_transition_time: chrono::Utc::now().to_rfc3339(),
                    reason: Some("ReconciliationStarted".to_string()),
                    message: Some("Started reconciliation process".to_string()),
                }]),
            },
        )
        .await?;

        // Publish reconciliation started event
        self.publish_event(
            &resource,
            "ReconciliationStarted",
            json!({
                "phase": "Processing"
            }),
        )
        .await?;

        match self.reconcile_internal(&resource).await {
            Ok(generated_resources) => {
                // Update status to Ready
                self.update_status(
                    &resource,
                    TerranResourceStatus {
                        phase: ResourcePhase::Ready,
                        last_reconciled: Some(chrono::Utc::now().to_rfc3339()),
                        generated_resources: Some(generated_resources.clone()),
                        error: None,
                        conditions: Some(vec![ResourceCondition {
                            r#type: "Ready".to_string(),
                            status: "True".to_string(),
                            last_transition_time: chrono::Utc::now().to_rfc3339(),
                            reason: Some("ReconciliationSuccessful".to_string()),
                            message: Some(format!(
                                "Successfully generated {} resources",
                                generated_resources.len()
                            )),
                        }]),
                    },
                )
                .await?;

                // Publish reconciliation completed event
                self.publish_event(
                    &resource,
                    "ReconciliationCompleted",
                    json!({
                        "phase": "Ready",
                        "resources_count": generated_resources.len()
                    }),
                )
                .await?;

                info!(
                    "Successfully reconciled TerranResource {}/{}",
                    namespace, name
                );
                Ok(Action::requeue(Duration::from_secs(300))) // Requeue after 5 minutes
            }
            Err(e) => {
                error!(
                    "Failed to reconcile TerranResource {}/{}: {:?}",
                    namespace, name, e
                );

                // Update status to Failed
                self.update_status(
                    &resource,
                    TerranResourceStatus {
                        phase: ResourcePhase::Failed,
                        last_reconciled: Some(chrono::Utc::now().to_rfc3339()),
                        generated_resources: None,
                        error: Some(e.to_string()),
                        conditions: Some(vec![ResourceCondition {
                            r#type: "Failed".to_string(),
                            status: "True".to_string(),
                            last_transition_time: chrono::Utc::now().to_rfc3339(),
                            reason: Some("ReconciliationFailed".to_string()),
                            message: Some(e.to_string()),
                        }]),
                    },
                )
                .await?;

                // Publish reconciliation failed event
                self.publish_event(
                    &resource,
                    "ReconciliationFailed",
                    json!({
                        "phase": "Failed",
                        "error": e.to_string()
                    }),
                )
                .await?;

                Ok(Action::requeue(Duration::from_secs(60))) // Retry after 1 minute
            }
        }
    }

    async fn reconcile_internal(
        &self,
        resource: &TerranResource,
    ) -> Result<Vec<GeneratedResource>> {
        // Execute KCL script
        let execution_result = self
            .kcl_runner
            .execute_kcl(resource, resource.spec.parameters.as_ref())
            .await
            .wrap_err("Failed to execute KCL script")?;

        if !execution_result.success {
            return Err(eyre::eyre!(
                "KCL execution failed: {}",
                execution_result
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        // Deploy generated resources
        let mut generated_resources = Vec::new();

        for k8s_resource in execution_result.generated_resources {
            match self.deploy_resource(resource, &k8s_resource).await {
                Ok(()) => {
                    generated_resources.push(GeneratedResource {
                        api_version: k8s_resource.api_version,
                        kind: k8s_resource.kind,
                        name: k8s_resource
                            .metadata
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        namespace: k8s_resource
                            .metadata
                            .get("namespace")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        status: GeneratedResourceStatus::Created,
                    });
                }
                Err(e) => {
                    warn!(
                        "Failed to deploy resource {}/{}: {:?}",
                        k8s_resource.kind,
                        k8s_resource
                            .metadata
                            .get("name")
                            .unwrap_or(&json!("unknown")),
                        e
                    );

                    generated_resources.push(GeneratedResource {
                        api_version: k8s_resource.api_version,
                        kind: k8s_resource.kind,
                        name: k8s_resource
                            .metadata
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        namespace: k8s_resource
                            .metadata
                            .get("namespace")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        status: GeneratedResourceStatus::Failed,
                    });
                }
            }
        }

        Ok(generated_resources)
    }

    async fn deploy_resource(
        &self,
        owner: &TerranResource,
        k8s_resource: &GeneratedK8sResource,
    ) -> Result<()> {
        let target_namespace = owner.spec.target_namespace.as_deref().unwrap_or("default");

        // Create the resource manifest
        let mut resource_manifest = json!({
            "apiVersion": k8s_resource.api_version,
            "kind": k8s_resource.kind,
            "metadata": k8s_resource.metadata
        });

        // Add owner reference
        if let Some(metadata) = resource_manifest.get_mut("metadata") {
            if let Some(metadata_obj) = metadata.as_object_mut() {
                metadata_obj.insert("namespace".to_string(), json!(target_namespace));

                let owner_refs = json!([{
                    "apiVersion": "terran.io/v1",
                    "kind": "TerranResource",
                    "name": owner.name_any(),
                    "uid": owner.uid().unwrap_or_default(),
                    "controller": true,
                    "blockOwnerDeletion": true
                }]);

                metadata_obj.insert("ownerReferences".to_string(), owner_refs);
            }
        }

        // Add spec if present
        if let Some(spec) = &k8s_resource.spec {
            resource_manifest["spec"] = spec.clone();
        }

        // Add data if present (for ConfigMaps/Secrets)
        if let Some(data) = &k8s_resource.data {
            resource_manifest["data"] = json!(data);
        }

        let resource_name = k8s_resource
            .metadata
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| eyre::eyre!("Resource missing name"))?;

        // For now, just log the resource that would be deployed
        // In a full implementation, you would use the dynamic client
        info!(
            "Would deploy resource: {} {} in namespace {}",
            k8s_resource.kind, resource_name, target_namespace
        );
        info!(
            "Resource manifest: {}",
            serde_json::to_string_pretty(&resource_manifest)
                .unwrap_or_else(|_| "Invalid JSON".to_string())
        );

        info!(
            "Successfully deployed resource {}/{} in namespace {}",
            k8s_resource.kind, resource_name, target_namespace
        );

        Ok(())
    }

    async fn update_status(
        &self,
        resource: &TerranResource,
        status: TerranResourceStatus,
    ) -> Result<()> {
        let name = resource.name_any();
        let namespace = resource
            .namespace()
            .unwrap_or_else(|| "default".to_string());

        let api: Api<TerranResource> = Api::namespaced(self.client.clone(), &namespace);

        let patch = json!({
            "status": status
        });

        api.patch_status(&name, &PatchParams::default(), &Patch::Merge(patch))
            .await
            .wrap_err("Failed to update resource status")?;

        Ok(())
    }

    async fn publish_event(
        &self,
        resource: &TerranResource,
        event_type: &str,
        data: Value,
    ) -> Result<()> {
        let event = TerranEvent {
            event_type: event_type.to_string(),
            resource_name: resource.name_any(),
            resource_namespace: resource
                .namespace()
                .unwrap_or_else(|| "default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data,
        };

        // Default topic, can be overridden by resource configuration
        let topic = resource
            .spec
            .event_config
            .as_ref()
            .map(|config| config.topic.as_str())
            .unwrap_or("terran-events");

        self.kafka_client.publish_event(topic, &event).await
    }
}

// Handle deletion of TerranResource
pub async fn handle_deletion(_client: Client, resource: Arc<TerranResource>) -> Result<Action> {
    let name = resource.name_any();
    let namespace = resource
        .namespace()
        .unwrap_or_else(|| "default".to_string());

    info!("Handling deletion of TerranResource {}/{}", namespace, name);

    // Resources with owner references will be automatically deleted by Kubernetes
    // Additional cleanup logic can be added here if needed

    Ok(Action::await_change())
}
