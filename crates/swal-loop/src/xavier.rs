use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct XavierHit {
    pub path: String,
    pub content: String,
    pub score: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum XavierError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Parse error: {0}")]
    Parse(String),
}

#[async_trait::async_trait]
pub trait XavierTransport: Send + Sync {
    async fn store(&self, path: &str, content: &str) -> Result<(), XavierError>;
    async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<XavierHit>, XavierError>;
}

pub struct HttpTransport {
    pub base_url: String,
    pub token: Option<String>,
}

impl HttpTransport {
    pub fn new(base_url: String, token: Option<String>) -> Self {
        Self { base_url, token }
    }

    async fn send_post(&self, path: &str, body: &str) -> Result<String, XavierError> {
        let url = self.base_url.trim_end_matches('/');
        let (host, port, path_prefix) = if url.starts_with("http://") {
            let rest = &url[7..];
            let slash_idx = rest.find('/');
            let (host_port, path_prefix) = if let Some(idx) = slash_idx {
                (&rest[..idx], &rest[idx..])
            } else {
                (rest, "")
            };
            let colon_idx = host_port.find(':');
            let (host, port) = if let Some(idx) = colon_idx {
                (&host_port[..idx], host_port[idx+1..].parse::<u16>().map_err(|e| XavierError::Http(format!("Invalid port: {}", e)))?)
            } else {
                (host_port, 80)
            };
            (host.to_string(), port, path_prefix.to_string())
        } else {
            return Err(XavierError::Http("Only http:// scheme is supported by minimal HTTP client".to_string()));
        };

        let full_path = format!("{}{}", path_prefix, path);
        let addr = format!("{}:{}", host, port);

        let mut stream = tokio::net::TcpStream::connect(&addr)
            .await
            .map_err(|e| XavierError::Http(format!("Connection to {} failed: {}", addr, e)))?;

        let mut request_header = format!(
            "POST {} HTTP/1.1\r\n\
             Host: {}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n",
            full_path, host, body.len()
        );

        if let Some(ref t) = self.token {
            request_header.push_str(&format!("X-Xavier-Token: {}\r\n", t));
        }
        request_header.push_str("\r\n");

        let mut request = request_header.into_bytes();
        request.extend_from_slice(body.as_bytes());

        stream.write_all(&request)
            .await
            .map_err(|e| XavierError::Http(format!("Failed to write request: {}", e)))?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response)
            .await
            .map_err(|e| XavierError::Http(format!("Failed to read response: {}", e)))?;

        let response_str = String::from_utf8_lossy(&response);
        let mut parts = response_str.splitn(2, "\r\n\r\n");
        let headers = parts.next().ok_or_else(|| XavierError::Http("No response headers".to_string()))?;
        let body_resp = parts.next().unwrap_or("");

        let status_line = headers.lines().next().ok_or_else(|| XavierError::Http("No status line".to_string()))?;
        let status_parts: Vec<&str> = status_line.split_whitespace().collect();
        if status_parts.len() < 2 {
            return Err(XavierError::Http(format!("Invalid status line: {}", status_line)));
        }
        let status_code = status_parts[1];

        if status_code == "401" {
            return Err(XavierError::Unauthorized);
        }

        if status_code != "200" && status_code != "201" && status_code != "204" {
            return Err(XavierError::Http(format!("Server returned error code {}: {}", status_code, body_resp)));
        }

        let is_chunked = headers.to_lowercase().contains("transfer-encoding: chunked");
        if is_chunked {
            let mut decoded = String::new();
            let mut remaining = body_resp;
            while !remaining.is_empty() {
                let mut parts = remaining.splitn(2, "\r\n");
                let size_str = parts.next().ok_or_else(|| XavierError::Parse("Invalid chunk size".to_string()))?.trim();
                let size = usize::from_str_radix(size_str, 16).map_err(|e| XavierError::Parse(format!("Invalid chunk hex: {}", e)))?;
                if size == 0 {
                    break;
                }
                let rest = parts.next().ok_or_else(|| XavierError::Parse("Missing chunk data".to_string()))?;
                if rest.len() < size {
                    return Err(XavierError::Parse("Incomplete chunk body".to_string()));
                }
                decoded.push_str(&rest[..size]);
                remaining = &rest[size..];
                if remaining.starts_with("\r\n") {
                    remaining = &remaining[2..];
                }
            }
            Ok(decoded)
        } else {
            Ok(body_resp.to_string())
        }
    }
}

#[async_trait::async_trait]
impl XavierTransport for HttpTransport {
    async fn store(&self, path: &str, content: &str) -> Result<(), XavierError> {
        let body = serde_json::json!({
            "path": path,
            "content": content
        });
        self.send_post("/v1/memories", &body.to_string()).await?;
        Ok(())
    }

    async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<XavierHit>, XavierError> {
        let limit_val = limit.unwrap_or(10);
        let body = serde_json::json!({
            "query": query,
            "limit": limit_val
        });
        let response_body = self.send_post("/v1/memories/search", &body.to_string()).await?;

        let json_val: serde_json::Value = serde_json::from_str(&response_body)
            .map_err(|e| XavierError::Parse(format!("Failed to parse JSON response: {}, Body: {}", e, response_body)))?;

        if let Some(hits_arr) = json_val.as_array() {
            let hits: Vec<XavierHit> = serde_json::from_value(serde_json::Value::Array(hits_arr.clone()))
                .map_err(|e| XavierError::Parse(format!("Failed to deserialize hits array: {}", e)))?;
            Ok(hits)
        } else if let Some(results) = json_val.get("results") {
            let hits: Vec<XavierHit> = serde_json::from_value(results.clone())
                .map_err(|e| XavierError::Parse(format!("Failed to deserialize results: {}", e)))?;
            Ok(hits)
        } else {
            Err(XavierError::Parse(format!("Unexpected response shape: {}", response_body)))
        }
    }
}

pub struct XavierClient {
    pub base_url: String,
    pub token: Option<String>,
    transport: Option<Arc<dyn XavierTransport>>,
}

impl XavierClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            token: None,
            transport: None,
        }
    }

    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }

    pub fn with_transport(mut self, transport: Arc<dyn XavierTransport>) -> Self {
        self.transport = Some(transport);
        self
    }

    pub async fn store(&self, path: &str, content: &str) -> Result<(), XavierError> {
        if let Some(ref t) = self.transport {
            t.store(path, content).await
        } else {
            let transport = HttpTransport::new(self.base_url.clone(), self.token.clone());
            transport.store(path, content).await
        }
    }

    pub async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<XavierHit>, XavierError> {
        if let Some(ref t) = self.transport {
            t.search(query, limit).await
        } else {
            let transport = HttpTransport::new(self.base_url.clone(), self.token.clone());
            transport.search(query, limit).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct InMemoryXavierTransport {
        memories: Mutex<Vec<XavierHit>>,
    }

    impl InMemoryXavierTransport {
        fn new() -> Self {
            Self {
                memories: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl XavierTransport for InMemoryXavierTransport {
        async fn store(&self, path: &str, content: &str) -> Result<(), XavierError> {
            let mut guard = self.memories.lock().unwrap();
            if let Some(existing) = guard.iter_mut().find(|m| m.path == path) {
                existing.content = content.to_string();
            } else {
                guard.push(XavierHit {
                    path: path.to_string(),
                    content: content.to_string(),
                    score: 1.0,
                });
            }
            Ok(())
        }

        async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<XavierHit>, XavierError> {
            let guard = self.memories.lock().unwrap();
            let limit_val = limit.unwrap_or(10);

            let mut results = Vec::new();
            for m in guard.iter() {
                if query.is_empty() || m.path.contains(query) || m.content.contains(query) {
                    results.push(m.clone());
                }
            }
            results.truncate(limit_val);
            Ok(results)
        }
    }

    #[tokio::test]
    async fn test_xavier_client_roundtrip() {
        let mock_transport = Arc::new(InMemoryXavierTransport::new());
        let client = XavierClient::new("http://localhost:8006")
            .with_token(Some("my-super-secret-token".to_string()))
            .with_transport(mock_transport.clone());

        // Assert store
        client.store("some/path", "hello xavier memory").await.expect("store failed");

        // Assert search with exact content
        let results = client.search("hello", None).await.expect("search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "some/path");
        assert_eq!(results[0].content, "hello xavier memory");
        assert_eq!(results[0].score, 1.0);

        // Assert search with no query matches
        let results2 = client.search("not-matching", None).await.expect("search failed");
        assert!(results2.is_empty());
    }
}
