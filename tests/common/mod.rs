//! Common test utilities and fixtures for integration tests
//!
//! This module provides shared test infrastructure including:
//! - MockLLM: A mock LLM provider for testing without real API calls
//! - MockMemoryStore: A simple in-memory store for testing
//! - Test fixtures: Common test scenarios and data

use agent_core::{Message, Result};
use async_trait::async_trait;
use llm::LLMProvider;
use memory::MemoryStore;
use std::sync::{Arc, Mutex};

/// Mock LLM provider that returns predefined responses
///
/// This mock allows testing the agent flow without making real API calls.
/// It tracks how many times it's been called and returns responses from a list.
///
/// # Example
///
/// ```
/// use tests::common::MockLLM;
///
/// let mock = MockLLM::new(vec![
///     "First response".to_string(),
///     "Second response".to_string(),
/// ]);
///
/// // First call returns "First response"
/// // Second call returns "Second response"
/// ```
pub struct MockLLM {
    responses: Vec<String>,
    call_count: Arc<Mutex<usize>>,
}

impl MockLLM {
    /// Creates a new MockLLM with predefined responses
    ///
    /// # Arguments
    ///
    /// * `responses` - A vector of responses to return in sequence
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Creates a MockLLM that returns a single response repeatedly
    pub fn with_single_response(response: String) -> Self {
        Self::new(vec![response])
    }

    /// Returns the number of times send_message was called
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    /// Resets the call count to zero
    #[allow(dead_code)]
    pub fn reset(&self) {
        *self.call_count.lock().unwrap() = 0;
    }
}

#[async_trait]
impl LLMProvider for MockLLM {
    async fn send_message(&self, _messages: &[Message]) -> Result<String> {
        let mut count = self.call_count.lock().unwrap();

        // If we have responses, cycle through them
        if !self.responses.is_empty() {
            let response = self.responses[*count % self.responses.len()].clone();
            *count += 1;
            Ok(response)
        } else {
            *count += 1;
            Err(agent_core::AgentError::LLMProvider(
                "MockLLM has no responses configured".to_string(),
            ))
        }
    }
}

/// Mock memory store for testing
///
/// Simple in-memory implementation that stores messages in a vector.
/// This is useful for testing without needing the full InMemoryStore implementation.
pub struct MockMemoryStore {
    messages: Vec<Message>,
}

impl MockMemoryStore {
    /// Creates a new empty MockMemoryStore
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Creates a MockMemoryStore with initial messages
    pub fn with_messages(messages: Vec<Message>) -> Self {
        Self { messages }
    }

    /// Returns all stored messages
    pub fn all_messages(&self) -> &[Message] {
        &self.messages
    }

    /// Returns the number of stored messages
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Returns true if no messages are stored
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl Default for MockMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore for MockMemoryStore {
    fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    fn get_recent(&self, limit: usize) -> Vec<Message> {
        self.messages
            .iter()
            .rev()
            .take(limit)
            .rev()
            .cloned()
            .collect()
    }

    fn get_within_budget(&self, _token_budget: usize) -> Vec<Message> {
        // For testing, just return all messages
        // A more sophisticated implementation could use actual token counting
        self.messages.clone()
    }

    fn clear(&mut self) {
        self.messages.clear();
    }
}

/// Test fixtures for common scenarios
pub mod fixtures {
    use super::*;

    /// Creates a simple calculator plan JSON for testing
    ///
    /// This plan includes a single calculator tool call to add two numbers.
    pub fn simple_calculator_plan() -> String {
        r#"{
            "reasoning": "To calculate 15 + 27, I'll use the calculator tool with the add operation",
            "steps": [
                {
                    "type": "tool_call",
                    "tool_name": "calculator",
                    "parameters": {
                        "operation": "add",
                        "a": 15,
                        "b": 27
                    }
                },
                {
                    "type": "reasoning",
                    "text": "The calculator returned 42 as the sum"
                },
                {
                    "type": "response",
                    "text": "15 + 27 equals 42"
                }
            ]
        }"#.to_string()
    }

    /// Creates a multi-step calculator plan JSON for testing
    ///
    /// This plan includes multiple calculator operations: (10 + 5) * 2
    #[allow(dead_code)]
    pub fn multi_step_calculator_plan() -> String {
        r#"{
            "reasoning": "To calculate (10 + 5) * 2, I'll first add 10 and 5, then multiply the result by 2",
            "steps": [
                {
                    "type": "tool_call",
                    "tool_name": "calculator",
                    "parameters": {
                        "operation": "add",
                        "a": 10,
                        "b": 5
                    }
                },
                {
                    "type": "reasoning",
                    "text": "First calculation gives us 15"
                },
                {
                    "type": "tool_call",
                    "tool_name": "calculator",
                    "parameters": {
                        "operation": "multiply",
                        "a": 15,
                        "b": 2
                    }
                },
                {
                    "type": "reasoning",
                    "text": "Second calculation gives us 30"
                },
                {
                    "type": "response",
                    "text": "The result of (10 + 5) * 2 is 30"
                }
            ]
        }"#.to_string()
    }

    /// Creates a plan with no tool calls (reasoning only)
    #[allow(dead_code)]
    pub fn reasoning_only_plan() -> String {
        r#"{
            "reasoning": "This is a simple question that doesn't require tools",
            "steps": [
                {
                    "type": "reasoning",
                    "text": "The capital of France is a well-known fact"
                },
                {
                    "type": "response",
                    "text": "The capital of France is Paris"
                }
            ]
        }"#
        .to_string()
    }

    /// Creates a plan with an invalid tool reference
    #[allow(dead_code)]
    pub fn invalid_tool_plan() -> String {
        r#"{
            "reasoning": "I'll use a non-existent tool",
            "steps": [
                {
                    "type": "tool_call",
                    "tool_name": "nonexistent_tool",
                    "parameters": {}
                }
            ]
        }"#
        .to_string()
    }

    /// Creates a plan with file reader tool call
    #[allow(dead_code)]
    pub fn file_reader_plan(file_path: &str) -> String {
        format!(
            r#"{{
                "reasoning": "I'll read the file to get its contents",
                "steps": [
                    {{
                        "type": "tool_call",
                        "tool_name": "file_reader",
                        "parameters": {{
                            "file_path": "{}"
                        }}
                    }},
                    {{
                        "type": "response",
                        "text": "Here are the file contents"
                    }}
                ]
            }}"#,
            file_path
        )
    }

    /// Creates a plan with multiple tool calls exceeding rate limits
    #[allow(dead_code)]
    pub fn rate_limit_exceeding_plan(num_calls: usize) -> String {
        let mut steps = Vec::new();

        for i in 0..num_calls {
            steps.push(format!(
                r#"{{
                    "type": "tool_call",
                    "tool_name": "calculator",
                    "parameters": {{
                        "operation": "add",
                        "a": {},
                        "b": 1
                    }}
                }}"#,
                i
            ));
        }

        format!(
            r#"{{
                "reasoning": "Performing {} calculations",
                "steps": [{}]
            }}"#,
            num_calls,
            steps.join(",")
        )
    }

    /// Creates a sample conversation history
    pub fn sample_conversation() -> Vec<Message> {
        vec![
            Message::system("You are a helpful assistant."),
            Message::user("Hello, my name is Alice."),
            Message::assistant("Hello Alice! Nice to meet you. How can I help you today?"),
            Message::user("What is 2 + 2?"),
            Message::assistant("2 + 2 equals 4."),
        ]
    }

    /// Creates a long conversation for testing memory limits
    #[allow(dead_code)]
    pub fn long_conversation(num_turns: usize) -> Vec<Message> {
        let mut messages = vec![Message::system("You are a helpful assistant.")];

        for i in 0..num_turns {
            messages.push(Message::user(format!("Question {}", i)));
            messages.push(Message::assistant(format!("Answer {}", i)));
        }

        messages
    }
}

/// Helper functions for test assertions
pub mod assertions {
    use agent_core::AgentError;

    /// Asserts that an error is an LLMProvider error
    #[allow(dead_code)]
    pub fn assert_llm_provider_error(result: &Result<(), AgentError>) {
        assert!(result.is_err(), "Expected LLMProvider error");
        if let Err(AgentError::LLMProvider(_)) = result {
            // Success
        } else {
            panic!("Expected LLMProvider error, got: {:?}", result);
        }
    }

    /// Asserts that an error is a GuardrailViolation error
    #[allow(dead_code)]
    pub fn assert_guardrail_violation(result: &Result<(), AgentError>) {
        assert!(result.is_err(), "Expected GuardrailViolation error");
        if let Err(AgentError::GuardrailViolation(_)) = result {
            // Success
        } else {
            panic!("Expected GuardrailViolation error, got: {:?}", result);
        }
    }

    /// Asserts that an error is a ToolExecution error
    #[allow(dead_code)]
    pub fn assert_tool_execution_error(result: &Result<(), AgentError>) {
        assert!(result.is_err(), "Expected ToolExecution error");
        if let Err(AgentError::ToolExecution { .. }) = result {
            // Success
        } else {
            panic!("Expected ToolExecution error, got: {:?}", result);
        }
    }

    /// Asserts that an error is a Planning error
    #[allow(dead_code)]
    pub fn assert_planning_error(result: &Result<(), AgentError>) {
        assert!(result.is_err(), "Expected Planning error");
        if let Err(AgentError::Planning(_)) = result {
            // Success
        } else {
            panic!("Expected Planning error, got: {:?}", result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_llm_single_response() {
        let mock = MockLLM::with_single_response("test response".to_string());

        let messages = vec![Message::user("test")];
        let response = mock.send_message(&messages).await.unwrap();

        assert_eq!(response, "test response");
        assert_eq!(mock.call_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_llm_multiple_responses() {
        let mock = MockLLM::new(vec!["first".to_string(), "second".to_string()]);

        let messages = vec![Message::user("test")];

        let response1 = mock.send_message(&messages).await.unwrap();
        assert_eq!(response1, "first");

        let response2 = mock.send_message(&messages).await.unwrap();
        assert_eq!(response2, "second");

        assert_eq!(mock.call_count(), 2);
    }

    #[tokio::test]
    async fn test_mock_llm_cycles_responses() {
        let mock = MockLLM::new(vec!["response".to_string()]);

        let messages = vec![Message::user("test")];

        // Should cycle back to first response
        mock.send_message(&messages).await.unwrap();
        let response = mock.send_message(&messages).await.unwrap();

        assert_eq!(response, "response");
        assert_eq!(mock.call_count(), 2);
    }

    #[test]
    fn test_mock_memory_store() {
        let mut store = MockMemoryStore::new();

        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        store.add_message(Message::user("test"));

        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);

        let messages = store.get_recent(10);
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_mock_memory_store_with_initial_messages() {
        let initial = vec![Message::user("first"), Message::assistant("second")];

        let store = MockMemoryStore::with_messages(initial);

        assert_eq!(store.len(), 2);
        assert_eq!(store.all_messages().len(), 2);
    }

    #[test]
    fn test_fixtures_simple_calculator_plan() {
        let plan = fixtures::simple_calculator_plan();
        assert!(plan.contains("calculator"));
        assert!(plan.contains("add"));
    }

    #[test]
    fn test_fixtures_sample_conversation() {
        let conversation = fixtures::sample_conversation();
        assert_eq!(conversation.len(), 5);
        assert!(conversation[0].content.contains("helpful assistant"));
    }
}
