use utoipa::OpenApi;
use rust_services::model::task::Task;
use crate::chat::types::{ChatRequest, ChatResponse, StreamChatResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handler,
        crate::get_tasks,
        crate::chat::chat_handler,
        crate::chat::chat_stream_handler,
        crate::metrics::metrics_handler,
    ),
    components(
        schemas(Task, ChatRequest, ChatResponse, StreamChatResponse)
    ),
    tags(
        (name = "tasks", description = "Task management endpoints"),
        (name = "chat", description = "Chat endpoints"),
        (name = "health", description = "Health check endpoints"),
        (name = "metrics", description = "Metrics endpoints"),
    ),
    info(
        title = "Zerg API",
        description = "API for Zerg application",
        version = "0.1.0"
    )
)]
pub struct ApiDoc;
