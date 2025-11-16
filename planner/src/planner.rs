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


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::types::ToolCall;
    use async_trait::async_trait;
    
    /// Mock LLM provider for testing
    struct MockLLM {
        responses: Vec<String>,
        call_index: std::sync::Arc<std::sync::Mutex<usize>>,
    }
    
    impl MockLLM {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses,
                call_index: std::sync::Arc::new(std::sync::Mutex::new(0)),
            }
        }
    }
    
    #[async_trait]
    impl llm::LLMProvider for MockLLM {
        async fn send_message(&self, _messages: &[Message]) -> Result<String> {
            let mut index = self.call_index.lock().unwrap();
            let response = self.responses.get(*index)
                .ok_or_else(|| agent_core::AgentError::LLMProvider(
                    format!("No response available for call {}", *index)
                ))?
                .clone();
            *index += 1;
            Ok(response)
        }
    }
    
    /// Mock memory store for testing
    struct MockMemoryStore {
        messages: Vec<Message>,
    }
    
    impl MockMemoryStore {
        fn new() -> Self {
            Self {
                messages: Vec::new(),
            }
        }
    }
    
    impl memory::MemoryStore for MockMemoryStore {
        fn add_message(&mut self, message: Message) {
            self.messages.push(message);
        }
        
        fn get_recent(&self, limit: usize) -> Vec<Message> {
            self.messages.iter()
                .rev()
                .take(limit)
                .rev()
                .cloned()
                .collect()
        }
        
        fn get_within_budget(&self, _token_budget: usize) -> Vec<Message> {
            self.messages.clone()
        }
        
        fn clear(&mut self) {
            self.messages.clear();
        }
    }
    
    #[test]
    fn test_parse_plan_with_valid_json() {
        // Test parsing a valid JSON plan response
        let planner = create_test_planner(vec![]);
        
        let json_response = r#"{
            "reasoning": "This is a test plan",
            "steps": [
                {
                    "type": "tool_call",
                    "tool_name": "calculator",
                    "parameters": {
                        "operation": "add",
                        "a": 5,
                        "b": 3
                    }
                },
                {
                    "type": "reasoning",
                    "text": "The calculation gives us 8"
                },
                {
                    "type": "response",
                    "text": "The answer is 8"
                }
            ]
        }"#;
        
        let plan = planner.parse_plan(json_response)
            .expect("Should parse valid JSON");
        
        assert_eq!(plan.reasoning, "This is a test plan");
        assert_eq!(plan.steps.len(), 3);
        
        // Verify first step is a tool call
        match &plan.steps[0] {
            Step::ToolCall(tool_call) => {
                assert_eq!(tool_call.tool_name, "calculator");
                assert_eq!(tool_call.parameters["operation"], "add");
                assert_eq!(tool_call.parameters["a"], 5);
                assert_eq!(tool_call.parameters["b"], 3);
            }
            _ => panic!("First step should be a tool call"),
        }
        
        // Verify second step is reasoning
        match &plan.steps[1] {
            Step::Reasoning { text } => {
                assert_eq!(text, "The calculation gives us 8");
            }
            _ => panic!("Second step should be reasoning"),
        }
        
        // Verify third step is response
        match &plan.steps[2] {
            Step::Response { text } => {
                assert_eq!(text, "The answer is 8");
            }
            _ => panic!("Third step should be response"),
        }
    }
    
    #[test]
    fn test_parse_plan_with_json_wrapped_in_text() {
        // Test that the planner can extract JSON from responses with extra text
        let planner = create_test_planner(vec![]);
        
        let response_with_extra_text = r#"
        Here's the plan I've created:
        
        {
            "reasoning": "Simple plan",
            "steps": [
                {
                    "type": "response",
                    "text": "Done"
                }
            ]
        }
        
        I hope this helps!
        "#;
        
        let plan = planner.parse_plan(response_with_extra_text)
            .expect("Should extract and parse JSON from text");
        
        assert_eq!(plan.reasoning, "Simple plan");
        assert_eq!(plan.steps.len(), 1);
    }
    
    #[test]
    fn test_parse_plan_with_malformed_json() {
        // Test that parsing fails gracefully with malformed JSON
        let planner = create_test_planner(vec![]);
        
        let malformed_json = "This is not valid JSON at all";
        
        let result = planner.parse_plan(malformed_json);
        assert!(result.is_err(), "Should fail with malformed JSON");
        
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("JSON") || error_msg.contains("parse"),
                "Error should mention JSON or parsing: {}", error_msg);
    }
    
    #[test]
    fn test_parse_plan_with_missing_fields() {
        // Test parsing JSON that's missing required fields
        let planner = create_test_planner(vec![]);
        
        let incomplete_json = r#"{
            "reasoning": "Missing steps field"
        }"#;
        
        let result = planner.parse_plan(incomplete_json);
        assert!(result.is_err(), "Should fail with missing fields");
    }
    
    #[test]
    fn test_validate_plan_with_valid_tools() {
        // Test that validation succeeds when all tools exist
        let planner = create_test_planner(vec![]);
        
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(tools::Calculator::new()));
        
        let plan = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "add", "a": 1, "b": 2})
                )),
                Step::Response { text: "Result is 3".to_string() },
            ],
            "Test plan".to_string(),
        );
        
        let result = planner.validate_plan(&plan, &registry);
        assert!(result.is_ok(), "Validation should succeed with valid tools");
    }
    
    #[test]
    fn test_validate_plan_with_invalid_tool() {
        // Test that validation fails when a tool doesn't exist
        let planner = create_test_planner(vec![]);
        
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(tools::Calculator::new()));
        
        let plan = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new(
                    "nonexistent_tool".to_string(),
                    json!({})
                )),
            ],
            "Test plan with invalid tool".to_string(),
        );
        
        let result = planner.validate_plan(&plan, &registry);
        assert!(result.is_err(), "Validation should fail with unknown tool");
        
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("nonexistent_tool"),
                "Error should mention the unknown tool: {}", error_msg);
        assert!(error_msg.contains("calculator"),
                "Error should list available tools: {}", error_msg);
    }
    
    #[test]
    fn test_validate_plan_with_multiple_invalid_tools() {
        // Test validation with multiple invalid tool references
        let planner = create_test_planner(vec![]);
        
        let registry = ToolRegistry::new(); // Empty registry
        
        let plan = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new(
                    "tool1".to_string(),
                    json!({})
                )),
                Step::ToolCall(ToolCall::new(
                    "tool2".to_string(),
                    json!({})
                )),
            ],
            "Test plan".to_string(),
        );
        
        let result = planner.validate_plan(&plan, &registry);
        assert!(result.is_err(), "Validation should fail on first invalid tool");
        
        // Should fail on the first invalid tool
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("tool1"),
                "Error should mention the first invalid tool: {}", error_msg);
    }
    
    #[test]
    fn test_validate_plan_without_tool_calls() {
        // Test that validation succeeds for plans with no tool calls
        let planner = create_test_planner(vec![]);
        
        let registry = ToolRegistry::new(); // Empty registry
        
        let plan = Plan::new(
            vec![
                Step::Reasoning { text: "Thinking...".to_string() },
                Step::Response { text: "Answer".to_string() },
            ],
            "Plan without tools".to_string(),
        );
        
        let result = planner.validate_plan(&plan, &registry);
        assert!(result.is_ok(), "Validation should succeed without tool calls");
    }
    
    #[test]
    fn test_build_system_prompt_with_no_tools() {
        // Test system prompt generation when no tools are available
        let planner = create_test_planner(vec![]);
        
        let tools: Vec<ToolInfo> = vec![];
        let prompt = planner.build_system_prompt(&tools);
        
        // Verify prompt contains basic instructions
        assert!(prompt.contains("planning assistant"),
                "Prompt should identify role");
        assert!(prompt.contains("JSON"),
                "Prompt should mention JSON format");
        assert!(prompt.contains("No tools are available"),
                "Prompt should indicate no tools");
        assert!(prompt.contains("reasoning") && prompt.contains("response"),
                "Prompt should mention available step types");
    }
    
    #[test]
    fn test_build_system_prompt_with_single_tool() {
        // Test system prompt generation with one tool
        let planner = create_test_planner(vec![]);
        
        let tools = vec![
            ToolInfo {
                name: "calculator".to_string(),
                description: "Performs arithmetic operations".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {"type": "string"},
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    }
                }),
            }
        ];
        
        let prompt = planner.build_system_prompt(&tools);
        
        // Verify prompt includes tool information
        assert!(prompt.contains("calculator"),
                "Prompt should include tool name");
        assert!(prompt.contains("arithmetic operations"),
                "Prompt should include tool description");
        assert!(prompt.contains("parameters_schema") || prompt.contains("operation"),
                "Prompt should include parameter schema");
        assert!(prompt.contains("Available tools"),
                "Prompt should have tools section");
    }
    
    #[test]
    fn test_build_system_prompt_with_multiple_tools() {
        // Test system prompt generation with multiple tools
        let planner = create_test_planner(vec![]);
        
        let tools = vec![
            ToolInfo {
                name: "calculator".to_string(),
                description: "Math operations".to_string(),
                parameters_schema: json!({"type": "object"}),
            },
            ToolInfo {
                name: "file_reader".to_string(),
                description: "Read files".to_string(),
                parameters_schema: json!({"type": "object"}),
            },
            ToolInfo {
                name: "web_search".to_string(),
                description: "Search the web".to_string(),
                parameters_schema: json!({"type": "object"}),
            },
        ];
        
        let prompt = planner.build_system_prompt(&tools);
        
        // Verify all tools are included
        assert!(prompt.contains("calculator"), "Should include calculator");
        assert!(prompt.contains("file_reader"), "Should include file_reader");
        assert!(prompt.contains("web_search"), "Should include web_search");
        
        // Verify all descriptions are included
        assert!(prompt.contains("Math operations"), "Should include calculator description");
        assert!(prompt.contains("Read files"), "Should include file_reader description");
        assert!(prompt.contains("Search the web"), "Should include web_search description");
    }
    
    #[test]
    fn test_build_system_prompt_includes_guidelines() {
        // Test that system prompt includes planning guidelines
        let planner = create_test_planner(vec![]);
        
        let tools = vec![];
        let prompt = planner.build_system_prompt(&tools);
        
        // Verify guidelines are present
        assert!(prompt.contains("Guidelines"),
                "Prompt should have guidelines section");
        assert!(prompt.contains("sequential"),
                "Should mention sequential execution");
        assert!(prompt.contains("tool names match"),
                "Should mention tool name matching");
        assert!(prompt.contains("valid JSON"),
                "Should emphasize JSON format");
    }
    
    #[tokio::test]
    async fn test_create_plan_with_mock_llm() {
        // Test end-to-end plan creation with mocked LLM
        let plan_json = r#"{
            "reasoning": "Use calculator to add numbers",
            "steps": [
                {
                    "type": "tool_call",
                    "tool_name": "calculator",
                    "parameters": {"operation": "add", "a": 10, "b": 20}
                },
                {
                    "type": "response",
                    "text": "The sum is 30"
                }
            ]
        }"#;
        
        let planner = create_test_planner(vec![plan_json.to_string()]);
        
        let tools = vec![
            ToolInfo {
                name: "calculator".to_string(),
                description: "Math operations".to_string(),
                parameters_schema: json!({"type": "object"}),
            }
        ];
        
        let plan = planner.create_plan("What is 10 + 20?", &tools)
            .await
            .expect("Should create plan successfully");
        
        assert_eq!(plan.reasoning, "Use calculator to add numbers");
        assert_eq!(plan.steps.len(), 2);
        
        // Verify tool call
        match &plan.steps[0] {
            Step::ToolCall(tool_call) => {
                assert_eq!(tool_call.tool_name, "calculator");
                assert_eq!(tool_call.parameters["operation"], "add");
                assert_eq!(tool_call.parameters["a"], 10);
                assert_eq!(tool_call.parameters["b"], 20);
            }
            _ => panic!("First step should be tool call"),
        }
    }
    
    #[tokio::test]
    async fn test_create_plan_with_llm_error() {
        // Test that plan creation handles LLM errors gracefully
        let planner = create_test_planner(vec![]); // No responses available
        
        let tools = vec![];
        
        let result = planner.create_plan("Test query", &tools).await;
        assert!(result.is_err(), "Should fail when LLM has no response");
    }
    
    #[test]
    fn test_extract_json_with_pure_json() {
        // Test JSON extraction when response is pure JSON
        let planner = create_test_planner(vec![]);
        
        let json = r#"{"key": "value"}"#;
        let extracted = planner.extract_json(json)
            .expect("Should extract pure JSON");
        
        assert_eq!(extracted, json);
    }
    
    #[test]
    fn test_extract_json_with_whitespace() {
        // Test JSON extraction with surrounding whitespace
        let planner = create_test_planner(vec![]);
        
        let json_with_whitespace = "  \n  {\"key\": \"value\"}  \n  ";
        let extracted = planner.extract_json(json_with_whitespace)
            .expect("Should extract JSON with whitespace");
        
        assert_eq!(extracted, r#"{"key": "value"}"#);
    }
    
    #[test]
    fn test_extract_json_with_surrounding_text() {
        // Test JSON extraction when embedded in text
        let planner = create_test_planner(vec![]);
        
        let text_with_json = "Here is the plan: {\"key\": \"value\"} Hope this helps!";
        let extracted = planner.extract_json(text_with_json)
            .expect("Should extract JSON from text");
        
        assert_eq!(extracted, r#"{"key": "value"}"#);
    }
    
    #[test]
    fn test_extract_json_with_no_json() {
        // Test that extraction fails when no JSON is present
        let planner = create_test_planner(vec![]);
        
        let no_json = "This text has no JSON in it";
        let result = planner.extract_json(no_json);
        
        assert!(result.is_err(), "Should fail when no JSON present");
        
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("JSON"),
                "Error should mention JSON: {}", error_msg);
    }
    
    // Helper function to create a test planner
    fn create_test_planner(responses: Vec<String>) -> Planner {
        let mock_llm = Box::new(MockLLM::new(responses));
        let mock_memory = Box::new(MockMemoryStore::new());
        Planner::new(mock_llm, mock_memory)
    }
}
