use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use serde_json::{json, Value};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use swal_loop::skills::SkillLoader;
use swal_core::tool::{Tool, ToolError, ToolRegistry};
use swal_core::{async_trait, schemars};
use swal_loop::provider::{MockProvider, ProviderResponse};
use swal_loop::r#loop::AgentLoop;

use swal_gateway::mcp::McpToolInfo;

// Implementation of EchoTool for http_gateway test
pub struct EchoTool;

#[async_trait::async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes back input text"
    }

    fn input_schema(&self) -> schemars::schema::RootSchema {
        schemars::schema::RootSchema::default()
    }

    async fn execute(&self, args: Value) -> Result<Value, ToolError> {
        Ok(args)
    }
}

// Helper to send TCP raw HTTP request
async fn send_raw_http(addr: std::net::SocketAddr, method: &str, path: &str, body: Option<&str>) -> (u16, String) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let mut stream = None;
    for _ in 0..15 {
        if let Ok(s) = TcpStream::connect(addr).await {
            stream = Some(s);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let mut stream = stream.expect("Failed to connect to gateway server after retries");

    let req_body = body.unwrap_or("");
    let mut req = format!(
        "{} {} HTTP/1.1\r\n\
         Host: {}\r\n\
         Connection: close\r\n",
        method, path, addr
    );
    if !req_body.is_empty() {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", req_body.len()));
    }
    req.push_str("\r\n");
    req.push_str(req_body);

    stream.write_all(req.as_bytes()).await.expect("Write failed");
    stream.flush().await.expect("Flush failed");

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes).await.expect("Read failed");

    let response_str = String::from_utf8_lossy(&response_bytes).into_owned();
    let mut parts = response_str.splitn(2, "\r\n\r\n");
    let headers = parts.next().expect("No headers found");
    let body_out = parts.next().unwrap_or("").to_string();

    let status_line = headers.lines().next().expect("No status line");
    let status_code = status_line.split_whitespace().nth(1).expect("No status code").parse::<u16>().expect("Invalid status code");

    (status_code, body_out)
}

#[tokio::test]
async fn test_http_gateway() {
    let mut dir = std::env::temp_dir();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    dir.push(format!("swal_gateway_e2e_{}", now));
    std::fs::create_dir_all(&dir).unwrap();

    let skills = SkillLoader::new(dir.to_str().unwrap()).unwrap();
    let registry = ToolRegistry::new();
    registry.register(Arc::new(EchoTool));

    let mock_resp = ProviderResponse {
        content: "Mock loop response content".to_string(),
        tool_calls: vec![],
    };
    let provider = Arc::new(MockProvider::new(vec![mock_resp]));
    let agent_loop = Arc::new(AgentLoop::new(provider, registry, skills));

    // Find an ephemeral port by binding to 127.0.0.1:0
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind ephemeral listener");
    let addr = listener.local_addr().expect("Failed to get local address");
    drop(listener);

    let agent_loop_clone = agent_loop.clone();
    tokio::spawn(async move {
        let _ = swal_gateway::http::serve(agent_loop_clone, addr).await;
    });

    // 1. Verify health check
    let (status, body) = send_raw_http(addr, "GET", "/health", None).await;
    assert_eq!(status, 200);
    let val: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(val, json!({ "status": "ok" }));

    // 2. Verify run task
    let (status, body) = send_raw_http(
        addr,
        "POST",
        "/run",
        Some(&json!({ "task": "say hi" }).to_string()),
    )
    .await;
    assert_eq!(status, 200);
    let val: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(val["content"], "Mock loop response content");

    let _ = std::fs::remove_dir_all(&dir);
}

// Mock agent for test_mcp_gateway
struct MockMcpAgent {
    tools: Vec<McpToolInfo>,
}

impl swal_gateway::mcp::AgentHandle for MockMcpAgent {
    fn run_task(&self, _task: &str) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send + '_>> {
        Box::pin(async move {
            Ok(json!({ "content": "mock run_task response" }))
        })
    }

    fn execute_tool(&self, name: &str, args: Value) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send + '_>> {
        let name = name.to_string();
        Box::pin(async move {
            if name == "echo" {
                Ok(args)
            } else {
                Err(format!("Unknown tool: {}", name))
            }
        })
    }

    fn list_tools(&self) -> Vec<McpToolInfo> {
        self.tools.clone()
    }
}

#[tokio::test]
async fn test_mcp_gateway() {
    let mock = Arc::new(MockMcpAgent {
        tools: vec![McpToolInfo {
            name: "echo".to_string(),
            description: "echo desc".to_string(),
            input_schema: json!({}),
        }],
    });

    let app = swal_gateway::mcp::routes().with_state(mock);

    // Test POST /mcp/tools
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/tools")
                .header("content-type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_body["tools"][0]["name"], "echo");
}