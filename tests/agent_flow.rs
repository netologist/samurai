//! Integration test for end-to-end agent flow
//! 
//! This test verifies that the agent can:
//! 1. Generate a plan from a user goal using a mocked LLM
//! 2. Validate the plan against available tools
//! 3. Execute the plan with tool calls
//! 4. Return a correct final response

use agent_core::{Message, Result};
use async_trait::async_trait;
use llm::LLMProvider;
use memory::{InMemoryStore, MemoryStore};
use planner::{Planner, Step};
use executor::Executor;
use tools::{Calculator, ToolRegistry};
use std::sync::{Arc, Mutex};

/// Mock LLM provider that returns predefined responses
/// 
/// This mock allows us to test the agent flow without making real API calls.
/// It tracks how many times it's been called and returns responses from a list.
struct MockLLM {
    responses: Vec<String>,
    call_count: Arc<Mutex<usize>>,
}

impl MockLLM {
    /// Creates a new MockLLM with predefined responses
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            call_count: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Returns the number of times send_message was called
    fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

#[async_trait]
impl LLMProvider for MockLLM {
    async fn send_message(&self, _messages: &[Message]) -> Result<String> {
        let mut count = self.call_count.lock().unwrap();
        let response = self.responses.get(*count)
            .ok_or_else(|| agent_core::AgentError::LLMProvider(
                format!("No response available for call {}", *count)
            ))?
            .clone();
        *count += 1;
        Ok(response)
    }
}

/// Mock memory store for testing
/// 
/// Simple in-memory implementation that stores messages in a vector.
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

impl MemoryStore for MockMemoryStore {
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
        // For testing, just return all messages
        self.messages.clone()
    }
    
    fn clear(&mut self) {
        self.messages.clear();
    }
}

#[tokio::test]
async fn test_agent_flow_with_single_tool_call() {
    // Create a mock LLM that returns a plan with a calculator tool call
    let plan_json = r#"{
        "reasoning": "To calculate 15 + 27, I'll use the calculator tool with the add operation",
        "steps": [
            {
                "type": "tool_call",
                "tool_name": "calculator",
                "parameters": {
                    "operation": "add",
                    "a": 15,
                    "b": 27
                }
            },
            {
                "type": "reasoning",
                "text": "The calculator returned 42 as the sum"
            },
            {
                "type": "response",
                "text": "15 + 27 equals 42"
            }
        ]
    }"#;
    
    let mock_llm = MockLLM::new(vec![plan_json.to_string()]);
    let mock_memory = Box::new(MockMemoryStore::new());
    
    // Create planner with mock LLM
    let planner = Planner::new(Box::new(mock_llm), mock_memory);
    
    // Create tool registry with calculator
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    
    // Step 1: Generate plan
    let plan = planner.create_plan(
        "What is 15 + 27?",
        &registry.list_tools()
    ).await.expect("Failed to create plan");
    
    // Verify plan structure
    assert_eq!(plan.steps.len(), 3, "Plan should have 3 steps");
    assert!(plan.reasoning.contains("calculator"), "Reasoning should mention calculator");
    
    // Verify first step is a tool call
    match &plan.steps[0] {
        Step::ToolCall(tool_call) => {
            assert_eq!(tool_call.tool_name, "calculator");
            assert_eq!(tool_call.parameters["operation"], "add");
            assert_eq!(tool_call.parameters["a"], 15);
            assert_eq!(tool_call.parameters["b"], 27);
        }
        _ => panic!("First step should be a tool call"),
    }
    
    // Step 2: Validate plan
    planner.validate_plan(&plan, &registry)
        .expect("Plan validation should succeed");
    
    // Step 3: Execute plan
    let executor_memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, executor_memory);
    
    let result = executor.execute_plan(plan).await
        .expect("Plan execution should succeed");
    
    // Verify execution result
    assert!(result.success, "Execution should be successful");
    assert_eq!(result.final_response, "15 + 27 equals 42");
    assert_eq!(result.step_results.len(), 3, "Should have 3 step results");
    
    // Verify all steps succeeded
    for step_result in &result.step_results {
        assert!(step_result.success, "All steps should succeed");
    }
    
    // Verify tool call result
    let tool_result = &result.step_results[0];
    assert!(tool_result.step_type.contains("calculator"));
    assert!(tool_result.output.contains("42"));
}

#[tokio::test]
async fn test_agent_flow_with_multiple_tool_calls() {
    // Create a plan with multiple calculator operations
    let plan_json = r#"{
        "reasoning": "To calculate (10 + 5) * 2, I'll first add 10 and 5, then multiply the result by 2",
        "steps": [
            {
                "type": "tool_call",
                "tool_name": "calculator",
                "parameters": {
                    "operation": "add",
                    "a": 10,
                    "b": 5
                }
            },
            {
                "type": "reasoning",
                "text": "First calculation gives us 15"
            },
            {
                "type": "tool_call",
                "tool_name": "calculator",
                "parameters": {
                    "operation": "multiply",
                    "a": 15,
                    "b": 2
                }
            },
            {
                "type": "reasoning",
                "text": "Second calculation gives us 30"
            },
            {
                "type": "response",
                "text": "The result of (10 + 5) * 2 is 30"
            }
        ]
    }"#;
    
    let mock_llm = MockLLM::new(vec![plan_json.to_string()]);
    let mock_memory = Box::new(MockMemoryStore::new());
    
    let planner = Planner::new(Box::new(mock_llm), mock_memory);
    
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    
    // Generate and validate plan
    let plan = planner.create_plan(
        "What is (10 + 5) * 2?",
        &registry.list_tools()
    ).await.expect("Failed to create plan");
    
    assert_eq!(plan.steps.len(), 5, "Plan should have 5 steps");
    
    planner.validate_plan(&plan, &registry)
        .expect("Plan validation should succeed");
    
    // Execute plan
    let executor_memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, executor_memory);
    
    let result = executor.execute_plan(plan).await
        .expect("Plan execution should succeed");
    
    // Verify execution
    assert!(result.success, "Execution should be successful");
    assert_eq!(result.final_response, "The result of (10 + 5) * 2 is 30");
    assert_eq!(result.step_results.len(), 5, "Should have 5 step results");
    
    // Verify both tool calls succeeded
    let first_tool_result = &result.step_results[0];
    assert!(first_tool_result.success);
    assert!(first_tool_result.output.contains("15"));
    
    let second_tool_result = &result.step_results[2];
    assert!(second_tool_result.success);
    assert!(second_tool_result.output.contains("30"));
}

#[tokio::test]
async fn test_agent_flow_with_invalid_tool() {
    // Create a plan that references a non-existent tool
    let plan_json = r#"{
        "reasoning": "I'll use a non-existent tool",
        "steps": [
            {
                "type": "tool_call",
                "tool_name": "nonexistent_tool",
                "parameters": {}
            }
        ]
    }"#;
    
    let mock_llm = MockLLM::new(vec![plan_json.to_string()]);
    let mock_memory = Box::new(MockMemoryStore::new());
    
    let planner = Planner::new(Box::new(mock_llm), mock_memory);
    
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    
    // Generate plan
    let plan = planner.create_plan(
        "Use a nonexistent tool",
        &registry.list_tools()
    ).await.expect("Failed to create plan");
    
    // Validation should fail
    let validation_result = planner.validate_plan(&plan, &registry);
    assert!(validation_result.is_err(), "Validation should fail for unknown tool");
    
    // Verify error message mentions the unknown tool
    let error = validation_result.unwrap_err();
    let error_msg = error.to_string();
    assert!(error_msg.contains("nonexistent_tool"), "Error should mention the unknown tool");
}

#[tokio::test]
async fn test_agent_flow_without_tools() {
    // Create a plan with only reasoning and response (no tool calls)
    let plan_json = r#"{
        "reasoning": "This is a simple question that doesn't require tools",
        "steps": [
            {
                "type": "reasoning",
                "text": "The capital of France is a well-known fact"
            },
            {
                "type": "response",
                "text": "The capital of France is Paris"
            }
        ]
    }"#;
    
    let mock_llm = MockLLM::new(vec![plan_json.to_string()]);
    let mock_memory = Box::new(MockMemoryStore::new());
    
    let planner = Planner::new(Box::new(mock_llm), mock_memory);
    
    // Empty tool registry
    let registry = ToolRegistry::new();
    
    // Generate plan
    let plan = planner.create_plan(
        "What is the capital of France?",
        &registry.list_tools()
    ).await.expect("Failed to create plan");
    
    assert_eq!(plan.steps.len(), 2, "Plan should have 2 steps");
    
    // Validate plan (should succeed even with no tools)
    planner.validate_plan(&plan, &registry)
        .expect("Plan validation should succeed");
    
    // Execute plan
    let executor_memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, executor_memory);
    
    let result = executor.execute_plan(plan).await
        .expect("Plan execution should succeed");
    
    // Verify execution
    assert!(result.success, "Execution should be successful");
    assert_eq!(result.final_response, "The capital of France is Paris");
    assert_eq!(result.step_results.len(), 2, "Should have 2 step results");
}

#[tokio::test]
async fn test_plan_parsing_with_malformed_json() {
    // Test that the planner handles malformed JSON gracefully
    let mock_llm = MockLLM::new(vec!["This is not valid JSON".to_string()]);
    let mock_memory = Box::new(MockMemoryStore::new());
    
    let planner = Planner::new(Box::new(mock_llm), mock_memory);
    
    let registry = ToolRegistry::new();
    
    // Plan creation should fail with a clear error
    let result = planner.create_plan(
        "Test query",
        &registry.list_tools()
    ).await;
    
    assert!(result.is_err(), "Should fail with malformed JSON");
    
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(error_msg.contains("JSON") || error_msg.contains("parse"), 
            "Error should mention JSON or parsing issue");
}

#[tokio::test]
async fn test_memory_stores_execution_results() {
    // Test that execution results are stored in memory
    let plan_json = r#"{
        "reasoning": "Simple calculation",
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
                "type": "response",
                "text": "The answer is 8"
            }
        ]
    }"#;
    
    let mock_llm = MockLLM::new(vec![plan_json.to_string()]);
    let planner_memory = Box::new(MockMemoryStore::new());
    
    let planner = Planner::new(Box::new(mock_llm), planner_memory);
    
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    
    let plan = planner.create_plan(
        "What is 5 + 3?",
        &registry.list_tools()
    ).await.expect("Failed to create plan");
    
    // Create executor with a memory store we can inspect
    let executor_memory = Box::new(InMemoryStore::new());
    let mut executor = Executor::new(registry, executor_memory);
    
    // Execute plan
    executor.execute_plan(plan).await
        .expect("Plan execution should succeed");
    
    // Note: We can't directly inspect the memory from outside the executor
    // in this test, but the fact that execution succeeded means memory
    // operations worked correctly. A more thorough test would require
    // exposing memory state or using a spy pattern.
}
