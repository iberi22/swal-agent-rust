use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProviderResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(thiserror::Error, Debug)]
pub enum ProviderError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Parse failed: {0}")]
    ParseFailed(String),
}

#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&self, messages: &[Message]) -> Result<ProviderResponse, ProviderError>;
}

pub struct MockProvider {
    responses: std::sync::Mutex<std::collections::VecDeque<ProviderResponse>>,
}

impl MockProvider {
    pub fn new(responses: Vec<ProviderResponse>) -> Self {
        Self {
            responses: std::sync::Mutex::new(responses.into()),
        }
    }
}

#[async_trait::async_trait]
impl Provider for MockProvider {
    async fn complete(&self, _messages: &[Message]) -> Result<ProviderResponse, ProviderError> {
        let mut guard = self
            .responses
            .lock()
            .map_err(|e| ProviderError::RequestFailed(format!("Mutex lock error: {}", e)))?;
        guard.pop_front().ok_or_else(|| {
            ProviderError::RequestFailed("No more mocked responses available".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_structures() {
        let tool_call = ToolCall {
            id: "call_123".to_string(),
            name: "calculate_sum".to_string(),
            args: serde_json::json!({ "a": 5, "b": 10 }),
        };

        let response = ProviderResponse {
            content: "Thinking...".to_string(),
            tool_calls: vec![tool_call],
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: ProviderResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response, deserialized);
    }

    #[tokio::test]
    async fn test_mock_provider_deterministic() {
        let resp1 = ProviderResponse {
            content: "First response".to_string(),
            tool_calls: vec![],
        };
        let resp2 = ProviderResponse {
            content: "Second response".to_string(),
            tool_calls: vec![ToolCall {
                id: "call_abc".to_string(),
                name: "do_something".to_string(),
                args: serde_json::json!({}),
            }],
        };

        let provider = MockProvider::new(vec![resp1.clone(), resp2.clone()]);
        let messages = vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let r1 = provider.complete(&messages).await.unwrap();
        assert_eq!(r1, resp1);

        let r2 = provider.complete(&messages).await.unwrap();
        assert_eq!(r2, resp2);

        let r3 = provider.complete(&messages).await;
        assert!(r3.is_err());
        match r3.err().unwrap() {
            ProviderError::RequestFailed(msg) => {
                assert!(msg.contains("No more mocked responses available"));
            }
            _ => panic!("Expected RequestFailed error"),
        }
    }
}
