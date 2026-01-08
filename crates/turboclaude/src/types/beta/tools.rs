//! Beta tool types for computer use, code execution, and other agentic tools
//!
//! These tools enable Claude to interact with computers, execute code, and perform
//! advanced agentic tasks.

use crate::types::CacheControl;
use serde::{Deserialize, Serialize};

/// Allowed callers for tools that support caller restrictions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AllowedCaller {
    /// Tool can be called directly by the model
    Direct,
    /// Tool can be called from code execution context (2025-08-25)
    #[serde(rename = "code_execution_20250825")]
    CodeExecution20250825,
}

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

    /// Web search tool (2025-03-05 version)
    ///
    /// Enables Claude to search the web and retrieve real-time information.
    /// Requires beta header: "anthropic-beta": "web-search-2025-03-05"
    ///
    /// Pricing: $10 per 1,000 searches + standard token costs
    #[serde(rename = "web_search_20250305")]
    WebSearch {
        /// Tool name (must be "web_search")
        name: String,
        /// Cache control settings
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
        /// Allowed callers for this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_callers: Option<Vec<AllowedCaller>>,
        /// Whether to defer loading this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
        /// Strict mode for tool execution
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },

    /// Web fetch tool (2025-09-10 version)
    ///
    /// Enables Claude to fetch content from URLs.
    #[serde(rename = "web_fetch_20250910")]
    WebFetch {
        /// Tool name (must be "web_fetch")
        name: String,
        /// Cache control settings
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
        /// Allowed callers for this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_callers: Option<Vec<AllowedCaller>>,
        /// Whether to defer loading this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
        /// Strict mode for tool execution
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },

    /// Computer use tool (2025-11-24 version)
    ///
    /// Latest version with zoom support and input examples.
    #[serde(rename = "computer_20251124")]
    ComputerUse20251124 {
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
        /// Allowed callers for this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_callers: Option<Vec<AllowedCaller>>,
        /// Whether to defer loading this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
        /// Enable zoomed screenshot action
        #[serde(skip_serializing_if = "Option::is_none")]
        enable_zoom: Option<bool>,
        /// Example inputs for the model
        #[serde(skip_serializing_if = "Option::is_none")]
        input_examples: Option<Vec<serde_json::Value>>,
        /// Strict mode for tool execution
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },

    /// Tool search BM25 tool (2025-11-19 version)
    ///
    /// Enables tool discovery using BM25 search algorithm.
    #[serde(rename = "tool_search_tool_bm25_20251119")]
    ToolSearchBm25 {
        /// Tool name (must be "tool_search_tool_bm25")
        name: String,
        /// Cache control settings
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
        /// Allowed callers for this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_callers: Option<Vec<AllowedCaller>>,
        /// Whether to defer loading this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
        /// Strict mode for tool execution
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },

    /// Tool search regex tool (2025-11-19 version)
    ///
    /// Enables tool discovery using regex patterns.
    #[serde(rename = "tool_search_tool_regex_20251119")]
    ToolSearchRegex {
        /// Tool name (must be "tool_search_tool_regex")
        name: String,
        /// Cache control settings
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
        /// Allowed callers for this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_callers: Option<Vec<AllowedCaller>>,
        /// Whether to defer loading this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
        /// Strict mode for tool execution
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },

    /// MCP toolset for integrating MCP servers
    #[serde(rename = "mcp_toolset")]
    McpToolset {
        /// MCP server name
        mcp_server_name: String,
        /// Cache control settings
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
        /// Per-tool configuration overrides
        #[serde(skip_serializing_if = "Option::is_none")]
        configs: Option<std::collections::HashMap<String, McpToolConfig>>,
        /// Default configuration for all tools
        #[serde(skip_serializing_if = "Option::is_none")]
        default_config: Option<McpToolDefaultConfig>,
    },

    /// Memory tool (2025-08-18 version)
    ///
    /// Enables persistent memory across conversations.
    #[serde(rename = "memory_20250818")]
    Memory {
        /// Tool name
        name: String,
        /// Cache control settings
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
        /// Allowed callers for this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_callers: Option<Vec<AllowedCaller>>,
        /// Whether to defer loading this tool
        #[serde(skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
        /// Strict mode for tool execution
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
}

/// MCP tool configuration for per-tool overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolConfig {
    /// Whether to defer loading this tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
    /// Whether this tool is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// Default MCP tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefaultConfig {
    /// Whether to defer loading tools by default
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
    /// Whether tools are enabled by default
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
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

    /// Create a web search tool
    ///
    /// Enables Claude to automatically search the web when it would improve the answer.
    /// Claude decides when to use web search and provides citations for information.
    ///
    /// # Requirements
    ///
    /// - Requires beta header: `"anthropic-beta": "web-search-2025-03-05"`
    /// - Available for: Claude 3.7 Sonnet, Claude 3.5 Sonnet (upgraded), Claude 3.5 Haiku
    /// - Pricing: $10 per 1,000 searches + standard token costs
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::BetaToolParam;
    ///
    /// let web_search = BetaToolParam::web_search();
    /// ```
    pub fn web_search() -> Self {
        Self::WebSearch {
            name: "web_search".to_string(),
            cache_control: None,
            allowed_callers: None,
            defer_loading: None,
            strict: None,
        }
    }

    /// Create a web fetch tool
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::BetaToolParam;
    ///
    /// let web_fetch = BetaToolParam::web_fetch();
    /// ```
    pub fn web_fetch() -> Self {
        Self::WebFetch {
            name: "web_fetch".to_string(),
            cache_control: None,
            allowed_callers: None,
            defer_loading: None,
            strict: None,
        }
    }

    /// Create a computer use tool (2025-11-24 version) with zoom support
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
    /// let computer = BetaToolParam::computer_use_v2(1920, 1080);
    /// ```
    pub fn computer_use_v2(width: u32, height: u32) -> Self {
        Self::ComputerUse20251124 {
            name: "computer".to_string(),
            display_width_px: width,
            display_height_px: height,
            display_number: None,
            cache_control: None,
            allowed_callers: None,
            defer_loading: None,
            enable_zoom: None,
            input_examples: None,
            strict: None,
        }
    }

    /// Create a tool search BM25 tool
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::BetaToolParam;
    ///
    /// let tool_search = BetaToolParam::tool_search_bm25();
    /// ```
    pub fn tool_search_bm25() -> Self {
        Self::ToolSearchBm25 {
            name: "tool_search_tool_bm25".to_string(),
            cache_control: None,
            allowed_callers: None,
            defer_loading: None,
            strict: None,
        }
    }

    /// Create a tool search regex tool
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::BetaToolParam;
    ///
    /// let tool_search = BetaToolParam::tool_search_regex();
    /// ```
    pub fn tool_search_regex() -> Self {
        Self::ToolSearchRegex {
            name: "tool_search_tool_regex".to_string(),
            cache_control: None,
            allowed_callers: None,
            defer_loading: None,
            strict: None,
        }
    }

    /// Create an MCP toolset
    ///
    /// # Arguments
    ///
    /// * `server_name` - The MCP server name
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::BetaToolParam;
    ///
    /// let mcp = BetaToolParam::mcp_toolset("my_server");
    /// ```
    pub fn mcp_toolset(server_name: impl Into<String>) -> Self {
        Self::McpToolset {
            mcp_server_name: server_name.into(),
            cache_control: None,
            configs: None,
            default_config: None,
        }
    }

    /// Create a memory tool
    ///
    /// # Example
    ///
    /// ```rust
    /// use turboclaude::types::beta::BetaToolParam;
    ///
    /// let memory = BetaToolParam::memory();
    /// ```
    pub fn memory() -> Self {
        Self::Memory {
            name: "memory".to_string(),
            cache_control: None,
            allowed_callers: None,
            defer_loading: None,
            strict: None,
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
            Self::Function {
                cache_control: cc, ..
            }
            | Self::Bash {
                cache_control: cc, ..
            }
            | Self::ComputerUse {
                cache_control: cc, ..
            }
            | Self::TextEditor {
                cache_control: cc, ..
            }
            | Self::CodeExecution {
                cache_control: cc, ..
            }
            | Self::WebSearch {
                cache_control: cc, ..
            }
            | Self::WebFetch {
                cache_control: cc, ..
            }
            | Self::ComputerUse20251124 {
                cache_control: cc, ..
            }
            | Self::ToolSearchBm25 {
                cache_control: cc, ..
            }
            | Self::ToolSearchRegex {
                cache_control: cc, ..
            }
            | Self::McpToolset {
                cache_control: cc, ..
            }
            | Self::Memory {
                cache_control: cc, ..
            } => {
                *cc = Some(cache_control);
            }
        }
        self
    }

    /// Set allowed callers for this tool (for tools that support it)
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        match &mut self {
            Self::WebSearch {
                allowed_callers: ac,
                ..
            }
            | Self::WebFetch {
                allowed_callers: ac,
                ..
            }
            | Self::ComputerUse20251124 {
                allowed_callers: ac,
                ..
            }
            | Self::ToolSearchBm25 {
                allowed_callers: ac,
                ..
            }
            | Self::ToolSearchRegex {
                allowed_callers: ac,
                ..
            }
            | Self::Memory {
                allowed_callers: ac,
                ..
            } => {
                *ac = Some(allowed_callers);
            }
            _ => {} // Other tools don't support allowed_callers
        }
        self
    }

    /// Set defer loading for this tool (for tools that support it)
    pub fn with_defer_loading(mut self, defer: bool) -> Self {
        match &mut self {
            Self::WebSearch {
                defer_loading: dl, ..
            }
            | Self::WebFetch {
                defer_loading: dl, ..
            }
            | Self::ComputerUse20251124 {
                defer_loading: dl, ..
            }
            | Self::ToolSearchBm25 {
                defer_loading: dl, ..
            }
            | Self::ToolSearchRegex {
                defer_loading: dl, ..
            }
            | Self::Memory {
                defer_loading: dl, ..
            } => {
                *dl = Some(defer);
            }
            _ => {} // Other tools don't support defer_loading
        }
        self
    }

    /// Enable zoom for computer use (v2) tool
    pub fn with_zoom_enabled(mut self, enable: bool) -> Self {
        if let Self::ComputerUse20251124 { enable_zoom, .. } = &mut self {
            *enable_zoom = Some(enable);
        }
        self
    }

    /// Set input examples for computer use (v2) tool
    pub fn with_input_examples(mut self, examples: Vec<serde_json::Value>) -> Self {
        if let Self::ComputerUse20251124 { input_examples, .. } = &mut self {
            *input_examples = Some(examples);
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
            }),
        );

        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["type"], "function");
        assert_eq!(json["name"], "get_weather");
        assert_eq!(json["description"], "Get weather for a location");
    }

    #[test]
    fn test_tool_with_cache_control() {
        let bash = BetaToolParam::bash().with_cache_control(CacheControl::ephemeral());

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
            BetaToolParam::ComputerUse {
                display_width_px,
                display_height_px,
                ..
            } => {
                assert_eq!(display_width_px, 1024);
                assert_eq!(display_height_px, 768);
            }
            _ => panic!("Expected ComputerUse variant"),
        }
    }

    #[test]
    fn test_web_fetch_tool_creation() {
        let web_fetch = BetaToolParam::web_fetch();
        let json = serde_json::to_value(&web_fetch).unwrap();

        assert_eq!(json["type"], "web_fetch_20250910");
        assert_eq!(json["name"], "web_fetch");
    }

    #[test]
    fn test_computer_use_v2_tool_creation() {
        let computer = BetaToolParam::computer_use_v2(1920, 1080)
            .with_zoom_enabled(true)
            .with_input_examples(vec![serde_json::json!({"action": "click"})]);
        let json = serde_json::to_value(&computer).unwrap();

        assert_eq!(json["type"], "computer_20251124");
        assert_eq!(json["name"], "computer");
        assert_eq!(json["display_width_px"], 1920);
        assert_eq!(json["display_height_px"], 1080);
        assert_eq!(json["enable_zoom"], true);
        assert!(json["input_examples"].is_array());
    }

    #[test]
    fn test_tool_search_bm25_creation() {
        let tool = BetaToolParam::tool_search_bm25();
        let json = serde_json::to_value(&tool).unwrap();

        assert_eq!(json["type"], "tool_search_tool_bm25_20251119");
        assert_eq!(json["name"], "tool_search_tool_bm25");
    }

    #[test]
    fn test_tool_search_regex_creation() {
        let tool = BetaToolParam::tool_search_regex();
        let json = serde_json::to_value(&tool).unwrap();

        assert_eq!(json["type"], "tool_search_tool_regex_20251119");
        assert_eq!(json["name"], "tool_search_tool_regex");
    }

    #[test]
    fn test_mcp_toolset_creation() {
        let tool = BetaToolParam::mcp_toolset("my_server");
        let json = serde_json::to_value(&tool).unwrap();

        assert_eq!(json["type"], "mcp_toolset");
        assert_eq!(json["mcp_server_name"], "my_server");
    }

    #[test]
    fn test_memory_tool_creation() {
        let tool = BetaToolParam::memory();
        let json = serde_json::to_value(&tool).unwrap();

        assert_eq!(json["type"], "memory_20250818");
        assert_eq!(json["name"], "memory");
    }

    #[test]
    fn test_allowed_callers_serialization() {
        let tool = BetaToolParam::web_search()
            .with_allowed_callers(vec![AllowedCaller::Direct, AllowedCaller::CodeExecution20250825]);
        let json = serde_json::to_value(&tool).unwrap();

        let callers = json["allowed_callers"].as_array().unwrap();
        assert_eq!(callers.len(), 2);
        assert_eq!(callers[0], "direct");
        assert_eq!(callers[1], "code_execution_20250825");
    }

    #[test]
    fn test_defer_loading() {
        let tool = BetaToolParam::tool_search_bm25().with_defer_loading(true);
        let json = serde_json::to_value(&tool).unwrap();

        assert_eq!(json["defer_loading"], true);
    }
}
