use anyhow::Result;
use kube::Client;
use std::process::Command;
use tracing::{info, instrument, warn};

use crate::crd::{CiCdConfig, CiCdProvider, Pipeline};
use crate::error::Error;

pub struct CiCdManager {
    client: Client,
}

impl CiCdManager {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
    
    #[instrument(skip(self))]
    pub async fn setup_cicd(
        &self,
        config: &CiCdConfig,
        namespace: &str,
    ) -> Result<(), Error> {
        match config.provider {
            CiCdProvider::Tekton => self.setup_tekton(config, namespace).await,
            CiCdProvider::ArgoWorkflows => self.setup_argo_workflows(config, namespace).await,
        }
    }
    
    #[instrument(skip(self))]
    async fn setup_tekton(&self, config: &CiCdConfig, namespace: &str) -> Result<(), Error> {
        info!("Setting up Tekton CI/CD");
        
        // Install Tekton if not present
        self.install_tekton().await?;
        
        // Create pipelines
        for pipeline in &config.pipelines {
            self.create_tekton_pipeline(pipeline, namespace).await?;
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn install_tekton(&self) -> Result<(), Error> {
        info!("Installing Tekton Pipelines");
        
        // Check if Tekton is already installed
        let check_cmd = Command::new("kubectl")
            .args(&["get", "namespace", "tekton-pipelines"])
            .output();
        
        if check_cmd.is_ok() && check_cmd.unwrap().status.success() {
            info!("Tekton already installed");
            return Ok(());
        }
        
        // Install Tekton Pipelines
        let install_cmd = Command::new("kubectl")
            .args(&[
                "apply", "-f",
                "https://storage.googleapis.com/tekton-releases/pipeline/latest/release.yaml"
            ])
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to install Tekton: {}", e)))?;
        
        if !install_cmd.status.success() {
            let stderr = String::from_utf8_lossy(&install_cmd.stderr);
            return Err(Error::CommandError(format!("Tekton install failed: {}", stderr)));
        }
        
        // Install Tekton Dashboard (optional)
        let dashboard_cmd = Command::new("kubectl")
            .args(&[
                "apply", "-f",
                "https://storage.googleapis.com/tekton-releases/dashboard/latest/release.yaml"
            ])
            .output();
        
        if let Ok(output) = dashboard_cmd {
            if output.status.success() {
                info!("Tekton Dashboard installed");
            } else {
                warn!("Failed to install Tekton Dashboard (optional)");
            }
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn create_tekton_pipeline(
        &self,
        pipeline: &Pipeline,
        namespace: &str,
    ) -> Result<(), Error> {
        info!("Creating Tekton pipeline: {}", pipeline.name);
        
        // Create Pipeline resource
        let pipeline_yaml = self.generate_tekton_pipeline_yaml(pipeline, namespace)?;
        self.apply_yaml_resource(&pipeline_yaml).await?;
        
        // Create TriggerBinding and TriggerTemplate if git trigger is configured
        if let Some(git_trigger) = &pipeline.trigger.git {
            let trigger_yaml = self.generate_tekton_trigger_yaml(pipeline, git_trigger, namespace)?;
            self.apply_yaml_resource(&trigger_yaml).await?;
        }
        
        Ok(())
    }
    
    fn generate_tekton_pipeline_yaml(
        &self,
        pipeline: &Pipeline,
        namespace: &str,
    ) -> Result<String, Error> {
        let mut tasks = Vec::new();
        
        for (i, step) in pipeline.steps.iter().enumerate() {
            let task_name = format!("{}-{}", pipeline.name, i);
            
            let env_vars = if let Some(env) = &step.env {
                env.iter()
                    .map(|(k, v)| format!("        - name: {}\n          value: \"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            };
            
            let working_dir = step.working_dir.as_deref().unwrap_or("/workspace");
            
            let commands = step.commands
                .iter()
                .map(|cmd| format!("        - {}", cmd))
                .collect::<Vec<_>>()
                .join("\n");
            
            let task_yaml = format!(
                r#"
  - name: {}
    taskSpec:
      steps:
      - name: {}
        image: {}
        workingDir: {}
        script: |
          #!/bin/sh
{}
        env:
{}"#,
                task_name, step.name, step.image, working_dir, commands, env_vars
            );
            
            tasks.push(task_yaml);
        }
        
        let pipeline_yaml = format!(
            r#"
apiVersion: tekton.dev/v1beta1
kind: Pipeline
metadata:
  name: {}
  namespace: {}
spec:
  workspaces:
  - name: shared-data
  tasks:
{}"#,
            pipeline.name,
            namespace,
            tasks.join("")
        );
        
        Ok(pipeline_yaml)
    }
    
    fn generate_tekton_trigger_yaml(
        &self,
        pipeline: &Pipeline,
        _git_trigger: &crate::crd::GitTrigger,
        namespace: &str,
    ) -> Result<String, Error> {
        let trigger_yaml = format!(
            r#"
apiVersion: triggers.tekton.dev/v1beta1
kind: TriggerBinding
metadata:
  name: {}-binding
  namespace: {}
spec:
  params:
  - name: git-repo-url
    value: $(body.repository.url)
  - name: git-revision
    value: $(body.head_commit.id)
---
apiVersion: triggers.tekton.dev/v1beta1
kind: TriggerTemplate
metadata:
  name: {}-template
  namespace: {}
spec:
  params:
  - name: git-repo-url
  - name: git-revision
  resourcetemplates:
  - apiVersion: tekton.dev/v1beta1
    kind: PipelineRun
    metadata:
      generateName: {}-run-
    spec:
      pipelineRef:
        name: {}
      workspaces:
      - name: shared-data
        volumeClaimTemplate:
          spec:
            accessModes:
            - ReadWriteOnce
            resources:
              requests:
                storage: 1Gi
---
apiVersion: triggers.tekton.dev/v1beta1
kind: EventListener
metadata:
  name: {}-listener
  namespace: {}
spec:
  serviceAccountName: tekton-triggers-sa
  triggers:
  - name: {}-trigger
    bindings:
    - ref: {}-binding
    template:
      ref: {}-template
"#,
            pipeline.name, namespace,
            pipeline.name, namespace,
            pipeline.name, pipeline.name,
            pipeline.name, namespace,
            pipeline.name, pipeline.name, pipeline.name
        );
        
        Ok(trigger_yaml)
    }
    
    #[instrument(skip(self))]
    async fn setup_argo_workflows(&self, config: &CiCdConfig, namespace: &str) -> Result<(), Error> {
        info!("Setting up Argo Workflows CI/CD");
        
        // Install Argo Workflows if not present
        self.install_argo_workflows().await?;
        
        // Create workflows
        for pipeline in &config.pipelines {
            self.create_argo_workflow(pipeline, namespace).await?;
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn install_argo_workflows(&self) -> Result<(), Error> {
        info!("Installing Argo Workflows");
        
        // Check if Argo Workflows is already installed
        let check_cmd = Command::new("kubectl")
            .args(&["get", "namespace", "argo"])
            .output();
        
        if check_cmd.is_ok() && check_cmd.unwrap().status.success() {
            info!("Argo Workflows already installed");
            return Ok(());
        }
        
        // Create namespace
        let create_ns_cmd = Command::new("kubectl")
            .args(&["create", "namespace", "argo"])
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to create argo namespace: {}", e)))?;
        
        if !create_ns_cmd.status.success() {
            let stderr = String::from_utf8_lossy(&create_ns_cmd.stderr);
            // Ignore error if namespace already exists
            if !stderr.contains("already exists") {
                return Err(Error::CommandError(format!("Failed to create argo namespace: {}", stderr)));
            }
        }
        
        // Install Argo Workflows
        let install_cmd = Command::new("kubectl")
            .args(&[
                "apply", "-n", "argo", "-f",
                "https://github.com/argoproj/argo-workflows/releases/latest/download/install.yaml"
            ])
            .output()
            .map_err(|e| Error::CommandError(format!("Failed to install Argo Workflows: {}", e)))?;
        
        if !install_cmd.status.success() {
            let stderr = String::from_utf8_lossy(&install_cmd.stderr);
            return Err(Error::CommandError(format!("Argo Workflows install failed: {}", stderr)));
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn create_argo_workflow(
        &self,
        pipeline: &Pipeline,
        namespace: &str,
    ) -> Result<(), Error> {
        info!("Creating Argo Workflow: {}", pipeline.name);
        
        // Create WorkflowTemplate
        let workflow_yaml = self.generate_argo_workflow_yaml(pipeline, namespace)?;
        self.apply_yaml_resource(&workflow_yaml).await?;
        
        // Create CronWorkflow if schedule trigger is configured
        if let Some(schedule) = &pipeline.trigger.schedule {
            let cron_yaml = self.generate_argo_cron_workflow_yaml(pipeline, schedule, namespace)?;
            self.apply_yaml_resource(&cron_yaml).await?;
        }
        
        Ok(())
    }
    
    fn generate_argo_workflow_yaml(
        &self,
        pipeline: &Pipeline,
        namespace: &str,
    ) -> Result<String, Error> {
        let mut templates = Vec::new();
        
        // Main DAG template
        let mut dag_tasks = Vec::new();
        
        for (i, step) in pipeline.steps.iter().enumerate() {
            let task_name = format!("step-{}", i);
            
            dag_tasks.push(format!(
                r#"    - name: {}
      template: {}-template"#,
                task_name, task_name
            ));
            
            let env_vars = if let Some(env) = &step.env {
                env.iter()
                    .map(|(k, v)| format!(
                        r#"        - name: {}
          value: "{}""#, k, v))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            };
            
            let working_dir = step.working_dir.as_deref().unwrap_or("/workspace");
            
            let commands = step.commands
                .iter()
                .map(|cmd| format!("          - {}", cmd))
                .collect::<Vec<_>>()
                .join("\n");
            
            let template_yaml = format!(
                r#"  - name: {}-template
    container:
      image: {}
      workingDir: {}
      command: [sh, -c]
      args:
        - |
{}
      env:
{}"#,
                task_name, step.image, working_dir, commands, env_vars
            );
            
            templates.push(template_yaml);
        }
        
        let dag_template = format!(
            r#"  - name: main
    dag:
      tasks:
{}"#,
            dag_tasks.join("\n")
        );
        
        templates.insert(0, dag_template);
        
        let workflow_yaml = format!(
            r#"
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: {}
  namespace: {}
spec:
  entrypoint: main
  templates:
{}"#,
            pipeline.name,
            namespace,
            templates.join("\n")
        );
        
        Ok(workflow_yaml)
    }
    
    fn generate_argo_cron_workflow_yaml(
        &self,
        pipeline: &Pipeline,
        schedule: &str,
        namespace: &str,
    ) -> Result<String, Error> {
        let cron_yaml = format!(
            r#"
apiVersion: argoproj.io/v1alpha1
kind: CronWorkflow
metadata:
  name: {}-cron
  namespace: {}
spec:
  schedule: "{}"
  workflowSpec:
    entrypoint: main
    workflowTemplateRef:
      name: {}
"#,
            pipeline.name, namespace, schedule, pipeline.name
        );
        
        Ok(cron_yaml)
    }
    
    #[instrument(skip(self))]
    async fn apply_yaml_resource(&self, yaml: &str) -> Result<(), Error> {
        let mut cmd = Command::new("kubectl");
        cmd.args(&["apply", "-f", "-"]);
        cmd.stdin(std::process::Stdio::piped());
        
        let mut child = cmd
            .spawn()
            .map_err(|e| Error::CommandError(format!("Failed to spawn kubectl: {}", e)))?;
        
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(yaml.as_bytes())
                .map_err(|e| Error::IoError(format!("Failed to write to kubectl stdin: {}", e)))?;
        }
        
        let output = child
            .wait_with_output()
            .map_err(|e| Error::CommandError(format!("Failed to wait for kubectl: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandError(format!("Failed to apply resource: {}", stderr)));
        }
        
        Ok(())
    }
}