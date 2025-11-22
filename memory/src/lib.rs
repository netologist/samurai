//! Memory system for AI agent conversation storage and retrieval.
//!
//! This crate provides a flexible memory system for storing and retrieving conversation
//! history with token-aware context management. It includes:
//!
//! - `MemoryStore` trait for different storage backends
//! - `InMemoryStore` implementation using Vec for MVP
//! - Token counting functionality using tiktoken-rs for OpenAI models
//! - `ConversationHistory` wrapper with convenience methods
//!
//! # Examples
//!
//! ```
//! use memory::{InMemoryStore, ConversationHistory, MemoryStore};
//!
//! let store = InMemoryStore::new();
//! let mut history = ConversationHistory::new(store);
//!
//! history.add_user_message("Hello!".to_string());
//! history.add_assistant_message("Hi there!".to_string());
//!
//! let recent = history.get_recent(10);
//! assert_eq!(recent.len(), 2);
//! ```

mod history;
mod in_memory;
mod store;
mod token_counter;

pub use history::ConversationHistory;
pub use in_memory::InMemoryStore;
pub use store::MemoryStore;
pub use token_counter::count_tokens;
