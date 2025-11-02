use std::error::Error;

/// A boundary between two error types.
///
/// This trait enables declarative error conversion at module/crate boundaries.
/// It provides a clean abstraction for converting between error types while
/// preserving context and enabling the `?` operator to work seamlessly.
///
/// # Design Philosophy
///
/// Error boundaries eliminate boilerplate `map_err()` calls by declaratively
/// defining conversion rules at module/crate boundaries. This follows the
/// principle of "errors as values" while maintaining type safety.
///
/// # Example
///
/// ```
/// use turboclaude_core::error::ErrorBoundary;
/// use turboclaude_core::error_boundary;
/// use std::io;
///
/// #[derive(Debug, thiserror::Error)]
/// enum AppError {
///     #[error("IO error: {0}")]
///     Io(String),
///     #[error("Parse error: {0}")]
///     Parse(String),
/// }
///
/// // Define the boundary - this generates both ErrorBoundary impl AND From impl
/// error_boundary!(io::Error => AppError, |e| {
///     AppError::Io(e.to_string())
/// });
///
/// // Now you can use ? operator directly
/// fn read_config() -> Result<String, AppError> {
///     let content = std::fs::read_to_string("config.toml")?; // Auto-converts!
///     Ok(content)
/// }
/// ```
pub trait ErrorBoundary {
    /// The inner error type (source of conversion).
    type Inner: Error + Send + Sync;

    /// The outer error type (target of conversion).
    type Outer: Error + Send + Sync;

    /// Convert from inner error to outer error.
    ///
    /// This method defines the conversion logic, preserving as much
    /// context as possible from the original error.
    fn convert(inner: Self::Inner) -> Self::Outer;
}

/// Macro to define error boundaries with automatic `From` implementation.
///
/// This macro generates a `From` implementation for error conversion,
/// enabling seamless use of the `?` operator.
///
/// # Syntax
///
/// ```ignore
/// error_boundary!(SourceError => TargetError, |err_var| {
///     // conversion logic returning TargetError
/// });
/// ```
///
/// # Example
///
/// ```
/// use turboclaude_core::error_boundary;
/// use std::io;
///
/// #[derive(Debug, thiserror::Error)]
/// enum MyError {
///     #[error("IO: {0}")]
///     Io(String),
///     #[error("Network: {0}")]
///     Network(String),
/// }
///
/// // Define IO error boundary
/// error_boundary!(io::Error => MyError, |e| {
///     MyError::Io(format!("{}", e))
/// });
///
/// // Now you can use ? with io::Error functions
/// fn read_file() -> Result<Vec<u8>, MyError> {
///     let data = std::fs::read("/tmp/test.txt")?;
///     Ok(data)
/// }
/// ```
///
/// # Multiple Boundaries
///
/// You can define multiple boundaries for the same target error:
///
/// ```
/// use turboclaude_core::error_boundary;
/// use std::io;
///
/// #[derive(Debug, thiserror::Error)]
/// enum AppError {
///     #[error("IO: {0}")]
///     Io(String),
///     #[error("JSON: {0}")]
///     Json(String),
/// }
///
/// error_boundary!(io::Error => AppError, |e| {
///     AppError::Io(e.to_string())
/// });
///
/// error_boundary!(serde_json::Error => AppError, |e| {
///     AppError::Json(e.to_string())
/// });
///
/// // Now both error types convert automatically
/// fn load_config() -> Result<serde_json::Value, AppError> {
///     let content = std::fs::read_to_string("config.json")?; // io::Error
///     let json = serde_json::from_str(&content)?; // serde_json::Error
///     Ok(json)
/// }
/// ```
#[macro_export]
macro_rules! error_boundary {
    ($inner:ty => $outer:ty, |$err:ident| $body:expr) => {
        // The From impl is what enables `?` operator
        impl ::std::convert::From<$inner> for $outer {
            fn from($err: $inner) -> $outer {
                $body
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::io;

    #[derive(Debug, thiserror::Error, PartialEq)]
    enum TestError {
        #[error("IO: {0}")]
        Io(String),
        #[error("Parse: {0}")]
        Parse(String),
        #[error("Unknown: {0}")]
        Unknown(String),
    }

    // Define error boundary for io::Error
    error_boundary!(io::Error => TestError, |e| {
        TestError::Io(e.to_string())
    });

    #[test]
    fn test_error_boundary_direct_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let test_error: TestError = io_error.into();

        match test_error {
            TestError::Io(msg) => {
                assert!(msg.contains("file not found"));
            }
            _ => panic!("Expected Io variant"),
        }
    }

    #[test]
    fn test_error_boundary_with_question_mark() {
        fn do_io() -> Result<String, TestError> {
            // This should auto-convert io::Error to TestError via From impl
            let content = std::fs::read_to_string("/nonexistent/path/that/does/not/exist")?;
            Ok(content)
        }

        let result = do_io();
        assert!(result.is_err());

        match result.unwrap_err() {
            TestError::Io(msg) => {
                assert!(msg.contains("No such file") || msg.contains("cannot find"));
            }
            _ => panic!("Expected Io variant"),
        }
    }

    #[test]
    #[allow(non_local_definitions)]
    fn test_multiple_error_boundaries() {
        // Define another boundary for the same target type
        error_boundary!(std::num::ParseIntError => TestError, |e| {
            TestError::Parse(e.to_string())
        });

        fn parse_number(s: &str) -> Result<i32, TestError> {
            let num = s.parse::<i32>()?; // auto-converts ParseIntError
            Ok(num)
        }

        let result = parse_number("not_a_number");
        assert!(result.is_err());

        match result.unwrap_err() {
            TestError::Parse(msg) => {
                assert!(msg.contains("invalid digit"));
            }
            _ => panic!("Expected Parse variant"),
        }
    }

    #[test]
    #[allow(non_local_definitions)]
    fn test_chained_error_conversions() {
        error_boundary!(std::fmt::Error => TestError, |_e| {
            TestError::Unknown("format error".to_string())
        });

        fn complex_operation() -> Result<(), TestError> {
            // Chain multiple operations with different error types
            let _content = std::fs::read_to_string("/nonexistent")?;
            Ok(())
        }

        let result = complex_operation();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_context_preservation() {
        let io_error = io::Error::new(
            io::ErrorKind::PermissionDenied,
            "access denied to /etc/shadow",
        );

        let test_error: TestError = io_error.into();

        match test_error {
            TestError::Io(msg) => {
                assert!(msg.contains("access denied"));
                assert!(msg.contains("/etc/shadow"));
            }
            _ => panic!("Expected Io variant"),
        }
    }
}
