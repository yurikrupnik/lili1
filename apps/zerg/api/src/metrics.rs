use axum::{
    extract::MatchedPath,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use metrics::{counter, gauge, histogram};
use std::time::Instant;

static METRICS_HANDLE: std::sync::OnceLock<metrics_exporter_prometheus::PrometheusHandle> = std::sync::OnceLock::new();

pub fn setup_metrics() -> eyre::Result<()> {
    let handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .map_err(|e| eyre::eyre!("Failed to install Prometheus recorder: {}", e))?;

    METRICS_HANDLE.set(handle)
        .map_err(|_| eyre::eyre!("Failed to set metrics handle"))?;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;

            gauge!("process_cpu_seconds_total").set(get_cpu_usage());
            gauge!("process_resident_memory_bytes").set(get_memory_usage() as f64);
            gauge!("tokio_runtime_workers_count").set(get_active_workers() as f64);
            gauge!("tokio_runtime_spawned_tasks_count").set(get_spawned_tasks() as f64);
        }
    });

    Ok(())
}

pub async fn metrics_middleware(
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str())
        .unwrap_or("unknown")
        .to_string();

    let response = next.run(req).await;

    let status = response.status();
    let duration = start.elapsed();

    counter!("http_requests_total",
        "method" => method.clone(),
        "status" => status.as_u16().to_string(),
        "path" => path.clone()
    ).increment(1);

    histogram!("http_request_duration_seconds",
        "method" => method,
        "status" => status.as_u16().to_string(),
        "path" => path
    ).record(duration.as_secs_f64());

    response
}

#[utoipa::path(
    get,
    path = "/metrics",
    tag = "metrics",
    responses(
        (status = 200, description = "Prometheus metrics", body = String, content_type = "text/plain"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn metrics_handler() -> Result<String, StatusCode> {
    match METRICS_HANDLE.get() {
        Some(handle) => Ok(handle.render()),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn get_cpu_usage() -> f64 {
    0.0
}

fn get_memory_usage() -> u64 {
    use std::fs;
    if let Ok(contents) = fs::read_to_string("/proc/self/status") {
        for line in contents.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    print!("{:?}", value);
                    if let Ok(kb) = value.parse::<u64>() {
                        return kb * 1024;
                    }
                }
            }
        }
    }
    0
}

fn get_active_workers() -> u64 {
    1
}

fn get_spawned_tasks() -> u64 {
    1
}
