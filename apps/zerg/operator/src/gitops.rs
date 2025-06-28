use anyhow::Result;
use kube::Client;
use std::process::Command;
use tracing::{info, instrument};

use crate::crd::{GitOpsConfig, GitOpsProvider};
use crate::error::Error;

pub struct GitOpsManager {
    client: Client,
}

impl GitOpsManager {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
    
    #[instrument(skip(self))]
    pub async fn setup_gitops(
        &self,
        config: &GitOpsConfig,
        namespace: &str,
    ) -> Result<(), Error> {
        match config.provider {
            GitOpsProvider::Flux => self.setup_flux(config, namespace).await,
            GitOpsProvider::ArgoCD => self.setup_argocd(config, namespace).await,
        }
    }
    
    #[instrument(skip(self))]
    async fn setup_flux(&self, config: &GitOpsConfig, namespace: &str) -> Result<(), Error> {
        info!("Setting up Flux GitOps");
        
        // Check if Flux is already installed
        let check_cmd = Command::new("flux")
            .args(&["check", "--pre"])
            .output();
        
        match check_cmd {
            Ok(output) if output.status.success() => {
                info!("Flux prerequisites satisfied");
            }
            _ => {
                info!("Installing Flux");
                self.install_flux().await?;
            }
        }
        
        // Bootstrap Flux with the git repository
        let bootstrap_cmd = Command::new("flux")
            .args(&[
                "bootstrap", "git",
                "--url", &config.repository,
                "--branch", &config.branch,
                "--path", &config.path,
                "--namespace", "flux-system",
            ])
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to bootstrap Flux: {}", e)))?;
        
        if !bootstrap_cmd.status.success() {
            let stderr = String::from_utf8_lossy(&bootstrap_cmd.stderr);
            return Err(Error::CommandError(format!("Flux bootstrap failed: {}", stderr)));
        }
        
        // Create GitRepository resource
        self.create_flux_git_repository(config, namespace).await?;
        
        // Create Kustomization resource
        self.create_flux_kustomization(config, namespace).await?;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn install_flux(&self) -> Result<(), Error> {
        info!("Installing Flux components");
        
        let install_cmd = Command::new("flux")
            .args(&["install"])
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to install Flux: {}", e)))?;
        
        if !install_cmd.status.success() {
            let stderr = String::from_utf8_lossy(&install_cmd.stderr);
            return Err(Error::CommandError(format!("Flux install failed: {}", stderr)));
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn create_flux_git_repository(
        &self,
        config: &GitOpsConfig,
        namespace: &str,
    ) -> Result<(), Error> {
        let git_repo_yaml = format!(
            r#"
apiVersion: source.toolkit.fluxcd.io/v1beta2
kind: GitRepository
metadata:
  name: zerg-repo
  namespace: {}
spec:
  interval: 5m
  url: {}
  ref:
    branch: {}
"#,
            namespace, config.repository, config.branch
        );
        
        // Apply the GitRepository resource
        let mut cmd = Command::new("kubectl");
        cmd.args(&["apply", "-f", "-"]);
        cmd.stdin(std::process::Stdio::piped());
        
        let mut child = cmd
            .spawn()
            .map_err(|e| Error::CommandError(format!("Failed to spawn kubectl: {}", e)))?;
        
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(git_repo_yaml.as_bytes())
                .map_err(|e| Error::IoError(format!("Failed to write to kubectl stdin: {}", e)))?;
        }
        
        let output = child
            .wait_with_output()
            .map_err(|e| Error::CommandError(format!("Failed to wait for kubectl: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandError(format!("Failed to create GitRepository: {}", stderr)));
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn create_flux_kustomization(
        &self,
        config: &GitOpsConfig,
        namespace: &str,
    ) -> Result<(), Error> {
        let prune = config.sync_policy
            .as_ref()
            .map(|p| p.prune)
            .unwrap_or(false);
        
        let kustomization_yaml = format!(
            r#"
apiVersion: kustomize.toolkit.fluxcd.io/v1beta2
kind: Kustomization
metadata:
  name: zerg-kustomization
  namespace: {}
spec:
  interval: 5m
  sourceRef:
    kind: GitRepository
    name: zerg-repo
  path: "{}"
  prune: {}
  targetNamespace: {}
"#,
            namespace, config.path, prune, namespace
        );
        
        // Apply the Kustomization resource
        let mut cmd = Command::new("kubectl");
        cmd.args(&["apply", "-f", "-"]);
        cmd.stdin(std::process::Stdio::piped());
        
        let mut child = cmd
            .spawn()
            .map_err(|e| Error::CommandError(format!("Failed to spawn kubectl: {}", e)))?;
        
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(kustomization_yaml.as_bytes())
                .map_err(|e| Error::IoError(format!("Failed to write to kubectl stdin: {}", e)))?;
        }
        
        let output = child
            .wait_with_output()
            .map_err(|e| Error::CommandError(format!("Failed to wait for kubectl: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandError(format!("Failed to create Kustomization: {}", stderr)));
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn setup_argocd(&self, config: &GitOpsConfig, namespace: &str) -> Result<(), Error> {
        info!("Setting up ArgoCD GitOps");
        
        // Install ArgoCD if not present
        self.install_argocd().await?;
        
        // Create ArgoCD Application
        self.create_argocd_application(config, namespace).await?;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn install_argocd(&self) -> Result<(), Error> {
        info!("Installing ArgoCD");
        
        // Check if ArgoCD namespace exists
        let check_ns_cmd = Command::new("kubectl")
            .args(&["get", "namespace", "argocd"])
            .output();
        
        if check_ns_cmd.is_err() || !check_ns_cmd.unwrap().status.success() {
            // Create namespace
            let create_ns_cmd = Command::new("kubectl")
                .args(&["create", "namespace", "argocd"])
                .output()
                .map_err(|e| Error::CommandError(format!("Failed to create argocd namespace: {}", e)))?;
            
            if !create_ns_cmd.status.success() {
                let stderr = String::from_utf8_lossy(&create_ns_cmd.stderr);
                return Err(Error::CommandError(format!("Failed to create argocd namespace: {}", stderr)));
            }
        }
        
        // Install ArgoCD
        let install_cmd = Command::new("kubectl")
            .args(&[
                "apply", "-n", "argocd", "-f",
                "https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml"
            ])
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to install ArgoCD: {}", e)))?;
        
        if !install_cmd.status.success() {
            let stderr = String::from_utf8_lossy(&install_cmd.stderr);
            return Err(Error::CommandError(format!("ArgoCD install failed: {}", stderr)));
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn create_argocd_application(
        &self,
        config: &GitOpsConfig,
        namespace: &str,
    ) -> Result<(), Error> {
        let sync_policy = if let Some(policy) = &config.sync_policy {
            if policy.automated {
                format!(
                    r#"
  syncPolicy:
    automated:
      prune: {}
      selfHeal: {}
"#,
                    policy.prune, policy.self_heal
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        let app_yaml = format!(
            r#"
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: zerg-app
  namespace: argocd
spec:
  project: default
  source:
    repoURL: {}
    targetRevision: {}
    path: {}
  destination:
    server: https://kubernetes.default.svc
    namespace: {}{}
"#,
            config.repository, config.branch, config.path, namespace, sync_policy
        );
        
        // Apply the Application resource
        let mut cmd = Command::new("kubectl");
        cmd.args(&["apply", "-f", "-"]);
        cmd.stdin(std::process::Stdio::piped());
        
        let mut child = cmd
            .spawn()
            .map_err(|e| Error::CommandError(format!("Failed to spawn kubectl: {}", e)))?;
        
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(app_yaml.as_bytes())
                .map_err(|e| Error::IoError(format!("Failed to write to kubectl stdin: {}", e)))?;
        }
        
        let output = child
            .wait_with_output()
            .map_err(|e| Error::CommandError(format!("Failed to wait for kubectl: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandError(format!("Failed to create ArgoCD Application: {}", stderr)));
        }
        
        Ok(())
    }
}