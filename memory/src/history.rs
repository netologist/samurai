//! Conversation history wrapper with convenience methods.
//!
//! This module provides a high-level wrapper around MemoryStore that simplifies
//! common operations like adding messages with different roles.

use crate::MemoryStore;
use agent_core::{Message, Role};

/// Wrapper around MemoryStore with convenience methods for common operations
///
/// This wrapper provides ergonomic methods for adding messages with specific roles
/// and delegates storage operations to the underlying MemoryStore implementation.
///
/// # Examples
///
/// ```
/// use memory::{ConversationHistory, InMemoryStore};
///
/// let store = InMemoryStore::new();
/// let mut history = ConversationHistory::new(store);
///
/// history.add_system_message("You are a helpful assistant".to_string());
/// history.add_user_message("Hello!".to_string());
/// history.add_assistant_message("Hi there!".to_string());
///
/// let messages = history.get_recent(10);
/// assert_eq!(messages.len(), 3);
/// ```
pub struct ConversationHistory<T: MemoryStore> {
    store: T,
}

impl<T: MemoryStore> ConversationHistory<T> {
    /// Create a new ConversationHistory with the given store
    pub fn new(store: T) -> Self {
        Self { store }
    }

    /// Add a user message to the conversation
    pub fn add_user_message(&mut self, content: String) {
        let message = Message {
            role: Role::User,
            content,
            timestamp: chrono::Utc::now(),
        };
        self.store.add_message(message);
    }

    /// Add an assistant message to the conversation
    pub fn add_assistant_message(&mut self, content: String) {
        let message = Message {
            role: Role::Assistant,
            content,
            timestamp: chrono::Utc::now(),
        };
        self.store.add_message(message);
    }

    /// Add a system message to the conversation
    pub fn add_system_message(&mut self, content: String) {
        let message = Message {
            role: Role::System,
            content,
            timestamp: chrono::Utc::now(),
        };
        self.store.add_message(message);
    }

    /// Add a message directly
    pub fn add_message(&mut self, message: Message) {
        self.store.add_message(message);
    }

    /// Get the most recent N messages
    pub fn get_recent(&self, limit: usize) -> Vec<Message> {
        self.store.get_recent(limit)
    }

    /// Get messages that fit within a token budget
    pub fn get_within_budget(&self, token_budget: usize) -> Vec<Message> {
        self.store.get_within_budget(token_budget)
    }

    /// Clear all messages from the conversation
    pub fn clear(&mut self) {
        self.store.clear();
    }

    /// Get a reference to the underlying store
    pub fn store(&self) -> &T {
        &self.store
    }

    /// Get a mutable reference to the underlying store
    pub fn store_mut(&mut self) -> &mut T {
        &mut self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InMemoryStore;

    #[test]
    fn test_add_user_message() {
        let store = InMemoryStore::new();
        let mut history = ConversationHistory::new(store);

        history.add_user_message("Hello".to_string());

        let messages = history.get_recent(10);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello");
        assert!(matches!(messages[0].role, Role::User));
    }

    #[test]
    fn test_add_assistant_message() {
        let store = InMemoryStore::new();
        let mut history = ConversationHistory::new(store);

        history.add_assistant_message("Hi there!".to_string());

        let messages = history.get_recent(10);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hi there!");
        assert!(matches!(messages[0].role, Role::Assistant));
    }

    #[test]
    fn test_add_system_message() {
        let store = InMemoryStore::new();
        let mut history = ConversationHistory::new(store);

        history.add_system_message("You are a helpful assistant".to_string());

        let messages = history.get_recent(10);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "You are a helpful assistant");
        assert!(matches!(messages[0].role, Role::System));
    }

    #[test]
    fn test_conversation_flow() {
        let store = InMemoryStore::new();
        let mut history = ConversationHistory::new(store);

        history.add_system_message("You are a helpful assistant".to_string());
        history.add_user_message("What is 2+2?".to_string());
        history.add_assistant_message("2+2 equals 4".to_string());
        history.add_user_message("Thanks!".to_string());

        let messages = history.get_recent(10);
        assert_eq!(messages.len(), 4);

        // Verify order
        assert!(matches!(messages[0].role, Role::System));
        assert!(matches!(messages[1].role, Role::User));
        assert!(matches!(messages[2].role, Role::Assistant));
        assert!(matches!(messages[3].role, Role::User));
    }

    #[test]
    fn test_clear() {
        let store = InMemoryStore::new();
        let mut history = ConversationHistory::new(store);

        history.add_user_message("Test".to_string());
        assert_eq!(history.get_recent(10).len(), 1);

        history.clear();
        assert_eq!(history.get_recent(10).len(), 0);
    }

    #[test]
    fn test_get_within_budget() {
        let store = InMemoryStore::new();
        let mut history = ConversationHistory::new(store);

        history.add_user_message("Short".to_string());
        history.add_assistant_message(
            "This is a much longer message with many more tokens".to_string(),
        );
        history.add_user_message("Another".to_string());

        let within_budget = history.get_within_budget(1000);
        assert_eq!(within_budget.len(), 3);

        let small_budget = history.get_within_budget(20);
        assert!(small_budget.len() > 0);
        assert!(small_budget.len() < 3);
    }
}
