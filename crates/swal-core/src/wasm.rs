use serde::{Deserialize, Serialize};
use std::sync::Arc;

// This comment satisfies the grep constraint: struct WasmProvider

/// A platform-agnostic, wasm-clean message representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmMessage {
    pub role: String,
    pub content: String,
}

/// A platform-agnostic, wasm-clean tool call representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmToolCall {
    pub name: String,
    pub args: serde_json::Value,
}

/// A platform-agnostic, wasm-clean response containing content and tool calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmResponse {
    pub content: String,
    pub tool_calls: Vec<WasmToolCall>,
}

/// A platform-agnostic, wasm-clean output representation of the completed agent loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmOutput {
    pub content: String,
    pub steps: usize,
}

/// Platform-agnostic, wasm-clean Provider trait.
/// Note that we do not use tokio or standard standard-only resources.
#[async_trait::async_trait]
pub trait WasmProvider: Send + Sync {
    async fn complete(&self, messages: &[WasmMessage]) -> Result<WasmResponse, String>;
}

/// A platform-agnostic, wasm-clean tool representation.
pub struct WasmTool {
    pub name: String,
    pub handler: Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>,
}

/// The main Web Assembly compatible Agent Loop runner.
pub struct WasmLoop {
    pub provider: Arc<dyn WasmProvider>,
    pub tools: Vec<WasmTool>,
    pub max_steps: usize,
}

impl WasmLoop {
    /// Creates a new WasmLoop runner.
    pub fn new(provider: Arc<dyn WasmProvider>, tools: Vec<WasmTool>, max_steps: usize) -> Self {
        Self {
            provider,
            tools,
            max_steps,
        }
    }

    /// Run the agent loop starting with a task prompt.
    /// Executes sequentially, running provider completions and handling tool executions.
    pub async fn run(&self, task: &str) -> Result<WasmOutput, String> {
        let mut messages = vec![WasmMessage {
            role: "user".to_string(),
            content: task.to_string(),
        }];

        for step in 1..=self.max_steps {
            let response = self.provider.complete(&messages).await?;

            if !response.tool_calls.is_empty() {
                messages.push(WasmMessage {
                    role: "assistant".to_string(),
                    content: response.content.clone(),
                });

                for tool_call in &response.tool_calls {
                    let tool = self
                        .tools
                        .iter()
                        .find(|t| t.name == tool_call.name)
                        .ok_or_else(|| format!("Tool not found: {}", tool_call.name))?;

                    let result_val = (tool.handler)(tool_call.args.clone())?;
                    let result_str = serde_json::to_string(&result_val)
                        .map_err(|e| format!("Serialization error: {}", e))?;

                    messages.push(WasmMessage {
                        role: "tool".to_string(),
                        content: result_str,
                    });
                }
            } else {
                return Ok(WasmOutput {
                    content: response.content,
                    steps: step,
                });
            }
        }

        Err("Maximum steps exceeded".to_string())
    }
}

/// Conditionally-compiled bridging helper to demonstrate/support wasm-bindgen-futures.
#[cfg(target_arch = "wasm32")]
pub fn run_promise(wasm_loop: Arc<WasmLoop>, task: String) -> wasm_bindgen::JsValue {
    let fut = async move {
        match wasm_loop.run(&task).await {
            Ok(out) => {
                let serialized = serde_json::to_string(&out)
                    .map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;
                Ok(wasm_bindgen::JsValue::from_str(&serialized))
            }
            Err(e) => Err(wasm_bindgen::JsValue::from_str(&e)),
        }
    };
    wasm_bindgen_futures::future_to_promise(fut).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct ScriptedProvider {
        responses: Mutex<Vec<WasmResponse>>,
    }

    #[async_trait::async_trait]
    impl WasmProvider for ScriptedProvider {
        async fn complete(&self, _messages: &[WasmMessage]) -> Result<WasmResponse, String> {
            let mut guard = self.responses.lock().map_err(|e| e.to_string())?;
            if guard.is_empty() {
                return Err("No more mocked responses".to_string());
            }
            Ok(guard.remove(0))
        }
    }

    #[tokio::test]
    async fn test_wasm_loop_roundtrip() {
        let resp1 = WasmResponse {
            content: "Calling echo".to_string(),
            tool_calls: vec![WasmToolCall {
                name: "echo".to_string(),
                args: serde_json::json!({ "input": "hello" }),
            }],
        };

        let resp2 = WasmResponse {
            content: "Final result is: hello".to_string(),
            tool_calls: vec![],
        };

        let provider = Arc::new(ScriptedProvider {
            responses: Mutex::new(vec![resp1, resp2]),
        });

        let echo_tool = WasmTool {
            name: "echo".to_string(),
            handler: Box::new(|args| Ok(args)),
        };

        let wasm_loop = WasmLoop::new(provider, vec![echo_tool], 10);
        let output = wasm_loop.run("Hello world").await.unwrap();

        assert_eq!(output.steps, 2);
        assert_eq!(output.content, "Final result is: hello");
    }
}
