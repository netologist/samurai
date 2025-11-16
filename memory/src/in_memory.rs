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
    fn test_get_recent_zero_limit() {
        let mut store = InMemoryStore::new();
        
        let msg = Message {
            role: Role::User,
            content: "Test message".to_string(),
            timestamp: Utc::now(),
        };
        
        store.add_message(msg);
        
        let recent = store.get_recent(0);
        assert_eq!(recent.len(), 0);
    }
    
    #[test]
    fn test_get_recent_empty_store() {
        let store = InMemoryStore::new();
        
        let recent = store.get_recent(10);
        assert_eq!(recent.len(), 0);
    }
    
    #[test]
    fn test_get_recent_all_messages() {
        let mut store = InMemoryStore::new();
        
        for i in 0..5 {
            let msg = Message {
                role: Role::User,
                content: format!("Message {}", i),
                timestamp: Utc::now(),
            };
            store.add_message(msg);
        }
        
        let recent = store.get_recent(5);
        assert_eq!(recent.len(), 5);
        assert_eq!(recent[0].content, "Message 0");
        assert_eq!(recent[4].content, "Message 4");
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
    fn test_get_within_budget_zero() {
        let mut store = InMemoryStore::new();
        
        let msg = Message {
            role: Role::User,
            content: "Test message".to_string(),
            timestamp: Utc::now(),
        };
        
        store.add_message(msg);
        
        let within_budget = store.get_within_budget(0);
        assert_eq!(within_budget.len(), 0);
    }
    
    #[test]
    fn test_get_within_budget_empty_store() {
        let store = InMemoryStore::new();
        
        let within_budget = store.get_within_budget(100);
        assert_eq!(within_budget.len(), 0);
    }
    
    #[test]
    fn test_get_within_budget_exact_fit() {
        let mut store = InMemoryStore::new();
        
        let msg1 = Message {
            role: Role::User,
            content: "First".to_string(),
            timestamp: Utc::now(),
        };
        let msg2 = Message {
            role: Role::User,
            content: "Second".to_string(),
            timestamp: Utc::now(),
        };
        
        store.add_message(msg1.clone());
        store.add_message(msg2.clone());
        
        // Calculate exact token count for both messages
        let token_count = count_tokens(&msg1) + count_tokens(&msg2);
        
        let within_budget = store.get_within_budget(token_count);
        assert_eq!(within_budget.len(), 2);
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
    
    #[test]
    fn test_clear_multiple_times() {
        let mut store = InMemoryStore::new();
        
        let msg = Message {
            role: Role::User,
            content: "Test message".to_string(),
            timestamp: Utc::now(),
        };
        
        store.add_message(msg.clone());
        store.clear();
        store.clear(); // Clear again on empty store
        
        assert_eq!(store.get_recent(10).len(), 0);
        
        // Add message after clearing
        store.add_message(msg);
        assert_eq!(store.get_recent(10).len(), 1);
    }
    
    #[test]
    fn test_message_storage_order() {
        let mut store = InMemoryStore::new();
        
        for i in 0..10 {
            let msg = Message {
                role: Role::User,
                content: format!("Message {}", i),
                timestamp: Utc::now(),
            };
            store.add_message(msg);
        }
        
        let all_messages = store.get_recent(10);
        
        // Verify messages are in chronological order
        for (i, msg) in all_messages.iter().enumerate() {
            assert_eq!(msg.content, format!("Message {}", i));
        }
    }
    
    #[test]
    fn test_different_roles() {
        let mut store = InMemoryStore::new();
        
        let system_msg = Message {
            role: Role::System,
            content: "System message".to_string(),
            timestamp: Utc::now(),
        };
        let user_msg = Message {
            role: Role::User,
            content: "User message".to_string(),
            timestamp: Utc::now(),
        };
        let assistant_msg = Message {
            role: Role::Assistant,
            content: "Assistant message".to_string(),
            timestamp: Utc::now(),
        };
        
        store.add_message(system_msg.clone());
        store.add_message(user_msg.clone());
        store.add_message(assistant_msg.clone());
        
        let recent = store.get_recent(3);
        assert_eq!(recent.len(), 3);
        assert!(matches!(recent[0].role, Role::System));
        assert!(matches!(recent[1].role, Role::User));
        assert!(matches!(recent[2].role, Role::Assistant));
    }
}
