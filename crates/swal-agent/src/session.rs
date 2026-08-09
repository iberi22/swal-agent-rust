use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use swal_store::session::{SessionStore, Store, Message};
use crate::config::Config;

/// A handle to a specific persistence session.
#[allow(dead_code)]
#[derive(Clone)]
pub struct SessionHandle {
    pub store: Arc<SessionStore>,
    pub session_id: String,
}

#[allow(dead_code)]
impl SessionHandle {
    /// Opens the default SQLite database and creates a new session.
    pub fn open(_config: &Config) -> anyhow::Result<Self> {
        let store = SessionStore::open_default()?;
        let store = Arc::new(store);

        let session_id = format!(
            "{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        store.create_session(&session_id, "New Session")?;

        Ok(Self { store, session_id })
    }

    /// Appends a new message to this session.
    pub async fn append(&self, role: &str, content: &str) -> anyhow::Result<()> {
        self.store.append_message(&self.session_id, role, content)?;
        Ok(())
    }

    /// Lists all messages in this session.
    pub async fn list_messages(&self) -> anyhow::Result<Vec<Message>> {
        let messages = self.store.get_messages(&self.session_id)?;
        Ok(messages)
    }
}

/// A wrapper around SessionHandle to preserve the public `Session` export.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Session {
    pub handle: SessionHandle,
}

#[allow(dead_code)]
impl Session {
    /// Creates a Session wrapper from a SessionHandle.
    pub fn from_handle(handle: SessionHandle) -> Self {
        Self { handle }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_append_and_list() {
        let store = Arc::new(SessionStore::open_in_memory().expect("failed to open in-memory store"));
        let session_id = "test_session_id".to_string();
        store.create_session(&session_id, "Test Session").expect("failed to create session");

        let handle = SessionHandle {
            store,
            session_id,
        };

        handle.append("user", "Hello").await.expect("failed to append user message");
        handle.append("assistant", "Hi there").await.expect("failed to append assistant message");

        let messages = handle.list_messages().await.expect("failed to list messages");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].content, "Hi there");

        // Verify from_handle creates a Session
        let session = Session::from_handle(handle.clone());
        assert_eq!(session.handle.session_id, "test_session_id");
    }
}
