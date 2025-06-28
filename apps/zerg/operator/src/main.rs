use anyhow::Result;
use clap::Parser;
use kube::Client;
use std::sync::Arc;
use tracing::{info, instrument};
use tracing_subscriber::{prelude::*, EnvFilter};

mod crd;
mod controller;
mod dependencies;
mod gitops;
mod cicd;
mod config;
mod error;

use controller::DependencyController;

#[derive(Parser)]
#[command(name = "zerg-operator")]
#[command(about = "Kubernetes operator for managing dependencies and GitOps workflows")]
struct Args {
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    #[arg(short, long, default_value = "zerg-system")]
    namespace: String,
    
    #[arg(short, long, default_value = "/etc/zerg/config.yaml")]
    config_path: String,
}

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::new(&args.log_level))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Zerg Operator");
    
    // Create Kubernetes client
    let client = Client::try_default().await?;
    
    // Load configuration
    let config = config::load_config(&args.config_path).await?;
    
    // Create and start the controller
    let controller = DependencyController::new(client, Arc::new(config));
    controller.run().await?;
    
    Ok(())
}