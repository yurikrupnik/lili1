use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub environment: Environment,
    pub jwt_secret: String,
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u64,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| eyre!("Invalid PORT value"))?;

        let environment = match env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            _ => Environment::Development,
        };

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            if matches!(environment, Environment::Production) {
                panic!("JWT_SECRET must be set in production");
            }
            "dev-secret-key-change-in-production".to_string()
        });

        if jwt_secret.len() < 32 {
            return Err(eyre!("JWT_SECRET must be at least 32 characters long"));
        }

        let rate_limit_requests = env::var("RATE_LIMIT_REQUESTS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .map_err(|_| eyre!("Invalid RATE_LIMIT_REQUESTS value"))?;

        let rate_limit_window_secs = env::var("RATE_LIMIT_WINDOW_SECS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .map_err(|_| eyre!("Invalid RATE_LIMIT_WINDOW_SECS value"))?;

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            host,
            port,
            environment,
            jwt_secret,
            rate_limit_requests,
            rate_limit_window_secs,
            cors_origins,
        })
    }

    pub fn server_url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            environment: Environment::Development,
            jwt_secret: "dev-secret-key-change-in-production".to_string(),
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
            cors_origins: vec!["*".to_string()],
        }
    }
}
