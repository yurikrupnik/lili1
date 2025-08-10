use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::instrument;

use crate::error::AppError;

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    entries: Arc<Mutex<HashMap<String, RateLimitEntry>>>,
    max_requests: u32,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_duration: Duration) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_duration,
        }
    }

    pub fn check_rate_limit(&self, key: &str) -> Result<bool, AppError> {
        let now = Instant::now();
        let mut entries = self.entries.lock().map_err(|_| AppError::InternalServerError)?;

        entries.retain(|_, entry| {
            now.duration_since(entry.window_start) < self.window_duration
        });

        let entry = entries.entry(key.to_string()).or_insert(RateLimitEntry {
            count: 0,
            window_start: now,
        });

        if now.duration_since(entry.window_start) >= self.window_duration {
            entry.count = 0;
            entry.window_start = now;
        }

        if entry.count >= self.max_requests {
            tracing::warn!(
                key = %key,
                count = entry.count,
                max_requests = self.max_requests,
                "Rate limit exceeded"
            );
            return Ok(false);
        }

        entry.count += 1;
        Ok(true)
    }
}

fn get_client_ip(headers: &HeaderMap) -> String {
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_for_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_for_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            return real_ip_str.to_string();
        }
    }

    "unknown".to_string()
}

pub fn create_rate_limiter() -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>> + Clone {
    let rate_limiter = RateLimiter::new(100, Duration::from_secs(60));

    move |request: Request, next: Next| {
        let rate_limiter = rate_limiter.clone();
        Box::pin(async move {
            rate_limit_middleware(request, next, rate_limiter).await
        })
    }
}

#[instrument(name = "rate_limit_middleware", skip_all)]
async fn rate_limit_middleware(
    request: Request,
    next: Next,
    rate_limiter: RateLimiter,
) -> Result<Response, AppError> {
    let client_ip = get_client_ip(request.headers());
    
    match rate_limiter.check_rate_limit(&client_ip) {
        Ok(true) => {
            tracing::debug!(
                client_ip = %client_ip,
                path = %request.uri().path(),
                "Rate limit check passed"
            );
            Ok(next.run(request).await)
        }
        Ok(false) => {
            tracing::warn!(
                client_ip = %client_ip,
                path = %request.uri().path(),
                "Rate limit exceeded"
            );
            Err(AppError::RateLimitExceeded)
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                client_ip = %client_ip,
                "Rate limit check failed"
            );
            Err(e)
        }
    }
}