use serde_json::{json, Value};
use std::sync::Arc;
use swal_core::tool::{Tool, ToolError, ToolRegistry};

/// Inline implementation of the fallback `echo` tool.
pub struct EchoTool;

#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
struct EchoArgs {
    /// The text to echo back.
    pub text: String,
}

#[async_trait::async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes back input text"
    }

    fn input_schema(&self) -> schemars::schema::RootSchema {
        schemars::schema_for!(EchoArgs)
    }

    async fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let text = args.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        Ok(json!({ "text": text }))
    }
}

/// Inline implementation of the fallback `read_file` tool.
pub struct ReadFileTool;

#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
struct ReadFileArgs {
    /// The path of the file to read.
    pub path: String,
}

#[async_trait::async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file from the host filesystem"
    }

    fn input_schema(&self) -> schemars::schema::RootSchema {
        schemars::schema_for!(ReadFileArgs)
    }

    async fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::Execution("Missing 'path' parameter".to_string()))?;

        let content = std::fs::read_to_string(path)
            .map_err(|e| ToolError::Execution(format!("Failed to read file '{}': {}", path, e)))?;

        Ok(json!({ "content": content }))
    }
}

/// Registers the default native tools.
/// This implementation takes the inline fallback path since gestalt's tools API is
/// not directly reusable without complex external dependency/trait mappings.
pub fn register_defaults(reg: &ToolRegistry) {
    reg.register(Arc::new(EchoTool));
    reg.register(Arc::new(ReadFileTool));
}

/// Builds and returns a fully populated ToolRegistry containing the default tools.
pub fn make_registry() -> ToolRegistry {
    let reg = ToolRegistry::new();
    register_defaults(&reg);
    reg
}
