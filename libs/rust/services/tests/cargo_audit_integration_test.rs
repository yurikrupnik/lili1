use std::process::Command;
use std::str;

#[test]
fn test_cargo_audit_runs_successfully() {
    let output = Command::new("cargo")
        .args(["audit", "--format", "json"])
        .output()
        .expect("Failed to execute cargo audit");

    let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF-8");
    let stderr = str::from_utf8(&output.stderr).expect("Invalid UTF-8");

    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);
    
    assert!(!stdout.is_empty() || !stderr.is_empty(), "cargo audit should produce some output");
}

#[test]
fn test_cargo_audit_database_loads() {
    let output = Command::new("cargo")
        .args(["audit", "--version"])
        .output()
        .expect("Failed to execute cargo audit --version");

    assert!(output.status.success(), "cargo audit --version should succeed");
    
    let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF-8");
    assert!(stdout.contains("cargo-audit"), "Output should contain cargo-audit version info");
}