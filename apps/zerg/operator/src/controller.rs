use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures_util::StreamExt;
use kube::{
    api::{Api, Patch, PatchParams, ResourceExt},
    client::Client,
    runtime::{
        controller::{Action, Controller},
        finalizer::{finalizer, Event as Finalizer},
    },
};
use tracing::{error, info, instrument};

use crate::{
    config::Config,
    crd::{DependencyManager, DependencyManagerStatus, Phase},
    dependencies::DependencyInstaller,
    error::Error,
    gitops::GitOpsManager,
    cicd::CiCdManager,
};

pub struct DependencyController {
    client: Client,
    config: Arc<Config>,
}

impl DependencyController {
    pub fn new(client: Client, config: Arc<Config>) -> Self {
        Self { client, config }
    }
    
    #[instrument(skip(self))]
    pub async fn run(self) -> Result<()> {
        let api: Api<DependencyManager> = Api::all(self.client.clone());
        
        Controller::new(api, Default::default())
            .run(reconcile, error_policy, Arc::new(self))
            .for_each(|result| async move {
                match result {
                    Ok(_) => info!("Reconciliation successful"),
                    Err(e) => error!("Reconciliation error: {}", e),
                }
            })
            .await;
            
        Ok(())
    }
}

#[instrument(skip(ctx))]
async fn reconcile(
    obj: Arc<DependencyManager>,
    ctx: Arc<DependencyController>,
) -> Result<Action, Error> {
    let name = obj.name_any();
    let namespace = obj.namespace().unwrap_or_default();
    
    info!("Reconciling DependencyManager {} in namespace {}", name, namespace);
    
    let api: Api<DependencyManager> = Api::namespaced(ctx.client.clone(), &namespace);
    
    finalizer(&api, "zerg.io/finalizer", obj, |event| async {
        match event {
            Finalizer::Apply(dm) => reconcile_dependency_manager(dm, ctx.clone()).await,
            Finalizer::Cleanup(dm) => cleanup_dependency_manager(dm, ctx.clone()).await,
        }
    })
    .await
    .map_err(|e| Error::FinalizerError(Box::new(e)))
}

#[instrument(skip(ctx))]
async fn reconcile_dependency_manager(
    dm: Arc<DependencyManager>,
    ctx: Arc<DependencyController>,
) -> Result<Action, Error> {
    let name = dm.name_any();
    let namespace = dm.namespace().unwrap_or_default();
    
    info!("Applying DependencyManager {}", name);
    
    // Update status to Installing
    update_status(&ctx.client, &dm, Phase::Installing, None).await?;
    
    // Install dependencies
    let installer = DependencyInstaller::new(ctx.client.clone());
    let mut dependency_statuses = Vec::new();
    
    for dep in &dm.spec.dependencies {
        if !dep.enabled {
            continue;
        }
        
        info!("Installing dependency: {}", dep.name);
        
        match installer.install_dependency(dep, &namespace).await {
            Ok(status) => {
                info!("Successfully installed dependency: {}", dep.name);
                dependency_statuses.push(status);
            }
            Err(e) => {
                error!("Failed to install dependency {}: {}", dep.name, e);
                update_status(&ctx.client, &dm, Phase::Failed, Some(format!("Failed to install {}: {}", dep.name, e))).await?;
                return Ok(Action::requeue(Duration::from_secs(300)));
            }
        }
    }
    
    // Setup GitOps if configured
    if let Some(gitops_config) = &dm.spec.gitops {
        info!("Setting up GitOps with provider: {:?}", gitops_config.provider);
        
        let gitops_manager = GitOpsManager::new(ctx.client.clone());
        if let Err(e) = gitops_manager.setup_gitops(gitops_config, &namespace).await {
            error!("Failed to setup GitOps: {}", e);
            update_status(&ctx.client, &dm, Phase::Failed, Some(format!("GitOps setup failed: {}", e))).await?;
            return Ok(Action::requeue(Duration::from_secs(300)));
        }
    }
    
    // Setup CI/CD if configured
    if let Some(cicd_config) = &dm.spec.cicd {
        info!("Setting up CI/CD with provider: {:?}", cicd_config.provider);
        
        let cicd_manager = CiCdManager::new(ctx.client.clone());
        if let Err(e) = cicd_manager.setup_cicd(cicd_config, &namespace).await {
            error!("Failed to setup CI/CD: {}", e);
            update_status(&ctx.client, &dm, Phase::Failed, Some(format!("CI/CD setup failed: {}", e))).await?;
            return Ok(Action::requeue(Duration::from_secs(300)));
        }
    }
    
    // Update status to Ready
    update_status(&ctx.client, &dm, Phase::Ready, None).await?;
    
    info!("Successfully reconciled DependencyManager {}", name);
    Ok(Action::requeue(Duration::from_secs(3600))) // Requeue every hour
}

#[instrument(skip(_ctx))]
async fn cleanup_dependency_manager(
    dm: Arc<DependencyManager>,
    _ctx: Arc<DependencyController>,
) -> Result<Action, Error> {
    let name = dm.name_any();
    info!("Cleaning up DependencyManager {}", name);
    
    // TODO: Implement cleanup logic
    // - Uninstall dependencies if configured
    // - Clean up GitOps resources
    // - Clean up CI/CD pipelines
    
    Ok(Action::await_change())
}

async fn update_status(
    client: &Client,
    dm: &DependencyManager,
    phase: Phase,
    error_message: Option<String>,
) -> Result<(), Error> {
    let name = dm.name_any();
    let namespace = dm.namespace().unwrap_or_default();
    let api: Api<DependencyManager> = Api::namespaced(client.clone(), &namespace);
    
    let mut status = DependencyManagerStatus {
        phase,
        dependencies: None,
        gitops_status: None,
        cicd_status: None,
        last_reconciled: Some(chrono::Utc::now().to_rfc3339()),
        conditions: None,
    };
    
    if let Some(msg) = error_message {
        // Add error condition
        status.conditions = Some(vec![crate::crd::Condition {
            type_: "Ready".to_string(),
            status: "False".to_string(),
            last_transition_time: chrono::Utc::now().to_rfc3339(),
            reason: Some("ReconciliationError".to_string()),
            message: Some(msg),
        }]);
    }
    
    let patch = serde_json::json!({
        "status": status
    });
    
    api.patch_status(&name, &PatchParams::default(), &Patch::Merge(&patch)).await
        .map_err(|e| Error::KubeError(e))?;
    
    Ok(())
}

fn error_policy(_obj: Arc<DependencyManager>, error: &Error, _ctx: Arc<DependencyController>) -> Action {
    error!("Reconciliation error: {}", error);
    Action::requeue(Duration::from_secs(60))
}