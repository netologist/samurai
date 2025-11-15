//! In-memory implementation of the MemoryStore trait.
//!
//! This module provides a simple Vec-based implementation suitable for MVP
//! and testing. Messages are stored in memory and lost when the process ends.

use agent_core::Message;
use crate::{MemoryStore, count_tokens};

/// In-memory implementation of MemoryStore using a Vec
///
/// This implementation stores all messages in a Vec and provides efficient
/// retrieval operations. Messages are stored in chronological order.
///
/// # Examples
///
/// ```
/// use memory::{InMemoryStore, MemoryStore};
/// use agent_core::{Message, Role};
/// use chrono::Utc;
///
/// let mut store = InMemoryStore::new();
/// let message = Message {
///     role: Role::User,
///     content: "Hello".to_string(),
///     timestamp: Utc::now(),
/// };
/// store.add_message(message);
///
/// let recent = store.get_recent(10);
/// assert_eq!(recent.len(), 1);
/// ```
pub struct InMemoryStore {
    messages: Vec<Message>,
}

impl InMemoryStore {
    /// Create a new empty InMemoryStore
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore for InMemoryStore {
    fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
    
    fn get_recent(&self, limit: usize) -> Vec<Message> {
        // Get the last N messages and return them in chronological order
        self.messages
            .iter()
            .rev()
            .take(limit)
            .rev()
            .cloned()
            .collect()
    }
    
    fn get_within_budget(&self, token_budget: usize) -> Vec<Message> {
        let mut result = Vec::new();
        let mut total_tokens = 0;
        
        // Iterate from most recent to oldest
        for message in self.messages.iter().rev() {
            let message_tokens = count_tokens(message);
            
            // Check if adding this message would exceed the budget
            if total_tokens + message_tokens > token_budget {
                break;
            }
            
            total_tokens += message_tokens;
            result.push(message.clone());
        }
        
        // Reverse to return messages in chronological order
        result.reverse();
        result
    }
    
    fn clear(&mut self) {
        self.messages.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{Message, Role};
    use chrono::Utc;
    
    #[test]
    fn test_add_and_get_recent() {
        let mut store = InMemoryStore::new();
        
        let msg1 = Message {
            role: Role::User,
            content: "First message".to_string(),
            timestamp: Utc::now(),
        };
        let msg2 = Message {
            role: Role::Assistant,
            content: "Second message".to_string(),
            timestamp: Utc::now(),
        };
        let msg3 = Message {
            role: Role::User,
            content: "Third message".to_string(),
            timestamp: Utc::now(),
        };
        
        store.add_message(msg1.clone());
        store.add_message(msg2.clone());
        store.add_message(msg3.clone());
        
        let recent = store.get_recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].content, "Second message");
        assert_eq!(recent[1].content, "Third message");
    }
    
    #[test]
    fn test_get_recent_more_than_available() {
        let mut store = InMemoryStore::new();
        
        let msg = Message {
            role: Role::User,
            content: "Only message".to_string(),
            timestamp: Utc::now(),
        };
        
        store.add_message(msg.clone());
        
        let recent = store.get_recent(10);
        assert_eq!(recent.len(), 1);
    }
    
    #[test]
    fn test_get_within_budget() {
        let mut store = InMemoryStore::new();
        
        let msg1 = Message {
            role: Role::User,
            content: "Short".to_string(),
            timestamp: Utc::now(),
        };
        let msg2 = Message {
            role: Role::Assistant,
            content: "This is a longer message with more tokens".to_string(),
            timestamp: Utc::now(),
        };
        let msg3 = Message {
            role: Role::User,
            content: "Another message".to_string(),
            timestamp: Utc::now(),
        };
        
        store.add_message(msg1.clone());
        store.add_message(msg2.clone());
        store.add_message(msg3.clone());
        
        // Get messages within a small budget (should only get the most recent)
        let within_budget = store.get_within_budget(20);
        assert!(within_budget.len() > 0);
        
        // Get messages within a larger budget (should get more messages)
        let within_larger_budget = store.get_within_budget(1000);
        assert_eq!(within_larger_budget.len(), 3);
        
        // Verify chronological order
        assert_eq!(within_larger_budget[0].content, "Short");
        assert_eq!(within_larger_budget[2].content, "Another message");
    }
    
    #[test]
    fn test_clear() {
        let mut store = InMemoryStore::new();
        
        let msg = Message {
            role: Role::User,
            content: "Test message".to_string(),
            timestamp: Utc::now(),
        };
        
        store.add_message(msg);
        assert_eq!(store.get_recent(10).len(), 1);
        
        store.clear();
        assert_eq!(store.get_recent(10).len(), 0);
    }
}
