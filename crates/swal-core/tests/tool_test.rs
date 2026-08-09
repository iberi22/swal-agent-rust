use std::sync::Arc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};
use swal_core::tool::{Tool, ToolError, ToolRegistry};

// Define an argument struct for the Adder tool.
#[derive(JsonSchema, Deserialize)]
struct AdderArgs {
    a: i32,
    b: i32,
}

struct AdderTool;

#[async_trait::async_trait]
impl Tool for AdderTool {
    fn name(&self) -> &str {
        "adder"
    }

    fn description(&self) -> &str {
        "Adds two integers together."
    }

    fn input_schema(&self) -> schemars::schema::RootSchema {
        schemars::schema_for!(AdderArgs)
    }

    async fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let parsed: AdderArgs = serde_json::from_value(args)
            .map_err(|e| ToolError::Serialization(e.to_string()))?;
        let sum = parsed.a + parsed.b;
        Ok(json!({ "sum": sum }))
    }
}

// Define an argument struct for the Echo tool.
#[derive(JsonSchema, Deserialize)]
struct EchoArgs {
    message: String,
}

struct EchoTool;

#[async_trait::async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes back the message provided."
    }

    fn input_schema(&self) -> schemars::schema::RootSchema {
        schemars::schema_for!(EchoArgs)
    }

    async fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let parsed: EchoArgs = serde_json::from_value(args)
            .map_err(|e| ToolError::Serialization(e.to_string()))?;
        Ok(json!({ "echoed": parsed.message }))
    }
}

#[tokio::test]
async fn test_tool_registry_workflow() {
    let registry = ToolRegistry::new();

    // Register tools
    registry.register(Arc::new(AdderTool));
    registry.register(Arc::new(EchoTool));

    // Verify list (names should be sorted alphabetically: "adder", "echo")
    let tool_list = registry.list();
    assert_eq!(tool_list, vec!["adder".to_string(), "echo".to_string()]);

    // Verify Adder tool execution
    let adder_args = json!({ "a": 12, "b": 30 });
    let adder_res = registry.execute("adder", adder_args).await.unwrap();
    assert_eq!(adder_res, json!({ "sum": 42 }));

    // Verify Echo tool execution
    let echo_args = json!({ "message": "Hello Swal Core!" });
    let echo_res = registry.execute("echo", echo_args).await.unwrap();
    assert_eq!(echo_res, json!({ "echoed": "Hello Swal Core!" }));

    // Verify Serialization error (missing parameter)
    let bad_adder_args = json!({ "a": 12 });
    let err_res = registry.execute("adder", bad_adder_args).await;
    assert!(matches!(err_res, Err(ToolError::Serialization(_))));

    // Verify NotFound error
    let missing_res = registry.execute("subtractor", json!({})).await;
    assert!(matches!(missing_res, Err(ToolError::NotFound(_))));
}
