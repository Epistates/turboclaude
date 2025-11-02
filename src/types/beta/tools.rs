//! Beta tool types for computer use, code execution, and other agentic tools
//!
//! These tools enable Claude to interact with computers, execute code, and perform
//! advanced agentic tasks.

use serde::{Deserialize, Serialize};
use crate::types::CacheControl;

/// Union of all beta tool types
///
/// This enum represents all available beta tools including computer use,
/// bash execution, text editing, code execution, and more.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BetaToolParam {
    /// Custom function tool (standard tool API)
    #[serde(rename = "function")]
    Function {
        /// Tool name
        name: String,
        /// Tool description
        description: String,
        /// JSON schema for input parameters
        input_schema: serde_json::Value,
        /// Cache control settings
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },

    /// Bash/shell command execution (2025-01-24 version)
    #[serde(rename = "bash_20250124")]
    Bash {
        /// Tool name (must be "bash")
        name: String,
        /// Cache control settings
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },

    /// Computer use tool for GUI interaction (2025-01-24 version)
    #[serde(rename = "computer_20250124")]
    ComputerUse {
        /// Tool name (must be "computer")
        name: String,
        /// Display width in pixels
        display_width_px: u32,
        /// Display height in pixels
        display_height_px: u32,
        /// Display number (for multi-monitor setups)
        #[serde(skip_serializing_if = "Option::is_none")]
        display_number: Option<u32>,
        /// Cache control settings
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },

    /// Text editor tool (2025-01-24 version)
    #[serde(rename = "text_editor_20250124")]
    TextEditor {
        /// Tool name (must be "str_replace_editor")
        name: String,
        /// Cache control settings
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },

    /// Code execution tool (2025-08-25 version)
    #[serde(rename = "code_execution_20250825")]
    CodeExecution {
        /// Tool name (must be "code_execution")
        name: String,
        /// Cache control settings
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
}

impl BetaToolParam {
    /// Create a bash tool
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::BetaToolParam;
    ///
    /// let bash = BetaToolParam::bash();
    /// ```
    pub fn bash() -> Self {
        Self::Bash {
            name: "bash".to_string(),
            cache_control: None,
        }
    }

    /// Create a computer use tool
    ///
    /// # Arguments
    ///
    /// * `width` - Display width in pixels
    /// * `height` - Display height in pixels
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::BetaToolParam;
    ///
    /// let computer = BetaToolParam::computer_use(1920, 1080);
    /// ```
    pub fn computer_use(width: u32, height: u32) -> Self {
        Self::ComputerUse {
            name: "computer".to_string(),
            display_width_px: width,
            display_height_px: height,
            display_number: None,
            cache_control: None,
        }
    }

    /// Create a text editor tool
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::BetaToolParam;
    ///
    /// let editor = BetaToolParam::text_editor();
    /// ```
    pub fn text_editor() -> Self {
        Self::TextEditor {
            name: "str_replace_editor".to_string(),
            cache_control: None,
        }
    }

    /// Create a code execution tool
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::BetaToolParam;
    ///
    /// let code_exec = BetaToolParam::code_execution();
    /// ```
    pub fn code_execution() -> Self {
        Self::CodeExecution {
            name: "code_execution".to_string(),
            cache_control: None,
        }
    }

    /// Create a function tool from JSON schema
    ///
    /// # Arguments
    ///
    /// * `name` - Tool name
    /// * `description` - Tool description
    /// * `input_schema` - JSON schema for input parameters
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::BetaToolParam;
    /// use serde_json::json;
    ///
    /// let tool = BetaToolParam::function(
    ///     "get_weather",
    ///     "Get weather for a location",
    ///     json!({
    ///         "type": "object",
    ///         "properties": {
    ///             "location": {"type": "string"}
    ///         },
    ///         "required": ["location"]
    ///     })
    /// );
    /// ```
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self::Function {
            name: name.into(),
            description: description.into(),
            input_schema,
            cache_control: None,
        }
    }

    /// Set cache control for this tool
    pub fn with_cache_control(mut self, cache_control: CacheControl) -> Self {
        match &mut self {
            Self::Function { cache_control: cc, .. }
            | Self::Bash { cache_control: cc, .. }
            | Self::ComputerUse { cache_control: cc, .. }
            | Self::TextEditor { cache_control: cc, .. }
            | Self::CodeExecution { cache_control: cc, .. } => {
                *cc = Some(cache_control);
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash_tool_creation() {
        let bash = BetaToolParam::bash();
        let json = serde_json::to_value(&bash).unwrap();

        assert_eq!(json["type"], "bash_20250124");
        assert_eq!(json["name"], "bash");
    }

    #[test]
    fn test_computer_use_tool_creation() {
        let computer = BetaToolParam::computer_use(1920, 1080);
        let json = serde_json::to_value(&computer).unwrap();

        assert_eq!(json["type"], "computer_20250124");
        assert_eq!(json["name"], "computer");
        assert_eq!(json["display_width_px"], 1920);
        assert_eq!(json["display_height_px"], 1080);
    }

    #[test]
    fn test_text_editor_tool_creation() {
        let editor = BetaToolParam::text_editor();
        let json = serde_json::to_value(&editor).unwrap();

        assert_eq!(json["type"], "text_editor_20250124");
        assert_eq!(json["name"], "str_replace_editor");
    }

    #[test]
    fn test_code_execution_tool_creation() {
        let code_exec = BetaToolParam::code_execution();
        let json = serde_json::to_value(&code_exec).unwrap();

        assert_eq!(json["type"], "code_execution_20250825");
        assert_eq!(json["name"], "code_execution");
    }

    #[test]
    fn test_function_tool_creation() {
        let tool = BetaToolParam::function(
            "get_weather",
            "Get weather for a location",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                },
                "required": ["location"]
            })
        );

        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["type"], "function");
        assert_eq!(json["name"], "get_weather");
        assert_eq!(json["description"], "Get weather for a location");
    }

    #[test]
    fn test_tool_with_cache_control() {
        let bash = BetaToolParam::bash()
            .with_cache_control(CacheControl::ephemeral());

        let json = serde_json::to_value(&bash).unwrap();
        assert!(json.get("cache_control").is_some());
    }

    #[test]
    fn test_tool_deserialization() {
        let json = r#"{
            "type": "computer_20250124",
            "name": "computer",
            "display_width_px": 1024,
            "display_height_px": 768
        }"#;

        let tool: BetaToolParam = serde_json::from_str(json).unwrap();
        match tool {
            BetaToolParam::ComputerUse { display_width_px, display_height_px, .. } => {
                assert_eq!(display_width_px, 1024);
                assert_eq!(display_height_px, 768);
            }
            _ => panic!("Expected ComputerUse variant"),
        }
    }
}
