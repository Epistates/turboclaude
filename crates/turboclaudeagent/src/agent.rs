//! Agent definitions

/// Agent definition for specialized agent personas
pub struct AgentDefinition {
    /// Agent name
    pub name: String,
}

impl AgentDefinition {
    /// Create a new agent
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
