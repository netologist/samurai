use crate::agent::Agent;
use agent_core::Result;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// Run the agent in REPL (Read-Eval-Print Loop) mode
///
/// This function provides an interactive conversation interface where users
/// can have multi-turn conversations with the agent. It supports:
/// - Line editing and history with rustyline
/// - Colored output for better readability
/// - Special commands like "exit" to quit
/// - Conversation history display
///
/// # Arguments
/// * `agent` - The agent to use for processing queries
///
/// # Returns
/// * `Result<()>` - Ok when the user exits, error on fatal failures
///
/// # Errors
/// Returns an error if:
/// - The readline editor cannot be initialized
/// - Fatal I/O errors occur
pub async fn run(mut agent: Agent) -> Result<()> {
    // Create readline editor with history support
    let mut rl = DefaultEditor::new().map_err(|e| {
        agent_core::AgentError::Execution(format!("Failed to initialize REPL: {}", e))
    })?;

    println!("{}", "AI Agent REPL - Interactive Mode".bright_cyan().bold());
    println!("Type your queries and press Enter. Type 'exit' to quit.");
    println!("Type 'history' to show conversation history.");
    println!();

    loop {
        // Display prompt and read user input
        let readline = rl.readline(">> ");

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
                        println!("{}", "Goodbye!".bright_green());
                        break;
                    }
                    "history" => {
                        println!("\n{}", "Conversation History:".bright_cyan().bold());
                        println!("(History display not yet implemented)");
                        println!();
                        continue;
                    }
                    "help" => {
                        println!("\n{}", "Available commands:".bright_cyan().bold());
                        println!("  {}  - Exit the REPL", "exit, quit".bright_yellow());
                        println!("  {}     - Show conversation history", "history".bright_yellow());
                        println!("  {}        - Show this help message", "help".bright_yellow());
                        println!();
                        continue;
                    }
                    _ => {}
                }

                // Process the query with the agent
                match agent.process(trimmed).await {
                    Ok(response) => {
                        // Print response with colored output (green for success)
                        println!("\n{}\n", response.bright_white());
                    }
                    Err(e) => {
                        // Print error with colored output (red for errors)
                        eprintln!("\n{} {}\n", "Error:".bright_red().bold(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C pressed
                println!("^C");
                println!("{}", "Use 'exit' or 'quit' to exit the REPL".bright_yellow());
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
