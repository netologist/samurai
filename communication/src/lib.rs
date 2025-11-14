//! Communication layer for HTTP API interactions
//!
//! This crate provides HTTP client utilities with retry logic and error handling
//! for communicating with LLM provider APIs.
//!
//! # Features
//! - JSON POST requests with automatic serialization/deserialization
//! - Configurable timeouts
//! - Exponential backoff retry logic
//! - Proper error handling and conversion
//!
//! # Example
//! ```no_run
//! use communication::{ApiClient, with_retry};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize)]
//! struct Request {
//!     message: String,
//! }
//!
//! #[derive(Deserialize)]
//! struct Response {
//!     reply: String,
//! }
//!
//! # async fn example() -> agent_core::Result<()> {
//! let client = ApiClient::new();
//! let request = Request { message: "Hello".to_string() };
//!
//! // Simple request
//! let response: Response = client.post_json("https://api.example.com", &request).await?;
//!
//! // Request with retry
//! let response: Response = with_retry(
//!     || client.post_json("https://api.example.com", &request),
//!     3
//! ).await?;
//! # Ok(())
//! # }
//! ```

mod client;
mod retry;

pub use client::ApiClient;
pub use retry::with_retry;
