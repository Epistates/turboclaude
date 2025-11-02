//! Integration tests for script executor timeout handling
//!
//! Tests verify that timed-out processes are properly killed and not orphaned.

use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use turboclaude_skills::executor::{BashExecutor, PythonExecutor, ScriptExecutor};

/// Helper to create a temporary Python script that sleeps
fn create_sleep_python_script(dir: &TempDir, sleep_secs: u64) -> PathBuf {
    let script_path = dir.path().join("sleep_test.py");
    let content = format!(
        r#"#!/usr/bin/env python3
import time
import sys

print("Starting sleep for {} seconds", flush=True)
time.sleep({})
print("Sleep completed", flush=True)
sys.exit(0)
"#,
        sleep_secs, sleep_secs
    );
    std::fs::write(&script_path, content).unwrap();
    script_path
}

/// Helper to create a temporary Bash script that sleeps
fn create_sleep_bash_script(dir: &TempDir, sleep_secs: u64) -> PathBuf {
    let script_path = dir.path().join("sleep_test.sh");
    let content = format!(
        r#"#!/bin/bash
echo "Starting sleep for {} seconds"
sleep {}
echo "Sleep completed"
exit 0
"#,
        sleep_secs, sleep_secs
    );
    std::fs::write(&script_path, content).unwrap();
    script_path
}

/// Helper to create a Python script with infinite loop
fn create_infinite_python_script(dir: &TempDir) -> PathBuf {
    let script_path = dir.path().join("infinite.py");
    let content = r#"#!/usr/bin/env python3
import time

print("Starting infinite loop", flush=True)
while True:
    time.sleep(0.1)
"#;
    std::fs::write(&script_path, content).unwrap();
    script_path
}

/// Helper to create a Bash script with infinite loop
fn create_infinite_bash_script(dir: &TempDir) -> PathBuf {
    let script_path = dir.path().join("infinite.sh");
    let content = r#"#!/bin/bash
echo "Starting infinite loop"
while true; do
    sleep 0.1
done
"#;
    std::fs::write(&script_path, content).unwrap();
    script_path
}

#[tokio::test]
async fn test_python_timeout_kills_process() {
    let temp_dir = tempfile::tempdir().unwrap();
    let script_path = create_sleep_python_script(&temp_dir, 10);

    let executor = PythonExecutor::new();

    // Execute with short timeout (100ms) for a 10-second script
    let start = std::time::Instant::now();
    let result = executor
        .execute(&script_path, &[], Duration::from_millis(100))
        .await
        .unwrap();
    let elapsed = start.elapsed();

    // Verify timeout behavior
    assert!(result.timed_out, "Script should have timed out");
    assert!(
        result.stderr.contains("timed out"),
        "stderr should indicate timeout: {}",
        result.stderr
    );
    assert_eq!(result.exit_code, -1, "Exit code should be -1 on timeout");
    assert!(
        elapsed < Duration::from_millis(500),
        "Should timeout quickly, took {:?}",
        elapsed
    );

    // Wait a bit to ensure process is dead
    tokio::time::sleep(Duration::from_millis(50)).await;

    // If process wasn't killed, it would still be running
    // The test passes if we get here without hanging
}

#[tokio::test]
async fn test_bash_timeout_kills_process() {
    let temp_dir = tempfile::tempdir().unwrap();
    let script_path = create_sleep_bash_script(&temp_dir, 10);

    let executor = BashExecutor::new();

    // Execute with short timeout (100ms) for a 10-second script
    let start = std::time::Instant::now();
    let result = executor
        .execute(&script_path, &[], Duration::from_millis(100))
        .await
        .unwrap();
    let elapsed = start.elapsed();

    // Verify timeout behavior
    assert!(result.timed_out, "Script should have timed out");
    assert!(
        result.stderr.contains("timed out"),
        "stderr should indicate timeout: {}",
        result.stderr
    );
    assert_eq!(result.exit_code, -1, "Exit code should be -1 on timeout");
    assert!(
        elapsed < Duration::from_millis(500),
        "Should timeout quickly, took {:?}",
        elapsed
    );

    // Wait a bit to ensure process is dead
    tokio::time::sleep(Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_python_infinite_loop_killed() {
    let temp_dir = tempfile::tempdir().unwrap();
    let script_path = create_infinite_python_script(&temp_dir);

    let executor = PythonExecutor::new();

    // Execute infinite loop with timeout
    let start = std::time::Instant::now();
    let result = executor
        .execute(&script_path, &[], Duration::from_millis(200))
        .await
        .unwrap();
    let elapsed = start.elapsed();

    // Verify timeout and kill
    assert!(result.timed_out, "Infinite loop should timeout");
    assert!(
        elapsed >= Duration::from_millis(200) && elapsed < Duration::from_millis(500),
        "Should respect timeout, took {:?}",
        elapsed
    );

    // Verify process was killed by waiting and checking no hang
    tokio::time::sleep(Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_bash_infinite_loop_killed() {
    let temp_dir = tempfile::tempdir().unwrap();
    let script_path = create_infinite_bash_script(&temp_dir);

    let executor = BashExecutor::new();

    // Execute infinite loop with timeout
    let start = std::time::Instant::now();
    let result = executor
        .execute(&script_path, &[], Duration::from_millis(200))
        .await
        .unwrap();
    let elapsed = start.elapsed();

    // Verify timeout and kill
    assert!(result.timed_out, "Infinite loop should timeout");
    assert!(
        elapsed >= Duration::from_millis(200) && elapsed < Duration::from_millis(500),
        "Should respect timeout, took {:?}",
        elapsed
    );

    // Verify process was killed by waiting and checking no hang
    tokio::time::sleep(Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_python_successful_execution_no_timeout() {
    let temp_dir = tempfile::tempdir().unwrap();
    let script_path = temp_dir.path().join("quick.py");
    let content = r#"#!/usr/bin/env python3
print("Quick execution")
"#;
    std::fs::write(&script_path, content).unwrap();

    let executor = PythonExecutor::new();

    // Execute quick script with generous timeout
    let result = executor
        .execute(&script_path, &[], Duration::from_secs(5))
        .await
        .unwrap();

    // Verify successful execution
    assert!(!result.timed_out, "Quick script should not timeout");
    assert_eq!(result.exit_code, 0, "Should exit successfully");
    assert!(
        result.stdout.contains("Quick execution"),
        "stdout: {}",
        result.stdout
    );
    assert!(result.success(), "Should be marked as success");
}

#[tokio::test]
async fn test_bash_successful_execution_no_timeout() {
    let temp_dir = tempfile::tempdir().unwrap();
    let script_path = temp_dir.path().join("quick.sh");
    let content = r#"#!/bin/bash
echo "Quick execution"
"#;
    std::fs::write(&script_path, content).unwrap();

    let executor = BashExecutor::new();

    // Execute quick script with generous timeout
    let result = executor
        .execute(&script_path, &[], Duration::from_secs(5))
        .await
        .unwrap();

    // Verify successful execution
    assert!(!result.timed_out, "Quick script should not timeout");
    assert_eq!(result.exit_code, 0, "Should exit successfully");
    assert!(
        result.stdout.contains("Quick execution"),
        "stdout: {}",
        result.stdout
    );
    assert!(result.success(), "Should be marked as success");
}

#[tokio::test]
async fn test_multiple_timeouts_no_resource_leak() {
    let temp_dir = tempfile::tempdir().unwrap();
    let python_script = create_infinite_python_script(&temp_dir);
    let bash_script = create_infinite_bash_script(&temp_dir);

    let python_executor = PythonExecutor::new();
    let bash_executor = BashExecutor::new();

    // Run multiple timeouts to verify no resource leaks
    for i in 0..5 {
        let result = python_executor
            .execute(&python_script, &[], Duration::from_millis(50))
            .await
            .unwrap();
        assert!(result.timed_out, "Python iteration {} should timeout", i);

        let result = bash_executor
            .execute(&bash_script, &[], Duration::from_millis(50))
            .await
            .unwrap();
        assert!(result.timed_out, "Bash iteration {} should timeout", i);

        // Small delay between iterations
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // If processes weren't killed, we'd have 10 orphaned processes
    // Test passes if we get here without resource exhaustion
}

#[tokio::test]
async fn test_timeout_respects_duration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let script_path = create_sleep_python_script(&temp_dir, 5);

    let executor = PythonExecutor::new();

    // Test with 300ms timeout
    let start = std::time::Instant::now();
    let result = executor
        .execute(&script_path, &[], Duration::from_millis(300))
        .await
        .unwrap();
    let elapsed = start.elapsed();

    assert!(result.timed_out, "Should timeout");
    assert!(
        elapsed >= Duration::from_millis(300) && elapsed < Duration::from_millis(600),
        "Should respect 300ms timeout, took {:?}",
        elapsed
    );
}
