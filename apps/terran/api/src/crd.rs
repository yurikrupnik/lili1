use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "terran.io",
    version = "v1",
    kind = "TerranResource",
    plural = "terranresources",
    namespaced,
    derive = "Default",
    status = "TerranResourceStatus",
    printcolumn = r#"{"name":"Phase", "type":"string", "jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#
)]
pub struct TerranResourceSpec {
    /// KCL script configuration
    pub kcl_config: KclConfig,
    /// Target namespace for resource deployment
    pub target_namespace: Option<String>,
    /// Additional parameters for KCL execution
    pub parameters: Option<BTreeMap<String, String>>,
    /// Event configuration for Kafka integration
    pub event_config: Option<EventConfig>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct KclConfig {
    /// KCL script source (inline or reference)
    pub source: KclSource,
    /// Entry point function name
    pub entry_point: Option<String>,
    /// KCL module dependencies
    pub dependencies: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(tag = "type")]
pub enum KclSource {
    #[serde(rename = "inline")]
    Inline { script: String },
    #[serde(rename = "configmap")]
    ConfigMap { name: String, key: String },
    #[serde(rename = "secret")]
    Secret { name: String, key: String },
    #[serde(rename = "git")]
    Git {
        repository: String,
        path: String,
        branch: Option<String>,
    },
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct EventConfig {
    /// Kafka topic for events
    pub topic: String,
    /// Event types to listen for
    pub event_types: Vec<String>,
    /// Kafka consumer group
    pub consumer_group: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct TerranResourceStatus {
    /// Current phase of the resource
    pub phase: ResourcePhase,
    /// Last reconciliation timestamp
    pub last_reconciled: Option<String>,
    /// Generated resources from KCL execution
    pub generated_resources: Option<Vec<GeneratedResource>>,
    /// Error message if reconciliation failed
    pub error: Option<String>,
    /// Conditions for the resource
    pub conditions: Option<Vec<ResourceCondition>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum ResourcePhase {
    Pending,
    Processing,
    Ready,
    Failed,
    Deleting,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct GeneratedResource {
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
    pub status: GeneratedResourceStatus,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum GeneratedResourceStatus {
    Created,
    Updated,
    Failed,
    Deleted,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ResourceCondition {
    pub r#type: String,
    pub status: String,
    pub last_transition_time: String,
    pub reason: Option<String>,
    pub message: Option<String>,
}

impl Default for TerranResourceSpec {
    fn default() -> Self {
        Self {
            kcl_config: KclConfig {
                source: KclSource::Inline {
                    script: "# Default KCL script\nresult = {}".to_string(),
                },
                entry_point: None,
                dependencies: None,
            },
            target_namespace: Some("default".to_string()),
            parameters: None,
            event_config: None,
        }
    }
}

impl Default for ResourcePhase {
    fn default() -> Self {
        ResourcePhase::Pending
    }
}
