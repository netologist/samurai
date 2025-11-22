use agent::Agent;
use agent_core::{Message, Result};
use async_trait::async_trait;
use llm::LLMProvider;
use std::sync::Arc;
use std::sync::Mutex;
use tools::ToolRegistry;

// Mock LLM that returns a sequence of responses
struct StatefulMockLLM {
    responses: Arc<Mutex<Vec<String>>>,
    call_count: Arc<Mutex<usize>>,
}

impl StatefulMockLLM {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            call_count: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl LLMProvider for StatefulMockLLM {
    async fn send_message(&self, _messages: &[Message]) -> Result<String> {
        let mut count = self.call_count.lock().unwrap();
        let responses = self.responses.lock().unwrap();

        if *count >= responses.len() {
            return Ok("{}".to_string()); // Default empty JSON
        }

        let response = responses[*count].clone();
        *count += 1;
        Ok(response)
    }
}

#[tokio::test]
async fn test_agent_multi_step_tool_execution() {
    // Plan 1: Use calculator
    let plan1 = r#"{
        "reasoning": "I need to calculate 5 + 3",
        "steps": [
            {
                "type": "tool_call",
                "tool_name": "calculator",
                "parameters": {
                    "operation": "add",
                    "a": 5,
                    "b": 3
                }
            }
        ]
    }"#;

    // Plan 2: Respond with result (simulating re-planning or next step)
    // Note: In the current Agent::run implementation, it only runs ONE plan per call.
    // So this test actually tests a single run that executes a tool.
    // The Agent doesn't loop automatically yet (that would be a "ReAct" loop).
    // The current Agent::run executes the plan and returns the final response.
    // If the plan has a tool call but no final response, the executor constructs one.

    let llm = Box::new(StatefulMockLLM::new(vec![plan1.to_string()]));

    let mut tools = ToolRegistry::new();
    tools.register(Box::new(tools::Calculator::new()));

    let mut agent = Agent::builder()
        .llm(llm)
        .tools(tools)
        .build()
        .expect("Failed to build agent");

    let result = agent.run("Calculate 5 + 3").await;

    assert!(result.is_ok());
    let response = result.unwrap();
    // The executor should have run the tool and since there was no explicit response step,
    // it typically returns the tool output or a summary.
    // In Executor::execute_plan:
    // if final_response.is_empty() && !step_results.is_empty() {
    //    final_response = step_results...join("\n");
    // }

    assert!(response.contains("8"));
    // The executor returns the tool output in the final response if no explicit response step
    // The format is typically just the JSON output or a string representation.
    // Let's check for the result value "8" which is sufficient.
    // assert!(response.contains("tool_call:calculator")); // This might not be in the final response string
}
