use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed")]
    Unauthorized,
    
    #[error("Access forbidden")]
    Forbidden,
    
    #[error("Resource not found")]
    NotFound,
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Request timeout")]
    Timeout,
    
    #[error("Internal server error")]
    InternalServerError,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Database error")]
    DatabaseError,
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "Authentication required".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", "Access denied".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "Resource not found".to_string()),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", self.to_string()),
            AppError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT_EXCEEDED", "Rate limit exceeded".to_string()),
            AppError::Timeout => (StatusCode::REQUEST_TIMEOUT, "TIMEOUT", "Request timeout".to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", self.to_string()),
            AppError::JwtError(_) => (StatusCode::UNAUTHORIZED, "JWT_ERROR", "Invalid authentication token".to_string()),
            AppError::JsonError(_) => (StatusCode::BAD_REQUEST, "JSON_ERROR", "Invalid JSON format".to_string()),
            AppError::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database operation failed".to_string()),
            AppError::ConfigError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR", "Configuration error".to_string()),
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".to_string()),
        };

        let error_response = json!({
            "error": {
                "code": error_code,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });

        tracing::error!(
            error = %self,
            status_code = %status,
            "Request error occurred"
        );

        (status, Json(error_response)).into_response()
    }
}

impl From<eyre::Error> for AppError {
    fn from(err: eyre::Error) -> Self {
        tracing::error!("Eyre error: {}", err);
        AppError::InternalServerError
    }
}

pub type AppResult<T> = Result<T, AppError>;