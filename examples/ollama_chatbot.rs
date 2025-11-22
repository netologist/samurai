//! Simple chatbot example using Ollama
//!
//! This example demonstrates a basic chatbot using local Ollama models.
//! No API keys required!
//!
//! # Prerequisites
//!
//! 1. Install Ollama: https://ollama.ai
//! 2. Pull a model: `ollama pull llama2`
//! 3. Ensure Ollama is running (it usually auto-starts)
//!
//! # Running
//!
//! ```bash
//! cargo run --example ollama_chatbot
//! ```

use agent_core::{Message, Result};
use config::LLMConfig;
use llm::create_provider;
use memory::{InMemoryStore, MemoryStore};

use colored::Colorize;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "=== Ollama Chatbot ===".bright_cyan().bold());
    println!("{}", "A simple chatbot using local Ollama models".bright_black());
    println!("{}", "No API keys required!".bright_green());
    println!();

    // Create Ollama configuration
    let config = LLMConfig {
        provider: "ollama".to_string(),
        model: "llama2".to_string(), // You can change this to mistral, phi, etc.
        api_key: String::new(),      // Not needed for Ollama
        base_url: Some("http://localhost:11434".to_string()),
        temperature: 0.7,
        max_tokens: 2000,
    };

    println!("{} {}", "Using model:".bright_black(), config.model.bright_white());
    println!("{} {}", "Provider:".bright_black(), config.provider.bright_white());
    println!();

    // Create LLM provider
    let llm = match create_provider(&config) {
        Ok(provider) => provider,
        Err(e) => {
            eprintln!("{}", format!("❌ Failed to create Ollama provider: {}", e).red());
            eprintln!();
            eprintln!("{}", "Make sure Ollama is installed and running:".yellow());
            eprintln!("  1. Install from https://ollama.ai");
            eprintln!("  2. Run: ollama pull llama2");
            eprintln!("  3. Ollama should start automatically");
            return Err(e);
        }
    };

    // Create memory store for conversation history
    let mut memory = InMemoryStore::new();

    // Add system message
    memory.add_message(Message::system(
        "You are a helpful AI assistant running locally via Ollama. Be concise and friendly.",
    ));

    println!("{}", "💬 Chat with the bot (type 'quit' to exit)".bright_green());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!();

    // Simple REPL
    loop {
        // Print prompt
        print!("{} ", "You:".bright_blue().bold());
        io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Check for exit command
        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!();
            println!("{}", "👋 Goodbye!".bright_cyan());
            break;
        }

        // Skip empty input
        if input.is_empty() {
            continue;
        }

        // Add user message to memory
        memory.add_message(Message::user(input));

        // Get recent conversation context
        let messages = memory.get_recent(10);

        // Send to LLM
        print!("{} ", "Bot:".bright_green().bold());
        io::stdout().flush()?;

        match llm.send_message(&messages).await {
            Ok(response) => {
                println!("{}", response);

                // Add assistant response to memory
                memory.add_message(Message::assistant(&response));
            }
            Err(e) => {
                eprintln!("{}", format!("❌ Error: {}", e).red());
                eprintln!();
                eprintln!("{}", "Is Ollama running? Try: ollama serve".yellow());
            }
        }

        println!();
    }

    Ok(())
}
