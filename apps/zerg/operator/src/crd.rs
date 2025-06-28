use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(CustomResource, Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[kube(
    group = "zerg.io",
    version = "v1",
    kind = "DependencyManager",
    plural = "dependencymanagers",
    namespaced
)]
#[kube(status = "DependencyManagerStatus")]
pub struct DependencyManagerSpec {
    /// Dependencies to install and manage
    pub dependencies: Vec<Dependency>,
    
    /// GitOps configuration
    pub gitops: Option<GitOpsConfig>,
    
    /// CI/CD pipeline configuration
    pub cicd: Option<CiCdConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct Dependency {
    /// Name of the dependency
    pub name: String,
    
    /// Type of dependency (helm, kustomize, yaml)
    #[serde(rename = "type")]
    pub type_: DependencyType,
    
    /// Repository or source information
    pub source: DependencySource,
    
    /// Version or chart version
    pub version: Option<String>,
    
    /// Target namespace
    pub namespace: Option<String>,
    
    /// Values for Helm charts
    pub values: Option<HashMap<String, serde_json::Value>>,
    
    /// Dependencies that must be installed before this one
    pub depends_on: Option<Vec<String>>,
    
    /// Whether this dependency is enabled
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Helm,
    Kustomize,
    Yaml,
    Operator,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct DependencySource {
    /// Repository URL
    pub repo: String,
    
    /// Chart name (for Helm)
    pub chart: Option<String>,
    
    /// Path within repository
    pub path: Option<String>,
    
    /// Git reference (branch, tag, commit)
    #[serde(rename = "ref")]
    pub ref_: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct GitOpsConfig {
    /// GitOps provider (flux, argocd)
    pub provider: GitOpsProvider,
    
    /// Git repository for GitOps
    pub repository: String,
    
    /// Branch to use
    pub branch: String,
    
    /// Path within repository
    pub path: String,
    
    /// Sync policy
    pub sync_policy: Option<SyncPolicy>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum GitOpsProvider {
    Flux,
    ArgoCD,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct SyncPolicy {
    /// Automated sync
    pub automated: bool,
    
    /// Self heal
    pub self_heal: bool,
    
    /// Prune resources
    pub prune: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct CiCdConfig {
    /// CI/CD provider (tekton, argo-workflows)
    pub provider: CiCdProvider,
    
    /// Pipeline definitions
    pub pipelines: Vec<Pipeline>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CiCdProvider {
    Tekton,
    ArgoWorkflows,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct Pipeline {
    /// Pipeline name
    pub name: String,
    
    /// Trigger configuration
    pub trigger: PipelineTrigger,
    
    /// Steps to execute
    pub steps: Vec<PipelineStep>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct PipelineTrigger {
    /// Git webhook trigger
    pub git: Option<GitTrigger>,
    
    /// Schedule trigger
    pub schedule: Option<String>,
    
    /// Manual trigger
    pub manual: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct GitTrigger {
    /// Repository URL
    pub repository: String,
    
    /// Branch patterns
    pub branches: Vec<String>,
    
    /// Event types (push, pull_request)
    pub events: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct PipelineStep {
    /// Step name
    pub name: String,
    
    /// Docker image to use
    pub image: String,
    
    /// Commands to run
    pub commands: Vec<String>,
    
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
    
    /// Working directory
    pub working_dir: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct DependencyManagerStatus {
    /// Overall status
    pub phase: Phase,
    
    /// Status of individual dependencies
    pub dependencies: Option<Vec<DependencyStatus>>,
    
    /// GitOps status
    pub gitops_status: Option<GitOpsStatus>,
    
    /// CI/CD status
    pub cicd_status: Option<CiCdStatus>,
    
    /// Last reconciliation time
    pub last_reconciled: Option<String>,
    
    /// Conditions
    pub conditions: Option<Vec<Condition>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum Phase {
    Pending,
    Installing,
    Ready,
    Failed,
    Updating,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct DependencyStatus {
    /// Dependency name
    pub name: String,
    
    /// Installation status
    pub status: DependencyInstallStatus,
    
    /// Installed version
    pub version: Option<String>,
    
    /// Last update time
    pub last_updated: Option<String>,
    
    /// Error message if failed
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum DependencyInstallStatus {
    Pending,
    Installing,
    Installed,
    Failed,
    Updating,
    Uninstalling,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct GitOpsStatus {
    /// Provider status
    pub provider: GitOpsProvider,
    
    /// Sync status
    pub sync_status: String,
    
    /// Last sync time
    pub last_sync: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct CiCdStatus {
    /// Provider status
    pub provider: CiCdProvider,
    
    /// Pipeline statuses
    pub pipelines: Vec<PipelineStatus>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct PipelineStatus {
    /// Pipeline name
    pub name: String,
    
    /// Current status
    pub status: String,
    
    /// Last run time
    pub last_run: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct Condition {
    /// Condition type
    #[serde(rename = "type")]
    pub type_: String,
    
    /// Status (True, False, Unknown)
    pub status: String,
    
    /// Last transition time
    pub last_transition_time: String,
    
    /// Reason for the condition
    pub reason: Option<String>,
    
    /// Human readable message
    pub message: Option<String>,
}