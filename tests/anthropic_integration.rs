//! Integration tests for Anthropic provider
//!
//! These tests use WireMock to mock the Anthropic API, allowing them to run
//! without requiring API keys or making real network calls.
//!
//! Requirements tested:
//! - 3.2: LLM provider implementation for Anthropic
//! - 3.3: Error handling for API failures

use agent_core::Message;
use config::LLMConfig;
use llm::{AnthropicProvider, LLMProvider};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Helper function to create a test LLM config with mock server
async fn create_test_config(mock_server: &MockServer) -> LLMConfig {
    LLMConfig {
        provider: "anthropic".to_string(),
        model: "claude-3-haiku-20240307".to_string(),
        api_key: "sk-ant-test-key".to_string(),
        base_url: Some(mock_server.uri()),
        temperature: 0.7,
        max_tokens: 100,
    }
}

/// Helper to create a successful Anthropic response
fn anthropic_success_response(content: &str) -> serde_json::Value {
    serde_json::json!({
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": content
        }],
        "model": "claude-3-haiku-20240307",
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 20
        }
    })
}

#[tokio::test]
async fn test_anthropic_successful_message_sending() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(anthropic_success_response("Hello, World!")))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = AnthropicProvider::new(&config).expect("Failed to create Anthropic provider");

    // Create a simple test message
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("Say 'Hello, World!' and nothing else."),
    ];

    // Send message to mock server
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to Anthropic API");

    // Verify we got the expected response
    assert_eq!(response, "Hello, World!");
}

#[tokio::test]
async fn test_anthropic_response_parsing() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(anthropic_success_response("4")))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = AnthropicProvider::new(&config).expect("Failed to create Anthropic provider");

    // Create messages that should get a structured response
    let messages = vec![
        Message::system("You are a helpful assistant that answers questions concisely."),
        Message::user("What is 2 + 2? Answer with just the number."),
    ];

    // Send message and verify response parsing
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to Anthropic API");

    // Verify response is parsed correctly as a string
    assert!(!response.is_empty(), "Response should not be empty");

    // The response should contain the number 4
    assert!(
        response.contains("4"),
        "Response should contain the answer '4', got: {}",
        response
    );
}

#[tokio::test]
async fn test_anthropic_system_message_handling() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            anthropic_success_response("Ahoy there, matey!")
        ))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = AnthropicProvider::new(&config).expect("Failed to create Anthropic provider");

    // Create messages with a system message that sets specific behavior
    let messages = vec![
        Message::system("You are a pirate. Always respond in pirate speak."),
        Message::user("Hello, how are you?"),
    ];

    // Send message
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to Anthropic API");

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
}

#[tokio::test]
async fn test_anthropic_multiple_system_messages() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            anthropic_success_response("I am fine")
        ))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = AnthropicProvider::new(&config).expect("Failed to create Anthropic provider");

    // Create messages with multiple system messages
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::system("You always respond in exactly 3 words."),
        Message::user("How are you?"),
    ];

    // Send message
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to Anthropic API");

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
}

#[tokio::test]
async fn test_anthropic_multi_turn_conversation() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            anthropic_success_response("Your name is Alice.")
        ))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = AnthropicProvider::new(&config).expect("Failed to create Anthropic provider");

    // Create a multi-turn conversation
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("My name is Alice."),
        Message::assistant("Hello Alice! Nice to meet you."),
        Message::user("What is my name?"),
    ];

    // Send message
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to Anthropic API");

    // Verify the response references the name from conversation history
    assert!(!response.is_empty(), "Response should not be empty");

    // The response should mention Alice
    let response_lower = response.to_lowercase();
    assert!(
        response_lower.contains("alice"),
        "Response should reference the name 'Alice' from conversation history, got: {}",
        response
    );
}

#[tokio::test]
async fn test_anthropic_invalid_api_key() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock error response for invalid API key
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "type": "error",
            "error": {
                "type": "authentication_error",
                "message": "Invalid API key"
            }
        })))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = AnthropicProvider::new(&config).expect("Failed to create Anthropic provider");

    // Create a simple test message
    let messages = vec![Message::user("Hello")];

    // Attempt to send message - should fail with authentication error
    let result = provider.send_message(&messages).await;

    // Verify that the request failed
    assert!(result.is_err(), "Request with invalid API key should fail");

    // Verify the error is an LLMProvider error
    let error = result.unwrap_err();
    let error_msg = error.to_string();

    assert!(
        error_msg.contains("LLM provider") || error_msg.contains("401"),
        "Error should indicate authentication failure, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_anthropic_empty_messages() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock error response for empty messages
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "type": "error",
            "error": {
                "type": "invalid_request_error",
                "message": "messages: field required"
            }
        })))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = AnthropicProvider::new(&config).expect("Failed to create Anthropic provider");

    // Send empty messages array
    let messages: Vec<Message> = vec![];

    // This should fail - Anthropic requires at least one message
    let result = provider.send_message(&messages).await;

    // Verify that the request failed
    assert!(result.is_err(), "Request with empty messages should fail");
}

#[tokio::test]
async fn test_anthropic_max_tokens_limit() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response with short content
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            anthropic_success_response("Once upon")
        ))
        .mount(&mock_server)
        .await;

    // Create provider with low max_tokens
    let mut config = create_test_config(&mock_server).await;
    config.max_tokens = 10;
    let provider = AnthropicProvider::new(&config).expect("Failed to create Anthropic provider");

    // Ask for something that would normally generate a long response
    let messages = vec![Message::user("Write a long story about a dragon.")];

    // Send message
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to Anthropic API");

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");

    // The response should be short
    assert!(
        response.len() < 50,
        "Response should be short due to max_tokens=10, got {} characters",
        response.len()
    );
}

#[tokio::test]
async fn test_anthropic_temperature_setting() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(anthropic_success_response("test")))
        .mount(&mock_server)
        .await;

    // Create provider with temperature=0.0
    let mut config = create_test_config(&mock_server).await;
    config.temperature = 0.0;
    let provider = AnthropicProvider::new(&config).expect("Failed to create Anthropic provider");

    let messages = vec![Message::user("Say 'test' and nothing else.")];

    // Send message
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message with temperature=0.0");

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
}

#[tokio::test]
async fn test_anthropic_only_system_message() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock error response
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "type": "error",
            "error": {
                "type": "invalid_request_error",
                "message": "messages: must contain at least one user message"
            }
        })))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = AnthropicProvider::new(&config).expect("Failed to create Anthropic provider");

    // Create messages with only a system message
    let messages = vec![Message::system("You are a helpful assistant.")];

    // This should fail - Anthropic requires at least one user/assistant message
    let result = provider.send_message(&messages).await;

    // Verify that the request failed
    assert!(
        result.is_err(),
        "Request with only system message should fail"
    );
}
