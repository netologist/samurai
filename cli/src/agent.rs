//! Agent orchestration module.
//!
//! This module provides the main Agent struct that coordinates all framework
//! components (LLM, memory, planner, executor, tools, guardrails) to process
//! user queries.

use agent_core::Result;
use config::AgentConfig;
use executor::Executor;
use guardrails::{FilePathGuardrail, GuardrailRegistry, RateLimitGuardrail};
use llm::create_provider;
use memory::{InMemoryStore, MemoryStore};
use planner::Planner;
use tools::{Calculator, FileReader, ToolRegistry, WebSearchStub};

/// Main agent structure that orchestrates all framework components.
///
/// The Agent coordinates:
/// - Memory: Stores conversation history
/// - Planner: Generates plans from user queries
/// - Executor: Runs plans and invokes tools
/// - Guardrails: Validates plans before execution
pub struct Agent {
    memory: Box<dyn MemoryStore>,
    planner: Planner,
    executor: Executor,
    guardrails: GuardrailRegistry,
}

impl Agent {
    /// Create a new agent from configuration
    ///
    /// # Arguments
    /// * `config` - Agent configuration loaded from file or environment
    ///
    /// # Returns
    /// * `Result<Self>` - Initialized agent or error
    ///
    /// # Errors
    /// Returns an error if:
    /// - LLM provider initialization fails
    /// - Configuration is invalid
    pub fn new(config: AgentConfig) -> Result<Self> {
        // Create memory store
        let memory = Box::new(InMemoryStore::new());

        // Create tool registry and register default tools
        let mut tools = ToolRegistry::new();

        // Register tools based on config or use defaults
        if config.tools.is_empty() || config.tools.contains(&"calculator".to_string()) {
            tools.register(Box::new(Calculator::new()));
        }
        if config.tools.is_empty() || config.tools.contains(&"file_reader".to_string()) {
            tools.register(Box::new(FileReader::new()));
        }
        if config.tools.is_empty() || config.tools.contains(&"web_search".to_string()) {
            tools.register(Box::new(WebSearchStub::new()));
        }

        // Create planner with LLM and memory
        let planner_memory = Box::new(InMemoryStore::new());
        let planner_llm = create_provider(&config.llm)?;
        let planner = Planner::new(planner_llm, planner_memory);

        // Create executor with tools and memory
        let executor_memory = Box::new(InMemoryStore::new());
        let executor = Executor::new(tools, executor_memory);

        // Create guardrails registry and register default guardrails
        let mut guardrails = GuardrailRegistry::new();

        // Register guardrails based on config
        if config.guardrails.contains(&"file_path".to_string()) {
            // Default to allowing /tmp and current directory
            let allowed_paths = vec![
                std::path::PathBuf::from("/tmp"),
                std::env::current_dir().unwrap_or_default(),
            ];
            guardrails.register(Box::new(FilePathGuardrail::new(allowed_paths)));
        }
        if config.guardrails.contains(&"rate_limit".to_string()) {
            // Default to 100 calls per minute
            guardrails.register(Box::new(RateLimitGuardrail::new(100)));
        }

        Ok(Self {
            memory,
            planner,
            executor,
            guardrails,
        })
    }

    /// Process a user query and return a response
    ///
    /// This method orchestrates the complete agent workflow:
    /// 1. Adds the user query to memory
    /// 2. Uses the planner to create a plan from the query
    /// 3. Validates the plan with guardrails
    /// 4. Executes the plan with the executor
    /// 5. Returns the final response
    ///
    /// # Arguments
    /// * `query` - The user's query or request
    ///
    /// # Returns
    /// * `Result<String>` - The agent's response or an error
    ///
    /// # Errors
    /// Returns an error if:
    /// - Plan generation fails
    /// - Guardrail validation fails
    /// - Plan execution fails
    pub async fn process(&mut self, query: &str) -> Result<String> {
        // Add user query to memory
        let user_message = agent_core::Message::user(query);
        self.memory.add_message(user_message);

        // Use planner to create plan from query
        let available_tools = self.executor.list_tools();
        let plan = self.planner.create_plan(query, &available_tools).await?;

        // Validate plan with guardrails
        self.guardrails.validate_all(&plan)?;

        // Execute plan with executor
        let result = self.executor.execute_plan(plan).await?;

        // Return final response
        Ok(result.final_response)
    }
}
