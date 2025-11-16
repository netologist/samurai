//! Integration tests for OpenAI provider
//! 
//! These tests call the real OpenAI API and are marked with #[ignore] to prevent
//! them from running in CI or during regular test runs. To run these tests:
//! 
//! ```bash
//! export OPENAI_API_KEY="your-api-key-here"
//! cargo test --test openai_integration -- --ignored
//! ```
//! 
//! Requirements tested:
//! - 3.2: LLM provider implementation for OpenAI
//! - 3.3: Error handling for API failures

use agent_core::Message;
use config::LLMConfig;
use llm::{LLMProvider, OpenAIProvider};

/// Helper function to create a test LLM config
/// 
/// Reads the API key from the OPENAI_API_KEY environment variable
fn create_test_config() -> LLMConfig {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable must be set for integration tests");
    
    LLMConfig {
        provider: "openai".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        api_key,
        temperature: 0.7,
        max_tokens: 100,
    }
}

/// Helper function to create a test config with an invalid API key
fn create_invalid_config() -> LLMConfig {
    LLMConfig {
        provider: "openai".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        api_key: "sk-invalid-key-for-testing".to_string(),
        temperature: 0.7,
        max_tokens: 100,
    }
}

#[tokio::test]
#[ignore] // Requires OPENAI_API_KEY and makes real API call
async fn test_openai_successful_message_sending() {
    // Create provider with valid configuration
    let config = create_test_config();
    let provider = OpenAIProvider::new(&config)
        .expect("Failed to create OpenAI provider");
    
    // Create a simple test message
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("Say 'Hello, World!' and nothing else."),
    ];
    
    // Send message to OpenAI API
    let response = provider.send_message(&messages).await
        .expect("Failed to send message to OpenAI API");
    
    // Verify we got a non-empty response
    assert!(!response.is_empty(), "Response should not be empty");
    
    // Verify the response contains expected content
    // Note: We can't guarantee exact response due to LLM variability,
    // but we can check it's reasonable
    assert!(response.len() > 5, "Response should be a reasonable length");
    
    println!("OpenAI response: {}", response);
}

#[tokio::test]
#[ignore] // Requires OPENAI_API_KEY and makes real API call
async fn test_openai_response_parsing() {
    // Create provider with valid configuration
    let config = create_test_config();
    let provider = OpenAIProvider::new(&config)
        .expect("Failed to create OpenAI provider");
    
    // Create messages that should get a structured response
    let messages = vec![
        Message::system("You are a helpful assistant that answers questions concisely."),
        Message::user("What is 2 + 2? Answer with just the number."),
    ];
    
    // Send message and verify response parsing
    let response = provider.send_message(&messages).await
        .expect("Failed to send message to OpenAI API");
    
    // Verify response is parsed correctly as a string
    assert!(!response.is_empty(), "Response should not be empty");
    
    // The response should contain the number 4
    assert!(
        response.contains("4"),
        "Response should contain the answer '4', got: {}",
        response
    );
    
    println!("Parsed response: {}", response);
}

#[tokio::test]
#[ignore] // Requires OPENAI_API_KEY and makes real API call
async fn test_openai_multi_turn_conversation() {
    // Test that the provider can handle multi-turn conversations
    let config = create_test_config();
    let provider = OpenAIProvider::new(&config)
        .expect("Failed to create OpenAI provider");
    
    // Create a multi-turn conversation
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("My name is Alice."),
        Message::assistant("Hello Alice! Nice to meet you."),
        Message::user("What is my name?"),
    ];
    
    // Send message
    let response = provider.send_message(&messages).await
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
    
    println!("Multi-turn response: {}", response);
}

#[tokio::test]
#[ignore] // Makes real API call with invalid key
async fn test_openai_invalid_api_key() {
    // Create provider with invalid API key
    let config = create_invalid_config();
    let provider = OpenAIProvider::new(&config)
        .expect("Failed to create OpenAI provider");
    
    // Create a simple test message
    let messages = vec![
        Message::user("Hello"),
    ];
    
    // Attempt to send message - should fail with authentication error
    let result = provider.send_message(&messages).await;
    
    // Verify that the request failed
    assert!(result.is_err(), "Request with invalid API key should fail");
    
    // Verify the error is an LLMProvider error with authentication message
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    
    assert!(
        error_msg.contains("LLM provider") || error_msg.contains("authentication") || error_msg.contains("Invalid API key"),
        "Error should indicate authentication failure, got: {}",
        error_msg
    );
    
    println!("Expected authentication error: {}", error_msg);
}

#[tokio::test]
#[ignore] // Makes real API call
async fn test_openai_empty_messages() {
    // Test error handling when no messages are provided
    let config = create_test_config();
    let provider = OpenAIProvider::new(&config)
        .expect("Failed to create OpenAI provider");
    
    // Send empty messages array
    let messages: Vec<Message> = vec![];
    
    // This should fail - OpenAI requires at least one message
    let result = provider.send_message(&messages).await;
    
    // Verify that the request failed
    assert!(result.is_err(), "Request with empty messages should fail");
    
    let error = result.unwrap_err();
    println!("Expected error for empty messages: {}", error);
}

#[tokio::test]
#[ignore] // Makes real API call
async fn test_openai_system_message_handling() {
    // Test that system messages are properly handled
    let config = create_test_config();
    let provider = OpenAIProvider::new(&config)
        .expect("Failed to create OpenAI provider");
    
    // Create messages with a system message that sets specific behavior
    let messages = vec![
        Message::system("You are a pirate. Always respond in pirate speak."),
        Message::user("Hello, how are you?"),
    ];
    
    // Send message
    let response = provider.send_message(&messages).await
        .expect("Failed to send message to OpenAI API");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    
    // The response should reflect the system message instruction
    // (though we can't guarantee exact pirate speak, the system message should influence it)
    println!("Pirate response: {}", response);
}

#[tokio::test]
#[ignore] // Makes real API call
async fn test_openai_max_tokens_limit() {
    // Test that max_tokens configuration is respected
    let mut config = create_test_config();
    config.max_tokens = 10; // Very small limit
    
    let provider = OpenAIProvider::new(&config)
        .expect("Failed to create OpenAI provider");
    
    // Ask for something that would normally generate a long response
    let messages = vec![
        Message::user("Write a long story about a dragon."),
    ];
    
    // Send message
    let response = provider.send_message(&messages).await
        .expect("Failed to send message to OpenAI API");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    
    // The response should be relatively short due to max_tokens limit
    // Note: Token count != character count, but this gives us a rough check
    assert!(
        response.len() < 200,
        "Response should be short due to max_tokens=10, got {} characters",
        response.len()
    );
    
    println!("Limited response ({} chars): {}", response.len(), response);
}

#[tokio::test]
#[ignore] // Makes real API call
async fn test_openai_temperature_setting() {
    // Test that temperature configuration is accepted
    // Note: We can't easily verify the temperature is actually used,
    // but we can verify the request succeeds with different temperatures
    let mut config = create_test_config();
    config.temperature = 0.0; // Deterministic
    
    let provider = OpenAIProvider::new(&config)
        .expect("Failed to create OpenAI provider");
    
    let messages = vec![
        Message::user("Say 'test' and nothing else."),
    ];
    
    // Send message
    let response = provider.send_message(&messages).await
        .expect("Failed to send message with temperature=0.0");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    
    println!("Response with temperature=0.0: {}", response);
}
