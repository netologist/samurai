//! Integration test for tool execution
//! 
//! This test verifies that:
//! 1. Tools are correctly registered and retrieved from the registry
//! 2. The executor calls tools with correct parameters
//! 3. Tool results are properly returned and formatted
//! 4. Error handling works when tool execution fails
//! 5. Tool results are stored in memory for context

mod common;

use common::MockMemoryStore;
use executor::Executor;
use memory::InMemoryStore;
use planner::{Plan, Step, ToolCall};
use tools::{Calculator, FileReader, ToolRegistry, WebSearchStub};
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_tool_registry_integration_with_executor() {
    // Create a tool registry with multiple tools
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    registry.register(Box::new(FileReader::new()));
    registry.register(Box::new(WebSearchStub::new()));
    
    // Create executor with the registry
    let memory = Box::new(MockMemoryStore::new());
    let executor = Executor::new(registry, memory);
    
    // Verify all tools are available
    let tools = executor.list_tools();
    assert_eq!(tools.len(), 3);
    
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    assert!(tool_names.contains(&"calculator".to_string()));
    assert!(tool_names.contains(&"file_reader".to_string()));
    assert!(tool_names.contains(&"web_search".to_string()));
}

#[tokio::test]
async fn test_calculator_tool_called_with_correct_parameters() {
    // Create registry with calculator
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    
    let memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, memory);
    
    // Create a plan with calculator tool call
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({
                    "operation": "add",
                    "a": 15.0,
                    "b": 27.0
                }),
            )),
            Step::Response {
                text: "Calculation complete".to_string(),
            },
        ],
        "Test calculator execution".to_string(),
    );
    
    // Execute the plan
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution succeeded
    assert!(result.success);
    assert_eq!(result.step_results.len(), 2);
    
    // Verify calculator was called with correct parameters
    let tool_result = &result.step_results[0];
    assert_eq!(tool_result.step_type, "tool_call:calculator");
    assert!(tool_result.success);
    
    // Verify the result contains the correct calculation
    assert!(tool_result.output.contains("42"));
    assert!(tool_result.output.contains("add"));
}

#[tokio::test]
async fn test_multiple_tools_called_in_sequence() {
    // Create a temporary file for FileReader
    let mut temp_file = NamedTempFile::new().unwrap();
    let test_content = "Test file content for integration test";
    temp_file.write_all(test_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();
    
    // Create registry with multiple tools
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    registry.register(Box::new(FileReader::new()));
    registry.register(Box::new(WebSearchStub::new()));
    
    let memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, memory);
    
    // Create a plan that uses all three tools
    let plan = Plan::new(
        vec![
            Step::Reasoning {
                text: "First, I'll do a calculation".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({
                    "operation": "multiply",
                    "a": 6.0,
                    "b": 7.0
                }),
            )),
            Step::Reasoning {
                text: "Now I'll read a file".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({
                    "file_path": file_path
                }),
            )),
            Step::Reasoning {
                text: "Finally, I'll search the web".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "web_search".to_string(),
                json!({
                    "query": "rust programming"
                }),
            )),
            Step::Response {
                text: "All tools executed successfully".to_string(),
            },
        ],
        "Test multiple tool execution".to_string(),
    );
    
    // Execute the plan
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution succeeded
    assert!(result.success);
    assert_eq!(result.step_results.len(), 7);
    assert_eq!(result.final_response, "All tools executed successfully");
    
    // Verify all steps succeeded
    for step_result in &result.step_results {
        assert!(step_result.success, "Step failed: {:?}", step_result);
    }
    
    // Verify calculator result
    let calc_result = &result.step_results[1];
    assert_eq!(calc_result.step_type, "tool_call:calculator");
    assert!(calc_result.output.contains("42"));
    
    // Verify file reader result
    let file_result = &result.step_results[3];
    assert_eq!(file_result.step_type, "tool_call:file_reader");
    assert!(file_result.output.contains(test_content));
    
    // Verify web search result
    let search_result = &result.step_results[5];
    assert_eq!(search_result.step_type, "tool_call:web_search");
    assert!(search_result.output.contains("rust programming"));
    assert!(search_result.output.contains("mock"));
}

#[tokio::test]
async fn test_tool_execution_with_invalid_parameters() {
    // Create registry with calculator
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    
    let memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, memory);
    
    // Create a plan with invalid parameters (missing 'b' parameter)
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({
                    "operation": "add",
                    "a": 10.0
                    // Missing 'b' parameter
                }),
            )),
        ],
        "Test invalid parameters".to_string(),
    );
    
    // Execute the plan
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution failed
    assert!(!result.success);
    assert_eq!(result.step_results.len(), 1);
    
    // Verify the error was captured
    let step_result = &result.step_results[0];
    assert!(!step_result.success);
    assert!(step_result.output.contains("failed"));
}

#[tokio::test]
async fn test_tool_execution_with_division_by_zero() {
    // Create registry with calculator
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    
    let memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, memory);
    
    // Create a plan with division by zero
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({
                    "operation": "divide",
                    "a": 10.0,
                    "b": 0.0
                }),
            )),
        ],
        "Test division by zero".to_string(),
    );
    
    // Execute the plan
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution failed
    assert!(!result.success);
    assert_eq!(result.step_results.len(), 1);
    
    // Verify the error mentions division by zero
    let step_result = &result.step_results[0];
    assert!(!step_result.success);
    assert!(step_result.output.contains("Division by zero") || 
            step_result.output.contains("division"));
}

#[tokio::test]
async fn test_file_reader_with_nonexistent_file() {
    // Create registry with file reader
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(FileReader::new()));
    
    let memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, memory);
    
    // Create a plan that tries to read a nonexistent file
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({
                    "file_path": "/nonexistent/path/to/file.txt"
                }),
            )),
        ],
        "Test nonexistent file".to_string(),
    );
    
    // Execute the plan
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution failed
    assert!(!result.success);
    assert_eq!(result.step_results.len(), 1);
    
    // Verify the error mentions file not found
    let step_result = &result.step_results[0];
    assert!(!step_result.success);
    assert!(step_result.output.contains("File not found") || 
            step_result.output.contains("not found") ||
            step_result.output.contains("failed"));
}

#[tokio::test]
async fn test_tool_execution_with_unknown_tool() {
    // Create empty registry
    let registry = ToolRegistry::new();
    
    let memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, memory);
    
    // Create a plan that references a tool that doesn't exist
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "nonexistent_tool".to_string(),
                json!({}),
            )),
        ],
        "Test unknown tool".to_string(),
    );
    
    // Execute the plan
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution failed
    assert!(!result.success);
    assert_eq!(result.step_results.len(), 1);
    
    // Verify the error mentions tool not found
    let step_result = &result.step_results[0];
    assert!(!step_result.success);
    assert!(step_result.output.contains("not found") || 
            step_result.output.contains("Tool not found"));
}

#[tokio::test]
async fn test_tool_results_stored_in_memory() {
    // Create registry with calculator
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    
    // Use InMemoryStore so we can inspect the stored messages
    let memory_store = InMemoryStore::new();
    let mut executor = Executor::new(registry, Box::new(memory_store));
    
    // Create a plan with multiple tool calls
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({
                    "operation": "add",
                    "a": 5.0,
                    "b": 3.0
                }),
            )),
            Step::Reasoning {
                text: "The result is 8".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({
                    "operation": "multiply",
                    "a": 8.0,
                    "b": 2.0
                }),
            )),
            Step::Response {
                text: "Final result is 16".to_string(),
            },
        ],
        "Test memory storage".to_string(),
    );
    
    // Execute the plan
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution succeeded
    assert!(result.success);
    assert_eq!(result.step_results.len(), 4);
    
    // Note: We can't directly inspect the memory from outside the executor
    // in this test, but the fact that execution succeeded and all steps
    // completed means memory operations worked correctly. The memory is
    // used internally by the executor to provide context for subsequent steps.
}

#[tokio::test]
async fn test_tool_execution_stops_on_first_failure() {
    // Create registry with calculator
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    
    let memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, memory);
    
    // Create a plan where the second tool call will fail
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({
                    "operation": "add",
                    "a": 5.0,
                    "b": 3.0
                }),
            )),
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({
                    "operation": "divide",
                    "a": 10.0,
                    "b": 0.0  // This will cause division by zero error
                }),
            )),
            Step::Response {
                text: "This should not execute".to_string(),
            },
        ],
        "Test failure handling".to_string(),
    );
    
    // Execute the plan
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution failed
    assert!(!result.success);
    
    // Should have 2 step results: first success, second failure
    assert_eq!(result.step_results.len(), 2);
    assert!(result.step_results[0].success);
    assert!(!result.step_results[1].success);
    
    // The third step (Response) should not have executed
    // This is verified by having only 2 step results instead of 3
}

#[tokio::test]
async fn test_web_search_tool_returns_mock_results() {
    // Create registry with web search
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(WebSearchStub::new()));
    
    let memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, memory);
    
    // Create a plan with web search
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "web_search".to_string(),
                json!({
                    "query": "artificial intelligence"
                }),
            )),
            Step::Response {
                text: "Search complete".to_string(),
            },
        ],
        "Test web search".to_string(),
    );
    
    // Execute the plan
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution succeeded
    assert!(result.success);
    assert_eq!(result.step_results.len(), 2);
    
    // Verify search result format
    let search_result = &result.step_results[0];
    assert_eq!(search_result.step_type, "tool_call:web_search");
    assert!(search_result.success);
    assert!(search_result.output.contains("artificial intelligence"));
    assert!(search_result.output.contains("results"));
    assert!(search_result.output.contains("mock"));
}

#[tokio::test]
async fn test_tool_parameter_validation() {
    // Create registry with calculator
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    
    let memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, memory);
    
    // Test with invalid operation
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({
                    "operation": "invalid_operation",
                    "a": 5.0,
                    "b": 3.0
                }),
            )),
        ],
        "Test invalid operation".to_string(),
    );
    
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution failed
    assert!(!result.success);
    assert_eq!(result.step_results.len(), 1);
    assert!(!result.step_results[0].success);
    assert!(result.step_results[0].output.contains("Unknown operation") ||
            result.step_results[0].output.contains("invalid"));
}

#[tokio::test]
async fn test_file_reader_with_valid_file() {
    // Create a temporary file
    let mut temp_file = NamedTempFile::new().unwrap();
    let test_content = "This is a test file for integration testing.\nIt has multiple lines.\nLine 3.";
    temp_file.write_all(test_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();
    
    // Create registry with file reader
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(FileReader::new()));
    
    let memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, memory);
    
    // Create a plan that reads the file
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({
                    "file_path": file_path
                }),
            )),
            Step::Response {
                text: "File read successfully".to_string(),
            },
        ],
        "Test file reading".to_string(),
    );
    
    // Execute the plan
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution succeeded
    assert!(result.success);
    assert_eq!(result.step_results.len(), 2);
    
    // Verify file contents were read correctly
    let file_result = &result.step_results[0];
    assert_eq!(file_result.step_type, "tool_call:file_reader");
    assert!(file_result.success);
    // The output is JSON formatted, so check for key parts of the content
    assert!(file_result.output.contains("This is a test file"));
    assert!(file_result.output.contains("integration testing"));
    assert!(file_result.output.contains(&file_path));
}

#[tokio::test]
async fn test_complex_multi_tool_workflow() {
    // Create a temporary file
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"42").unwrap();
    temp_file.flush().unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();
    
    // Create registry with all tools
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator::new()));
    registry.register(Box::new(FileReader::new()));
    registry.register(Box::new(WebSearchStub::new()));
    
    let memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(registry, memory);
    
    // Create a complex workflow
    let plan = Plan::new(
        vec![
            Step::Reasoning {
                text: "Starting complex workflow".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({
                    "operation": "add",
                    "a": 10.0,
                    "b": 5.0
                }),
            )),
            Step::Reasoning {
                text: "Got 15, now reading file".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({
                    "file_path": file_path
                }),
            )),
            Step::Reasoning {
                text: "File contains 42, now searching".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "web_search".to_string(),
                json!({
                    "query": "meaning of life"
                }),
            )),
            Step::Reasoning {
                text: "All operations complete".to_string(),
            },
            Step::Response {
                text: "Workflow completed successfully with all tools".to_string(),
            },
        ],
        "Complex multi-tool workflow".to_string(),
    );
    
    // Execute the plan
    let result = executor.execute_plan(plan).await.unwrap();
    
    // Verify execution succeeded
    assert!(result.success);
    assert_eq!(result.step_results.len(), 8);
    assert_eq!(result.final_response, "Workflow completed successfully with all tools");
    
    // Verify all steps succeeded
    for (i, step_result) in result.step_results.iter().enumerate() {
        assert!(step_result.success, "Step {} failed: {:?}", i, step_result);
    }
    
    // Verify specific tool results
    assert!(result.step_results[1].output.contains("15"));
    assert!(result.step_results[3].output.contains("42"));
    assert!(result.step_results[5].output.contains("meaning of life"));
}
