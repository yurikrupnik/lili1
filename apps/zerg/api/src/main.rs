mod shared;

use axum::{response::Html, routing::get, Json, Router};
use eyre::Result;
use rust_services::{envs::Env, model::task::Task, tracing::init_tracing};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();
    tracing::info!("Starting API server");

    let app_router = Router::new()
        .route("/", get(handler))
        .route("/tasks", get(get_tasks));
    let url = Env::get_url().map_err(|e| eyre::eyre!("Failed to get URL: {}", e))?;
    let listener = tokio::net::TcpListener::bind(&url).await?;

    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, app_router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World from Local Services!</h1>")
}

async fn get_tasks() -> Json<Vec<Task>> {
    // use uuid::Uuid;

    let sample_tasks = vec![
        Task {
            id: String::from("a"),
            title: "Setup local services".to_string(),
            description: Some("Integrate local services into the API app".to_string()),
            completed: true,
        },
        Task {
          id: String::from("b"),
            title: "Add task routes".to_string(),
            description: Some("Create REST API endpoints for tasks".to_string()),
            completed: false,
        },
    ];

    Json(sample_tasks)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Signal received, starting graceful shutdown");
}

// // #![deny(clippy::unwrap_used)]
// // #![deny(clippy::expect_used)]
// // // #![deny(clippy::clone)]
// // #![deny(clippy::panic)]
// // #![deny(unused_must_use)]
// use api::routes;
// use app_state::AppState;
// use axum::response::IntoResponse;
// use axum::routing::get;
// use axum::Extension;
// use generals::{envs::Env, tracing::init_tracing};
// use log::info;
// use services::influx::services::InfluxConfig;
// use services::{
//   influx::connector::connect as influx_connect, mongo::connector::connect as mongo_connect,
//   postgres::connector::connect as postgres_connect, redis::connector::connect as redis_connect,
// };
// use shared::{app::app, shutdown::shutdown_signal};
// use std::sync::{Arc, Mutex};
// use swagger::ApiDoc;
// use eyre::{eyre, OptionExt, Result};
// use shared::handlers::health_check;
//
// mod api;
// mod app_state;
// // mod error;
// mod swagger;
//
//
// #[tokio::main]
// async fn main() -> Result<()> {
//   // Initialize tracing (logging)
//   init_tracing();
//   tracing::info!("Initializing application");
//
//   let dbs = tokio::join!(
//         postgres_connect(None, None), // Connect to Postgres
//         influx_connect(),             // Connect to InfluxDB
//         mongo_connect("aris"),        // Connect to MongoDB
//         redis_connect(),              // Setup Redis pool
//     );
//
//   let (postgres_pool_result, influx_client_result, db_result, redis_pool_result) = dbs;
//
//   // Handle results of connections
//   let influx_client = influx_client_result?;
//   let db = db_result?;
//   let redis_pool = redis_pool_result?;
//   let postgres_pool = postgres_pool_result?;
//   let redis_client = redis::Client::open("redis://localhost:6379")?;
//   let redis_pub = redis_client.get_async_pubsub().await?;
//
//   // migration
//   // sqlx::migrate!().run(&postgres_pool).await?;
//
//   // app state
//   let org = "my-org".to_string();
//   let bucket = "home".to_string();
//   let config = Arc::new(InfluxConfig {
//     client: influx_client,
//     org,
//     bucket,
//   });
//   let pubsub = Arc::new(Mutex::new(redis_pub));
//   let state = AppState::new(db, redis_pool, postgres_pool, pubsub);
//
//   let router = app::<ApiDoc, AppState>(state.clone(), routes()).layer(Extension(config));
//   // run our app with hyper, listening globally on port
//   let listener = tokio::net::TcpListener::bind(Env::get_url()).await?;
//
//   info!("Server starting on {}", listener.local_addr()?);
//
//   // Start the server.
//   axum::serve(listener, router.into_make_service())
//     .with_graceful_shutdown(shutdown_signal())
//     .await
//     .inspect_err(|e| {
//       tracing::error!("Server encountered an error: {:?}", e);
//       std::process::exit(1);
//     })?;
//
//   Ok(())
// }
