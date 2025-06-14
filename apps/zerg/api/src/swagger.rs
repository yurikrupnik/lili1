//
// use super::api::{
//   book::controller as book,
//   operations::controller as operation,
//   sql_book::controller as sql_book,
//   sse::controller as sse,
//   task::controller as task,
//   // task_redis_pubsub::controller as task_redis_pubsub,
//   // task_redis_response::controller as task_redis_response, // , todo::controller as todo,
//   todo::controller as todo,
// };
// use
// ::OpenApi;
//
// /// Main structure to generate OpenAPI documentation
// #[derive(OpenApi)]
// #[openapi(
//   paths(
//     todo::get_todo,
//     todo::get_todos,
//     todo::create_todo,
//     todo::update_todo,
//     todo::delete_todo,
//     todo::drop_todos,
//     book::get_book,
//     book::get_books,
//     book::create_book,
//     book::update_book,
//     book::delete_book,
//     book::drop_books,
//     sql_book::get_book,
//     sql_book::get_books,
//     sql_book::drop_books,
//     sql_book::delete_book,
//     sql_book::update_book,
//     sql_book::create_book,
//     sse::sse_handler,
//     // task_redis_response::get_task,
//     // task_redis_response::get_tasks,
//     // task_redis_response::create_task_redis,
//     // task_redis_response::update_task,
//     // task_redis_response::delete_task,
//     // task_redis_response::drop_tasks,
//     // task_redis_pubsub::get_task,
//     // task_redis_pubsub::get_tasks,
//     // task_redis_pubsub::create_task_redis,
//     // task_redis_pubsub::update_task,
//     // task_redis_pubsub::delete_task,
//     // task_redis_pubsub::drop_tasks,
//     // task::get_task,
//     // task::get_tasks,
//     // task::create_task,
//     // task::update_task,
//     // task::delete_task,
//     // task::drop_tasks,
//     // operation::read_buckets,
//     // operation::delete_bucket,
//   ),
//   components(schemas())
// )]
// pub(crate) struct ApiDoc;
