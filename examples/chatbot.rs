//! Simple Chatbot Example
//!
//! This example demonstrates a basic conversational agent with no tools.
//! It showcases:
//! - Loading configuration from a YAML file
//! - Initializing an agent with LLM and memory only
//! - Running an interactive REPL loop for multi-turn conversations
//!
//! The chatbot maintains conversation context through the memory system
//! and can engage in natural dialogue without any external tool capabilities.
//!
//! # Usage
//!
//! ```bash
//! # Set your OpenAI API key
//! export OPENAI_API_KEY=your_api_key_here
//!
//! # Run the chatbot example
//! cargo run --example chatbot
//! ```

use agent_core::Result;
use colored::Colorize;
use config::AgentConfig;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "=== Simple Chatbot Example ===".bright_cyan().bold());
    println!("This example demonstrates a basic conversational agent.");
    println!();

    // Load configuration from the chatbot config file
    let config_path = std::path::Path::new("examples/configs/chatbot.yaml");
    let config = config::load_from_file(config_path)?;

    // Initialize the agent with the configuration
    // This chatbot uses LLM and memory but no tools for simple conversation
    let mut agent = create_simple_agent(config)?;

    // Run the interactive REPL
    run_chatbot_repl(&mut agent).await?;

    Ok(())
}

/// Create a simple agent with LLM and memory only (no tools)
fn create_simple_agent(config: AgentConfig) -> Result<SimpleAgent> {
    // Create memory store for conversation history
    let memory = Box::new(memory::InMemoryStore::new());

    // Create LLM provider from config
    let llm = llm::create_provider(&config.llm)?;

    Ok(SimpleAgent { llm, memory })
}

/// Simple agent structure for chatbot (no tools, planner, or executor)
struct SimpleAgent {
    llm: Box<dyn llm::LLMProvider>,
    memory: Box<dyn memory::MemoryStore>,
}

impl SimpleAgent {
    /// Process a user message and return a response
    async fn chat(&mut self, user_input: &str) -> Result<String> {
        // Add user message to memory
        let user_message = agent_core::Message::user(user_input);
        self.memory.add_message(user_message.clone());

        // Get recent conversation history for context
        let history = self.memory.get_recent(10);

        // Send messages to LLM and get response
        let response = self.llm.send_message(&history).await?;

        // Add assistant response to memory
        let assistant_message = agent_core::Message::assistant(&response);
        self.memory.add_message(assistant_message);

        Ok(response)
    }
}

/// Run the chatbot REPL (Read-Eval-Print Loop)
async fn run_chatbot_repl(agent: &mut SimpleAgent) -> Result<()> {
    // Create readline editor with history support
    let mut rl = DefaultEditor::new().map_err(|e| {
        agent_core::AgentError::Execution(format!("Failed to initialize REPL: {}", e))
    })?;

    println!("{}", "Chatbot REPL - Type your messages and press Enter".bright_green());
    println!("Commands: 'exit' to quit, 'help' for help");
    println!();

    loop {
        // Display prompt and read user input
        let readline = rl.readline("You: ");

        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                // Skip empty lines
                if trimmed.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(trimmed);

                // Handle special commands
                match trimmed {
                    "exit" | "quit" => {
                        println!("{}", "Goodbye! Thanks for chatting!".bright_green());
                        break;
                    }
                    "help" => {
                        println!("\n{}", "Available commands:".bright_cyan().bold());
                        println!("  {}  - Exit the chatbot", "exit, quit".bright_yellow());
                        println!("  {}        - Show this help message", "help".bright_yellow());
                        println!();
                        continue;
                    }
                    _ => {}
                }

                // Process the message with the agent
                match agent.chat(trimmed).await {
                    Ok(response) => {
                        // Print response with colored output
                        println!("{} {}\n", "Bot:".bright_cyan().bold(), response);
                    }
                    Err(e) => {
                        // Print error with colored output
                        eprintln!("{} {}\n", "Error:".bright_red().bold(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C pressed
                println!("^C");
                println!("{}", "Use 'exit' or 'quit' to exit".bright_yellow());
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D pressed
                println!("{}", "Goodbye!".bright_green());
                break;
            }
            Err(err) => {
                // Fatal error
                return Err(agent_core::AgentError::Execution(format!(
                    "REPL error: {}",
                    err
                )));
            }
        }
    }

    Ok(())
}
