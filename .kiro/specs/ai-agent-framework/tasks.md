# Implementation Plan

This implementation plan provides a series of prompts for implementing the AI agent framework. Each task builds incrementally on previous work, with all code integrated and functional at each step.

- [x] 1. Set up workspace structure and core crate
- [x] 1.1 Initialize cargo workspace with root Cargo.toml defining workspace members and shared dependencies
  - Create workspace with `cargo new --lib ai-agent-framework`
  - Add [workspace] section with members array
  - Add [workspace.dependencies] for shared crates (serde, thiserror, tokio)
  - _Requirements: 1.1, 1.2_

- [x] 1.2 Create core crate with Message and Role types
  - Create core crate with `cargo new --lib core`
  - Define Message struct with role, content, and timestamp fields in `core/src/message.rs`
  - Define Role enum (System, User, Assistant) with serde derives
  - Add helper methods for creating messages (Message::system, Message::user, Message::assistant)
  - _Requirements: 1.1, 1.3_

- [x] 1.3 Implement AgentError type with thiserror
  - Create `core/src/error.rs` with AgentError enum
  - Add variants for Config, LLMProvider, ToolExecution, GuardrailViolation, Memory, Planning, Execution errors
  - Implement From traits for std::io::Error and serde_json::Error
  - Define Result<T> type alias as std::result::Result<T, AgentError>
  - _Requirements: 12.1, 12.2, 12.3_

- [x] 1.4 Create core lib.rs with public exports
  - Re-export Message, Role, AgentError, and Result from lib.rs
  - Add module documentation explaining core crate purpose
  - _Requirements: 1.3_

- [ ]* 1.5 Write unit tests for core types
  - Test Message creation with different roles
  - Test error conversion from io::Error and serde_json::Error
  - Verify serde serialization/deserialization of Message
  - _Requirements: 12.2_

- [x] 1.6 Create root README with project overview
  - Document project structure and crate purposes
  - Add getting started instructions
  - Include architecture overview
  - _Requirements: 1.4_


- [x] 2. Implement configuration management
- [x] 2.1 Create config crate with configuration structures
  - Create config crate with `cargo new --lib config`
  - Define AgentConfig struct with llm, memory, tools, and guardrails fields in `config/src/lib.rs`
  - Define LLMConfig struct with provider, model, api_key, temperature, and max_tokens fields
  - Define MemoryConfig struct with max_messages and token_budget fields
  - Add serde Deserialize derives to all config structs
  - _Requirements: 2.1, 2.2_

- [x] 2.2 Implement file-based configuration loading
  - Implement `load_from_file(path: &Path) -> Result<AgentConfig>` function
  - Use serde_yaml to parse YAML config files
  - Add proper error handling with context about which file failed to load
  - _Requirements: 2.2_

- [x] 2.3 Implement environment variable configuration
  - Implement `from_env() -> Result<AgentConfig>` function
  - Read OPENAI_API_KEY, ANTHROPIC_API_KEY, MODEL, TEMPERATURE from environment
  - Provide sensible defaults for optional parameters (temperature: 0.7, max_tokens: 2000)
  - _Requirements: 2.4, 2.5_

- [x] 2.4 Implement configuration merging and validation
  - Implement `merge(file_config: AgentConfig, env_config: AgentConfig) -> AgentConfig` function
  - Environment variables should override file-based values
  - Implement `validate(config: &AgentConfig) -> Result<()>` to check required fields
  - Ensure api_key, provider, and model are present
  - _Requirements: 2.3, 2.4_

- [ ]* 2.5 Write unit tests for configuration loading
  - Test parsing valid YAML config files
  - Test error handling for missing required fields
  - Test environment variable override behavior
  - Test merge logic with various combinations
  - _Requirements: 2.2, 2.3, 2.4_

- [x] 2.6 Create example configuration files
  - Create `examples/configs/simple.yaml` with OpenAI configuration
  - Create `examples/configs/anthropic.yaml` with Anthropic configuration
  - Add comments explaining each configuration option
  - _Requirements: 2.2_


- [x] 3. Build HTTP communication layer
- [x] 3.1 Create communication crate with ApiClient
  - Create communication crate with `cargo new --lib communication`
  - Define ApiClient struct wrapping reqwest::Client in `communication/src/client.rs`
  - Implement `new()` constructor with default timeout of 30 seconds
  - Add timeout configuration field
  - _Requirements: 3.1, 3.3_

- [x] 3.2 Implement JSON POST method with error handling
  - Implement `post_json<T, R>(&self, url: &str, body: &T) -> Result<R>` method
  - Use reqwest to send JSON POST requests
  - Handle network errors, timeout errors, and deserialization errors
  - Convert reqwest errors to AgentError::LLMProvider
  - _Requirements: 3.3_

- [x] 3.3 Implement retry logic with exponential backoff
  - Implement `with_retry<F, T>(operation: F, max_attempts: u32) -> Result<T>` function
  - Use exponential backoff starting at 1 second, doubling each retry
  - Maximum 3 attempts for transient failures
  - Only retry on network errors and 5xx status codes, not 4xx errors
  - _Requirements: 3.5_

- [ ]* 3.4 Write unit tests for HTTP client
  - Test successful POST request with mock server
  - Test retry behavior with simulated failures
  - Test timeout handling
  - Test error conversion from reqwest errors
  - _Requirements: 3.3, 3.5_


- [x] 4. Implement LLM provider interface and OpenAI integration
- [x] 4.1 Create llm crate with LLMProvider trait
  - Create llm crate with `cargo new --lib llm`
  - Define LLMProvider async trait in `llm/src/provider.rs`
  - Add `send_message(&self, messages: &[Message]) -> Result<String>` method
  - Use async_trait macro for async trait support
  - _Requirements: 3.1, 3.2_

- [x] 4.2 Create OpenAI request and response types
  - Create `llm/src/openai/types.rs` with OpenAI API types
  - Define ChatCompletionRequest with model, messages, temperature, max_tokens fields
  - Define ChatCompletionResponse with choices array
  - Define Message format matching OpenAI API (role and content)
  - Add serde derives for serialization
  - _Requirements: 3.2_

- [x] 4.3 Implement OpenAIProvider
  - Create `llm/src/openai/mod.rs` with OpenAIProvider struct
  - Store api_key, model, temperature, max_tokens, and ApiClient
  - Implement constructor `new(config: &LLMConfig) -> Result<Self>`
  - Convert framework Message types to OpenAI format
  - _Requirements: 3.2_

- [x] 4.4 Implement send_message for OpenAI
  - Implement LLMProvider trait for OpenAIProvider
  - Build ChatCompletionRequest from messages
  - Call OpenAI API at https://api.openai.com/v1/chat/completions
  - Add Authorization header with Bearer token
  - Extract response text from choices[0].message.content
  - Handle API errors (auth failures, rate limits, invalid requests)
  - _Requirements: 3.2, 3.3_

- [ ]* 4.5 Write integration test for OpenAI provider
  - Create test that calls real OpenAI API (mark with #[ignore])
  - Verify successful message sending and response parsing
  - Test error handling for invalid API key
  - _Requirements: 3.2, 3.3_


- [x] 5. Add Anthropic provider and provider factory
- [x] 5.1 Create Anthropic request and response types
  - Create `llm/src/anthropic/types.rs` with Anthropic API types
  - Define MessagesRequest with model, messages, system, temperature, max_tokens fields
  - Define MessagesResponse with content array
  - Note: Anthropic separates system messages into dedicated field
  - Add serde derives for serialization
  - _Requirements: 3.2_

- [x] 5.2 Implement AnthropicProvider
  - Create `llm/src/anthropic/mod.rs` with AnthropicProvider struct
  - Store api_key, model, temperature, max_tokens, and ApiClient
  - Implement constructor `new(config: &LLMConfig) -> Result<Self>`
  - Convert framework Message types to Anthropic format, separating system messages
  - _Requirements: 3.2_

- [x] 5.3 Implement send_message for Anthropic
  - Implement LLMProvider trait for AnthropicProvider
  - Build MessagesRequest from messages
  - Call Anthropic API at https://api.anthropic.com/v1/messages
  - Add x-api-key header and anthropic-version: 2023-06-01 header
  - Extract response text from content[0].text
  - Handle API errors appropriately
  - _Requirements: 3.2, 3.3_

- [x] 5.4 Implement provider factory
  - Create `llm/src/factory.rs` with `create_provider(config: &LLMConfig) -> Result<Box<dyn LLMProvider>>` function
  - Match on config.provider string ("openai" or "anthropic")
  - Return appropriate provider instance
  - Return error for unknown provider types
  - _Requirements: 3.2_

- [x] 5.5 Update llm lib.rs with public exports
  - Re-export LLMProvider trait, create_provider function
  - Re-export OpenAIProvider and AnthropicProvider
  - Add module documentation
  - _Requirements: 3.1, 3.2_

- [ ]* 5.6 Write integration test for Anthropic provider
  - Create test that calls real Anthropic API (mark with #[ignore])
  - Verify successful message sending and response parsing
  - Test system message handling
  - _Requirements: 3.2, 3.3_


- [x] 6. Build memory system with token awareness
- [x] 6.1 Create memory crate with MemoryStore trait
  - Create memory crate with `cargo new --lib memory`
  - Define MemoryStore trait in `memory/src/store.rs`
  - Add methods: `add_message(&mut self, message: Message)`, `get_recent(&self, limit: usize) -> Vec<Message>`, `get_within_budget(&self, token_budget: usize) -> Vec<Message>`, `clear(&mut self)`
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 6.2 Implement InMemoryStore
  - Create `memory/src/in_memory.rs` with InMemoryStore struct
  - Use Vec<Message> for storage
  - Implement MemoryStore trait methods
  - Implement `get_recent` to return last N messages in chronological order
  - _Requirements: 4.1, 4.2, 4.4_

- [x] 6.3 Add token counting functionality
  - Add tiktoken-rs dependency for OpenAI token counting
  - Implement helper function `count_tokens(message: &Message) -> usize`
  - Use cl100k_base encoding (GPT-3.5/GPT-4)
  - _Requirements: 4.3_

- [x] 6.4 Implement token-aware retrieval
  - Implement `get_within_budget` method in InMemoryStore
  - Iterate messages from most recent, counting tokens
  - Stop when adding next message would exceed budget
  - Return messages in chronological order
  - _Requirements: 4.3_

- [x] 6.5 Create ConversationHistory wrapper
  - Create `memory/src/history.rs` with ConversationHistory struct
  - Wrap MemoryStore with helper methods
  - Add convenience methods for common operations
  - _Requirements: 4.2_

- [x] 6.6 Update memory lib.rs with public exports
  - Re-export MemoryStore trait, InMemoryStore, ConversationHistory
  - Add module documentation
  - _Requirements: 4.1, 4.4_

- [ ]* 6.7 Write unit tests for memory system
  - Test message storage and retrieval
  - Test get_recent with various limits
  - Test get_within_budget with different token budgets
  - Test clear functionality
  - _Requirements: 4.2, 4.3_


- [ ] 7. Implement tool system and registry
- [ ] 7.1 Create tools crate with Tool trait
  - Create tools crate with `cargo new --lib tools`
  - Define Tool async trait in `tools/src/tool.rs`
  - Add methods: `name(&self) -> &str`, `description(&self) -> &str`, `parameters_schema(&self) -> Value`, `async execute(&self, params: Value) -> Result<Value>`
  - Use async_trait macro
  - _Requirements: 5.1, 5.2_

- [ ] 7.2 Implement ToolRegistry
  - Create `tools/src/registry.rs` with ToolRegistry struct
  - Use HashMap<String, Box<dyn Tool>> for storage
  - Implement `register(&mut self, tool: Box<dyn Tool>)` method
  - Implement `get(&self, name: &str) -> Option<&dyn Tool>` method
  - Implement `list_tools(&self) -> Vec<ToolInfo>` to return tool names and descriptions
  - _Requirements: 5.2_

- [ ] 7.3 Implement Calculator tool
  - Create `tools/src/calculator.rs` with Calculator struct
  - Implement Tool trait with arithmetic operations (add, subtract, multiply, divide)
  - Define JSON schema for parameters (operation, a, b)
  - Parse parameters and perform calculation
  - Return result as JSON value
  - _Requirements: 5.4_

- [ ] 7.4 Implement FileReader tool
  - Create `tools/src/file_reader.rs` with FileReader struct
  - Implement Tool trait for reading file contents
  - Define JSON schema for parameters (file_path)
  - Read file contents using std::fs::read_to_string
  - Return contents as JSON string value
  - Handle file not found and permission errors
  - _Requirements: 5.4_

- [ ] 7.5 Implement WebSearchStub tool
  - Create `tools/src/web_search.rs` with WebSearchStub struct
  - Implement Tool trait that returns mock search results
  - Define JSON schema for parameters (query)
  - Return hardcoded mock results for demonstration
  - _Requirements: 5.4_

- [ ] 7.6 Update tools lib.rs with public exports
  - Re-export Tool trait, ToolRegistry
  - Re-export Calculator, FileReader, WebSearchStub
  - Add module documentation
  - _Requirements: 5.1, 5.4_

- [ ]* 7.7 Write unit tests for tools
  - Test Calculator with various operations
  - Test FileReader with existing and non-existing files
  - Test WebSearchStub returns expected format
  - Test ToolRegistry registration and retrieval
  - Test parameter validation
  - _Requirements: 5.2, 5.3, 5.4_


- [ ] 8. Create planning system with LLM reasoning
- [ ] 8.1 Create planner crate with Plan and Step types
  - Create planner crate with `cargo new --lib planner`
  - Define Plan struct in `planner/src/types.rs` with steps and reasoning fields
  - Define Step enum with ToolCall, Reasoning, and Response variants
  - Define ToolCall struct with tool_name and parameters fields
  - Add Clone and Debug derives
  - _Requirements: 6.1, 6.3_

- [ ] 8.2 Implement Planner struct
  - Create `planner/src/planner.rs` with Planner struct
  - Store LLMProvider and MemoryStore references
  - Implement constructor `new(llm: Box<dyn LLMProvider>, memory: Box<dyn MemoryStore>) -> Self`
  - _Requirements: 6.1, 6.2_

- [ ] 8.3 Create system prompt template for planning
  - Implement `build_system_prompt(&self, available_tools: &[ToolInfo]) -> String` method
  - Create prompt that instructs LLM to generate structured plans
  - Include available tools with descriptions and parameter schemas
  - Specify expected output format (JSON with steps array)
  - _Requirements: 6.2_

- [ ] 8.4 Implement plan generation
  - Implement `create_plan(&self, goal: &str, available_tools: &[ToolInfo]) -> Result<Plan>` method
  - Build system prompt with available tools
  - Create messages array with system prompt and user goal
  - Call LLM to generate plan
  - _Requirements: 6.1, 6.2_

- [ ] 8.5 Implement plan parsing
  - Implement `parse_plan(&self, response: &str) -> Result<Plan>` method
  - Parse LLM response as JSON
  - Extract steps array and convert to Step enum variants
  - Handle parsing errors gracefully
  - _Requirements: 6.3_

- [ ] 8.6 Implement plan validation
  - Implement `validate_plan(&self, plan: &Plan, registry: &ToolRegistry) -> Result<()>` method
  - Check that all tool calls reference tools in registry
  - Return error if unknown tool is referenced
  - _Requirements: 6.4, 6.5_

- [ ] 8.7 Update planner lib.rs with public exports
  - Re-export Planner, Plan, Step, ToolCall types
  - Add module documentation
  - _Requirements: 6.1_

- [ ]* 8.8 Write unit tests for planner
  - Test plan parsing with valid JSON responses
  - Test plan validation with valid and invalid tool references
  - Test system prompt generation includes all tools
  - Mock LLM responses for testing
  - _Requirements: 6.2, 6.4, 6.5_


- [ ] 9. Build execution system for running plans
- [ ] 9.1 Create executor crate with result types
  - Create executor crate with `cargo new --lib executor`
  - Define ExecutionResult struct in `executor/src/types.rs` with success, final_response, and step_results fields
  - Define StepResult struct with step_type, output, and success fields
  - Add Debug derives
  - _Requirements: 7.1, 7.3_

- [ ] 9.2 Implement Executor struct
  - Create `executor/src/executor.rs` with Executor struct
  - Store ToolRegistry and MemoryStore references
  - Implement constructor `new(tools: ToolRegistry, memory: Box<dyn MemoryStore>) -> Self`
  - _Requirements: 7.1, 7.2_

- [ ] 9.3 Implement execute_plan method
  - Implement `execute_plan(&mut self, plan: Plan) -> Result<ExecutionResult>` method
  - Iterate through plan steps sequentially
  - Call execute_step for each step
  - Collect step results
  - Add results to memory for context
  - Build final response from step results
  - _Requirements: 7.1, 7.3_

- [ ] 9.4 Implement execute_step method
  - Implement `execute_step(&mut self, step: &Step) -> Result<StepResult>` method
  - Pattern match on Step variants
  - For ToolCall, call handle_tool_call
  - For Reasoning, return reasoning text as result
  - For Response, return response text as result
  - _Requirements: 7.2, 7.3_

- [ ] 9.5 Implement handle_tool_call method
  - Implement `handle_tool_call(&mut self, tool_call: &ToolCall) -> Result<StepResult>` method
  - Look up tool in registry by name
  - Return error if tool not found
  - Execute tool with provided parameters
  - Capture tool result and wrap in StepResult
  - Handle tool execution errors
  - _Requirements: 7.2, 7.3, 7.5_

- [ ] 9.6 Implement result storage in memory
  - After each step execution, create Message with result
  - Add message to memory store
  - This provides context for subsequent steps
  - _Requirements: 7.5_

- [ ] 9.7 Update executor lib.rs with public exports
  - Re-export Executor, ExecutionResult, StepResult types
  - Add module documentation
  - _Requirements: 7.1_

- [ ]* 9.8 Write unit tests for executor
  - Test execute_plan with multi-step plan
  - Test execute_step with each Step variant
  - Test handle_tool_call with valid and invalid tool names
  - Test error handling when step fails
  - Mock tools and memory for testing
  - _Requirements: 7.2, 7.3, 7.4, 7.5_


- [ ] 10. Implement safety guardrails system
- [ ] 10.1 Create guardrails crate with Guardrail trait
  - Create guardrails crate with `cargo new --lib guardrails`
  - Define Guardrail trait in `guardrails/src/guardrail.rs`
  - Add methods: `name(&self) -> &str`, `validate(&self, plan: &Plan) -> Result<()>`
  - _Requirements: 8.1_

- [ ] 10.2 Implement GuardrailRegistry
  - Create `guardrails/src/registry.rs` with GuardrailRegistry struct
  - Use Vec<Box<dyn Guardrail>> for storage
  - Implement `register(&mut self, guardrail: Box<dyn Guardrail>)` method
  - Implement `validate_all(&self, plan: &Plan) -> Result<()>` method that checks all guardrails
  - Stop on first violation and return error
  - _Requirements: 8.1, 8.3, 8.4_

- [ ] 10.3 Implement FilePathGuardrail
  - Create `guardrails/src/file_path.rs` with FilePathGuardrail struct
  - Store allowed_paths: Vec<PathBuf> field
  - Implement Guardrail trait
  - Check all ToolCall steps for file_reader tool
  - Extract file_path parameter and validate it's within allowed paths
  - Return GuardrailViolation error if path not allowed
  - _Requirements: 8.2, 8.3, 8.4_

- [ ] 10.4 Implement RateLimitGuardrail
  - Create `guardrails/src/rate_limit.rs` with RateLimitGuardrail struct
  - Store max_calls_per_minute field and call tracking
  - Implement Guardrail trait
  - Count ToolCall steps in plan
  - Check if adding these calls would exceed rate limit
  - Return GuardrailViolation error if limit exceeded
  - _Requirements: 8.2, 8.3, 8.4_

- [ ] 10.5 Update guardrails lib.rs with public exports
  - Re-export Guardrail trait, GuardrailRegistry
  - Re-export FilePathGuardrail, RateLimitGuardrail
  - Add module documentation
  - _Requirements: 8.1, 8.2_

- [ ]* 10.6 Write unit tests for guardrails
  - Test FilePathGuardrail with allowed and disallowed paths
  - Test RateLimitGuardrail with plans under and over limit
  - Test GuardrailRegistry validates all registered guardrails
  - Test that validation stops on first violation
  - _Requirements: 8.3, 8.4_


- [ ] 11. Create rules engine for behavior customization
- [ ] 11.1 Create rules crate with Rule trait and PlanningContext
  - Create rules crate with `cargo new --lib rules`
  - Define PlanningContext struct in `rules/src/context.rs` with system_prompt, constraints, and metadata fields
  - Define Rule trait in `rules/src/rule.rs` with methods: `name(&self) -> &str`, `priority(&self) -> u32`, `apply(&self, context: &mut PlanningContext)`
  - _Requirements: 9.1, 9.2_

- [ ] 11.2 Implement RuleEngine
  - Create `rules/src/engine.rs` with RuleEngine struct
  - Use Vec<Box<dyn Rule>> for storage
  - Implement `add_rule(&mut self, rule: Box<dyn Rule>)` method
  - Implement `apply_all(&self, context: &mut PlanningContext)` method
  - Sort rules by priority before applying
  - _Requirements: 9.2, 9.3_

- [ ] 11.3 Implement ResponseLengthRule
  - Create `rules/src/response_length.rs` with ResponseLengthRule struct
  - Store max_words field
  - Implement Rule trait with priority 100
  - In apply method, add constraint to context about word limit
  - _Requirements: 9.4_

- [ ] 11.4 Implement ToneRule
  - Create `rules/src/tone.rs` with ToneRule struct
  - Store tone field (enum: Formal, Casual, Technical)
  - Implement Rule trait with priority 50
  - In apply method, modify system_prompt to include tone guidance
  - _Requirements: 9.4_

- [ ] 11.5 Update rules lib.rs with public exports
  - Re-export Rule trait, RuleEngine, PlanningContext
  - Re-export ResponseLengthRule, ToneRule
  - Add module documentation
  - _Requirements: 9.1, 9.4_

- [ ]* 11.6 Write unit tests for rules
  - Test ResponseLengthRule adds constraint to context
  - Test ToneRule modifies system prompt
  - Test RuleEngine applies rules in priority order
  - Test multiple rules can be applied to same context
  - _Requirements: 9.2, 9.3, 9.4_


- [ ] 12. Build command-line interface
- [ ] 12.1 Create cli crate with argument parsing
  - Create cli crate with `cargo new --bin cli`
  - Define CliArgs struct in `cli/src/args.rs` using clap derive
  - Add fields: config (PathBuf), query (Option<String>), verbose (bool)
  - Add clap attributes for help text and argument parsing
  - _Requirements: 10.1, 10.2_

- [ ] 12.2 Implement agent initialization
  - Create `cli/src/agent.rs` with Agent struct
  - Store all framework components (llm, memory, tools, planner, executor, guardrails, rules)
  - Implement `new(config: AgentConfig) -> Result<Self>` constructor
  - Initialize LLM provider from config
  - Create memory store, tool registry, planner, executor
  - Register default tools (Calculator, FileReader, WebSearchStub)
  - Register default guardrails if configured
  - _Requirements: 10.3_

- [ ] 12.3 Implement process method for agent
  - Implement `process(&mut self, query: &str) -> Result<String>` method on Agent
  - Add user query to memory
  - Use planner to create plan from query
  - Validate plan with guardrails
  - Execute plan with executor
  - Return final response
  - _Requirements: 10.3_

- [ ] 12.4 Implement single-turn mode
  - Create `cli/src/single.rs` with `run(agent: &mut Agent, query: &str) -> Result<()>` function
  - Call agent.process with query
  - Print response to stdout
  - Handle errors and print user-friendly messages
  - _Requirements: 10.4_

- [ ] 12.5 Implement REPL mode
  - Create `cli/src/repl.rs` with `run(agent: Agent) -> Result<()>` function
  - Use rustyline for line editing and history
  - Display prompt ">> " for user input
  - Process each line with agent.process
  - Display responses with colored output
  - Support "exit" command to quit
  - Show conversation history on request
  - _Requirements: 10.4, 10.5_

- [ ] 12.6 Implement main function
  - Create `cli/src/main.rs` with tokio main function
  - Parse command-line arguments with CliArgs
  - Load configuration from file
  - Initialize agent
  - Branch to single-turn or REPL mode based on query argument
  - Handle errors with anyhow and display user-friendly messages
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [ ] 12.7 Add error handling and logging
  - Use anyhow for error propagation in main and CLI functions
  - Add colored output for errors (red) and success (green)
  - Add verbose logging when --verbose flag is set
  - _Requirements: 10.3, 12.5_


- [ ] 13. Create example agents
- [ ] 13.1 Set up examples directory structure
  - Create examples directory in workspace root
  - Create examples/configs subdirectory for configuration files
  - Update workspace Cargo.toml to include example binaries
  - _Requirements: 11.1_

- [ ] 13.2 Create simple chatbot example
  - Create `examples/chatbot.rs` with tokio main function
  - Load config from examples/configs/chatbot.yaml
  - Initialize agent with LLM and memory only (no tools)
  - Implement simple REPL loop for conversation
  - Add comments explaining what this example demonstrates
  - _Requirements: 11.1, 11.2, 11.3_

- [ ] 13.3 Create chatbot configuration file
  - Create `examples/configs/chatbot.yaml`
  - Configure OpenAI provider with gpt-3.5-turbo
  - Set temperature to 0.7 and max_tokens to 500
  - Use environment variable for API key
  - Add comments explaining each setting
  - _Requirements: 11.1_

- [ ] 13.4 Create research assistant example
  - Create `examples/research.rs` with tokio main function
  - Load config from examples/configs/research.yaml
  - Initialize agent with WebSearchStub and FileReader tools
  - Demonstrate multi-step research workflow
  - Add comments explaining tool usage
  - _Requirements: 11.1, 11.2, 11.3_

- [ ] 13.5 Create research assistant configuration file
  - Create `examples/configs/research.yaml`
  - Configure with tools enabled (web_search, file_reader)
  - Set higher token budget for longer conversations
  - Add comments explaining research-specific settings
  - _Requirements: 11.1_

- [ ] 13.6 Create file manager example
  - Create `examples/file_manager.rs` with tokio main function
  - Load config from examples/configs/file_manager.yaml
  - Initialize agent with FileReader tool
  - Add FilePathGuardrail restricting to specific directory
  - Demonstrate safe file operations
  - Add comments explaining guardrail usage
  - _Requirements: 11.1, 11.2, 11.3_

- [ ] 13.7 Create file manager configuration file
  - Create `examples/configs/file_manager.yaml`
  - Configure with file_reader tool enabled
  - Specify allowed file paths in guardrails section
  - Add comments explaining safety settings
  - _Requirements: 11.1_

- [ ] 13.8 Create README for examples
  - Create `examples/README.md` explaining each example
  - Include instructions for running each example
  - Document required environment variables (API keys)
  - Show expected interactions and outputs
  - _Requirements: 11.4_


- [ ] 14. Add integration tests and documentation
- [ ] 14.1 Create integration test infrastructure
  - Create tests directory at workspace root
  - Create `tests/common/mod.rs` for shared test utilities
  - Implement MockLLM provider for testing
  - Implement test fixtures for common scenarios
  - _Requirements: 11.5_

- [ ] 14.2 Write agent flow integration test
  - Create `tests/agent_flow.rs`
  - Test end-to-end agent execution with mocked LLM
  - Verify plan creation, validation, and execution
  - Test with multi-step plan including tool calls
  - Assert correct final response
  - _Requirements: 11.5_

- [ ] 14.3 Write tool execution integration test
  - Create `tests/tool_execution.rs`
  - Test tool registry with executor integration
  - Verify tools are called with correct parameters
  - Test error handling when tool execution fails
  - Test result storage in memory
  - _Requirements: 11.5_

- [ ] 14.4 Write guardrails integration test
  - Create `tests/guardrails_integration.rs`
  - Test that guardrails prevent invalid plans from executing
  - Verify FilePathGuardrail blocks unauthorized file access
  - Verify RateLimitGuardrail blocks excessive API calls
  - _Requirements: 11.5_

- [ ] 14.5 Update root README with comprehensive documentation
  - Add project overview and motivation
  - Include architecture diagram (ASCII or link to image)
  - Add getting started guide with installation steps
  - Document each crate's purpose and responsibilities
  - Add examples of basic usage
  - Include link to blog series
  - _Requirements: 1.4_

- [ ] 14.6 Add inline documentation to public APIs
  - Add rustdoc comments to all public traits, structs, and functions
  - Include examples in doc comments where helpful
  - Document error conditions and panics
  - Add module-level documentation for each crate
  - _Requirements: 12.2_

- [ ] 14.7 Generate and verify documentation
  - Run `cargo doc --open --no-deps` to generate docs
  - Verify all public APIs have documentation
  - Check that examples in doc comments compile
  - Fix any broken links or formatting issues
  - _Requirements: 12.2_

- [ ] 14.8 Create CONTRIBUTING.md
  - Document how to extend the framework
  - Explain how to add new LLM providers
  - Explain how to create custom tools
  - Explain how to implement custom guardrails and rules
  - Include code style guidelines
  - Add instructions for running tests
  - _Requirements: 1.4_

- [ ] 14.9 Run final quality checks
  - Run `cargo test --workspace` to verify all tests pass
  - Run `cargo clippy --workspace -- -D warnings` to check code quality
  - Run `cargo fmt --all` to format code
  - Build release binaries with `cargo build --release`
  - Verify all examples run successfully
  - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5_
