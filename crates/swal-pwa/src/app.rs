use leptos::prelude::*;
use crate::worker;

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
    let (sessions, _set_sessions) = signal(vec![
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
                        "Send"
                    </button>
                </form>
            </main>
        </div>
    }
}
