use async_trait::async_trait;
use dashmap::DashMap;
use serde_json::Value;
use std::fmt;
use std::sync::Arc;

/// Error enum for Tool operations
#[derive(Debug, Clone)]
pub enum ToolError {
    Serialization(String),
    Execution(String),
    NotFound(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Self::Execution(msg) => write!(f, "Execution error: {}", msg),
            Self::NotFound(msg) => write!(f, "Tool not found: {}", msg),
        }
    }
}

impl std::error::Error for ToolError {}

/// Platform-agnostic trait representing an executable tool.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name of the tool.
    fn name(&self) -> &str;

    /// Description of what the tool does.
    fn description(&self) -> &str;

    /// Schema defining the inputs the tool accepts.
    fn input_schema(&self) -> schemars::schema::RootSchema;

    /// Executes the tool with the given JSON arguments.
    async fn execute(&self, args: Value) -> Result<Value, ToolError>;
}

/// Registry for managing and executing dynamic tools.
pub struct ToolRegistry {
    tools: DashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Creates a new, empty ToolRegistry.
    pub fn new() -> Self {
        Self {
            tools: DashMap::new(),
        }
    }

    /// Registers a new tool into the registry.
    pub fn register(&self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Lists the names of all registered tools, sorted alphabetically.
    pub fn list(&self) -> Vec<String> {
        let mut names: Vec<String> = self.tools.iter().map(|entry| entry.key().clone()).collect();
        names.sort();
        names
    }

    /// Executes a registered tool by name with the given arguments.
    pub async fn execute(&self, name: &str, args: Value) -> Result<Value, ToolError> {
        let tool = {
            let entry = self
                .tools
                .get(name)
                .ok_or_else(|| ToolError::NotFound(name.to_string()))?;
            entry.value().clone()
        };
        tool.execute(args).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
