use clap::Parser;
// use cli::format_option;
// use proc_macros::DbResource;
// use redis::{RedisWrite, ToRedisArgs};
// use redis_derive::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::types::Uuid;
use sqlx::{FromRow, Row};
// use tabled::Tabled;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
// #[schema(
//   example = json!(
//     {
//       "id": "00000000-0000-0000-0000-000000000000",
//       "title": "Task title",
//       "description": "Task description",
//       "completed": false
//     }
//   )
// )]
pub struct Task {
    #[schema(default = "00000000-0000-0000-0000-000000000000")]
    pub id: Uuid,
    /// Title of the task
    pub title: String,
    // #[tabled(display_with = "format_option")]
    /// description of the task
    pub description: Option<String>,
    /// completed state of the task
    pub completed: bool,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct UpdateTask {
    /// Task title
    #[validate(length(min = 2))]
    pub title: Option<String>,
    /// Task description
    pub description: Option<String>,
    /// Task completed state
    pub completed: Option<bool>,
    // pub id: Vec<String>,
}

#[derive(Debug, Validate, Deserialize, Serialize, ToSchema)]
pub struct CreateTask {
    /// Task title
    #[validate(length(min = 2))]
    title: String,
    /// Task description
    #[validate(length(min = 4))]
    description: String,
    /// Task state
    completed: bool,
}

// impl ToRedisArgs for CreateTask {
//   fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
//     // Manually convert to JSON and push as one argument
//     let as_json = serde_json::to_string(self).unwrap();
//     out.write_arg(as_json.as_bytes());
//   }
// }

impl<'r> FromRow<'r, PgRow> for Task {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Task {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            completed: row.try_get("completed")?,
        })
    }
}

#[derive(Parser, Debug)]
pub struct UpdateTaskArgs {
    /// Task title
    #[clap(long, short)]
    pub title: Option<String>,
    /// Task description
    #[clap(long, short)]
    pub description: Option<String>,
    /// Task completed state
    #[clap(long, short)]
    pub completed: Option<bool>,
}

#[derive(Parser, Debug)]
pub struct CreateTaskArgs {
    /// Task title
    title: String,
    /// Task description
    description: String,
    /// Task state
    #[clap(short, long)]
    completed: bool,
}

impl From<UpdateTaskArgs> for UpdateTask {
    fn from(args: UpdateTaskArgs) -> Self {
        UpdateTask {
            title: args.title,
            description: args.description,
            completed: args.completed,
        }
    }
}

impl From<CreateTaskArgs> for CreateTask {
    fn from(args: CreateTaskArgs) -> Self {
        CreateTask {
            title: args.title,
            description: args.description,
            completed: args.completed,
        }
    }
}
