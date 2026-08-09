use swal_store::session::{SessionStore, Store};

#[test]
fn test_session_store_crud_roundtrip() {
    // 1. Initialize an in-memory session store for clean, isolated, network-free testing.
    let store =
        SessionStore::open_in_memory().expect("Failed to initialize in-memory SessionStore");

    // Verify initial row counts
    assert_eq!(store.count_rows("sessions").unwrap(), 0);
    assert_eq!(store.count_rows("messages").unwrap(), 0);

    // 2. Create a session
    let session_id = "test-session-123";
    let summary = "A test chat session";
    let session = store
        .create_session(session_id, summary)
        .expect("Failed to create session");

    assert_eq!(session.id, session_id);
    assert_eq!(session.summary, summary);
    assert!(session.created_at > 0);
    assert_eq!(session.created_at, session.updated_at);

    // Verify row counts after creation
    assert_eq!(store.count_rows("sessions").unwrap(), 1);
    assert_eq!(store.count_rows("messages").unwrap(), 0);

    // 3. Append first message
    let msg1 = store
        .append_message(session_id, "user", "Hello agent!")
        .expect("Failed to append first message");

    assert_eq!(msg1.session_id, session_id);
    assert_eq!(msg1.role, "user");
    assert_eq!(msg1.content, "Hello agent!");
    assert!(msg1.ts >= session.created_at);

    // Verify row counts after first message
    assert_eq!(store.count_rows("sessions").unwrap(), 1);
    assert_eq!(store.count_rows("messages").unwrap(), 1);

    // 4. Append second message
    let msg2 = store
        .append_message(session_id, "assistant", "Hello! How can I help you today?")
        .expect("Failed to append second message");

    assert_eq!(msg2.session_id, session_id);
    assert_eq!(msg2.role, "assistant");
    assert_eq!(msg2.content, "Hello! How can I help you today?");
    assert!(msg2.ts >= msg1.ts);

    // Verify row counts after second message
    assert_eq!(store.count_rows("sessions").unwrap(), 1);
    assert_eq!(store.count_rows("messages").unwrap(), 2);

    // 5. Read back session & messages and verify
    let fetched_session = store
        .get_session(session_id)
        .expect("Failed to get session")
        .expect("Session not found");

    assert_eq!(fetched_session.id, session_id);
    assert_eq!(fetched_session.summary, summary);
    // Updated_at should have been updated upon appending messages
    assert!(fetched_session.updated_at >= fetched_session.created_at);

    let messages = store
        .get_messages(session_id)
        .expect("Failed to get messages");

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].id, msg1.id);
    assert_eq!(messages[0].content, "Hello agent!");
    assert_eq!(messages[1].id, msg2.id);
    assert_eq!(messages[1].content, "Hello! How can I help you today?");

    // Verify list_sessions returns our session
    let all_sessions = store.list_sessions().expect("Failed to list sessions");
    assert_eq!(all_sessions.len(), 1);
    assert_eq!(all_sessions[0].id, session_id);

    // 6. Delete session
    store
        .delete_session(session_id)
        .expect("Failed to delete session");

    // Verify row counts after deletion (should be empty cascade)
    assert_eq!(store.count_rows("sessions").unwrap(), 0);
    assert_eq!(store.count_rows("messages").unwrap(), 0);

    // Verify list_sessions is empty
    let all_sessions_after = store.list_sessions().expect("Failed to list sessions");
    assert_eq!(all_sessions_after.len(), 0);
}
