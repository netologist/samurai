use crate::agent::Agent;
use agent_core::Result;
use colored::Colorize;

/// Run the agent in single-turn mode with a single query
///
/// This function processes one query and prints the response to stdout.
/// It's useful for scripting and one-off queries.
///
/// # Arguments
/// * `agent` - The agent to use for processing
/// * `query` - The query to process
///
/// # Returns
/// * `Result<()>` - Ok if successful, error otherwise
///
/// # Errors
/// Returns an error if:
/// - The agent fails to process the query
/// - Output cannot be written to stdout
pub async fn run(agent: &mut Agent, query: &str) -> Result<()> {
    // Process the query
    let response = agent.process(query).await?;

    // Print response to stdout with success color
    println!("{}", response.bright_white());

    Ok(())
}
