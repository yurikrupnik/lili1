pub mod todo;
pub mod error;

// use crate::app_state::AppState;
// use axum::{routing::get, Router};
// use book::router::router as book_router;
// use operations::router::router as operations_router;
// use sql_book::router::router as sql_book_router;
// use sse::router::router as see_router;
// use std::time::Duration;
// use task::router::router as task_router;
// // use task_redis_pubsub::router::router as task_redis_pubsub_router;
// use task_redis_response::router::router as task_redis_response_router;
// use todo::router::router as todo_router;
// use tokio::time::sleep;
//
// pub mod book;
// pub mod operations;
// pub mod sql_book;
// pub mod sse;
// pub mod task;
// // pub mod task_redis_pubsub;
// pub mod task_redis_response;
// pub mod todo;
// mod todo;
//
// pub fn routes() -> Router<AppState> {
//   Router::new()
//     .merge(todo_router())
//     .merge(book_router())
//     .route("/slow", get(|| sleep(Duration::from_secs(5))))
//     .route("/forever", get(std::future::pending::<()>))
//     .merge(operations_router())
//     .merge(sql_book_router())
//     .merge(see_router())
//     .merge(task_router())
//     // .merge(task_redis_pubsub_router())
//     .merge(task_redis_response_router())
// }
