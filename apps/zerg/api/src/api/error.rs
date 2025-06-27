use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use strum::{EnumString, Display, EnumIter};
use serde_json::json;

#[derive(Debug, EnumString, Display, EnumIter)]
pub enum ApiError {
  #[strum(serialize = "user_not_found")]
  UserNotFound,
  #[strum(serialize = "invalid_credentials")]
  InvalidCredentials,
  #[strum(serialize = "insufficient_permissions")]
  InsufficientPermissions,
  #[strum(serialize = "validation_failed")]
  ValidationFailed,
  #[strum(serialize = "internal_server_error")]
  InternalServerError,
}
