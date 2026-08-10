//! Compaction Module — Context window management and message compaction.
//!
//! # Reuse Analysis
//! synapse-agentic provides a `SessionContext` and `LLMSummarizer` under its `context` module.
//! However, `LLMSummarizer` is asynchronous, relies on an external `LLMProvider`, and has a
//! different Message type structure, making a direct synchronous wrapping of a function like
//! `pub fn compact(messages: &[Message], max_tokens: usize) -> Vec<Message>` impractical.
//!
//! Therefore, we implement the v1 fallback: a deterministic, robust context trimming strategy
//! that retains the initial system prompt (if present) and the last N messages to fit within
//! a specified token heuristic threshold, inserting a system marker to indicate the compaction boundaries.

use crate::provider::Message;

/// Heuristic to determine if compaction should be triggered based on estimated character/token length.
///
/// We estimate tokens using a standard char-to-token ratio (1 token is roughly 4 characters).
/// If the total character length of the messages is greater than `max_tokens * 4`, compaction is needed.
pub fn should_compact(messages: &[Message], max_tokens: usize) -> bool {
    let threshold = max_tokens * 4;
    let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
    total_chars > threshold
}

/// Compacts a sequence of messages to fit within `max_tokens` (based on the character heuristic threshold).
///
/// Keeps the first system message (the system prompt) if present.
/// Prepends a system summary marker: `[compacted: kept last N of M messages]`.
/// Keeps the last N messages such that the total size of the first system message, the summary marker,
/// and the last N messages is under the character threshold.
pub fn compact(messages: &[Message], max_tokens: usize) -> Vec<Message> {
    if !should_compact(messages, max_tokens) {
        return messages.to_vec();
    }

    let total_messages = messages.len();
    if total_messages == 0 {
        return Vec::new();
    }

    let threshold = max_tokens * 4;

    // Check if the first message is a system prompt
    let has_system_first = messages[0].role == "system";
    let system_first_msg = if has_system_first {
        Some(&messages[0])
    } else {
        None
    };

    // Find the maximum number N of trailing messages we can keep
    let max_possible_n = if has_system_first {
        total_messages - 1
    } else {
        total_messages
    };

    let mut best_n = 0;

    for n in (0..=max_possible_n).rev() {
        let system_len = system_first_msg.map(|m| m.content.len()).unwrap_or(0);
        let marker_content = format!("[compacted: kept last {} of {} messages]", n, total_messages);
        let marker_len = marker_content.len();

        let last_n_start = total_messages - n;
        let last_n_len: usize = messages[last_n_start..]
            .iter()
            .map(|m| m.content.len())
            .sum();

        let total_len = system_len + marker_len + last_n_len;
        if total_len <= threshold {
            best_n = n;
            break;
        }
    }

    // Ensure we keep at least 1 message if possible
    if best_n == 0 && max_possible_n >= 1 {
        best_n = 1;
    }

    let mut compacted = Vec::new();
    if let Some(sys_msg) = system_first_msg {
        compacted.push(sys_msg.clone());
    }

    let marker_content = format!("[compacted: kept last {} of {} messages]", best_n, total_messages);
    compacted.push(Message {
        role: "system".to_string(),
        content: marker_content,
    });

    let last_n_start = total_messages - best_n;
    for msg in &messages[last_n_start..] {
        compacted.push(msg.clone());
    }

    compacted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_compact_boundary() {
        // max_tokens = 10, so threshold is 40 characters
        let max_tokens = 10;

        // 39 characters -> should_compact = false
        let msg_short = vec![
            Message {
                role: "system".to_string(),
                content: "1234567890".to_string(), // 10
            },
            Message {
                role: "user".to_string(),
                content: "1234567890123456789".to_string(), // 19
            },
            Message {
                role: "assistant".to_string(),
                content: "1234567890".to_string(), // 10
            },
        ];
        assert_eq!(msg_short.iter().map(|m| m.content.len()).sum::<usize>(), 39);
        assert!(!should_compact(&msg_short, max_tokens));

        // 40 characters -> should_compact = false
        let msg_boundary = vec![
            Message {
                role: "system".to_string(),
                content: "1234567890".to_string(), // 10
            },
            Message {
                role: "user".to_string(),
                content: "12345678901234567890".to_string(), // 20
            },
            Message {
                role: "assistant".to_string(),
                content: "1234567890".to_string(), // 10
            },
        ];
        assert_eq!(msg_boundary.iter().map(|m| m.content.len()).sum::<usize>(), 40);
        assert!(!should_compact(&msg_boundary, max_tokens));

        // 41 characters -> should_compact = true
        let msg_long = vec![
            Message {
                role: "system".to_string(),
                content: "1234567890".to_string(), // 10
            },
            Message {
                role: "user".to_string(),
                content: "123456789012345678901".to_string(), // 21
            },
            Message {
                role: "assistant".to_string(),
                content: "1234567890".to_string(), // 10
            },
        ];
        assert_eq!(msg_long.iter().map(|m| m.content.len()).sum::<usize>(), 41);
        assert!(should_compact(&msg_long, max_tokens));
    }

    #[test]
    fn test_compact_short_list() {
        let max_tokens = 10;
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "System prompt".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ];

        let result = compact(&messages, max_tokens);
        // Under threshold, should be returned unchanged
        assert_eq!(result, messages);
    }

    #[test]
    fn test_compact_long_list() {
        let max_tokens = 25; // threshold = 100 chars
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "System prompt here".to_string(), // 18 chars
            },
            Message {
                role: "user".to_string(),
                content: "Very long message indeed that contains a huge amount of text, definitely exceeding any reasonable short limit and triggering compaction properly.".to_string(), // 151 chars
            },
            Message {
                role: "assistant".to_string(),
                content: "Brief response".to_string(), // 14 chars
            },
        ];

        let total_orig_chars: usize = messages.iter().map(|m| m.content.len()).sum();
        assert!(total_orig_chars > max_tokens * 4);

        let compacted = compact(&messages, max_tokens);

        // First message (system) should be kept
        assert_eq!(compacted[0].role, "system");
        assert_eq!(compacted[0].content, "System prompt here");

        // Marker message should be present
        assert_eq!(compacted[1].role, "system");
        assert!(compacted[1].content.contains("compacted"));

        // Overall size (total character length) should be reduced compared to the original
        let total_compacted_chars: usize = compacted.iter().map(|m| m.content.len()).sum();
        assert!(total_compacted_chars < total_orig_chars);
    }
}
