#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String, String>;
}

#[allow(dead_code)]
pub struct MockProvider;

#[async_trait::async_trait]
impl Provider for MockProvider {
    async fn complete(&self, _prompt: &str) -> Result<String, String> {
        Ok(String::new())
    }
}
