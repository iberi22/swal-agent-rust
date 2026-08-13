use leptos::prelude::*;
use crate::worker;

#[cfg(target_arch = "wasm32")]
use swal_store::{indexeddb::IndexedDbStore, session::Store};

/// Message structure for the chat view.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
}

/// Main application shell component.
#[component]
pub fn App() -> impl IntoView {
    // Reactive signals
    #[allow(unused_variables)]
    let (sessions, set_sessions) = signal(vec![
        "Session 1".to_string(),
        "Session 2".to_string(),
        "Session 3".to_string(),
    ]);

    let (messages, set_messages) = signal(vec![
        ChatMessage {
            sender: "System".to_string(),
            content: "Welcome to swal-pwa!".to_string(),
        }
    ]);

    let (input_text, set_input_text) = signal(String::new());

    #[cfg(target_arch = "wasm32")]
    {
        let set_sessions_clone = set_sessions.clone();
        wasm_bindgen_futures::spawn_local(async move {
            // Briefly delay query to allow background IndexedDbStore cache load to finalize
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                if let Some(w) = web_sys::window() {
                    let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 150);
                }
            });
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;

            match IndexedDbStore::open() {
                Ok(store) => {
                    match store.list_sessions() {
                        Ok(list) => {
                            if !list.is_empty() {
                                let names: Vec<String> = list.into_iter().map(|s| s.summary).collect();
                                set_sessions_clone.set(names);
                            }
                        }
                        Err(e) => {
                            web_sys::console::warn_1(&format!("Failed to list sessions: {:?}", e).into());
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::warn_1(&format!("Failed to open IndexedDbStore: {:?}", e).into());
                }
            }
        });
    }

    // Submit handler
    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let text = input_text.get();
        if !text.trim().is_empty() {
            // Add user message
            set_messages.update(|msgs| {
                msgs.push(ChatMessage {
                    sender: "User".to_string(),
                    content: text.clone(),
                });
            });

            // Call worker stub
            let worker_response = worker::run_task(&text);

            // Add system/worker response message
            set_messages.update(|msgs| {
                msgs.push(ChatMessage {
                    sender: "Worker".to_string(),
                    content: format!("Worker output: '{}'", worker_response),
                });
            });

            #[cfg(target_arch = "wasm32")]
            {
                match IndexedDbStore::open() {
                    Ok(store) => {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let session_id = format!("session_{}", now);
                        let summary = if text.len() > 30 {
                            format!("{}...", &text[..30])
                        } else {
                            text.clone()
                        };
                        if let Ok(_sess) = store.create_session(&session_id, &summary) {
                            let _ = store.append_message(&session_id, "user", &text);
                            let _ = store.append_message(&session_id, "worker", &worker_response);
                        }
                        if let Ok(list) = store.list_sessions() {
                            let names: Vec<String> = list.into_iter().map(|s| s.summary).collect();
                            set_sessions.set(names);
                        }
                    }
                    Err(e) => {
                        web_sys::console::warn_1(&format!("Failed to open IndexedDbStore on submit: {:?}", e).into());
                    }
                }
            }

            // Reset input field
            set_input_text.set(String::new());
        }
    };

    view! {
        <div style="display: flex; height: 100vh; font-family: sans-serif;">
            // Sidebar
            <aside style="width: 250px; border-right: 1px solid #ccc; padding: 1rem; background-color: #f9f9f9;">
                <h3>"Sessions"</h3>
                <ul style="list-style: none; padding: 0;">
                    {move || sessions.get().into_iter().map(|session| {
                        view! {
                            <li style="padding: 0.5rem 0; border-bottom: 1px solid #eee; cursor: pointer;">
                                {session}
                            </li>
                        }
                    }).collect::<Vec<_>>()}
                </ul>
            </aside>

            // Chat View
            <main style="flex: 1; display: flex; flex-direction: column; padding: 1rem;">
                // Chat history
                <div style="flex: 1; overflow-y: auto; margin-bottom: 1rem; border: 1px solid #eee; padding: 1rem; border-radius: 4px;">
                    {move || messages.get().into_iter().map(|msg| {
                        view! {
                            <div style="margin-bottom: 1rem;">
                                <strong>{msg.sender}": "</strong>
                                <span>{msg.content}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                // Input form
                <form on:submit=handle_submit style="display: flex; gap: 0.5rem;">
                    <input
                        type="text"
                        placeholder="Type a message..."
                        style="flex: 1; padding: 0.5rem; border: 1px solid #ccc; border-radius: 4px;"
                        prop:value=input_text
                        on:input=move |ev| set_input_text.set(event_target_value(&ev))
                    />
                    <button type="submit" style="padding: 0.5rem 1rem; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;">
                        "Run"
                    </button>
                </form>
            </main>
        </div>
    }
}
