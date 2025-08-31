use std::env;
use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    let rust_env = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    let is_production = rust_env.eq_ignore_ascii_case("production");
    
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    
    if is_production {
        let _ = tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_target(false)
            .try_init();
    } else {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .pretty()
            .try_init();
    }
    
    // metrics::setup_metrics()?;
    // tracing::info!("Starting API server");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    
    fn setup_test() {
        INIT.call_once(|| {
            env::remove_var("RUST_LOG");
            env::remove_var("RUST_ENV");
        });
    }
    
    #[test]
    fn test_init_tracing_development() {
        setup_test();
        env::set_var("RUST_ENV", "development");
        
        init_tracing();
        
        env::remove_var("RUST_ENV");
    }
    
    #[test]
    fn test_init_tracing_production() {
        setup_test();
        env::set_var("RUST_ENV", "production");
        
        init_tracing();
        
        env::remove_var("RUST_ENV");
    }
    
    #[test]
    fn test_init_tracing_default_env() {
        setup_test();
        env::remove_var("RUST_ENV");
        
        init_tracing();
    }
    
    #[test]
    fn test_init_tracing_with_rust_log() {
        setup_test();
        env::set_var("RUST_LOG", "debug");
        env::set_var("RUST_ENV", "development");
        
        init_tracing();
        
        env::remove_var("RUST_LOG");
        env::remove_var("RUST_ENV");
    }
}