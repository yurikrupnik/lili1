use eyre::Result;
use kube::Client;
#[allow(unused_imports)]
use tracing::info;

mod crd;
mod kafka;
mod kcl_runner;
mod operator;
mod reconciler;

use crate::{kafka::KafkaClient, operator::TerranOperator};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;
    let kafka_client = KafkaClient::new("localhost:9092").await?;

    let operator = TerranOperator::new(client, kafka_client);
    operator.run().await?;

    Ok(())
}
