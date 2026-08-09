use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Shared serde representation of a Chat Session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub summary: String,
}

/// Shared serde representation of a Chat Message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub ts: i64,
}

/// Errors returned by the store operations.
#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    SessionNotFound(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "SQLite error: {}", e),
            StoreError::Io(e) => write!(f, "IO error: {}", e),
            StoreError::SessionNotFound(id) => write!(f, "Session not found: {}", id),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Sqlite(e) => Some(e),
            StoreError::Io(e) => Some(e),
            StoreError::SessionNotFound(_) => None,
        }
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(err: rusqlite::Error) -> Self {
        StoreError::Sqlite(err)
    }
}

impl From<std::io::Error> for StoreError {
    fn from(err: std::io::Error) -> Self {
        StoreError::Io(err)
    }
}

/// The `Store` trait defines the interface for persisting chat sessions and messages.
///
/// We implement this synchronously because native SQLite operations are extremely fast
/// and run directly in-process. Synchronous APIs avoid the overhead of task scheduling
/// and async state machines, making the implementation simpler, cleaner, and highly performant
/// for native CLI and daemon operations.
pub trait Store {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Creates a new session with the given ID and summary.
    /// Sets `created_at` and `updated_at` to the current UTC timestamp in seconds.
    fn create_session(&self, id: &str, summary: &str) -> Result<Session, Self::Error>;

    /// Appends a new message to the specified session.
    /// Updates the session's `updated_at` timestamp.
    fn append_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<Message, Self::Error>;

    /// Retrieves a session by its ID.
    fn get_session(&self, id: &str) -> Result<Option<Session>, Self::Error>;

    /// Retrieves all messages belonging to a specified session, ordered by their insert sequence/timestamp.
    fn get_messages(&self, session_id: &str) -> Result<Vec<Message>, Self::Error>;

    /// Lists all sessions, typically ordered by `updated_at` descending.
    fn list_sessions(&self) -> Result<Vec<Session>, Self::Error>;

    /// Deletes a session and all its associated messages.
    fn delete_session(&self, id: &str) -> Result<(), Self::Error>;
}

/// A SQLite backend implementing the `Store` trait.
pub struct SessionStore {
    conn: Mutex<rusqlite::Connection>,
}

impl SessionStore {
    /// Opens a connection to a SQLite database at the specified path.
    /// If the parent directories do not exist, they will be created automatically.
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StoreError> {
        if let Some(parent) = path.as_ref().parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = rusqlite::Connection::open(path)?;
        Self::init_connection(conn)
    }

    /// Opens a connection to a SQLite database at the default path (`data/swal-agent.db`).
    pub fn open_default() -> Result<Self, StoreError> {
        Self::open("data/swal-agent.db")
    }

    /// Opens a temporary, in-memory SQLite database connection. Useful for testing.
    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = rusqlite::Connection::open_in_memory()?;
        Self::init_connection(conn)
    }

    /// Helper to count rows in a table. Used for testing/monitoring.
    pub fn count_rows(&self, table: &str) -> Result<usize, StoreError> {
        let conn = self.conn.lock().unwrap();
        let query = match table {
            "sessions" => "SELECT COUNT(*) FROM sessions;",
            "messages" => "SELECT COUNT(*) FROM messages;",
            _ => return Err(StoreError::Sqlite(rusqlite::Error::QueryReturnedNoRows)),
        };
        let count: usize = conn.query_row(query, [], |row| row.get(0))?;
        Ok(count)
    }

    fn init_connection(conn: rusqlite::Connection) -> Result<Self, StoreError> {
        // Apply WAL journal mode for performance and concurrency.
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Initialize schema tables if they do not exist.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                summary TEXT NOT NULL
            );",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                ts INTEGER NOT NULL
            );",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl Store for SessionStore {
    type Error = StoreError;

    fn create_session(&self, id: &str, summary: &str) -> Result<Session, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO sessions (id, created_at, updated_at, summary) VALUES (?1, ?2, ?3, ?4);",
            rusqlite::params![id, now, now, summary],
        )?;
        Ok(Session {
            id: id.to_string(),
            created_at: now,
            updated_at: now,
            summary: summary.to_string(),
        })
    }

    fn append_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<Message, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        // Check if the session exists.
        let session_exists: bool = conn
            .query_row(
                "SELECT 1 FROM sessions WHERE id = ?1;",
                rusqlite::params![session_id],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !session_exists {
            return Err(StoreError::SessionNotFound(session_id.to_string()));
        }

        conn.execute(
            "INSERT INTO messages (session_id, role, content, ts) VALUES (?1, ?2, ?3, ?4);",
            rusqlite::params![session_id, role, content, now],
        )?;
        let id = conn.last_insert_rowid();

        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2;",
            rusqlite::params![now, session_id],
        )?;

        Ok(Message {
            id,
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            ts: now,
        })
    }

    fn get_session(&self, id: &str) -> Result<Option<Session>, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, created_at, updated_at, summary FROM sessions WHERE id = ?1;")?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Session {
                id: row.get(0)?,
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
                summary: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_messages(&self, session_id: &str) -> Result<Vec<Message>, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, session_id, role, content, ts FROM messages WHERE session_id = ?1 ORDER BY id ASC;")?;
        let mapped = stmt.query_map(rusqlite::params![session_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                ts: row.get(4)?,
            })
        })?;

        let mut messages = Vec::new();
        for msg in mapped {
            messages.push(msg?);
        }
        Ok(messages)
    }

    fn list_sessions(&self) -> Result<Vec<Session>, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, created_at, updated_at, summary FROM sessions ORDER BY updated_at DESC;",
        )?;
        let mapped = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
                summary: row.get(3)?,
            })
        })?;

        let mut sessions = Vec::new();
        for s in mapped {
            sessions.push(s?);
        }
        Ok(sessions)
    }

    fn delete_session(&self, id: &str) -> Result<(), Self::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM messages WHERE session_id = ?1;",
            rusqlite::params![id],
        )?;
        conn.execute("DELETE FROM sessions WHERE id = ?1;", rusqlite::params![id])?;
        Ok(())
    }
}
