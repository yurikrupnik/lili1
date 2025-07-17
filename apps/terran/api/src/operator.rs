use crate::crd::TerranResource;
use crate::kafka::{KafkaClient, TerranEvent};
use crate::reconciler::TerranReconciler;
use eyre::{Result, WrapErr};
use futures::StreamExt;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::Api,
    client::Client,
    runtime::{
        controller::{Action, Controller},
        watcher::Config,
    },
    CustomResourceExt,
};
use std::{sync::Arc, time::Duration};
use tokio::{select, signal};
use tracing::{error, info};

const TERRAN_FINALIZER: &str = "terran.io/finalizer";

pub struct TerranOperator {
    client: Client,
    kafka_client: Arc<KafkaClient>,
}

impl TerranOperator {
    pub fn new(client: Client, kafka_client: KafkaClient) -> Self {
        Self {
            client,
            kafka_client: Arc::new(kafka_client),
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting Terran Kubernetes Operator");

        // Ensure CRD is installed
        self.ensure_crd().await?;

        // Health check Kafka connection
        self.kafka_client
            .health_check()
            .await
            .wrap_err("Kafka health check failed")?;

        // Setup event consumption from Kafka
        let kafka_client = self.kafka_client.clone();
        let client = self.client.clone();

        let kafka_handle = tokio::spawn(async move {
            if let Err(e) = Self::run_kafka_consumer(kafka_client, client).await {
                error!("Kafka consumer failed: {:?}", e);
            }
        });

        // Setup Kubernetes controller
        let reconciler = TerranReconciler::new(self.client.clone(), self.kafka_client.clone());

        let api: Api<TerranResource> = Api::all(self.client.clone());
        let controller = Controller::new(api.clone(), Config::default())
            .shutdown_on_signal()
            .run(
                move |resource, ctx| Self::reconcile_simple(resource, ctx, reconciler.clone()),
                Self::error_policy,
                Arc::new(()),
            )
            .filter_map(|x| async move { std::result::Result::ok(x) })
            .for_each(|_| futures::future::ready(()));

        info!("Terran operator is running");

        // Run both the controller and Kafka consumer
        select! {
            _ = controller => {
                info!("Controller shutdown");
            }
            _ = kafka_handle => {
                info!("Kafka consumer shutdown");
            }
            _ = signal::ctrl_c() => {
                info!("Received shutdown signal");
            }
        }

        info!("Terran operator shutdown complete");
        Ok(())
    }

    async fn reconcile_simple(
        resource: Arc<TerranResource>,
        _ctx: Arc<()>,
        reconciler: TerranReconciler,
    ) -> Result<Action, kube::Error> {
        match reconciler.reconcile(resource.clone()).await {
            Ok(action) => Ok(action),
            Err(e) => {
                error!("Reconciliation failed: {:?}", e);
                Ok(Action::requeue(Duration::from_secs(60)))
            }
        }
    }

    fn error_policy(_resource: Arc<TerranResource>, error: &kube::Error, _ctx: Arc<()>) -> Action {
        error!("Reconciliation error: {:?}", error);
        Action::requeue(Duration::from_secs(30))
    }

    async fn ensure_crd(&self) -> Result<()> {
        let crd_api: Api<CustomResourceDefinition> = Api::all(self.client.clone());
        let crd_name = "terranresources.terran.io";

        match crd_api.get(crd_name).await {
            Ok(_) => {
                info!("TerranResource CRD already exists");
                Ok(())
            }
            Err(_) => {
                info!("Creating TerranResource CRD");
                let crd = TerranResource::crd();
                crd_api
                    .create(&Default::default(), &crd)
                    .await
                    .wrap_err("Failed to create TerranResource CRD")?;

                info!("TerranResource CRD created successfully");

                // Wait a moment for the CRD to be ready
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok(())
            }
        }
    }

    async fn run_kafka_consumer(kafka_client: Arc<KafkaClient>, client: Client) -> Result<()> {
        // Subscribe to relevant topics
        kafka_client
            .subscribe_to_topics(&["terran-events", "k8s-events"])
            .await?;

        // Start consuming events
        kafka_client
            .consume_events(move |event| Self::handle_kafka_event(&client, event))
            .await
    }

    fn handle_kafka_event(_client: &Client, event: TerranEvent) -> Result<()> {
        info!("Received Kafka event: {:?}", event);

        // Handle different event types
        match event.event_type.as_str() {
            "ReconciliationTrigger" => {
                // Trigger reconciliation for specific resource
                info!(
                    "Triggering reconciliation for {}/{}",
                    event.resource_namespace, event.resource_name
                );

                // In a real implementation, you might want to trigger
                // reconciliation by updating an annotation on the resource
                // or using some other mechanism
            }
            "ExternalEvent" => {
                // Handle external events that might require reconciliation
                info!(
                    "Handling external event for {}/{}",
                    event.resource_namespace, event.resource_name
                );
            }
            _ => {
                info!("Unhandled event type: {}", event.event_type);
            }
        }

        Ok(())
    }
}
