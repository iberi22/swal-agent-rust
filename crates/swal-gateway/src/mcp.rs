use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    routing::post,
    Router,
};

/// Backward compatibility placeholder function for mcp_server.
#[allow(dead_code)]
pub fn mcp_server() -> String {
    "MCP Server Fallback HTTP endpoints are registered via routes()".to_string()
}

/// Platform-agnostic trait representing an agent's main loop control and tool execution capabilities.
/// This enables the HTTP/WS gateway to delegate user tasks and MCP tool requests.
///
/// Returns boxed Send futures to ensure dynamic compatibility (dyn AgentHandle) without requiring external macros.
pub trait AgentHandle: Send + Sync {
    /// Executes a given task on the agent and returns a JSON response.
    fn run_task(&self, task: &str) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send + '_>>;

    /// Executes a tool registered on the agent.
    fn execute_tool(&self, name: &str, args: Value) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send + '_>>;

    /// Lists the tool information for all tools configured on the agent.
    fn list_tools(&self) -> Vec<McpToolInfo>;
}

/// Standardized Model Context Protocol (MCP) Tool information representation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Response payload for the list tools endpoint.
#[derive(Serialize, Clone, Debug)]
pub struct McpToolsResponse {
    pub tools: Vec<McpToolInfo>,
}

/// Request payload for executing an MCP tool.
#[derive(Deserialize, Clone, Debug)]
pub struct McpCallRequest {
    pub name: String,
    pub arguments: Value,
}

/// Standardized text content segment in an MCP response.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct McpCallTextContent {
    pub r#type: String, // typically "text"
    pub text: String,
}

/// Response payload for executing an MCP tool.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct McpCallResponse {
    pub content: Vec<McpCallTextContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Exposes the fallback JSON-RPC-like HTTP endpoints for the MCP Server.
///
/// # Integration Point
/// Since no external MCP crate is added to dependencies, the fallback HTTP paths can be merged
/// into the main Axum Router in `http.rs`:
///
/// ```ignore
/// use std::sync::Arc;
/// use axum::Router;
/// use swal_gateway::mcp;
///
/// let agent: Arc<dyn mcp::AgentHandle> = todo!();
/// let app = Router::new()
///     .merge(mcp::routes().with_state(agent));
/// ```
pub fn routes() -> Router<Arc<dyn AgentHandle>> {
    Router::new()
        .route("/mcp/tools", post(mcp_tools_handler))
        .route("/mcp/call", post(mcp_call_handler))
}

/// Handler for retrieving the list of available tools.
async fn mcp_tools_handler(
    State(agent): State<Arc<dyn AgentHandle>>,
) -> impl IntoResponse {
    let tools = agent.list_tools();
    Json(McpToolsResponse { tools })
}

/// Handler for invoking a registered tool.
async fn mcp_call_handler(
    State(agent): State<Arc<dyn AgentHandle>>,
    Json(req): Json<McpCallRequest>,
) -> impl IntoResponse {
    match mcp_call(agent, &req.name, req.arguments).await {
        Ok(result_val) => {
            let text_val = if result_val.is_string() {
                result_val.as_str().unwrap().to_string()
            } else {
                result_val.to_string()
            };
            Json(McpCallResponse {
                content: vec![McpCallTextContent {
                    r#type: "text".to_string(),
                    text: text_val,
                }],
                is_error: None,
            })
        }
        Err(err) => {
            Json(McpCallResponse {
                content: vec![McpCallTextContent {
                    r#type: "text".to_string(),
                    text: format!("Error: {}", err),
                }],
                is_error: Some(true),
            })
        }
    }
}

/// Helper function to execute a tool via AgentHandle.
pub async fn mcp_call(
    agent: Arc<dyn AgentHandle>,
    name: &str,
    arguments: Value,
) -> Result<Value, String> {
    agent.execute_tool(name, arguments).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt; // for oneshot

    struct MockAgent {
        tools: Vec<McpToolInfo>,
    }

    impl AgentHandle for MockAgent {
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
    async fn test_mcp_call_helper() {
        let mock = Arc::new(MockAgent {
            tools: vec![McpToolInfo {
                name: "echo".to_string(),
                description: "echo desc".to_string(),
                input_schema: json!({}),
            }],
        });

        let res = mcp_call(mock.clone(), "echo", json!({ "hello": "world" })).await;
        assert_eq!(res, Ok(json!({ "hello": "world" })));

        let res_fail = mcp_call(mock, "unknown", json!({})).await;
        assert!(res_fail.is_err());
    }

    #[tokio::test]
    async fn test_mcp_routes() {
        let mock = Arc::new(MockAgent {
            tools: vec![McpToolInfo {
                name: "echo".to_string(),
                description: "echo desc".to_string(),
                input_schema: json!({}),
            }],
        });

        let app = routes().with_state(mock);

        // Test POST /mcp/tools
        let response = app
            .clone()
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

        // Test POST /mcp/call with valid tool
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/call")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&json!({
                        "name": "echo",
                        "arguments": { "test": 123 }
                    })).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json_body["content"][0]["type"], "text");
        assert_eq!(json_body["content"][0]["text"], "{\"test\":123}");

        // Test POST /mcp/call with failing/unknown tool
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/call")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&json!({
                        "name": "unknown",
                        "arguments": {}
                    })).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json_body["isError"], true);
    }
}
