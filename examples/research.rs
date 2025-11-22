//! Research Assistant Example
//!
//! This example demonstrates an agent with tool capabilities for research tasks.
//! It showcases:
//! - Loading configuration with tools enabled
//! - Initializing an agent with WebSearchStub and FileReader tools
//! - Using the planner to decompose research queries into steps
//! - Executing multi-step workflows with tool invocations
//! - Demonstrating how tools extend agent capabilities
//!
//! The research assistant can:
//! - Search for information (using WebSearchStub for demonstration)
//! - Read and analyze file contents
//! - Combine information from multiple sources
//! - Provide comprehensive research summaries
//!
//! # Usage
//!
//! ```bash
//! # Set your OpenAI API key
//! export OPENAI_API_KEY=your_api_key_here
//!
//! # Run the research assistant example
//! cargo run --example research
//! ```

use agent_core::Result;
use colored::Colorize;
use config::AgentConfig;
use executor::Executor;
use guardrails::GuardrailRegistry;
use memory::InMemoryStore;
use planner::Planner;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use tools::{FileReader, ToolRegistry, WebSearchStub};

#[tokio::main]
async fn main() -> Result<()> {
    println!(
        "{}",
        "=== Research Assistant Example ===".bright_cyan().bold()
    );
    println!("This example demonstrates an agent with tool capabilities.");
    println!("Available tools: WebSearch (stub), FileReader");
    println!();

    // Load configuration from the research config file
    let config_path = std::path::Path::new("examples/configs/research.yaml");
    let config = config::load_from_file(config_path)?;

    // Initialize the research assistant agent
    let mut agent = create_research_agent(config)?;

    // Run the interactive REPL
    run_research_repl(&mut agent).await?;

    Ok(())
}

/// Create a research assistant agent with tools enabled
fn create_research_agent(config: AgentConfig) -> Result<ResearchAgent> {
    // Create memory store for conversation history
    let memory = Box::new(InMemoryStore::new());

    // Create tool registry and register research tools
    let mut tools = ToolRegistry::new();

    // Register WebSearchStub - simulates web search capability
    // In a production system, this would integrate with a real search API
    tools.register(Box::new(WebSearchStub::new()));
    println!("{}", "  ✓ Registered WebSearchStub tool".bright_green());

    // Register FileReader - allows reading file contents
    // Useful for analyzing documents, code, or data files
    tools.register(Box::new(FileReader::new()));
    println!("{}", "  ✓ Registered FileReader tool".bright_green());

    println!();

    // Create planner with LLM and memory
    // The planner uses the LLM to decompose research queries into steps
    let planner_memory = Box::new(InMemoryStore::new());
    let planner_llm = llm::create_provider(&config.llm)?;
    let planner = Planner::new(planner_llm, planner_memory);

    // Create executor with tools and memory
    // The executor runs the planned steps and invokes tools
    let executor_memory = Box::new(InMemoryStore::new());
    let executor = Executor::new(tools, executor_memory);

    // Create guardrails registry (empty for this example)
    // Research tasks typically don't need strict guardrails
    let guardrails = GuardrailRegistry::new();

    Ok(ResearchAgent {
        memory,
        planner,
        executor,
        guardrails,
    })
}

/// Research agent structure with full framework capabilities
struct ResearchAgent {
    memory: Box<dyn memory::MemoryStore>,
    planner: Planner,
    executor: Executor,
    guardrails: GuardrailRegistry,
}

impl ResearchAgent {
    /// Process a research query using the full agent workflow
    async fn research(&mut self, query: &str) -> Result<String> {
        // Add user query to memory
        let user_message = agent_core::Message::user(query);
        self.memory.add_message(user_message);

        // Use planner to create a multi-step research plan
        // The planner will identify which tools to use and in what order
        let available_tools = self.executor.list_tools();
        println!("{}", "  → Planning research steps...".bright_blue());
        let plan = self.planner.create_plan(query, &available_tools).await?;

        println!(
            "{} {} steps",
            "  → Plan created with".bright_blue(),
            plan.steps.len()
        );

        // Validate plan with guardrails (if any are configured)
        self.guardrails.validate_all(&plan)?;

        // Execute the research plan
        // This will invoke tools as needed and collect results
        println!("{}", "  → Executing research plan...".bright_blue());
        let result = self.executor.execute_plan(plan).await?;

        // Add result to memory for context in future queries
        let assistant_message = agent_core::Message::assistant(&result.final_response);
        self.memory.add_message(assistant_message);

        Ok(result.final_response)
    }
}

/// Run the research assistant REPL
async fn run_research_repl(agent: &mut ResearchAgent) -> Result<()> {
    // Create readline editor with history support
    let mut rl = DefaultEditor::new().map_err(|e| {
        agent_core::AgentError::Execution(format!("Failed to initialize REPL: {}", e))
    })?;

    println!(
        "{}",
        "Research Assistant REPL - Ask research questions".bright_green()
    );
    println!("The assistant will use tools to gather and analyze information.");
    println!("Commands: 'exit' to quit, 'help' for help, 'examples' for sample queries");
    println!();

    loop {
        // Display prompt and read user input
        let readline = rl.readline("Research: ");

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
                        println!("{}", "Goodbye! Happy researching!".bright_green());
                        break;
                    }
                    "help" => {
                        println!("\n{}", "Available commands:".bright_cyan().bold());
                        println!(
                            "  {}  - Exit the research assistant",
                            "exit, quit".bright_yellow()
                        );
                        println!(
                            "  {}        - Show this help message",
                            "help".bright_yellow()
                        );
                        println!(
                            "  {}    - Show example research queries",
                            "examples".bright_yellow()
                        );
                        println!();
                        continue;
                    }
                    "examples" => {
                        println!("\n{}", "Example research queries:".bright_cyan().bold());
                        println!("  • Search for information about Rust async programming");
                        println!("  • Read the README.md file and summarize it");
                        println!(
                            "  • Find information about AI agents and explain the key concepts"
                        );
                        println!("  • Analyze the Cargo.toml file and list all dependencies");
                        println!();
                        continue;
                    }
                    _ => {}
                }

                // Process the research query with the agent
                println!();
                match agent.research(trimmed).await {
                    Ok(response) => {
                        // Print response with colored output
                        println!("\n{}", "Research Results:".bright_cyan().bold());
                        println!("{}\n", response);
                    }
                    Err(e) => {
                        // Print error with colored output
                        eprintln!("\n{} {}\n", "Error:".bright_red().bold(), e);
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
