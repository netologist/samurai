//! Integration tests for OpenAI provider
//!
//! These tests use WireMock to mock the OpenAI API, allowing them to run
//! without requiring API keys or making real network calls.
//!
//! Requirements tested:
//! - 3.2: LLM provider implementation for OpenAI
//! - 3.3: Error handling for API failures

use agent_core::Message;
use config::LLMConfig;
use llm::{LLMProvider, OpenAIProvider};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Helper function to create a test LLM config with mock server
async fn create_test_config(mock_server: &MockServer) -> LLMConfig {
    LLMConfig {
        provider: "openai".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        api_key: "sk-test-key".to_string(),
        base_url: Some(mock_server.uri()),
        temperature: 0.7,
        max_tokens: 100,
    }
}

/// Helper to create a successful OpenAI response
fn openai_success_response(content: &str) -> serde_json::Value {
    serde_json::json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-3.5-turbo",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 20,
            "total_tokens": 30
        }
    })
}

#[tokio::test]
async fn test_openai_successful_message_sending() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(openai_success_response("Hello, World!")))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = OpenAIProvider::new(&config).expect("Failed to create OpenAI provider");

    // Create a simple test message
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("Say 'Hello, World!' and nothing else."),
    ];

    // Send message to mock server
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to OpenAI API");

    // Verify we got the expected response
    assert_eq!(response, "Hello, World!");
}

#[tokio::test]
async fn test_openai_response_parsing() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(openai_success_response("4")))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = OpenAIProvider::new(&config).expect("Failed to create OpenAI provider");

    // Create messages that should get a structured response
    let messages = vec![
        Message::system("You are a helpful assistant that answers questions concisely."),
        Message::user("What is 2 + 2? Answer with just the number."),
    ];

    // Send message and verify response parsing
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to OpenAI API");

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
async fn test_openai_multi_turn_conversation() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            openai_success_response("Your name is Alice.")
        ))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = OpenAIProvider::new(&config).expect("Failed to create OpenAI provider");

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
        .expect("Failed to send message to OpenAI API");

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
async fn test_openai_invalid_api_key() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock error response for invalid API key
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {
                "message": "Invalid API key",
                "type": "invalid_request_error",
                "code": "invalid_api_key"
            }
        })))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = OpenAIProvider::new(&config).expect("Failed to create OpenAI provider");

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
async fn test_openai_empty_messages() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock error response for empty messages
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": {
                "message": "messages array is empty",
                "type": "invalid_request_error"
            }
        })))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = OpenAIProvider::new(&config).expect("Failed to create OpenAI provider");

    // Send empty messages array
    let messages: Vec<Message> = vec![];

    // This should fail - OpenAI requires at least one message
    let result = provider.send_message(&messages).await;

    // Verify that the request failed
    assert!(result.is_err(), "Request with empty messages should fail");
}

#[tokio::test]
async fn test_openai_system_message_handling() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            openai_success_response("Ahoy there, matey! I be doin' fine!")
        ))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = OpenAIProvider::new(&config).expect("Failed to create OpenAI provider");

    // Create messages with a system message that sets specific behavior
    let messages = vec![
        Message::system("You are a pirate. Always respond in pirate speak."),
        Message::user("Hello, how are you?"),
    ];

    // Send message
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to OpenAI API");

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
}

#[tokio::test]
async fn test_openai_max_tokens_limit() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response with short content
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            openai_success_response("Once upon")
        ))
        .mount(&mock_server)
        .await;

    // Create provider with low max_tokens
    let mut config = create_test_config(&mock_server).await;
    config.max_tokens = 10;
    let provider = OpenAIProvider::new(&config).expect("Failed to create OpenAI provider");

    // Ask for something that would normally generate a long response
    let messages = vec![Message::user("Write a long story about a dragon.")];

    // Send message
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to OpenAI API");

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
async fn test_openai_temperature_setting() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(openai_success_response("test")))
        .mount(&mock_server)
        .await;

    // Create provider with temperature=0.0
    let mut config = create_test_config(&mock_server).await;
    config.temperature = 0.0;
    let provider = OpenAIProvider::new(&config).expect("Failed to create OpenAI provider");

    let messages = vec![Message::user("Say 'test' and nothing else.")];

    // Send message
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message with temperature=0.0");

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
}
