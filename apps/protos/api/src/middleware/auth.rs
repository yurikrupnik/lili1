use axum::{
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::instrument;

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: String,
    pub role: String,
}

const EXCLUDED_PATHS: &[&str] = &["/health", "/docs", "/openapi.json"];

#[instrument(name = "auth_middleware", skip_all)]
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path().to_string();
    
    if EXCLUDED_PATHS.iter().any(|&excluded| path.starts_with(excluded)) {
        return Ok(next.run(request).await);
    }

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            auth.strip_prefix("Bearer ").unwrap_or("")
        }
        _ => {
            tracing::warn!(
                path = %path,
                "Missing or invalid Authorization header"
            );
            return Err(AppError::Unauthorized);
        }
    };

    if token.is_empty() {
        tracing::warn!(path = %path, "Empty authentication token");
        return Err(AppError::Unauthorized);
    }

    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string());

    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation,
    )?;

    let user = AuthenticatedUser {
        id: token_data.claims.sub,
        role: token_data.claims.role,
    };

    request.extensions_mut().insert(user.clone());

    tracing::info!(
        user_id = %user.id,
        user_role = %user.role,
        path = %path,
        "User authenticated successfully"
    );

    Ok(next.run(request).await)
}

pub fn create_jwt_token(user_id: &str, role: &str) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let expiration = now + chrono::Duration::hours(24);

    let claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: expiration.timestamp() as usize,
        role: role.to_string(),
    };

    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string());

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_ref()),
    )?;

    Ok(token)
}