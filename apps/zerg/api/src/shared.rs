// use crate::b64::b64u_decode;
use std::env;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::LazyLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use futures_util::{sink::SinkExt, stream::StreamExt};
use axum::{
  extract::{
    ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    State,
  },
  response::{Html, IntoResponse},
  routing::get,
  Router,
};

// use tokio::sync::broadcast;
// Example of caching DATABASE_URL with LazyLock
// static DATABASE_URL: LazyLock<Result<String>> = LazyLock::new(|| {
//   env::var("DATABASE_URL").map_err(|_| Error::MissingEnv("DATABASE_URL"))
// });

// let s  || {
// Env::get_env_parse("PORT")
// }

static PORT: LazyLock<Result<String>> = LazyLock::new(|| {
  Env::get_env_parse("PORT")
});
static HOST: LazyLock<Result<String>> = LazyLock::new(|| {
  Env::get_env("HOST")
});
static MONGO_URI: LazyLock<Result<String>> = LazyLock::new(|| {
  Env::get_env("MONGO_URI")
});

#[derive(Debug)]
pub struct Env {}

impl Env {
  pub fn get_enva(name: &'static str) -> Result<String> {
    // let config = Env::get_env("MONGO_URI");
    // config
    // tracing_subscriber::registry()
    //   .with(
    //     tracing_subscriber::EnvFilter::try_from_default_env()
    //       .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
    //   )
    //   .with(tracing_subscriber::fmt::layer())
    //   .init();
    env::var(name).map_err(|_| Error::MissingEnv(name))
  }
  pub fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::MissingEnv(name))
  }
  pub fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = Env::get_env(name)?;
    val.parse::<T>().map_err(|_| Error::WrongFormat(name))
  }
  pub fn get_port() -> Result<u16> {
    Env::get_env_parse("PORT")
  }
  // pub fn get_postgres() -> Result<String> {
  //     Env::get_env("DATABASE_URL")
  // }

  pub fn get_postgres() -> Result<String> {
    static DATABASE_URL: LazyLock<Result<String>> = LazyLock::new(|| {
      Env::get_env("DATABASE_URL")
    });
    DATABASE_URL.clone()
  }
  pub fn get_redis() -> Result<String> {
    static REDIS_URI: LazyLock<Result<String>> = LazyLock::new(|| {
      Env::get_env("REDIS_URI")
    });
    REDIS_URI.clone()
  }
  pub fn get_mongo() -> Result<String> {
    MONGO_URI.clone()
  }
  pub fn get_url() -> Result<String> {
    use std::net::{SocketAddrV4, Ipv4Addr};
    let port = Env::get_port()?;
    let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
    Ok(socket_addr.to_string())
  }
  
  // pub fn new() -> Result<Self> {
  //   Self::new({
  //     
  //   }).map(|_| Self {})
  // }
}

#[derive(Debug)]
pub struct ConfigProps {
  pub host: String,
  pub port: u16,
  pub mongo_uri: String,
  pub redis_uri: String,
  pub postgres_uri: String,
  pub influxdb_uri: String,
}

impl ConfigProps {
  pub fn new() -> Result<Self> {
    let host = Env::get_env("HOST")?;
    let port = Env::get_env_parse("PORT")?;
    let mongo_uri = Env::get_env("MONGO_URI")?;
    let redis_uri = Env::get_redis()?;
    // let postgres_uri = Env::get_env("POSTGRES_URI")?;
    let postgres_uri = Env::get_postgres()?;
    let influxdb_uri = Env::get_env("INFLUXDB_URL")?;
    Ok(Self {
      host,
      port,
      mongo_uri,
      redis_uri,
      postgres_uri,
      influxdb_uri
    })
  }
}

pub fn get_env(name: &'static str) -> Result<String> {
  env::var(name).map_err(|_| Error::MissingEnv(name))
}

pub fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
  let val = get_env(name)?;
  val.parse::<T>().map_err(|_| Error::WrongFormat(name))
}

// pub fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
//   b64u_decode(&get_env(name)?).map_err(|_| Error::WrongFormat(name))
// }

// region:    --- Error
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
  MissingEnv(&'static str),
  WrongFormat(&'static str),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
  fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
    write!(fmt, "{self:?}")
  }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate

// endregion: --- Error
