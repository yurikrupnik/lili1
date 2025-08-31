mod config;
mod error;
mod handlers;
mod middleware;
mod routes;

use axum::{
    http::{header, Method, StatusCode},
    middleware as axum_middleware,
    Router,
};
use eyre::Result;
use opentelemetry::global;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_stdout::SpanExporter;
use rust_services::tracing::init_tracing;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::{
    config::AppConfig,
    handlers::health_check,
    middleware::{auth::auth_middleware, rate_limit::create_rate_limiter, security::security_headers_middleware},
    routes::create_routes,
};

fn init_tracer_provider() {
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();
    global::set_tracer_provider(provider);
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();
    init_tracer_provider();

    let config = AppConfig::from_env()?;

    tracing::info!(
        service = "protos_api",
        version = env!("CARGO_PKG_VERSION"),
        environment = %config.environment,
        "Starting Protos API server"
    );

    let rate_limiter = create_rate_limiter();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_origin(Any);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Micros),
        );

    let sensitive_headers = SetSensitiveRequestHeadersLayer::new([
        header::AUTHORIZATION,
        header::COOKIE,
        header::SET_COOKIE,
    ]);

    let app = Router::new()
        .merge(create_routes())
        .route("/healths", axum::routing::get(health_check))
        .layer(axum_middleware::from_fn(auth_middleware))
        .layer(axum_middleware::from_fn(rate_limiter))
        .layer(TimeoutLayer::new(Duration::from_secs(31)))
        .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB limit
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new())
        .layer(trace_layer)
        .layer(cors)
        .layer(sensitive_headers)
        .layer(axum_middleware::from_fn(security_headers_middleware))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Route not found") });

    let listener = tokio::net::TcpListener::bind(&config.server_url()).await?;

    tracing::info!(
        address = %listener.local_addr()?,
        "Protos API server listening"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Protos API server shut down gracefully");
    Ok(())
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
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down");
        },
    }
}
