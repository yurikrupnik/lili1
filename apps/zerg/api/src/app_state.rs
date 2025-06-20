// 
// // use bb8::Pool;
// use r2d2::Pool;
// // use bb8_redis::RedisConnectionManager;
// use mongodb::Database;
// use redis::aio::PubSub;
// use redis::Client;
// use sqlx::PgPool;
// use std::sync::{Arc, Mutex};
// // use tokio::sync::Mutex; // or RwLock
// 
// #[derive(Clone)]
// pub struct AppState {
//   pub pool: PgPool,
//   pub db: Database,
//   pub redis: Pool<Client>,
//   pub pubsub: Arc<Mutex<PubSub>>,
// }
// 
// impl AppState {
//   pub fn new(
//     db: Database,
//     redis: Pool<Client>,
//     pool: PgPool,
//     pubsub: Arc<Mutex<PubSub>>,
//     // redis_pub: redis::aio::MultiplexedConnection,
//   ) -> Self {
//     Self {
//       db,
//       redis,
//       pool,
//       pubsub,
//     }
//   }
// }
