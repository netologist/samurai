//! Type definitions for Anthropic API requests and responses.

use serde::{Deserialize, Serialize};

/// Anthropic API message format.
///
/// Represents a single message in the conversation with role and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    /// The role of the message sender ("user" or "assistant")
    pub role: String,
    /// The content of the message
    pub content: String,
}

/// Request structure for Anthropic Messages API.
///
/// Note: Anthropic separates system messages into a dedicated field
/// rather than including them in the messages array.
///
/// This structure is serialized to JSON and sent to the Anthropic API.
#[derive(Debug, Serialize)]
pub struct MessagesRequest {
    /// The model to use (e.g., "claude-3-sonnet-20240229")
    pub model: String,
    /// The conversation messages (user and assistant only)
    pub messages: Vec<AnthropicMessage>,
    /// Optional system message (sent separately from messages array)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Sampling temperature (0.0 to 1.0)
    pub temperature: f32,
    /// Maximum number of tokens to generate
    pub max_tokens: usize,
}

/// Response structure from Anthropic Messages API.
///
/// This structure is deserialized from the JSON response.
#[derive(Debug, Deserialize)]
pub struct MessagesResponse {
    /// Unique identifier for the message
    pub id: String,
    /// Response type (always "message")
    #[serde(rename = "type")]
    pub response_type: String,
    /// Role of the response (always "assistant")
    pub role: String,
    /// Array of content blocks (usually contains one text block)
    pub content: Vec<ContentBlock>,
    /// The model used for the response
    pub model: String,
    /// Reason why the model stopped generating
    pub stop_reason: Option<String>,
}

/// Content block in the Anthropic response.
///
/// Anthropic responses contain an array of content blocks, typically
/// with a single text block containing the generated response.
#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    /// Type of content block (e.g., "text")
    #[serde(rename = "type")]
    pub content_type: String,
    /// The text content
    pub text: String,
}
