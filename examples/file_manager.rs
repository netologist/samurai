//! File Manager Example
//!
//! This example demonstrates an agent with file operations and safety guardrails.
//! It showcases:
//! - Loading configuration with FileReader tool enabled
//! - Implementing FilePathGuardrail to restrict file access
//! - Safe file operations within allowed directories
//! - How guardrails prevent unauthorized actions
//! - Security-conscious agent design
//!
//! The file manager can:
//! - Read file contents from allowed directories
//! - Analyze and summarize file information
//! - Answer questions about file contents
//! - Demonstrate guardrail enforcement
//!
//! Safety features:
//! - FilePathGuardrail restricts access to specific directories
//! - Attempts to access files outside allowed paths are blocked
//! - Clear error messages when guardrails are violated
//!
//! # Usage
//!
//! ```bash
//! # Set your OpenAI API key
//! export OPENAI_API_KEY=your_api_key_here
//!
//! # Run the file manager example
//! cargo run --example file_manager
//! ```

use agent_core::Result;
use colored::Colorize;
use config::AgentConfig;
use executor::Executor;
use guardrails::{FilePathGuardrail, GuardrailRegistry};
use memory::InMemoryStore;
use planner::Planner;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::path::PathBuf;
use tools::{FileReader, ToolRegistry};

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "=== File Manager Example ===".bright_cyan().bold());
    println!("This example demonstrates safe file operations with guardrails.");
    println!();

    // Load configuration from the file manager config file
    let config_path = std::path::Path::new("examples/configs/file_manager.yaml");
    let config = config::load_from_file(config_path)?;

    // Initialize the file manager agent with guardrails
    let mut agent = create_file_manager_agent(config)?;

    // Display allowed paths
    println!("{}", "Allowed file access paths:".bright_yellow().bold());
    println!(
        "  • Current directory: {}",
        std::env::current_dir()?.display()
    );
    println!("  • Examples directory: examples/");
    println!();
    println!(
        "{}",
        "Attempts to access files outside these paths will be blocked.".bright_yellow()
    );
    println!();

    // Run the interactive REPL
    run_file_manager_repl(&mut agent).await?;

    Ok(())
}

/// Create a file manager agent with FileReader tool and FilePathGuardrail
fn create_file_manager_agent(config: AgentConfig) -> Result<FileManagerAgent> {
    // Create memory store for conversation history
    let memory = Box::new(InMemoryStore::new());

    // Create tool registry and register FileReader tool
    let mut tools = ToolRegistry::new();

    // Register FileReader - allows reading file contents
    // This is the only tool needed for file management tasks
    tools.register(Box::new(FileReader::new()));
    println!("{}", "  ✓ Registered FileReader tool".bright_green());

    // Create planner with LLM and memory
    let planner_memory = Box::new(InMemoryStore::new());
    let planner_llm = llm::create_provider(&config.llm)?;
    let planner = Planner::new(planner_llm, planner_memory);

    // Create executor with tools and memory
    let executor_memory = Box::new(InMemoryStore::new());
    let executor = Executor::new(tools, executor_memory);

    // Create guardrails registry with FilePathGuardrail
    // This is the key security feature of this example
    let mut guardrails = GuardrailRegistry::new();

    // Define allowed paths for file access
    // In this example, we allow:
    // 1. Current working directory
    // 2. Examples directory (for demonstration)
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let examples_dir = current_dir.join("examples");

    let allowed_paths = vec![current_dir, examples_dir];

    // Register the FilePathGuardrail
    // This guardrail will validate all file operations before execution
    guardrails.register(Box::new(FilePathGuardrail::new(allowed_paths)));
    println!("{}", "  ✓ Registered FilePathGuardrail".bright_green());
    println!();

    Ok(FileManagerAgent {
        memory,
        planner,
        executor,
        guardrails,
    })
}

/// File manager agent structure with guardrails
struct FileManagerAgent {
    memory: Box<dyn memory::MemoryStore>,
    planner: Planner,
    executor: Executor,
    guardrails: GuardrailRegistry,
}

impl FileManagerAgent {
    /// Process a file management query with guardrail validation
    async fn manage_files(&mut self, query: &str) -> Result<String> {
        // Add user query to memory
        let user_message = agent_core::Message::user(query);
        self.memory.add_message(user_message);

        // Use planner to create a file management plan
        let available_tools = self.executor.list_tools();
        println!("{}", "  → Planning file operations...".bright_blue());
        let plan = self.planner.create_plan(query, &available_tools).await?;

        println!(
            "{} {} steps",
            "  → Plan created with".bright_blue(),
            plan.steps.len()
        );

        // Validate plan with guardrails
        // This is where FilePathGuardrail checks file access permissions
        println!("{}", "  → Validating with guardrails...".bright_blue());
        match self.guardrails.validate_all(&plan) {
            Ok(_) => {
                println!("{}", "  ✓ Guardrail validation passed".bright_green());
            }
            Err(e) => {
                // Guardrail violation detected
                println!(
                    "{}",
                    "  ✗ Guardrail violation detected!".bright_red().bold()
                );
                return Err(e);
            }
        }

        // Execute the file management plan
        println!("{}", "  → Executing file operations...".bright_blue());
        let result = self.executor.execute_plan(plan).await?;

        // Add result to memory
        let assistant_message = agent_core::Message::assistant(&result.final_response);
        self.memory.add_message(assistant_message);

        Ok(result.final_response)
    }
}

/// Run the file manager REPL
async fn run_file_manager_repl(agent: &mut FileManagerAgent) -> Result<()> {
    // Create readline editor with history support
    let mut rl = DefaultEditor::new().map_err(|e| {
        agent_core::AgentError::Execution(format!("Failed to initialize REPL: {}", e))
    })?;

    println!(
        "{}",
        "File Manager REPL - Ask about files in allowed directories".bright_green()
    );
    println!("The agent will use FileReader tool with guardrail protection.");
    println!("Commands: 'exit' to quit, 'help' for help, 'examples' for sample queries");
    println!();

    loop {
        // Display prompt and read user input
        let readline = rl.readline("FileManager: ");

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
                        println!("{}", "Goodbye! Stay safe!".bright_green());
                        break;
                    }
                    "help" => {
                        println!("\n{}", "Available commands:".bright_cyan().bold());
                        println!(
                            "  {}  - Exit the file manager",
                            "exit, quit".bright_yellow()
                        );
                        println!(
                            "  {}        - Show this help message",
                            "help".bright_yellow()
                        );
                        println!(
                            "  {}    - Show example file queries",
                            "examples".bright_yellow()
                        );
                        println!();
                        continue;
                    }
                    "examples" => {
                        println!(
                            "\n{}",
                            "Example file management queries:".bright_cyan().bold()
                        );
                        println!("  • Read the README.md file and summarize it");
                        println!("  • What's in the Cargo.toml file?");
                        println!(
                            "  • Read examples/configs/chatbot.yaml and explain the configuration"
                        );
                        println!("  • Try to read /etc/passwd (will be blocked by guardrail)");
                        println!();
                        println!(
                            "{}",
                            "Note: Attempts to access files outside allowed paths will be blocked."
                                .bright_yellow()
                        );
                        println!();
                        continue;
                    }
                    _ => {}
                }

                // Process the file management query with the agent
                println!();
                match agent.manage_files(trimmed).await {
                    Ok(response) => {
                        // Print response with colored output
                        println!("\n{}", "File Manager Results:".bright_cyan().bold());
                        println!("{}\n", response);
                    }
                    Err(e) => {
                        // Print error with colored output
                        // Guardrail violations will appear here
                        eprintln!("\n{} {}\n", "Error:".bright_red().bold(), e);

                        // Provide helpful message for guardrail violations
                        if e.to_string().contains("Guardrail") {
                            eprintln!(
                                "{}",
                                "This file access was blocked by the FilePathGuardrail."
                                    .bright_yellow()
                            );
                            eprintln!(
                                "{}",
                                "Only files in allowed directories can be accessed."
                                    .bright_yellow()
                            );
                            eprintln!();
                        }
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
