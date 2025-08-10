use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{roll_dice, health_check, DiceRollResponse, HealthResponse, DiceRollQuery};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::roll_dice,
        crate::handlers::health_check,
    ),
    components(
        schemas(DiceRollResponse, HealthResponse, DiceRollQuery)
    ),
    tags(
        (name = "dice", description = "Dice rolling operations"),
        (name = "health", description = "Health check operations")
    ),
    info(
        title = "Protos API",
        version = "0.1.0",
        description = "A secure API for rolling dice and health checks",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.example.com", description = "Production server")
    )
)]
pub struct ApiDoc;

pub fn create_routes() -> Router {
    Router::new()
        .route("/rolldice", get(roll_dice))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
}

pub mod api_routes {
    use super::*;
    use axum::routing::get;

    pub fn dice_routes() -> Router {
        Router::new()
            .route("/roll", get(roll_dice))
    }

    pub fn create_api_routes() -> Router {
        Router::new()
            .nest("/api/v1/dice", dice_routes())
    }
}