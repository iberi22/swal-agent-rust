use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use serde_json::Value;
use axum::extract::ws::{WebSocket, Message};

// Comment on issue 02: http.rs does not currently define or export a public AgentHandle trait,
// so we define a local AgentHandle trait in ws.rs instead to avoid editing http.rs.
// We use Pin<Box<dyn Future...>> to ensure the trait is dyn-compatible.
pub trait AgentHandle: Send + Sync {
    fn run_task(&self, task: &str) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send + '_>>;
}

pub type WebSocketStream = WebSocket;

/// Helper function to process individual WebSocket text messages.
/// Extracted to allow comprehensive unit testing without needing a live network connection.
pub async fn process_message(text: &str, agent: &dyn AgentHandle) -> Option<Message> {
    let trimmed = text.trim();
    if trimmed.eq_ignore_ascii_case("ping") {
        return Some(Message::Text("pong".to_string()));
    }
    if trimmed.eq_ignore_ascii_case("health") {
        return Some(Message::Text("ok".to_string()));
    }

    if let Ok(json) = serde_json::from_str::<Value>(trimmed) {
        if let Some(task_val) = json.get("task").and_then(|v: &Value| v.as_str()) {
            match agent.run_task(task_val).await {
                Ok(val) => {
                    let content = val.get("content")
                        .and_then(|v: &Value| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let steps = val.get("steps")
                        .and_then(|v: &Value| v.as_u64())
                        .unwrap_or(0) as usize;
                    let resp = serde_json::json!({
                        "content": content,
                        "steps": steps
                    });
                    return Some(Message::Text(resp.to_string()));
                }
                Err(err) => {
                    let resp = serde_json::json!({
                        "error": err
                    });
                    return Some(Message::Text(resp.to_string()));
                }
            }
        } else if let Some(echo_val) = json.get("echo") {
            let resp = serde_json::json!({
                "echo": echo_val
            });
            return Some(Message::Text(resp.to_string()));
        } else if let Some(ping_val) = json.get("ping") {
            let resp = serde_json::json!({
                "pong": ping_val
            });
            return Some(Message::Text(resp.to_string()));
        } else if json.get("health").is_some() {
            let resp = serde_json::json!({
                "status": "ok"
            });
            return Some(Message::Text(resp.to_string()));
        }
    }
    None
}

/// Real WebSocket server handler.
/// Receives client messages, runs tasks via AgentHandle, and sends back results.
pub async fn handle_ws(mut socket: WebSocketStream, agent: Arc<dyn AgentHandle>) {
    while let Some(msg_result) = socket.recv().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                if let Some(resp) = process_message(&text, agent.as_ref()).await {
                    if let Err(e) = socket.send(resp).await {
                        tracing::error!("Failed to send message: {:?}", e);
                        break;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket client closed connection");
                break;
            }
            Ok(_) => {
                // Ignore binary, ping, pong, etc.
            }
            Err(e) => {
                tracing::error!("WebSocket error: {:?}", e);
                break;
            }
        }
    }
}

// Old Ola 2.01 stub compatibility (if needed by other modules)
#[allow(dead_code)]
pub async fn accept(_conn: &str) {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct MockAgent {
        should_fail: bool,
    }

    impl AgentHandle for MockAgent {
        fn run_task(&self, task: &str) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send + '_>> {
            let should_fail = self.should_fail;
            let task = task.to_string();
            Box::pin(async move {
                if should_fail {
                    Err(format!("Failed to run task: {}", task))
                } else {
                    Ok(json!({
                        "content": format!("Echo: {}", task),
                        "steps": 3
                    }))
                }
            })
        }
    }

    #[tokio::test]
    async fn test_process_message_plain_ping() {
        let agent = Arc::new(MockAgent { should_fail: false });
        let resp = process_message("ping", agent.as_ref()).await.unwrap();
        if let Message::Text(text) = resp {
            assert_eq!(text, "pong");
        } else {
            panic!("Expected text message");
        }
    }

    #[tokio::test]
    async fn test_process_message_plain_health() {
        let agent = Arc::new(MockAgent { should_fail: false });
        let resp = process_message("health", agent.as_ref()).await.unwrap();
        if let Message::Text(text) = resp {
            assert_eq!(text, "ok");
        } else {
            panic!("Expected text message");
        }
    }

    #[tokio::test]
    async fn test_process_message_json_echo() {
        let agent = Arc::new(MockAgent { should_fail: false });
        let resp = process_message(r#"{"echo": "hello"}"#, agent.as_ref()).await.unwrap();
        if let Message::Text(text) = resp {
            let json: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["echo"], "hello");
        } else {
            panic!("Expected text message");
        }
    }

    #[tokio::test]
    async fn test_process_message_json_ping() {
        let agent = Arc::new(MockAgent { should_fail: false });
        let resp = process_message(r#"{"ping": "hello"}"#, agent.as_ref()).await.unwrap();
        if let Message::Text(text) = resp {
            let json: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["pong"], "hello");
        } else {
            panic!("Expected text message");
        }
    }

    #[tokio::test]
    async fn test_process_message_json_health() {
        let agent = Arc::new(MockAgent { should_fail: false });
        let resp = process_message(r#"{"health": true}"#, agent.as_ref()).await.unwrap();
        if let Message::Text(text) = resp {
            let json: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["status"], "ok");
        } else {
            panic!("Expected text message");
        }
    }

    #[tokio::test]
    async fn test_process_message_json_task_success() {
        let agent = Arc::new(MockAgent { should_fail: false });
        let resp = process_message(r#"{"task": "do work"}"#, agent.as_ref()).await.unwrap();
        if let Message::Text(text) = resp {
            let json: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["content"], "Echo: do work");
            assert_eq!(json["steps"], 3);
        } else {
            panic!("Expected text message");
        }
    }

    #[tokio::test]
    async fn test_process_message_json_task_error() {
        let agent = Arc::new(MockAgent { should_fail: true });
        let resp = process_message(r#"{"task": "do work"}"#, agent.as_ref()).await.unwrap();
        if let Message::Text(text) = resp {
            let json: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["error"], "Failed to run task: do work");
        } else {
            panic!("Expected text message");
        }
    }
}
