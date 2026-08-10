use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use serde_json::Value;
use serde::Deserialize;

pub mod anyhow {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
}

pub trait AgentHandle: Send + Sync {
    fn run_task<'a>(&'a self, task: &'a str) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send + 'a>>;
}

impl AgentHandle for swal_loop::r#loop::AgentLoop {
    fn run_task<'a>(&'a self, task: &'a str) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send + 'a>> {
        Box::pin(async move {
            match self.run(task).await {
                Ok(output) => {
                    serde_json::to_value(output).map_err(|e| e.to_string())
                }
                Err(e) => Err(e.to_string()),
            }
        })
    }
}

#[derive(Clone)]
struct AppState {
    agent: Arc<dyn AgentHandle>,
}

#[derive(Deserialize)]
struct RunRequest {
    task: String,
}

async fn health_handler() -> Json<Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn run_handler(
    State(state): State<AppState>,
    Json(payload): Json<RunRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    match state.agent.run_task(&payload.task).await {
        Ok(val) => Ok(Json(val)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err)),
    }
}

pub async fn serve(agent: Arc<dyn AgentHandle>, addr: SocketAddr) -> anyhow::Result<()> {
    let state = AppState { agent };
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/run", post(run_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use serde_json::json;
    use swal_loop::provider::{MockProvider, ProviderResponse};
    use swal_loop::r#loop::AgentLoop;
    use swal_loop::skills::SkillLoader;
    use swal_core::tool::ToolRegistry;

    #[tokio::test]
    async fn test_http_endpoints() {
        let mut dir = std::env::temp_dir();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("swal_gateway_test_{}", now));
        std::fs::create_dir_all(&dir).unwrap();

        let skills = SkillLoader::new(dir.to_str().unwrap()).unwrap();
        let registry = ToolRegistry::new();

        let mock_resp = ProviderResponse {
            content: "Mock loop response content".to_string(),
            tool_calls: vec![],
        };
        let provider = Arc::new(MockProvider::new(vec![mock_resp]));
        let agent_loop = Arc::new(AgentLoop::new(provider, registry, skills));

        let state = AppState { agent: agent_loop };
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/run", post(run_handler))
            .with_state(state);

        // 1. Test GET /health
        let req = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body_json, json!({ "status": "ok" }));

        // 2. Test POST /run
        let req = Request::builder()
            .method("POST")
            .uri("/run")
            .header("content-type", "application/json")
            .body(Body::from(json!({ "task": "hello mock" }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["content"], "Mock loop response content");
        assert_eq!(body_json["steps"], 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
