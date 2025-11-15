//! Planner crate for the AI Agent Framework
//! 
//! This crate provides the planning system that decomposes user goals into
//! executable steps using LLM reasoning. The planner generates structured plans
//! that the executor can run sequentially.
//! 
//! # Core Concepts
//! 
//! - **Plan**: A sequence of steps with reasoning about why the plan was chosen
//! - **Step**: Individual actions that can be tool calls, reasoning steps, or responses
//! - **ToolCall**: Structured invocation of a tool with parameters as JSON
//! - **Planner**: Orchestrates plan generation using LLM with system prompts
//! 
//! # Architecture
//! 
//! The planner uses a multi-step process:
//! 
//! 1. **System Prompt Generation**: Creates a detailed prompt that includes available
//!    tools, their descriptions, parameter schemas, and instructions for generating
//!    valid JSON plans.
//! 
//! 2. **LLM Invocation**: Sends the system prompt and user goal to the LLM provider
//!    to generate a plan using reasoning capabilities.
//! 
//! 3. **Plan Parsing**: Extracts and parses the JSON response from the LLM into
//!    a structured Plan object with typed steps.
//! 
//! 4. **Plan Validation**: Verifies that all tool references in the plan exist
//!    in the tool registry to prevent runtime errors.
//! 
//! # Example
//! 
//! ```rust,ignore
//! use planner::{Planner, Plan};
//! use llm::OpenAIProvider;
//! use memory::InMemoryStore;
//! use tools::{ToolRegistry, Calculator};
//! 
//! async fn example() -> agent_core::Result<()> {
//!     // Set up dependencies
//!     let llm = Box::new(OpenAIProvider::new(&config)?);
//!     let memory = Box::new(InMemoryStore::new());
//!     let planner = Planner::new(llm, memory);
//!     
//!     // Create a tool registry
//!     let mut registry = ToolRegistry::new();
//!     registry.register(Box::new(Calculator::new()));
//!     
//!     // Generate a plan
//!     let plan = planner.create_plan(
//!         "What is 15 + 27?",
//!         &registry.list_tools()
//!     ).await?;
//!     
//!     // Validate the plan
//!     planner.validate_plan(&plan, &registry)?;
//!     
//!     // The plan is now ready for execution
//!     println!("Generated plan with {} steps", plan.steps.len());
//!     Ok(())
//! }
//! ```
//! 
//! # Plan Format
//! 
//! Plans are represented as JSON with the following structure:
//! 
//! ```json
//! {
//!   "reasoning": "Explanation of the plan",
//!   "steps": [
//!     {
//!       "type": "tool_call",
//!       "tool_name": "calculator",
//!       "parameters": {"operation": "add", "a": 15, "b": 27}
//!     },
//!     {
//!       "type": "reasoning",
//!       "text": "The calculation shows the result"
//!     },
//!     {
//!       "type": "response",
//!       "text": "15 + 27 equals 42"
//!     }
//!   ]
//! }
//! ```

mod types;
mod planner;

// Re-export public types
pub use types::{Plan, Step, ToolCall};
pub use planner::Planner;
