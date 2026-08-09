use std::sync::Arc;
use serde_json::Value;
use swal_loop::provider::{MockProvider, ProviderResponse, ToolCall};
use swal_loop::r#loop::{AgentLoop, LoopError};
use swal_loop::skills::SkillLoader;
use swal_core::tool::{Tool, ToolRegistry, ToolError};

struct EchoTool;

#[async_trait::async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes back input"
    }

    fn input_schema(&self) -> schemars::schema::RootSchema {
        Default::default()
    }

    async fn execute(&self, args: Value) -> Result<Value, ToolError> {
        Ok(args)
    }
}

#[tokio::test]
async fn test_agent_loop_e2e_roundtrip() {
    let mut dir = std::env::temp_dir();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    dir.push(format!("swal_loop_e2e_test_{}", now));
    std::fs::create_dir_all(&dir).unwrap();

    let skills = SkillLoader::new(dir.to_str().unwrap()).unwrap();

    let registry = ToolRegistry::new();
    registry.register(Arc::new(EchoTool));

    let resp1 = ProviderResponse {
        content: "Calling echo".to_string(),
        tool_calls: vec![ToolCall {
            id: "call_123".to_string(),
            name: "echo".to_string(),
            args: serde_json::json!({ "text": "hi" }),
        }],
    };

    let resp2 = ProviderResponse {
        content: "done".to_string(),
        tool_calls: vec![],
    };

    let provider = Arc::new(MockProvider::new(vec![resp1, resp2]));

    let agent_loop = AgentLoop::new(provider, registry, skills);
    let output = agent_loop.run("say hi").await.unwrap();

    assert_eq!(output.content, "done", "Content should be 'done'");
    assert_eq!(output.tool_calls_executed, 1, "Should execute exactly 1 tool call");
    assert_eq!(output.steps, 2, "Should take exactly 2 steps");

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_agent_loop_max_steps() {
    let mut dir = std::env::temp_dir();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    dir.push(format!("swal_loop_e2e_test_max_steps_{}", now));
    std::fs::create_dir_all(&dir).unwrap();

    let skills = SkillLoader::new(dir.to_str().unwrap()).unwrap();
    let registry = ToolRegistry::new();
    registry.register(Arc::new(EchoTool));

    // A provider that returns a tool call, prompting the loop to continue
    let resp1 = ProviderResponse {
        content: "Calling echo again".to_string(),
        tool_calls: vec![ToolCall {
            id: "call_456".to_string(),
            name: "echo".to_string(),
            args: serde_json::json!({ "text": "looping" }),
        }],
    };

    let provider = Arc::new(MockProvider::new(vec![resp1]));

    // Construct the AgentLoop with max_steps = 1
    let agent_loop = AgentLoop::new(provider, registry, skills).with_max_steps(1);
    let result = agent_loop.run("trigger max steps").await;

    assert!(result.is_err(), "Expected MaxSteps error, but got Ok");
    match result.err().unwrap() {
        LoopError::MaxSteps => {}
        err => panic!("Expected LoopError::MaxSteps, but got {:?}", err),
    }

    let _ = std::fs::remove_dir_all(&dir);
}
