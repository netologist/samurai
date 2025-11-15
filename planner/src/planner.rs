use agent_core::{Message, Result};
use tools::{ToolInfo, ToolRegistry};
use crate::types::{Plan, Step};

/// The Planner orchestrates plan generation using LLM reasoning.
/// 
/// It takes a user goal and available tools, then uses the LLM to generate
/// a structured plan with steps that the executor can run.
pub struct Planner {
    llm: Box<dyn llm::LLMProvider>,
    memory: Box<dyn memory::MemoryStore>,
}

impl Planner {
    /// Creates a new Planner with the given LLM provider and memory store.
    /// 
    /// # Arguments
    /// * `llm` - The LLM provider to use for plan generation
    /// * `memory` - The memory store for conversation context
    /// 
    /// # Returns
    /// A new Planner instance
    pub fn new(llm: Box<dyn llm::LLMProvider>, memory: Box<dyn memory::MemoryStore>) -> Self {
        Self { llm, memory }
    }
    
    /// Builds a system prompt that instructs the LLM on how to generate plans.
    /// 
    /// The prompt includes:
    /// - Instructions on the expected JSON output format
    /// - Available tools with their descriptions and parameter schemas
    /// - Guidelines for creating effective plans
    /// 
    /// # Arguments
    /// * `available_tools` - List of tools the agent can use
    /// 
    /// # Returns
    /// A formatted system prompt string
    pub fn build_system_prompt(&self, available_tools: &[ToolInfo]) -> String {
        let mut prompt = String::from(
            "You are an AI planning assistant. Your job is to break down user goals into \
            executable steps. You must respond with a valid JSON object following this exact format:\n\n\
            {\n  \
              \"reasoning\": \"Your explanation of the plan\",\n  \
              \"steps\": [\n    \
                {\"type\": \"tool_call\", \"tool_name\": \"tool_name\", \"parameters\": {...}},\n    \
                {\"type\": \"reasoning\", \"text\": \"explanation\"},\n    \
                {\"type\": \"response\", \"text\": \"final response to user\"}\n  \
              ]\n\
            }\n\n"
        );
        
        if available_tools.is_empty() {
            prompt.push_str("No tools are available. You can only use reasoning and response steps.\n\n");
        } else {
            prompt.push_str("Available tools:\n\n");
            
            for tool in available_tools {
                prompt.push_str(&format!("- **{}**: {}\n", tool.name, tool.description));
                prompt.push_str(&format!("  Parameters schema: {}\n\n", 
                    serde_json::to_string_pretty(&tool.parameters_schema).unwrap_or_default()));
            }
        }
        
        prompt.push_str(
            "\nGuidelines:\n\
            1. Break complex goals into simple, sequential steps\n\
            2. Use tool_call steps to invoke tools with proper parameters\n\
            3. Use reasoning steps to explain your thought process\n\
            4. End with a response step that answers the user's question\n\
            5. Ensure all tool names match exactly the available tools\n\
            6. Validate that parameters match the tool's schema\n\n\
            Remember: Respond ONLY with valid JSON. Do not include any other text."
        );
        
        prompt
    }
    
    /// Creates a plan for achieving the given goal.
    /// 
    /// This method:
    /// 1. Builds a system prompt with available tools
    /// 2. Creates a message array with the system prompt and user goal
    /// 3. Calls the LLM to generate a plan
    /// 4. Parses the LLM response into a structured Plan
    /// 
    /// # Arguments
    /// * `goal` - The user's goal or request
    /// * `available_tools` - List of tools the agent can use
    /// 
    /// # Returns
    /// * `Result<Plan>` - The generated plan or an error
    pub async fn create_plan(&self, goal: &str, available_tools: &[ToolInfo]) -> Result<Plan> {
        // Build the system prompt with available tools
        let system_prompt = self.build_system_prompt(available_tools);
        
        // Create messages array with system prompt and user goal
        let messages = vec![
            Message::system(&system_prompt),
            Message::user(goal),
        ];
        
        // Call LLM to generate plan
        let response = self.llm.send_message(&messages).await?;
        
        // Parse the response into a Plan
        self.parse_plan(&response)
    }
    
    /// Parses an LLM response into a structured Plan.
    /// 
    /// The response is expected to be a JSON object with:
    /// - `reasoning`: String explaining the plan
    /// - `steps`: Array of step objects
    /// 
    /// This method handles parsing errors gracefully and provides
    /// helpful error messages when the LLM response is malformed.
    /// 
    /// # Arguments
    /// * `response` - The raw LLM response string
    /// 
    /// # Returns
    /// * `Result<Plan>` - The parsed plan or an error
    pub fn parse_plan(&self, response: &str) -> Result<Plan> {
        // Try to extract JSON from the response (LLMs sometimes add extra text)
        let json_str = self.extract_json(response)?;
        
        // Parse the JSON into a Plan
        let plan: Plan = serde_json::from_str(json_str)
            .map_err(|e| agent_core::AgentError::Planning(
                format!("Failed to parse plan JSON: {}. Response was: {}", e, json_str)
            ))?;
        
        Ok(plan)
    }
    
    /// Extracts JSON from a response that might contain extra text.
    /// 
    /// LLMs sometimes add explanatory text before or after the JSON.
    /// This method tries to find and extract just the JSON portion.
    /// 
    /// # Arguments
    /// * `response` - The raw response string
    /// 
    /// # Returns
    /// * `Result<&str>` - The extracted JSON string or an error
    fn extract_json<'a>(&self, response: &'a str) -> Result<&'a str> {
        let trimmed = response.trim();
        
        // If it starts with {, assume it's pure JSON
        if trimmed.starts_with('{') {
            return Ok(trimmed);
        }
        
        // Try to find JSON object boundaries
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                if end > start {
                    return Ok(&trimmed[start..=end]);
                }
            }
        }
        
        // If we can't find JSON, return an error
        Err(agent_core::AgentError::Planning(
            format!("Could not find valid JSON in response: {}", response)
        ))
    }
    
    /// Validates that a plan only references tools that exist in the registry.
    /// 
    /// This method checks all ToolCall steps in the plan and ensures that
    /// each referenced tool is available in the provided registry. This
    /// prevents runtime errors when the executor tries to invoke a tool
    /// that doesn't exist.
    /// 
    /// # Arguments
    /// * `plan` - The plan to validate
    /// * `registry` - The tool registry to check against
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if all tools exist, error otherwise
    pub fn validate_plan(&self, plan: &Plan, registry: &ToolRegistry) -> Result<()> {
        for step in &plan.steps {
            if let Step::ToolCall(tool_call) = step {
                // Check if the tool exists in the registry
                if registry.get(&tool_call.tool_name).is_none() {
                    return Err(agent_core::AgentError::Planning(
                        format!(
                            "Plan references unknown tool '{}'. Available tools: {}",
                            tool_call.tool_name,
                            registry.list_tools()
                                .iter()
                                .map(|t| t.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    ));
                }
            }
        }
        
        Ok(())
    }
}
