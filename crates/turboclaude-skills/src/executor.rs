//! Script execution for skills
//!
//! Provides safe execution of Python and Bash scripts with timeout handling,
//! output capture, and error management.
//!
//! # Example
//!
//! ```no_run
//! use turboclaude_skills::executor::{PythonExecutor, ScriptExecutor};
//! use std::path::Path;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let executor = PythonExecutor::new();
//! let output = executor.execute(
//!     Path::new("script.py"),
//!     &["arg1", "arg2"],
//!     Duration::from_secs(30),
//! ).await?;
//!
//! if output.success() {
//!     println!("Output: {}", output.stdout);
//! } else {
//!     eprintln!("Error: {}", output.stderr);
//! }
//! # Ok(())
//! # }
//! ```

use crate::error::{Result, SkillError};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command;

/// Validates script paths to prevent directory traversal attacks
///
/// Ensures script paths are:
/// - Absolute or properly canonicalized
/// - Within an expected base directory
/// - Not symlinks (optional, configurable)
#[derive(Debug, Clone)]
pub struct PathValidator {
    base_dir: PathBuf,
    allow_symlinks: bool,
}

impl PathValidator {
    /// Create a new path validator with a base directory
    ///
    /// # Arguments
    ///
    /// * `base_dir` - The base directory that scripts must be under
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            allow_symlinks: false,
        }
    }

    /// Allow symlinks (default: false)
    ///
    /// By default, symlinks are rejected for security. Set this to true
    /// to allow scripts to be symlinks (use with caution).
    #[must_use]
    pub fn allow_symlinks(mut self, allow: bool) -> Self {
        self.allow_symlinks = allow;
        self
    }

    /// Validate a script path
    ///
    /// Checks that:
    /// 1. The path exists and is a file
    /// 2. The canonical path is within `base_dir`
    /// 3. The path is not a symlink (unless allowed)
    ///
    /// # Errors
    ///
    /// Returns error if path is invalid, outside base directory, or is a disallowed symlink
    pub fn validate(&self, path: &Path) -> Result<PathBuf> {
        // Check if path exists
        if !path.exists() {
            return Err(SkillError::ScriptExecution(format!(
                "Script path does not exist: {}",
                path.display()
            )));
        }

        // Check if it's a file
        if !path.is_file() {
            return Err(SkillError::ScriptExecution(format!(
                "Script path is not a regular file: {}",
                path.display()
            )));
        }

        // Check symlink policy
        if path.is_symlink() && !self.allow_symlinks {
            return Err(SkillError::ScriptExecution(format!(
                "Symlinks are not allowed: {}",
                path.display()
            )));
        }

        // Canonicalize both paths for comparison
        let canonical_path = path.canonicalize().map_err(|e| {
            SkillError::ScriptExecution(format!("Failed to canonicalize script path: {e}"))
        })?;

        let canonical_base = self.base_dir.canonicalize().map_err(|e| {
            SkillError::ScriptExecution(format!("Failed to canonicalize base directory: {e}"))
        })?;

        // Check that canonical path is within base directory
        if !canonical_path.starts_with(&canonical_base) {
            return Err(SkillError::ScriptExecution(format!(
                "Script path is outside allowed directory: {}",
                path.display()
            )));
        }

        Ok(canonical_path)
    }
}

/// Result of script execution
///
/// Contains all output from the script including stdout, stderr, exit code,
/// timing information, and timeout status.
#[derive(Debug, Clone)]
pub struct ScriptOutput {
    /// Exit code (0 = success)
    pub exit_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Execution duration
    pub duration: Duration,

    /// Whether the script timed out
    pub timed_out: bool,
}

impl ScriptOutput {
    /// Check if script executed successfully
    ///
    /// Returns true only if exit code is 0 and script did not timeout.
    #[must_use]
    pub fn success(&self) -> bool {
        self.exit_code == 0 && !self.timed_out
    }
}

/// Trait for script executors
///
/// Implementors can execute scripts in different languages (Python, Bash, etc.)
/// with timeout handling and output capture.
#[async_trait]
pub trait ScriptExecutor: Send + Sync {
    /// Execute a script with arguments and timeout
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the script file
    /// * `args` - Command-line arguments to pass to the script
    /// * `timeout` - Maximum execution time before killing the process
    ///
    /// # Returns
    ///
    /// `ScriptOutput` containing exit code, stdout, stderr, duration, and timeout status
    ///
    /// # Errors
    ///
    /// Returns error if the script cannot be spawned or executed
    async fn execute(&self, path: &Path, args: &[&str], timeout: Duration) -> Result<ScriptOutput>;

    /// Check if this executor can handle the given file
    ///
    /// Typically checks file extension (.py for Python, .sh for Bash, etc.)
    fn can_execute(&self, path: &Path) -> bool;
}

/// Python script executor
///
/// Executes Python scripts using the python3 interpreter.
pub struct PythonExecutor {
    /// Python interpreter path
    python_path: String,
    /// Optional path validator for security
    path_validator: Option<PathValidator>,
}

impl PythonExecutor {
    /// Create a new Python executor with default path ("python3")
    #[must_use]
    pub fn new() -> Self {
        Self {
            python_path: "python3".to_string(),
            path_validator: None,
        }
    }

    /// Create with custom Python interpreter path
    ///
    /// # Example
    ///
    /// ```
    /// use turboclaude_skills::executor::PythonExecutor;
    ///
    /// let executor = PythonExecutor::with_path("/usr/local/bin/python3.11");
    /// ```
    #[must_use]
    pub fn with_path(python_path: impl Into<String>) -> Self {
        Self {
            python_path: python_path.into(),
            path_validator: None,
        }
    }

    /// Set a path validator for security
    ///
    /// When set, all script paths will be validated against the base directory.
    /// This prevents directory traversal attacks.
    ///
    /// # Example
    ///
    /// ```
    /// use turboclaude_skills::executor::{PythonExecutor, PathValidator};
    /// use std::path::PathBuf;
    ///
    /// let validator = PathValidator::new("/home/user/scripts");
    /// let executor = PythonExecutor::new().with_validator(validator);
    /// ```
    #[must_use]
    pub fn with_validator(mut self, validator: PathValidator) -> Self {
        self.path_validator = Some(validator);
        self
    }
}

impl Default for PythonExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ScriptExecutor for PythonExecutor {
    async fn execute(
        &self,
        path: &Path,
        args: &[&str],
        timeout_duration: Duration,
    ) -> Result<ScriptOutput> {
        let start = Instant::now();

        // Validate path if validator is configured
        if let Some(validator) = &self.path_validator {
            validator.validate(path)?;
        }

        // Build command
        let mut cmd = Command::new(&self.python_path);
        cmd.arg(path);
        cmd.args(args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn process with kill_on_drop to ensure cleanup
        let mut child = cmd
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| SkillError::ScriptExecution(format!("Failed to spawn Python: {e}")))?;

        let child_id = child.id();

        // Manually capture stdout/stderr while monitoring for timeout
        // We need to read output concurrently to avoid deadlocks
        use tokio::io::AsyncReadExt;

        let mut stdout_handle = child.stdout.take().unwrap();
        let mut stderr_handle = child.stderr.take().unwrap();

        let stdout_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            stdout_handle.read_to_end(&mut buf).await.ok();
            buf
        });

        let stderr_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            stderr_handle.read_to_end(&mut buf).await.ok();
            buf
        });

        // Use tokio::select! to handle timeout with proper kill
        let result = tokio::select! {
            status_result = child.wait() => {
                let duration = start.elapsed();
                match status_result {
                    Ok(status) => {
                        // Get output from background tasks
                        let stdout_buf = stdout_task.await.unwrap_or_default();
                        let stderr_buf = stderr_task.await.unwrap_or_default();

                        Ok(ScriptOutput {
                            exit_code: status.code().unwrap_or(-1),
                            stdout: String::from_utf8_lossy(&stdout_buf).to_string(),
                            stderr: String::from_utf8_lossy(&stderr_buf).to_string(),
                            duration,
                            timed_out: false,
                        })
                    }
                    Err(e) => Err(SkillError::ScriptExecution(format!(
                        "Python execution failed: {e}"
                    ))),
                }
            }

            () = tokio::time::sleep(timeout_duration) => {
                // Timeout - kill the process explicitly
                if let Err(e) = child.kill().await {
                    tracing::warn!(
                        "Failed to kill timed-out Python process {}: {}",
                        child_id.unwrap_or(0),
                        e
                    );
                }

                // Abort background tasks since we're timing out
                stdout_task.abort();
                stderr_task.abort();

                let duration = start.elapsed();
                Ok(ScriptOutput {
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Script timed out after {timeout_duration:?}"),
                    duration,
                    timed_out: true,
                })
            }
        };

        result
    }

    fn can_execute(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "py")
    }
}

/// Bash script executor
///
/// Executes Bash scripts using the bash interpreter.
pub struct BashExecutor {
    /// Bash interpreter path
    bash_path: String,
    /// Optional path validator for security
    path_validator: Option<PathValidator>,
}

impl BashExecutor {
    /// Create a new Bash executor with default path ("bash")
    #[must_use]
    pub fn new() -> Self {
        Self {
            bash_path: "bash".to_string(),
            path_validator: None,
        }
    }

    /// Create with custom Bash interpreter path
    ///
    /// # Example
    ///
    /// ```
    /// use turboclaude_skills::executor::BashExecutor;
    ///
    /// let executor = BashExecutor::with_path("/bin/bash");
    /// ```
    #[must_use]
    pub fn with_path(bash_path: impl Into<String>) -> Self {
        Self {
            bash_path: bash_path.into(),
            path_validator: None,
        }
    }

    /// Set a path validator for security
    ///
    /// When set, all script paths will be validated against the base directory.
    /// This prevents directory traversal attacks.
    ///
    /// # Example
    ///
    /// ```
    /// use turboclaude_skills::executor::{BashExecutor, PathValidator};
    /// use std::path::PathBuf;
    ///
    /// let validator = PathValidator::new("/home/user/scripts");
    /// let executor = BashExecutor::new().with_validator(validator);
    /// ```
    #[must_use]
    pub fn with_validator(mut self, validator: PathValidator) -> Self {
        self.path_validator = Some(validator);
        self
    }
}

impl Default for BashExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ScriptExecutor for BashExecutor {
    async fn execute(
        &self,
        path: &Path,
        args: &[&str],
        timeout_duration: Duration,
    ) -> Result<ScriptOutput> {
        let start = Instant::now();

        // Validate path if validator is configured
        if let Some(validator) = &self.path_validator {
            validator.validate(path)?;
        }

        // Build command
        let mut cmd = Command::new(&self.bash_path);
        cmd.arg(path);
        cmd.args(args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn process with kill_on_drop to ensure cleanup
        let mut child = cmd
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| SkillError::ScriptExecution(format!("Failed to spawn Bash: {e}")))?;

        let child_id = child.id();

        // Manually capture stdout/stderr while monitoring for timeout
        // We need to read output concurrently to avoid deadlocks
        use tokio::io::AsyncReadExt;

        let mut stdout_handle = child.stdout.take().unwrap();
        let mut stderr_handle = child.stderr.take().unwrap();

        let stdout_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            stdout_handle.read_to_end(&mut buf).await.ok();
            buf
        });

        let stderr_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            stderr_handle.read_to_end(&mut buf).await.ok();
            buf
        });

        // Use tokio::select! to handle timeout with proper kill
        let result = tokio::select! {
            status_result = child.wait() => {
                let duration = start.elapsed();
                match status_result {
                    Ok(status) => {
                        // Get output from background tasks
                        let stdout_buf = stdout_task.await.unwrap_or_default();
                        let stderr_buf = stderr_task.await.unwrap_or_default();

                        Ok(ScriptOutput {
                            exit_code: status.code().unwrap_or(-1),
                            stdout: String::from_utf8_lossy(&stdout_buf).to_string(),
                            stderr: String::from_utf8_lossy(&stderr_buf).to_string(),
                            duration,
                            timed_out: false,
                        })
                    }
                    Err(e) => Err(SkillError::ScriptExecution(format!(
                        "Bash execution failed: {e}"
                    ))),
                }
            }

            () = tokio::time::sleep(timeout_duration) => {
                // Timeout - kill the process explicitly
                if let Err(e) = child.kill().await {
                    tracing::warn!(
                        "Failed to kill timed-out Bash process {}: {}",
                        child_id.unwrap_or(0),
                        e
                    );
                }

                // Abort background tasks since we're timing out
                stdout_task.abort();
                stderr_task.abort();

                let duration = start.elapsed();
                Ok(ScriptOutput {
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Script timed out after {timeout_duration:?}"),
                    duration,
                    timed_out: true,
                })
            }
        };

        result
    }

    fn can_execute(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "sh")
    }
}

/// Composite executor that routes to the appropriate executor
///
/// Automatically selects the correct executor based on file extension.
/// Default executors: Python (.py), Bash (.sh)
pub struct CompositeExecutor {
    executors: Vec<Box<dyn ScriptExecutor>>,
}

impl CompositeExecutor {
    /// Create a new composite executor with default executors
    ///
    /// Default executors:
    /// - `PythonExecutor` for .py files
    /// - `BashExecutor` for .sh files
    #[must_use]
    pub fn new() -> Self {
        Self {
            executors: vec![
                Box::new(PythonExecutor::new()),
                Box::new(BashExecutor::new()),
            ],
        }
    }

    /// Create with custom executors
    ///
    /// # Example
    ///
    /// ```
    /// use turboclaude_skills::executor::{CompositeExecutor, PythonExecutor, BashExecutor};
    ///
    /// let executor = CompositeExecutor::with_executors(vec![
    ///     Box::new(PythonExecutor::with_path("/usr/bin/python3.11")),
    ///     Box::new(BashExecutor::new()),
    /// ]);
    /// ```
    #[must_use]
    pub fn with_executors(executors: Vec<Box<dyn ScriptExecutor>>) -> Self {
        Self { executors }
    }
}

impl Default for CompositeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ScriptExecutor for CompositeExecutor {
    async fn execute(&self, path: &Path, args: &[&str], timeout: Duration) -> Result<ScriptOutput> {
        // Find appropriate executor
        for executor in &self.executors {
            if executor.can_execute(path) {
                return executor.execute(path, args, timeout).await;
            }
        }

        Err(SkillError::UnsupportedScriptType(
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("unknown")
                .to_string(),
        ))
    }

    fn can_execute(&self, path: &Path) -> bool {
        self.executors.iter().any(|e| e.can_execute(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_validator_valid_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_file = temp_dir.path().join("script.py");
        std::fs::write(&script_file, "print('hello')").unwrap();

        let validator = PathValidator::new(temp_dir.path());
        let result = validator.validate(&script_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_validator_nonexistent_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent.py");

        let validator = PathValidator::new(temp_dir.path());
        let result = validator.validate(&nonexistent);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_validator_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        let validator = PathValidator::new(temp_dir.path());
        let result = validator.validate(&subdir);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_validator_traversal_attempt() {
        let temp_dir = tempfile::tempdir().unwrap();
        let scripts_dir = temp_dir.path().join("scripts");
        std::fs::create_dir(&scripts_dir).unwrap();

        let script_file = scripts_dir.join("script.py");
        std::fs::write(&script_file, "print('hello')").unwrap();

        let parent_dir = temp_dir.path().join("other");
        std::fs::create_dir(&parent_dir).unwrap();

        let validator = PathValidator::new(&scripts_dir);
        // Try to access file outside scripts_dir via ..
        let traversal_path = scripts_dir.join("../other");
        let result = validator.validate(&traversal_path);
        // Should fail because the canonical path is outside scripts_dir
        assert!(result.is_err());
    }

    #[test]
    fn test_python_executor_can_execute() {
        let executor = PythonExecutor::new();
        assert!(executor.can_execute(Path::new("script.py")));
        assert!(!executor.can_execute(Path::new("script.sh")));
        assert!(!executor.can_execute(Path::new("script.txt")));
    }

    #[test]
    fn test_bash_executor_can_execute() {
        let executor = BashExecutor::new();
        assert!(executor.can_execute(Path::new("script.sh")));
        assert!(!executor.can_execute(Path::new("script.py")));
        assert!(!executor.can_execute(Path::new("script.txt")));
    }

    #[test]
    fn test_composite_executor_can_execute() {
        let executor = CompositeExecutor::new();
        assert!(executor.can_execute(Path::new("script.py")));
        assert!(executor.can_execute(Path::new("script.sh")));
        assert!(!executor.can_execute(Path::new("script.txt")));
    }

    #[test]
    fn test_script_output_success() {
        let output = ScriptOutput {
            exit_code: 0,
            stdout: "Success".to_string(),
            stderr: String::new(),
            duration: Duration::from_millis(100),
            timed_out: false,
        };
        assert!(output.success());

        let failed = ScriptOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: "Error".to_string(),
            duration: Duration::from_millis(100),
            timed_out: false,
        };
        assert!(!failed.success());

        let timeout = ScriptOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::from_secs(30),
            timed_out: true,
        };
        assert!(!timeout.success());
    }

    #[test]
    fn test_python_executor_with_path() {
        let executor = PythonExecutor::with_path("/usr/local/bin/python3.11");
        assert_eq!(executor.python_path, "/usr/local/bin/python3.11");
    }

    #[test]
    fn test_bash_executor_with_path() {
        let executor = BashExecutor::with_path("/bin/bash");
        assert_eq!(executor.bash_path, "/bin/bash");
    }
}
