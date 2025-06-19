// use crate::b64::b64u_decode;
use std::env;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::LazyLock;

// Example of caching DATABASE_URL with LazyLock
static DATABASE_URL: LazyLock<Result<String>> = LazyLock::new(|| {
  env::var("DATABASE_URL").map_err(|_| Error::MissingEnv("DATABASE_URL"))
});

#[derive(Debug)]
pub struct Env {}

impl Env {
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
    DATABASE_URL.clone()
  }
  pub fn get_redis() -> Result<String> {
    Env::get_env("REDIS_HOST")
  }
  pub fn get_mongo() -> Result<String> {
    Env::get_env("MONGO_URI")
  }
  pub fn get_url() -> String {
    let port = Env::get_port().unwrap();
    format!("{}:{}", Ipv4Addr::UNSPECIFIED, port)
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

#[derive(Debug, Clone)]
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
