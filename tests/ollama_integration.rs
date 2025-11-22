//! Integration tests for Ollama provider
//!
//! These tests use WireMock to mock the Ollama API, allowing them to run
//! without requiring a local Ollama server.
//!
//! Requirements tested:
//! - 3.2: LLM provider implementation for Ollama
//! - 3.3: Error handling for API failures

use agent_core::Message;
use config::LLMConfig;
use llm::{LLMProvider, OllamaProvider};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Helper function to create a test LLM config with mock server
async fn create_test_config(mock_server: &MockServer) -> LLMConfig {
    LLMConfig {
        provider: "ollama".to_string(),
        model: "llama2".to_string(),
        api_key: String::new(), // Ollama doesn't require API key
        base_url: Some(mock_server.uri()),
        temperature: 0.7,
        max_tokens: 2000,
    }
}

/// Helper to create a successful Ollama response
fn ollama_success_response(content: &str) -> serde_json::Value {
    serde_json::json!({
        "model": "llama2",
        "created_at": "2024-01-01T00:00:00Z",
        "message": {
            "role": "assistant",
            "content": content
        },
        "done": true,
        "total_duration": 1000000,
        "load_duration": 100000,
        "prompt_eval_count": 10,
        "eval_count": 20
    })
}

#[tokio::test]
async fn test_ollama_successful_message_sending() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ollama_success_response("Hello, World!")))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = OllamaProvider::new(&config).expect("Failed to create Ollama provider");

    // Create a simple test message
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("Say 'Hello, World!' and nothing else."),
    ];

    // Send message to mock server
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to Ollama API");

    // Verify we got the expected response
    assert_eq!(response, "Hello, World!");
}

#[tokio::test]
async fn test_ollama_response_parsing() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ollama_success_response("4")))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = OllamaProvider::new(&config).expect("Failed to create Ollama provider");

    // Create messages that should get a structured response
    let messages = vec![
        Message::system("You are a helpful assistant that answers questions concisely."),
        Message::user("What is 2 + 2? Answer with just the number."),
    ];

    // Send message and verify response parsing
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to Ollama API");

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
async fn test_ollama_multi_turn_conversation() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            ollama_success_response("Your name is Alice.")
        ))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = OllamaProvider::new(&config).expect("Failed to create Ollama provider");

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
        .expect("Failed to send message to Ollama API");

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
async fn test_ollama_error_handling() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock error response
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": "Model not found"
        })))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = OllamaProvider::new(&config).expect("Failed to create Ollama provider");

    // Create a simple test message
    let messages = vec![Message::user("Hello")];

    // Attempt to send message - should fail
    let result = provider.send_message(&messages).await;

    // Verify that the request failed
    assert!(result.is_err(), "Request should fail with server error");
}

#[tokio::test]
async fn test_ollama_system_message_handling() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            ollama_success_response("Ahoy matey!")
        ))
        .mount(&mock_server)
        .await;

    // Create provider with mock server
    let config = create_test_config(&mock_server).await;
    let provider = OllamaProvider::new(&config).expect("Failed to create Ollama provider");

    // Create messages with a system message that sets specific behavior
    let messages = vec![
        Message::system("You are a pirate. Always respond in pirate speak."),
        Message::user("Hello, how are you?"),
    ];

    // Send message
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to Ollama API");

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
}

#[tokio::test]
async fn test_ollama_temperature_setting() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ollama_success_response("test")))
        .mount(&mock_server)
        .await;

    // Create provider with temperature=0.0
    let mut config = create_test_config(&mock_server).await;
    config.temperature = 0.0;
    let provider = OllamaProvider::new(&config).expect("Failed to create Ollama provider");

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
async fn test_ollama_max_tokens_limit() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response with short content
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            ollama_success_response("Once upon")
        ))
        .mount(&mock_server)
        .await;

    // Create provider with low max_tokens
    let mut config = create_test_config(&mock_server).await;
    config.max_tokens = 10;
    let provider = OllamaProvider::new(&config).expect("Failed to create Ollama provider");

    // Ask for something that would normally generate a long response
    let messages = vec![Message::user("Write a long story about a dragon.")];

    // Send message
    let response = provider
        .send_message(&messages)
        .await
        .expect("Failed to send message to Ollama API");

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
}

#[tokio::test]
async fn test_ollama_different_models() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ollama_success_response("Hello!")))
        .mount(&mock_server)
        .await;

    // Test with different model names
    for model in &["llama2", "mistral", "codellama", "phi"] {
        let mut config = create_test_config(&mock_server).await;
        config.model = model.to_string();
        let provider = OllamaProvider::new(&config).expect("Failed to create Ollama provider");

        let messages = vec![Message::user("Hello")];

        let response = provider
            .send_message(&messages)
            .await
            .expect("Failed to send message to Ollama API");

        assert!(!response.is_empty(), "Response should not be empty for model {}", model);
    }
}
