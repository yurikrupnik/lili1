// use crate::b64::b64u_decode;
use std::env;
// use std::fmt::Result;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[derive(Debug)]
pub struct Env {}

impl Env {
    pub fn is_prod() -> Result<bool> {
        let value = Env::get_env("RUST_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .eq_ignore_ascii_case("production");
        Ok(value)
        // let rust_env = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
        // let is_production = rust_env.eq_ignore_ascii_case("production");
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
    pub fn get_postgres() -> Result<String> {
        Env::get_env("DATABASE_URL")
    }

    pub fn get_redis() -> Result<String> {
        Env::get_env("REDIS_HOST")
    }
    pub fn get_mongo() -> Result<String> {
        Env::get_env("MONGO_URI")
    }
    pub fn get_url() -> Result<String> {
        Ok(format!("{}:{}", Ipv4Addr::UNSPECIFIED, Env::get_port()?))
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

#[derive(Debug)]
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
