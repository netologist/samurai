//! Token counting functionality for messages.
//!
//! This module provides token counting using tiktoken-rs with the cl100k_base
//! encoding used by GPT-3.5 and GPT-4 models.

use agent_core::Message;
use tiktoken_rs::cl100k_base;

/// Count the number of tokens in a message using the cl100k_base encoding (GPT-3.5/GPT-4)
///
/// This function counts tokens for both the role prefix and the message content.
/// The token count is approximate and includes overhead for role formatting.
///
/// # Examples
///
/// ```
/// use memory::count_tokens;
/// use agent_core::{Message, Role};
/// use chrono::Utc;
///
/// let message = Message {
///     role: Role::User,
///     content: "Hello, world!".to_string(),
///     timestamp: Utc::now(),
/// };
///
/// let count = count_tokens(&message);
/// assert!(count > 0);
/// ```
pub fn count_tokens(message: &Message) -> usize {
    let bpe = cl100k_base().expect("Failed to load cl100k_base encoding");

    // Count tokens for role prefix (approximate)
    let role_tokens = match message.role {
        agent_core::Role::System => 4,    // "system: "
        agent_core::Role::User => 4,      // "user: "
        agent_core::Role::Assistant => 4, // "assistant: "
    };

    // Count tokens in the content
    let content_tokens = bpe.encode_with_special_tokens(&message.content).len();

    role_tokens + content_tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{Message, Role};

    #[test]
    fn test_count_tokens_simple() {
        let message = Message {
            role: Role::User,
            content: "Hello, world!".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let count = count_tokens(&message);
        // Should be more than 0
        assert!(count > 0);
    }

    #[test]
    fn test_count_tokens_longer_message() {
        let message = Message {
            role: Role::Assistant,
            content: "This is a longer message with more words to count tokens for.".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let count = count_tokens(&message);
        // Longer message should have more tokens
        assert!(count > 10);
    }
}
