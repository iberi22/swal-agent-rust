use crate::provider::{Message, Provider};
use crate::skills::SkillLoader;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use swal_core::tool::ToolRegistry;

#[derive(thiserror::Error, Debug)]
pub enum LoopError {
    #[error("Maximum steps exceeded")]
    MaxSteps,
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Tool execution error: {0}")]
    Tool(String),
    #[error("No final response received")]
    NoFinalResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AgentOutput {
    pub content: String,
    pub steps: usize,
    pub tool_calls_executed: usize,
}

pub struct AgentLoop {
    provider: Arc<dyn Provider>,
    tools: ToolRegistry,
    skills: SkillLoader,
    max_steps: usize,
    session_id: Option<String>,
}

impl AgentLoop {
    pub fn new(provider: Arc<dyn Provider>, tools: ToolRegistry, skills: SkillLoader) -> Self {
        Self {
            provider,
            tools,
            skills,
            max_steps: 10,
            session_id: None,
        }
    }

    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub async fn run(&self, task: &str) -> Result<AgentOutput, LoopError> {
        let skills_list = {
            let skills_guard = self.skills.snapshot.read().map_err(|e| {
                LoopError::Provider(format!("Failed to read skills snapshot: {}", e))
            })?;

            let mut list = Vec::new();
            for skill in skills_guard.values() {
                list.push(format!("- {}: {}", skill.name, skill.description));
            }
            list
        };

        let system_prompt = format!(
            "You are a helpful assistant. You have access to the following skills:\n{}\n",
            skills_list.join("\n")
        );

        let mut messages = vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: task.to_string(),
            },
        ];

        let mut tool_calls_executed = 0;

        for step in 1..=self.max_steps {
            let response = self
                .provider
                .complete(&messages)
                .await
                .map_err(|e| LoopError::Provider(e.to_string()))?;

            if !response.tool_calls.is_empty() {
                messages.push(Message {
                    role: "assistant".to_string(),
                    content: response.content.clone(),
                });

                for tool_call in &response.tool_calls {
                    let result_val = self
                        .tools
                        .execute(&tool_call.name, tool_call.args.clone())
                        .await
                        .map_err(|e| LoopError::Tool(e.to_string()))?;

                    let result_str = serde_json::to_string(&result_val)
                        .unwrap_or_else(|_| result_val.to_string());

                    messages.push(Message {
                        role: "tool".to_string(),
                        content: result_str,
                    });
                    tool_calls_executed += 1;
                }
            } else {
                return Ok(AgentOutput {
                    content: response.content,
                    steps: step,
                    tool_calls_executed,
                });
            }
        }

        Err(LoopError::MaxSteps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{MockProvider, ProviderResponse, ToolCall};
    use serde_json::Value;
    use swal_core::tool::{Tool, ToolError};

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
    async fn test_agent_loop_roundtrip() {
        let mut dir = std::env::temp_dir();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("swal_loop_test_{}", now));
        std::fs::create_dir_all(&dir).unwrap();

        let skills = SkillLoader::new(dir.to_str().unwrap()).unwrap();

        let registry = ToolRegistry::new();
        registry.register(Arc::new(EchoTool));

        let resp1 = ProviderResponse {
            content: "Calling echo".to_string(),
            tool_calls: vec![ToolCall {
                id: "call_123".to_string(),
                name: "echo".to_string(),
                args: serde_json::json!({ "input": "hello" }),
            }],
        };

        let resp2 = ProviderResponse {
            content: "Echo returned: hello".to_string(),
            tool_calls: vec![],
        };

        let provider = Arc::new(MockProvider::new(vec![resp1, resp2]));

        let agent_loop = AgentLoop::new(provider, registry, skills);
        let output = agent_loop.run("Hello world").await.unwrap();

        assert_eq!(output.tool_calls_executed, 1);
        assert_eq!(output.content, "Echo returned: hello");
        assert_eq!(output.steps, 2);

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
