pub mod envs;
pub mod model;
pub mod tracing;
// mod app;

use clap::{Parser, Subcommand, ValueEnum};
use core::str;
// use strum::{Display, EnumString, IntoEnumIterator, VariantNames};
// use strum_macros::{Display, EnumIter, EnumString, EnumVariantNames};
use strum::{Display, EnumString, IntoEnumIterator, VariantNames};
use strum_macros::{EnumIter, EnumVariantNames};
// Define enums with both Strum and Clap derives
#[derive(Debug, Clone, Display, EnumString, EnumIter, EnumVariantNames, ValueEnum)]
#[strum(serialize_all = "lowercase")]
#[clap(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Display, EnumString, EnumIter, ValueEnum)]
#[strum(serialize_all = "lowercase")]
#[clap(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Yaml,
    Toml,
    PrettyJson,
    #[strum(serialize = "csv")]
    CommaSeparated,
}

#[derive(Debug, Clone, Display, EnumString, EnumIter, ValueEnum)]
#[strum(serialize_all = "lowercase")]
pub enum Environment {
    Development,
    #[strum(serialize = "local")]
    #[clap(alias = "d")]
    Staging,
    #[strum(serialize = "dev")]
    #[clap(alias = "s")]
    Production,
    #[strum(serialize = "prod")]
    #[clap(alias = "p")]
    DevelopmentShort,
    #[strum(serialize = "stage")]
    #[clap(alias = "stg")]
    ProductionShort,
}

#[derive(Debug, Parser)]
#[command(name = "myapp")]
#[command(about = "A CLI app demonstrating Strum with Clap")]
#[command(version)]
pub struct Cli {
    #[arg(short, long, value_enum, global = true, default_value = "info")]
    pub log_level: LogLevel,
    pub output_format: OutputFormat,
    pub environment: Environment,
}

// mod db_resource;
// mod reflective;
// use reflective::Reflective;
// use db_resource::DbResource;
// pub use db_resource::DbResource;
// pub use reflective::Reflective;

pub trait Reflective {
    fn name() -> &'static str;
    fn field_names() -> Vec<&'static str>;
    fn field_values(&self) -> Vec<String>;
}

pub trait DbResource {
    const URL: &'static str;
    const COLLECTION: &'static str;
    const TAG: &'static str;
    // const SHIT: &'static str;
    // fn shit(s: &str) -> String;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
