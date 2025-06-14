// use crate::app_state::AppState;
// use axum::{routing, Router};
// use proc_macros::DbResource;
// 
// use super::controller::{create_todo, delete_todo, drop_todos, get_todo, get_todos, update_todo};
// use super::model::Todo;
// 
// /// Todo endpoint router - includes full CRUD!
// pub fn router() -> Router<AppState> {
//   Router::new()
//     .route(
//       Todo::URL,
//       routing::get(get_todos).delete(drop_todos).post(create_todo),
//     )
//     .route(
//       Todo::URL_WITH_ID,
//       routing::put(update_todo).get(get_todo).delete(delete_todo),
//     )
// }