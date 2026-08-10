use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// NOTE: Neither `rmcp` nor `gestalt_mcp` was found in swal-loop's Cargo.toml dependencies.
// We are implementing the fallback client according to instructions.

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
}

#[derive(thiserror::Error, Debug)]
pub enum McpError {
    #[error("Connect error: {0}")]
    Connect(String),
    #[error("JSON-RPC error: {0}")]
    JsonRpc(String),
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
}

fn parse_url(url: &str) -> Result<(String, u16, String), String> {
    let s = if url.starts_with("http://") {
        &url[7..]
    } else if url.starts_with("https://") {
        return Err("https is not supported in this fallback client".to_string());
    } else {
        url
    };

    let (host_port, path) = match s.find('/') {
        Some(idx) => (&s[..idx], &s[idx..]),
        None => (s, "/"),
    };

    let (host, port) = match host_port.find(':') {
        Some(idx) => {
            let host = &host_port[..idx];
            let port_str = &host_port[idx+1..];
            let port = port_str.parse::<u16>().map_err(|e| e.to_string())?;
            (host.to_string(), port)
        }
        None => (host_port.to_string(), 80),
    };

    Ok((host, port, path.to_string()))
}

async fn send_json_rpc_http(url_str: &str, method: &str, params: Value) -> Result<Value, String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let (host, port, path) = parse_url(url_str)?;

    let addr = format!("{}:{}", host, port);

    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;

    let rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });

    let body = serde_json::to_string(&rpc_request).map_err(|e| e.to_string())?;

    let http_request = format!(
        "POST {} HTTP/1.1\r\n\
         Host: {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n\
         {}",
        path, host, body.len(), body
    );

    stream.write_all(http_request.as_bytes()).await.map_err(|e| format!("Write error: {}", e))?;
    stream.flush().await.map_err(|e| format!("Flush error: {}", e))?;

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes).await.map_err(|e| format!("Read error: {}", e))?;

    let response_str = String::from_utf8_lossy(&response_bytes);

    let mut parts = response_str.splitn(2, "\r\n\r\n");
    let headers = parts.next().ok_or_else(|| "No HTTP headers found".to_string())?;
    let body_str = parts.next().ok_or_else(|| "No HTTP body found".to_string())?;

    if !headers.contains("200 OK") {
        return Err(format!("Server returned non-200 status. Headers: {}", headers));
    }

    let rpc_response: Value = serde_json::from_str(body_str)
        .map_err(|e| format!("Failed to parse response JSON: {}. Body: {}", e, body_str))?;

    if let Some(error) = rpc_response.get("error") {
        return Err(format!("JSON-RPC error: {:?}", error));
    }

    let result = rpc_response.get("result").ok_or_else(|| "Missing 'result' field in JSON-RPC response".to_string())?;

    Ok(result.clone())
}

pub struct McpClient {
    url: Option<String>,
    initialized: bool,
    mock_handler: Option<Arc<dyn Fn(&str, Value) -> Result<Value, String> + Send + Sync>>,
}

impl McpClient {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            url: None,
            initialized: false,
            mock_handler: None,
        }
    }

    pub fn with_mock<F>(handler: F) -> Self
    where
        F: Fn(&str, Value) -> Result<Value, String> + Send + Sync + 'static,
    {
        Self {
            url: None,
            initialized: false,
            mock_handler: Some(Arc::new(handler)),
        }
    }

    pub async fn connect(&mut self, url: &str) -> Result<(), McpError> {
        let mut resolved_url = url.to_string();
        if resolved_url.starts_with("http://") {
            let rest = &resolved_url[7..];
            if !rest.contains('/') {
                resolved_url.push_str("/mcp");
            }
        } else if !resolved_url.contains('/') {
            resolved_url.push_str("/mcp");
        }

        // Run initialize RPC call
        let init_params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "swal-agent",
                "version": "0.1.0"
            }
        });

        self.url = Some(resolved_url);

        let _init_res = self.send_rpc("initialize", init_params).await?;

        self.initialized = true;

        // Send notifications/initialized
        let _ = self.send_rpc("notifications/initialized", serde_json::json!({})).await;

        Ok(())
    }

    pub async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        if !self.initialized {
            return Err(McpError::Connect("Not connected/initialized".to_string()));
        }

        let res = self.send_rpc("tools/list", serde_json::json!({})).await?;

        let tools_val = res.get("tools").and_then(|v| v.as_array()).ok_or_else(|| {
            McpError::JsonRpc("Invalid tools/list response structure".to_string())
        })?;

        let mut tools = Vec::new();
        for t in tools_val {
            let name = t.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let description = t.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();
            tools.push(McpTool { name, description });
        }

        Ok(tools)
    }

    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value, McpError> {
        if !self.initialized {
            return Err(McpError::Connect("Not connected/initialized".to_string()));
        }

        // Validate first that the tool is listed
        let tools = self.list_tools().await?;
        if !tools.iter().any(|t| t.name == name) {
            return Err(McpError::ToolNotFound(name.to_string()));
        }

        let params = serde_json::json!({
            "name": name,
            "arguments": args
        });

        self.send_rpc("tools/call", params).await
    }

    async fn send_rpc(&self, method: &str, params: Value) -> Result<Value, McpError> {
        if let Some(ref handler) = self.mock_handler {
            return handler(method, params).map_err(McpError::JsonRpc);
        }

        let url = self.url.as_ref().ok_or_else(|| McpError::Connect("Not connected".to_string()))?;
        send_json_rpc_http(url, method, params)
            .await
            .map_err(McpError::JsonRpc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_mcp_client_success() {
        let client_mock = McpClient::with_mock(|method, params| {
            match method {
                "initialize" => {
                    assert_eq!(params.get("protocolVersion").and_then(|v| v.as_str()), Some("2024-11-05"));
                    Ok(json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "serverInfo": {
                            "name": "mock-mcp-server",
                            "version": "1.0.0"
                        }
                    }))
                }
                "notifications/initialized" => {
                    Ok(json!({}))
                }
                "tools/list" => {
                    Ok(json!({
                        "tools": [
                            {
                                "name": "get_weather",
                                "description": "Get current weather"
                            },
                            {
                                "name": "send_email",
                                "description": "Send an email notification"
                            }
                        ]
                    }))
                }
                "tools/call" => {
                    let name = params.get("name").and_then(|n| n.as_str()).unwrap();
                    let args = params.get("arguments").unwrap();
                    if name == "get_weather" {
                        assert_eq!(args.get("location").and_then(|l| l.as_str()), Some("Seattle"));
                        Ok(json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": "The weather in Seattle is rainy, 52°F"
                                }
                            ],
                            "isError": false
                        }))
                    } else {
                        Err("Unsupported tool in mock call".to_string())
                    }
                }
                _ => Err(format!("Unexpected method: {}", method)),
            }
        });

        let mut client = client_mock;

        // Assert we can't do anything before connect
        assert!(client.list_tools().await.is_err());
        assert!(client.call_tool("get_weather", json!({"location": "Seattle"})).await.is_err());

        // Connect
        client.connect("http://localhost:8080").await.expect("Connect failed");

        // List tools
        let tools = client.list_tools().await.expect("List tools failed");
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "get_weather");
        assert_eq!(tools[0].description, "Get current weather");
        assert_eq!(tools[1].name, "send_email");
        assert_eq!(tools[1].description, "Send an email notification");

        // Call tool successfully
        let result = client.call_tool("get_weather", json!({"location": "Seattle"})).await.expect("Call tool failed");
        let content_text = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.get(0)).and_then(|item| item.get("text")).and_then(|t| t.as_str()).unwrap();
        assert_eq!(content_text, "The weather in Seattle is rainy, 52°F");

        // Call non-existent tool -> Expect ToolNotFound
        let err = client.call_tool("calculate_sum", json!({})).await.unwrap_err();
        assert!(matches!(err, McpError::ToolNotFound(name) if name == "calculate_sum"));
    }
}
