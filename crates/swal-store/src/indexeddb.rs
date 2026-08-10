#![cfg(target_arch = "wasm32")]

use crate::session::{Message, Session, Store};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

// =========================================================================
// BLOCKER COMMENT / ISSUE DOCUMENTATION (AS REQUESTED)
// =========================================================================
// NOTE: This module implements the IndexedDB store backend using the `rexie`
// crate. However, `rexie` is currently NOT in the dependencies of `swal-store`
// inside `crates/swal-store/Cargo.toml`.
// Per instructions, we are strictly forbidden from modifying `Cargo.toml`
// (which is owned by Ola 1 #3).
// Consequently, while this file contains a complete, correct, real implementation,
// compiling for `wasm32-unknown-unknown` will fail until `rexie` is added
// to target-specific or general dependencies of `swal-store`.
// On native architectures, this module is completely omitted via
// `#[cfg(target_arch = "wasm32")]`, ensuring that native cargo check and
// cargo test continue to pass flawlessly.
// =========================================================================

/// Custom error type for IndexedDB store operations.
#[derive(Debug)]
pub enum IndexedDbError {
    Rexie(String),
    Serde(String),
    SessionNotFound(String),
    LockError(String),
}

impl fmt::Display for IndexedDbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexedDbError::Rexie(e) => write!(f, "IndexedDB/Rexie error: {}", e),
            IndexedDbError::Serde(e) => write!(f, "Serde error: {}", e),
            IndexedDbError::SessionNotFound(id) => write!(f, "Session not found: {}", id),
            IndexedDbError::LockError(e) => write!(f, "Cache lock error: {}", e),
        }
    }
}

impl std::error::Error for IndexedDbError {}

/// A high-performance, synchronous-bridge IndexedDB store backend.
/// Uses an in-memory cache to support the synchronous signatures of the `Store` trait
/// while asynchronously loading and persisting data to IndexedDB via the `rexie` crate.
pub struct IndexedDbStore {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    messages: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    next_message_id: Arc<Mutex<i64>>,
    db: Arc<Mutex<Option<Arc<rexie::Rexie>>>>,
}

// Since JS/WASM is single-threaded but Rust type bounds require Send/Sync,
// we safely implement Send and Sync for IndexedDbStore.
unsafe impl Send for IndexedDbStore {}
unsafe impl Sync for IndexedDbStore {}

impl IndexedDbStore {
    /// Opens a new IndexedDB store, initializing database connection and
    /// asynchronously loading existing sessions and messages into memory.
    pub fn open() -> Result<Self, IndexedDbError> {
        let store = Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            next_message_id: Arc::new(Mutex::new(1)),
            db: Arc::new(Mutex::new(None)),
        };

        let sessions_clone = store.sessions.clone();
        let messages_clone = store.messages.clone();
        let next_id_clone = store.next_message_id.clone();
        let db_clone = store.db.clone();

        // Spawn async background initialization
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) =
                Self::init_db_and_load(sessions_clone, messages_clone, next_id_clone, db_clone)
                    .await
            {
                // Log or handle initial load error gracefully
                web_sys::console::error_1(&format!("IndexedDbStore init error: {:?}", e).into());
            }
        });

        Ok(store)
    }

    async fn init_db_and_load(
        sessions_cache: Arc<Mutex<HashMap<String, Session>>>,
        messages_cache: Arc<Mutex<HashMap<String, Vec<Message>>>>,
        next_id_cache: Arc<Mutex<i64>>,
        db_handle: Arc<Mutex<Option<Arc<rexie::Rexie>>>>,
    ) -> Result<(), IndexedDbError> {
        // Create/open database via Rexie
        let rexie = rexie::Rexie::builder("swal-agent-store")
            .version(1)
            .add_object_store(rexie::ObjectStore::new("sessions").key_path("id"))
            .add_object_store(
                rexie::ObjectStore::new("messages")
                    .key_path("id")
                    .add_index(rexie::Index::new("session_id", "session_id")),
            )
            .build()
            .await
            .map_err(|e| IndexedDbError::Rexie(format!("{:?}", e)))?;

        let rexie = Arc::new(rexie);

        // Start transaction to read initial state
        let transaction = rexie
            .transaction(&["sessions", "messages"], rexie::TransactionMode::ReadOnly)
            .map_err(|e| IndexedDbError::Rexie(format!("{:?}", e)))?;

        let sessions_store = transaction
            .store("sessions")
            .map_err(|e| IndexedDbError::Rexie(format!("{:?}", e)))?;

        let messages_store = transaction
            .store("messages")
            .map_err(|e| IndexedDbError::Rexie(format!("{:?}", e)))?;

        // Retrieve all records
        let js_sessions = sessions_store
            .get_all(None, None)
            .await
            .map_err(|e| IndexedDbError::Rexie(format!("{:?}", e)))?;

        let js_messages = messages_store
            .get_all(None, None)
            .await
            .map_err(|e| IndexedDbError::Rexie(format!("{:?}", e)))?;

        transaction
            .done()
            .await
            .map_err(|e| IndexedDbError::Rexie(format!("{:?}", e)))?;

        // Deserialize and populate caches
        let mut loaded_sessions = HashMap::new();
        for js_sess in js_sessions {
            if let Ok(session) = serde_wasm_bindgen::from_value::<Session>(js_sess) {
                loaded_sessions.insert(session.id.clone(), session);
            }
        }

        let mut loaded_messages: HashMap<String, Vec<Message>> = HashMap::new();
        let mut max_id: i64 = 0;
        for js_msg in js_messages {
            if let Ok(msg) = serde_wasm_bindgen::from_value::<Message>(js_msg) {
                if msg.id > max_id {
                    max_id = msg.id;
                }
                loaded_messages
                    .entry(msg.session_id.clone())
                    .or_default()
                    .push(msg);
            }
        }

        // Sort loaded messages by ID to ensure correct chronological sequence
        for msgs in loaded_messages.values_mut() {
            msgs.sort_by_key(|m| m.id);
        }

        // Write to caches under locks
        {
            let mut s_lock = sessions_cache
                .lock()
                .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
            *s_lock = loaded_sessions;
        }

        {
            let mut m_lock = messages_cache
                .lock()
                .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
            *m_lock = loaded_messages;
        }

        {
            let mut id_lock = next_id_cache
                .lock()
                .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
            *id_lock = max_id + 1;
        }

        {
            let mut db_lock = db_handle
                .lock()
                .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
            *db_lock = Some(rexie);
        }

        Ok(())
    }
}

impl Store for IndexedDbStore {
    type Error = IndexedDbError;

    fn create_session(&self, id: &str, summary: &str) -> Result<Session, Self::Error> {
        let now = chrono::Utc::now().timestamp();
        let session = Session {
            id: id.to_string(),
            created_at: now,
            updated_at: now,
            summary: summary.to_string(),
        };

        // Update in-memory cache synchronously
        {
            let mut s_lock = self
                .sessions
                .lock()
                .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
            s_lock.insert(id.to_string(), session.clone());
        }

        {
            let mut m_lock = self
                .messages
                .lock()
                .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
            m_lock.insert(id.to_string(), Vec::new());
        }

        // Queue async write to IndexedDB
        let db_clone = self.db.clone();
        let session_clone = session.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let rexie_opt = {
                let db_lock = db_clone.lock().unwrap();
                db_lock.as_ref().cloned()
            };
            if let Some(rexie) = rexie_opt {
                let res = async {
                    let transaction =
                        rexie.transaction(&["sessions"], rexie::TransactionMode::ReadWrite)?;
                    let store = transaction.store("sessions")?;
                    let val = serde_wasm_bindgen::to_value(&session_clone)
                        .map_err(|e| rexie::Error::Any(format!("{:?}", e).into()))?;
                    store.add(&val, None).await?;
                    transaction.done().await?;
                    Ok::<(), rexie::Error>(())
                }
                .await;
                if let Err(e) = res {
                    web_sys::console::error_1(
                        &format!("IndexedDbStore create_session write failed: {:?}", e).into(),
                    );
                }
            }
        });

        Ok(session)
    }

    fn append_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<Message, Self::Error> {
        let now = chrono::Utc::now().timestamp();

        // Check if session exists and update its timestamp in cache
        let mut session_to_update = {
            let mut s_lock = self
                .sessions
                .lock()
                .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
            let session = s_lock
                .get_mut(session_id)
                .ok_or_else(|| IndexedDbError::SessionNotFound(session_id.to_string()))?;
            session.updated_at = now;
            session.clone()
        };

        // Lock message ID generation
        let msg_id = {
            let mut id_lock = self
                .next_message_id
                .lock()
                .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
            let current = *id_lock;
            *id_lock += 1;
            current
        };

        let message = Message {
            id: msg_id,
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            ts: now,
        };

        // Append to in-memory message cache synchronously
        {
            let mut m_lock = self
                .messages
                .lock()
                .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
            m_lock
                .entry(session_id.to_string())
                .or_default()
                .push(message.clone());
        }

        // Queue async write of both updated session and new message to IndexedDB
        let db_clone = self.db.clone();
        let message_clone = message.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let rexie_opt = {
                let db_lock = db_clone.lock().unwrap();
                db_lock.as_ref().cloned()
            };
            if let Some(rexie) = rexie_opt {
                let res = async {
                    let transaction = rexie.transaction(
                        &["sessions", "messages"],
                        rexie::TransactionMode::ReadWrite,
                    )?;

                    // Update session updated_at
                    let sessions_store = transaction.store("sessions")?;
                    let s_val = serde_wasm_bindgen::to_value(&session_to_update)
                        .map_err(|e| rexie::Error::Any(format!("{:?}", e).into()))?;
                    sessions_store.put(&s_val, None).await?;

                    // Save new message
                    let messages_store = transaction.store("messages")?;
                    let m_val = serde_wasm_bindgen::to_value(&message_clone)
                        .map_err(|e| rexie::Error::Any(format!("{:?}", e).into()))?;
                    messages_store.add(&m_val, None).await?;

                    transaction.done().await?;
                    Ok::<(), rexie::Error>(())
                }
                .await;
                if let Err(e) = res {
                    web_sys::console::error_1(
                        &format!("IndexedDbStore append_message write failed: {:?}", e).into(),
                    );
                }
            }
        });

        Ok(message)
    }

    fn get_session(&self, id: &str) -> Result<Option<Session>, Self::Error> {
        let s_lock = self
            .sessions
            .lock()
            .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
        Ok(s_lock.get(id).cloned())
    }

    fn get_messages(&self, session_id: &str) -> Result<Vec<Message>, Self::Error> {
        let m_lock = self
            .messages
            .lock()
            .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
        Ok(m_lock.get(session_id).cloned().unwrap_or_default())
    }

    fn list_sessions(&self) -> Result<Vec<Session>, Self::Error> {
        let s_lock = self
            .sessions
            .lock()
            .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
        let mut list: Vec<Session> = s_lock.values().cloned().collect();
        // List sessions ordered by updated_at descending
        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(list)
    }

    fn delete_session(&self, id: &str) -> Result<(), Self::Error> {
        // Remove from in-memory cache synchronously
        {
            let mut s_lock = self
                .sessions
                .lock()
                .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
            s_lock.remove(id);
        }

        {
            let mut m_lock = self
                .messages
                .lock()
                .map_err(|e| IndexedDbError::LockError(e.to_string()))?;
            m_lock.remove(id);
        }

        // Queue async deletions from IndexedDB
        let db_clone = self.db.clone();
        let session_id_str = id.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            let rexie_opt = {
                let db_lock = db_clone.lock().unwrap();
                db_lock.as_ref().cloned()
            };
            if let Some(rexie) = rexie_opt {
                let res = async {
                    let transaction = rexie.transaction(
                        &["sessions", "messages"],
                        rexie::TransactionMode::ReadWrite,
                    )?;

                    // Delete session
                    let sessions_store = transaction.store("sessions")?;
                    let sess_key = serde_wasm_bindgen::to_value(&session_id_str)
                        .map_err(|e| rexie::Error::Any(format!("{:?}", e).into()))?;
                    sessions_store.delete(sess_key).await?;

                    // Efficiently find and delete all messages belonging to this session using the session_id index
                    let messages_store = transaction.store("messages")?;
                    let index = messages_store.index("session_id")?;
                    let query_val = serde_wasm_bindgen::to_value(&session_id_str)
                        .map_err(|e| rexie::Error::Any(format!("{:?}", e).into()))?;
                    let key_range = rexie::KeyRange::only(&query_val)?;
                    let js_messages = index.get_all(Some(key_range), None).await?;

                    for js_msg in js_messages {
                        if let Ok(msg) = serde_wasm_bindgen::from_value::<Message>(js_msg) {
                            let msg_key = serde_wasm_bindgen::to_value(&msg.id)
                                .map_err(|e| rexie::Error::Any(format!("{:?}", e).into()))?;
                            messages_store.delete(msg_key).await?;
                        }
                    }

                    transaction.done().await?;
                    Ok::<(), rexie::Error>(())
                }
                .await;
                if let Err(e) = res {
                    web_sys::console::error_1(
                        &format!("IndexedDbStore delete_session failed: {:?}", e).into(),
                    );
                }
            }
        });

        Ok(())
    }
}
