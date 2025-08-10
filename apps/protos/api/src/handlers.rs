use axum::{
    extract::Query,
    response::Json,
};
use opentelemetry::{
    global,
    trace::{Span, SpanKind, Status, Tracer},
    KeyValue,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::{ToSchema, OpenApi};
use validator::Validate;

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
    pub uptime: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DiceRollResponse {
    pub value: u8,
    pub timestamp: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct DiceRollQuery {
    #[validate(range(min = 1, max = 100))]
    pub sides: Option<u8>,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    summary = "Health check endpoint",
    description = "Returns the current health status of the API",
    responses(
        (status = 200, description = "API is healthy", body = HealthResponse),
        (status = 500, description = "Internal server error")
    )
)]
#[instrument(name = "health_check", skip_all)]
pub async fn health_check() -> AppResult<Json<HealthResponse>> {
    let start_time = std::time::SystemTime::now();
    let uptime = start_time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        uptime: format!("{}s", uptime.as_secs()),
    };

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/rolldice",
    tag = "dice",
    summary = "Roll a dice",
    description = "Rolls a dice with the specified number of sides (default: 6)",
    params(
        ("sides" = Option<u8>, Query, description = "Number of sides on the dice (1-100)")
    ),
    responses(
        (status = 200, description = "Dice rolled successfully", body = DiceRollResponse),
        (status = 400, description = "Invalid parameters"),
        (status = 500, description = "Internal server error")
    )
)]
#[instrument(name = "roll_dice", skip_all, fields(sides))]
pub async fn roll_dice(Query(params): Query<DiceRollQuery>) -> AppResult<Json<DiceRollResponse>> {
    params.validate().map_err(|e| {
        AppError::ValidationError(format!("Invalid parameters: {}", e))
    })?;

    let sides = params.sides.unwrap_or(6);
    tracing::Span::current().record("sides", sides);

    let tracer = global::tracer("protos_api");
    let mut span = tracer
        .span_builder("generate_random_number")
        .with_kind(SpanKind::Internal)
        .start(&tracer);

    if sides == 0 {
        span.set_status(Status::error("Invalid dice sides"));
        return Err(AppError::ValidationError("Dice must have at least 1 side".to_string()));
    }

    let random_value = rand::rng().random_range(1..=sides);
    
    span.add_event(
        "Generated random number".to_string(),
        vec![
            KeyValue::new("value", random_value as i64),
            KeyValue::new("sides", sides as i64),
        ],
    );
    span.set_status(Status::Ok);

    let response = DiceRollResponse {
        value: random_value,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    tracing::info!(
        dice_value = random_value,
        dice_sides = sides,
        "Dice rolled successfully"
    );

    Ok(Json(response))
}