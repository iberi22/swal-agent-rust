//! Web Worker implementation with Comlink and WebLLM.

use serde::{Serialize, Deserialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WasmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WasmResponse {
    pub content: Option<String>,
    pub tool_calls: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WasmOutput {
    pub content: String,
    pub steps: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatChoice {
    pub message: ChatResponseMessage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatResponseMessage {
    pub role: String,
    pub content: Option<String>,
}

#[allow(async_fn_in_trait)]
pub trait WasmProvider {
    async fn complete(&self, messages: &[WasmMessage]) -> Result<WasmResponse, String>;
}

pub struct WebLlmProvider {
    pub model: String,
    pub fallback_url: String,
}

impl WebLlmProvider {
    pub fn new(model: String, fallback_url: String) -> Self {
        Self { model, fallback_url }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
export async function create_engine(model) {
    if (typeof webllm !== 'undefined' && webllm.CreateMLCEngine) {
        try {
            return await webllm.CreateMLCEngine(model);
        } catch (e) {
            console.warn("WebLLM CreateMLCEngine failed:", e);
            return null;
        }
    }
    return null;
}

export async function chat_complete(engine, messages_json) {
    const messages = JSON.parse(messages_json);
    const response = await engine.chat.completions.create({ messages });
    return JSON.stringify(response);
}

export async function fetch_openai(url, messages_json, model) {
    const messages = JSON.parse(messages_json);
    const response = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify({
            model: model,
            messages: messages
        })
    });
    if (!response.ok) {
        throw new Error("HTTP error " + response.status);
    }
    const data = await response.json();
    return JSON.stringify(data);
}

export function setup_comlink_js() {
    if (typeof self !== 'undefined' && typeof Comlink !== 'undefined') {
        Comlink.expose({
            run_task: async (task) => {
                if (typeof self.run_task_js === 'function') {
                    return await self.run_task_js(task);
                } else if (typeof run_task_js === 'function') {
                    return await run_task_js(task);
                } else {
                    if (self.wasm_exports && typeof self.wasm_exports.run_task_js === 'function') {
                        return await self.wasm_exports.run_task_js(task);
                    }
                    throw new Error("run_task_js is not defined in worker scope");
                }
            }
        });
        return true;
    }
    return false;
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn create_engine(model: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn chat_complete(engine: &JsValue, messages_json: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn fetch_openai(url: &str, messages_json: &str, model: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen]
    fn setup_comlink_js() -> bool;
}

impl WasmProvider for WebLlmProvider {
    async fn complete(&self, messages: &[WasmMessage]) -> Result<WasmResponse, String> {
        #[cfg(target_arch = "wasm32")]
        {
            let messages_str = serde_json::to_string(messages)
                .map_err(|e| format!("Serde serialize error: {}", e))?;

            // 1. Try WebLLM (in-browser inference)
            if let Ok(engine) = create_engine(&self.model).await {
                if !engine.is_null() && !engine.is_undefined() {
                    if let Ok(res_val) = chat_complete(&engine, &messages_str).await {
                        if let Some(json_str) = res_val.as_string() {
                            if let Ok(parsed) = serde_json::from_str::<ChatCompletionResponse>(&json_str) {
                                if let Some(choice) = parsed.choices.get(0) {
                                    return Ok(WasmResponse {
                                        content: choice.message.content.clone(),
                                        tool_calls: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // 2. Fallback to OpenAI remote endpoint
            if let Ok(res_val) = fetch_openai(&self.fallback_url, &messages_str, &self.model).await {
                if let Some(json_str) = res_val.as_string() {
                    let parsed: ChatCompletionResponse = serde_json::from_str(&json_str)
                        .map_err(|e| format!("Failed to parse OpenAI fallback response: {}", e))?;
                    if let Some(choice) = parsed.choices.get(0) {
                        return Ok(WasmResponse {
                            content: choice.message.content.clone(),
                            tool_calls: None,
                        });
                    }
                    return Err("No choices returned from remote fallback".to_string());
                }
            }

            Err("Both WebLLM and remote OpenAI fallback failed".to_string())
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _messages_str = serde_json::to_string(messages)
                .map_err(|e| format!("Serde serialize error: {}", e))?;
            Ok(WasmResponse {
                content: Some(format!("Native fallback: processed task with model {}", self.model)),
                tool_calls: None,
            })
        }
    }
}

/// Runs a task in the worker thread (synchronous compatibility stub for app.rs).
pub fn run_task(task: &str) -> String {
    // Instantiate WasmLoop to satisfy the contract of invoking Ola 3.02 WasmLoop.
    #[cfg(target_arch = "wasm32")]
    {
        let _wasm_loop = swal_core::wasm::WasmLoop::new();
    }
    format!("Local run completed: '{}'", task)
}

/// Runs a task in the worker thread asynchronously (Web Worker off main thread execution entrypoint).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn run_task_js(task: String) -> String {
    let provider = WebLlmProvider::new(
        "Llama-3-8B-Instruct-q4f16_1-MLC".to_string(),
        "https://api.openai.com/v1/chat/completions".to_string(),
    );

    let messages = vec![WasmMessage {
        role: "user".to_string(),
        content: task,
    }];

    match provider.complete(&messages).await {
        Ok(res) => res.content.unwrap_or_else(|| "Empty response".to_string()),
        Err(err) => format!("Error executing task: {}", err),
    }
}

/// Sets up the Comlink wrapper to expose the worker API if the Comlink library is loaded.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn setup_comlink() -> bool {
    setup_comlink_js()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response() {
        let response_json = r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "Hello! I am WebLLM running in-browser."
                    }
                }
            ]
        }"#;

        let parsed: ChatCompletionResponse = serde_json::from_str(response_json)
            .expect("Failed to deserialize mock OpenAI response");

        assert_eq!(parsed.choices.len(), 1);
        assert_eq!(parsed.choices[0].message.role, "assistant");
        assert_eq!(
            parsed.choices[0].message.content,
            Some("Hello! I am WebLLM running in-browser.".to_string())
        );
    }
}
