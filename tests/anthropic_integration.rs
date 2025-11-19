//! Integration tests for Anthropic provider
//! 
//! These tests call the real Anthropic API and are marked with #[ignore] to prevent
//! them from running in CI or during regular test runs. To run these tests:
//! 
//! ```bash
//! export ANTHROPIC_API_KEY="your-api-key-here"
//! cargo test --test anthropic_integration -- --ignored
//! ```
//! 
//! Requirements tested:
//! - 3.2: LLM provider implementation for Anthropic
//! - 3.3: Error handling for API failures

use agent_core::Message;
use config::LLMConfig;
use llm::{AnthropicProvider, LLMProvider};

/// Helper function to create a test LLM config
/// 
/// Reads the API key from the ANTHROPIC_API_KEY environment variable
fn create_test_config() -> LLMConfig {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set for integration tests");
    
    LLMConfig {
        provider: "anthropic".to_string(),
        model: "claude-3-haiku-20240307".to_string(),
        api_key,
        base_url: None,
        temperature: 0.7,
        max_tokens: 100,
    }
}

/// Helper function to create a test config with an invalid API key
fn create_invalid_config() -> LLMConfig {
    LLMConfig {
        provider: "anthropic".to_string(),
        model: "claude-3-haiku-20240307".to_string(),
        api_key: "sk-ant-invalid-key-for-testing".to_string(),
        base_url: None,
        temperature: 0.7,
        max_tokens: 100,
    }
}

#[tokio::test]
#[ignore] // Requires ANTHROPIC_API_KEY and makes real API call
async fn test_anthropic_successful_message_sending() {
    // Create provider with valid configuration
    let config = create_test_config();
    let provider = AnthropicProvider::new(&config)
        .expect("Failed to create Anthropic provider");
    
    // Create a simple test message
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("Say 'Hello, World!' and nothing else."),
    ];
    
    // Send message to Anthropic API
    let response = provider.send_message(&messages).await
        .expect("Failed to send message to Anthropic API");
    
    // Verify we got a non-empty response
    assert!(!response.is_empty(), "Response should not be empty");
    
    // Verify the response contains expected content
    // Note: We can't guarantee exact response due to LLM variability,
    // but we can check it's reasonable
    assert!(response.len() > 5, "Response should be a reasonable length");
    
    println!("Anthropic response: {}", response);
}

#[tokio::test]
#[ignore] // Requires ANTHROPIC_API_KEY and makes real API call
async fn test_anthropic_response_parsing() {
    // Create provider with valid configuration
    let config = create_test_config();
    let provider = AnthropicProvider::new(&config)
        .expect("Failed to create Anthropic provider");
    
    // Create messages that should get a structured response
    let messages = vec![
        Message::system("You are a helpful assistant that answers questions concisely."),
        Message::user("What is 2 + 2? Answer with just the number."),
    ];
    
    // Send message and verify response parsing
    let response = provider.send_message(&messages).await
        .expect("Failed to send message to Anthropic API");
    
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
#[ignore] // Requires ANTHROPIC_API_KEY and makes real API call
async fn test_anthropic_system_message_handling() {
    // Test that system messages are properly handled
    // This is particularly important for Anthropic as they use a separate system field
    let config = create_test_config();
    let provider = AnthropicProvider::new(&config)
        .expect("Failed to create Anthropic provider");
    
    // Create messages with a system message that sets specific behavior
    let messages = vec![
        Message::system("You are a pirate. Always respond in pirate speak."),
        Message::user("Hello, how are you?"),
    ];
    
    // Send message
    let response = provider.send_message(&messages).await
        .expect("Failed to send message to Anthropic API");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    
    // The response should reflect the system message instruction
    // (though we can't guarantee exact pirate speak, the system message should influence it)
    println!("Pirate response: {}", response);
}

#[tokio::test]
#[ignore] // Requires ANTHROPIC_API_KEY and makes real API call
async fn test_anthropic_multiple_system_messages() {
    // Test that multiple system messages are properly combined
    // Anthropic's API requires system messages to be in a single field
    let config = create_test_config();
    let provider = AnthropicProvider::new(&config)
        .expect("Failed to create Anthropic provider");
    
    // Create messages with multiple system messages
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::system("You always respond in exactly 3 words."),
        Message::user("How are you?"),
    ];
    
    // Send message
    let response = provider.send_message(&messages).await
        .expect("Failed to send message to Anthropic API");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    
    // The response should be influenced by both system messages
    println!("Response with multiple system messages: {}", response);
}

#[tokio::test]
#[ignore] // Requires ANTHROPIC_API_KEY and makes real API call
async fn test_anthropic_multi_turn_conversation() {
    // Test that the provider can handle multi-turn conversations
    let config = create_test_config();
    let provider = AnthropicProvider::new(&config)
        .expect("Failed to create Anthropic provider");
    
    // Create a multi-turn conversation
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("My name is Alice."),
        Message::assistant("Hello Alice! Nice to meet you."),
        Message::user("What is my name?"),
    ];
    
    // Send message
    let response = provider.send_message(&messages).await
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
    
    println!("Multi-turn response: {}", response);
}

#[tokio::test]
#[ignore] // Makes real API call with invalid key
async fn test_anthropic_invalid_api_key() {
    // Create provider with invalid API key
    let config = create_invalid_config();
    let provider = AnthropicProvider::new(&config)
        .expect("Failed to create Anthropic provider");
    
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
async fn test_anthropic_empty_messages() {
    // Test error handling when no messages are provided
    let config = create_test_config();
    let provider = AnthropicProvider::new(&config)
        .expect("Failed to create Anthropic provider");
    
    // Send empty messages array
    let messages: Vec<Message> = vec![];
    
    // This should fail - Anthropic requires at least one message
    let result = provider.send_message(&messages).await;
    
    // Verify that the request failed
    assert!(result.is_err(), "Request with empty messages should fail");
    
    let error = result.unwrap_err();
    println!("Expected error for empty messages: {}", error);
}

#[tokio::test]
#[ignore] // Makes real API call
async fn test_anthropic_max_tokens_limit() {
    // Test that max_tokens configuration is respected
    let mut config = create_test_config();
    config.max_tokens = 10; // Very small limit
    
    let provider = AnthropicProvider::new(&config)
        .expect("Failed to create Anthropic provider");
    
    // Ask for something that would normally generate a long response
    let messages = vec![
        Message::user("Write a long story about a dragon."),
    ];
    
    // Send message
    let response = provider.send_message(&messages).await
        .expect("Failed to send message to Anthropic API");
    
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
async fn test_anthropic_temperature_setting() {
    // Test that temperature configuration is accepted
    // Note: We can't easily verify the temperature is actually used,
    // but we can verify the request succeeds with different temperatures
    let mut config = create_test_config();
    config.temperature = 0.0; // Deterministic
    
    let provider = AnthropicProvider::new(&config)
        .expect("Failed to create Anthropic provider");
    
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

#[tokio::test]
#[ignore] // Makes real API call
async fn test_anthropic_only_system_message() {
    // Test edge case: only system message, no user message
    let config = create_test_config();
    let provider = AnthropicProvider::new(&config)
        .expect("Failed to create Anthropic provider");
    
    // Create messages with only a system message
    let messages = vec![
        Message::system("You are a helpful assistant."),
    ];
    
    // This should fail - Anthropic requires at least one user/assistant message
    let result = provider.send_message(&messages).await;
    
    // Verify that the request failed
    assert!(result.is_err(), "Request with only system message should fail");
    
    let error = result.unwrap_err();
    println!("Expected error for only system message: {}", error);
}
