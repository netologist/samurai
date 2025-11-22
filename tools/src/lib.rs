//! Tools crate for the AI Agent Framework
//!
//! This crate provides the tool system that enables agents to perform external actions
//! beyond text generation. Tools can include calculations, file operations, web searches,
//! and any other capability that can be invoked programmatically.
//!
//! # Core Concepts
//!
//! - **Tool**: A trait defining the interface for all tools
//! - **ToolRegistry**: A registry for managing and retrieving available tools
//! - **ToolInfo**: Metadata about a tool for display and planning
//!
//! # Example
//!
//! ```rust
//! use tools::{ToolRegistry, Calculator, FileReader, WebSearchStub};
//!
//! let mut registry = ToolRegistry::new();
//! registry.register(Box::new(Calculator::new()));
//! registry.register(Box::new(FileReader::new()));
//! registry.register(Box::new(WebSearchStub::new()));
//!
//! // List all available tools
//! let tools = registry.list_tools();
//! for tool in tools {
//!     println!("{}: {}", tool.name, tool.description);
//! }
//!
//! // Get a specific tool
//! if let Some(calc) = registry.get("calculator") {
//!     // Use the calculator tool
//! }
//! ```

mod calculator;
mod file_reader;
mod registry;
mod tool;
mod web_search;

// Re-export public types and traits
pub use calculator::Calculator;
pub use file_reader::FileReader;
pub use registry::ToolRegistry;
pub use tool::{Tool, ToolInfo};
pub use web_search::WebSearchStub;
