use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents the role of a message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    /// System message that sets context or instructions
    System,
    /// Message from the user
    User,
    /// Message from the AI assistant
    Assistant,
}

/// Represents a single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender
    pub role: Role,
    /// The content of the message
    pub content: String,
    /// When the message was created
    pub timestamp: DateTime<Utc>,
}

impl Message {
    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            timestamp: Utc::now(),
        }
    }

    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            timestamp: Utc::now(),
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_system() {
        let msg = Message::system("You are a helpful assistant");
        assert_eq!(msg.role, Role::System);
        assert_eq!(msg.content, "You are a helpful assistant");
    }

    #[test]
    fn test_message_user() {
        let msg = Message::user("Hello, world!");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello, world!");
    }

    #[test]
    fn test_message_assistant() {
        let msg = Message::assistant("Hello! How can I help you?");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content, "Hello! How can I help you?");
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("test");
        let json = serde_json::to_string(&msg).expect("Failed to serialize message");
        let deserialized: Message =
            serde_json::from_str(&json).expect("Failed to deserialize message");
        assert_eq!(msg.role, deserialized.role);
        assert_eq!(msg.content, deserialized.content);
    }
}
