use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};
use std::time::Duration;
use tracing::instrument;

use crate::error::AppError;

#[instrument(name = "security_headers_middleware", skip_all)]
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        "nosniff".parse().unwrap(),
    );
    
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        "DENY".parse().unwrap(),
    );
    
    headers.insert(
        header::HeaderName::from_static("x-xss-protection"),
        "1; mode=block".parse().unwrap(),
    );
    
    headers.insert(
        header::HeaderName::from_static("referrer-policy"),
        "no-referrer".parse().unwrap(),
    );
    
    headers.insert(
        header::HeaderName::from_static("strict-transport-security"),
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    
    headers.insert(
        header::HeaderName::from_static("content-security-policy"),
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; font-src 'self'; object-src 'none'; frame-src 'none'; base-uri 'self'".parse().unwrap(),
    );
    
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        "camera=(), microphone=(), geolocation=(), payment=()".parse().unwrap(),
    );
    
    headers.insert(
        header::HeaderName::from_static("x-permitted-cross-domain-policies"),
        "none".parse().unwrap(),
    );

    tracing::debug!("Security headers added to response");
    
    Ok(response)
}