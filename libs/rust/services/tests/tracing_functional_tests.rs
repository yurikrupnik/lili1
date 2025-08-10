use rust_services::tracing::init_tracing;
use std::env;
use std::sync::Once;
use tracing::info;

static INIT: Once = Once::new();

fn reset_environment() {
    env::remove_var("RUST_LOG");
    env::remove_var("RUST_ENV");
}

#[test]
fn test_functional_production_environment() {
    INIT.call_once(|| {
        reset_environment();
    });
    
    env::set_var("RUST_ENV", "production");
    env::set_var("RUST_LOG", "warn");
    
    init_tracing();
    
    info!("Test log message for production");
    
    reset_environment();
}

#[test]
fn test_functional_development_environment() {
    reset_environment();
    env::set_var("RUST_ENV", "development");
    env::set_var("RUST_LOG", "debug");
    
    init_tracing();
    
    info!("Test log message for development");
    
    reset_environment();
}

#[test]
fn test_functional_case_insensitive_production() {
    reset_environment();
    env::set_var("RUST_ENV", "PRODUCTION");
    
    init_tracing();
    
    info!("Test log message for case insensitive production");
    
    reset_environment();
}

#[test]
fn test_functional_mixed_case_production() {
    reset_environment();
    env::set_var("RUST_ENV", "PrOdUcTiOn");
    
    init_tracing();
    
    info!("Test log message for mixed case production");
    
    reset_environment();
}

#[test]
fn test_functional_invalid_rust_log() {
    reset_environment();
    env::set_var("RUST_ENV", "development");
    env::set_var("RUST_LOG", "invalid_log_level");
    
    init_tracing();
    
    info!("Test log message with invalid RUST_LOG");
    
    reset_environment();
}

#[test]
fn test_functional_empty_rust_env() {
    reset_environment();
    env::set_var("RUST_ENV", "");
    
    init_tracing();
    
    info!("Test log message with empty RUST_ENV");
    
    reset_environment();
}

#[test]
fn test_functional_staging_environment() {
    reset_environment();
    env::set_var("RUST_ENV", "staging");
    
    init_tracing();
    
    info!("Test log message for staging environment");
    
    reset_environment();
}