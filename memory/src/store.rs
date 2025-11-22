//! Memory store trait definition.
//!
//! This module defines the `MemoryStore` trait that provides a unified interface
//! for different storage backends (in-memory, file-based, database, etc.).

use agent_core::Message;

/// Trait for different memory storage backends
///
/// Implementations of this trait provide storage and retrieval of conversation messages
/// with support for both count-based and token-based retrieval strategies.
pub trait MemoryStore: Send + Sync {
    /// Add a message to the memory store
    fn add_message(&mut self, message: Message);

    /// Get the most recent N messages in chronological order
    fn get_recent(&self, limit: usize) -> Vec<Message>;

    /// Get messages that fit within a token budget, in chronological order
    fn get_within_budget(&self, token_budget: usize) -> Vec<Message>;

    /// Clear all messages from the store
    fn clear(&mut self);
}
