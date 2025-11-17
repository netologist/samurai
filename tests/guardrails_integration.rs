//! Integration test for guardrails system
//! 
//! This test verifies that:
//! 1. Guardrails prevent invalid plans from executing
//! 2. FilePathGuardrail blocks unauthorized file access
//! 3. RateLimitGuardrail blocks excessive API calls
//! 4. GuardrailRegistry validates all registered guardrails
//! 5. Execution is prevented when guardrails are violated

mod common;

use common::{MockLLM, MockMemoryStore, fixtures};
use executor::Executor;
use guardrails::{FilePathGuardrail, GuardrailRegistry, RateLimitGuardrail};
use planner::{Plan, Planner, Step, ToolCall};
use serde_json::json;
use std::path::PathBuf;
use tools::{FileReader, ToolRegistry};

#[tokio::test]
async fn test_file_path_guardrail_blocks_unauthorized_access() {
    // Create a guardrail that only allows access to /tmp
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(FilePathGuardrail::new(vec![
        PathBuf::from("/tmp"),
    ])));

    // Create a plan that tries to read /etc/passwd (unauthorized)
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({
                    "file_path": "/etc/passwd"
                }),
            )),
            Step::Response {
                text: "File contents".to_string(),
            },
        ],
        "Attempting to read unauthorized file".to_string(),
    );

    // Validation should fail
    let result = guardrail_registry.validate_all(&plan);
    assert!(result.is_err(), "Should block unauthorized file access");

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("/etc/passwd") || error_msg.contains("not allowed"),
        "Error should mention the unauthorized path"
    );
}

#[tokio::test]
async fn test_file_path_guardrail_allows_authorized_access() {
    // Create a guardrail that allows access to /tmp
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(FilePathGuardrail::new(vec![
        PathBuf::from("/tmp"),
    ])));

    // Create a plan that reads from /tmp (authorized)
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({
                    "file_path": "/tmp/test.txt"
                }),
            )),
            Step::Response {
                text: "File contents".to_string(),
            },
        ],
        "Reading from authorized directory".to_string(),
    );

    // Validation should succeed
    let result = guardrail_registry.validate_all(&plan);
    assert!(result.is_ok(), "Should allow authorized file access");
}

#[tokio::test]
async fn test_file_path_guardrail_with_multiple_allowed_paths() {
    // Create a guardrail with multiple allowed paths
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(FilePathGuardrail::new(vec![
        PathBuf::from("/tmp"),
        PathBuf::from("/home/user/documents"),
    ])));

    // Test access to first allowed path
    let plan1 = Plan::new(
        vec![Step::ToolCall(ToolCall::new(
            "file_reader".to_string(),
            json!({"file_path": "/tmp/file1.txt"}),
        ))],
        "Test plan 1".to_string(),
    );
    assert!(guardrail_registry.validate_all(&plan1).is_ok());

    // Test access to second allowed path
    let plan2 = Plan::new(
        vec![Step::ToolCall(ToolCall::new(
            "file_reader".to_string(),
            json!({"file_path": "/home/user/documents/file2.txt"}),
        ))],
        "Test plan 2".to_string(),
    );
    assert!(guardrail_registry.validate_all(&plan2).is_ok());

    // Test access to disallowed path
    let plan3 = Plan::new(
        vec![Step::ToolCall(ToolCall::new(
            "file_reader".to_string(),
            json!({"file_path": "/etc/shadow"}),
        ))],
        "Test plan 3".to_string(),
    );
    assert!(guardrail_registry.validate_all(&plan3).is_err());
}

#[tokio::test]
async fn test_rate_limit_guardrail_blocks_excessive_calls() {
    // Create a guardrail with a low limit (3 calls per minute)
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(RateLimitGuardrail::new(3)));

    // Create a plan with 5 tool calls (exceeds limit)
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 1, "b": 1}),
            )),
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 2, "b": 2}),
            )),
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 3, "b": 3}),
            )),
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 4, "b": 4}),
            )),
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 5, "b": 5}),
            )),
        ],
        "Plan with too many tool calls".to_string(),
    );

    // Validation should fail
    let result = guardrail_registry.validate_all(&plan);
    assert!(result.is_err(), "Should block excessive API calls");

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Rate limit") || error_msg.contains("rate limit"),
        "Error should mention rate limit"
    );
}

#[tokio::test]
async fn test_rate_limit_guardrail_allows_within_limit() {
    // Create a guardrail with limit of 5 calls per minute
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(RateLimitGuardrail::new(5)));

    // Create a plan with 3 tool calls (within limit)
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 1, "b": 1}),
            )),
            Step::Reasoning {
                text: "First calculation done".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "multiply", "a": 2, "b": 3}),
            )),
            Step::Reasoning {
                text: "Second calculation done".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "subtract", "a": 10, "b": 5}),
            )),
            Step::Response {
                text: "All calculations complete".to_string(),
            },
        ],
        "Plan within rate limit".to_string(),
    );

    // Validation should succeed
    let result = guardrail_registry.validate_all(&plan);
    assert!(result.is_ok(), "Should allow calls within rate limit");
}

#[tokio::test]
async fn test_rate_limit_guardrail_accumulates_across_plans() {
    // Create a guardrail with limit of 5 calls per minute
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(RateLimitGuardrail::new(5)));

    // First plan with 2 calls - should pass
    let plan1 = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 1, "b": 1}),
            )),
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 2, "b": 2}),
            )),
        ],
        "First plan".to_string(),
    );
    assert!(
        guardrail_registry.validate_all(&plan1).is_ok(),
        "First plan should pass"
    );

    // Second plan with 2 calls - should pass (total 4)
    let plan2 = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 3, "b": 3}),
            )),
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 4, "b": 4}),
            )),
        ],
        "Second plan".to_string(),
    );
    assert!(
        guardrail_registry.validate_all(&plan2).is_ok(),
        "Second plan should pass"
    );

    // Third plan with 2 calls - should fail (would be 6 total, limit is 5)
    let plan3 = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 5, "b": 5}),
            )),
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 6, "b": 6}),
            )),
        ],
        "Third plan".to_string(),
    );
    assert!(
        guardrail_registry.validate_all(&plan3).is_err(),
        "Third plan should fail due to accumulated rate limit"
    );
}

#[tokio::test]
async fn test_multiple_guardrails_all_must_pass() {
    // Create registry with both file path and rate limit guardrails
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(FilePathGuardrail::new(vec![
        PathBuf::from("/tmp"),
    ])));
    guardrail_registry.register(Box::new(RateLimitGuardrail::new(3)));

    // Plan that passes file path but fails rate limit
    let plan1 = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"file_path": "/tmp/file1.txt"}),
            )),
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"file_path": "/tmp/file2.txt"}),
            )),
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"file_path": "/tmp/file3.txt"}),
            )),
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"file_path": "/tmp/file4.txt"}),
            )),
        ],
        "Too many file reads".to_string(),
    );
    assert!(
        guardrail_registry.validate_all(&plan1).is_err(),
        "Should fail rate limit check"
    );

    // Create a fresh registry for next test
    let mut guardrail_registry2 = GuardrailRegistry::new();
    guardrail_registry2.register(Box::new(FilePathGuardrail::new(vec![
        PathBuf::from("/tmp"),
    ])));
    guardrail_registry2.register(Box::new(RateLimitGuardrail::new(3)));

    // Plan that passes rate limit but fails file path
    let plan2 = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"file_path": "/etc/passwd"}),
            )),
        ],
        "Unauthorized file access".to_string(),
    );
    assert!(
        guardrail_registry2.validate_all(&plan2).is_err(),
        "Should fail file path check"
    );

    // Create a fresh registry for next test
    let mut guardrail_registry3 = GuardrailRegistry::new();
    guardrail_registry3.register(Box::new(FilePathGuardrail::new(vec![
        PathBuf::from("/tmp"),
    ])));
    guardrail_registry3.register(Box::new(RateLimitGuardrail::new(3)));

    // Plan that passes both guardrails
    let plan3 = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"file_path": "/tmp/file.txt"}),
            )),
            Step::Response {
                text: "File read successfully".to_string(),
            },
        ],
        "Valid plan".to_string(),
    );
    assert!(
        guardrail_registry3.validate_all(&plan3).is_ok(),
        "Should pass all guardrails"
    );
}

#[tokio::test]
async fn test_guardrails_prevent_plan_execution() {
    // Create a plan that violates file path guardrail
    let unauthorized_plan = fixtures::file_reader_plan("/etc/passwd");

    let mock_llm = MockLLM::new(vec![unauthorized_plan]);
    let mock_memory = Box::new(MockMemoryStore::new());

    let planner = Planner::new(Box::new(mock_llm), mock_memory);

    let mut registry = ToolRegistry::new();
    registry.register(Box::new(FileReader::new()));

    // Generate plan
    let plan = planner
        .create_plan("Read /etc/passwd", &registry.list_tools())
        .await
        .expect("Failed to create plan");

    // Create guardrail registry
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(FilePathGuardrail::new(vec![
        PathBuf::from("/tmp"),
    ])));

    // Validation should fail
    let validation_result = guardrail_registry.validate_all(&plan);
    assert!(
        validation_result.is_err(),
        "Guardrail should block unauthorized file access"
    );

    // Verify that we don't execute the plan when validation fails
    // In a real application, you would check validation before calling execute_plan
    // Here we just verify the validation failed as expected
    let error = validation_result.unwrap_err();
    assert!(error.to_string().contains("not allowed"));
}

#[tokio::test]
async fn test_guardrails_with_valid_plan_allows_execution() {
    // Create a plan that passes guardrails
    let authorized_plan = fixtures::file_reader_plan("/tmp/test.txt");

    let mock_llm = MockLLM::new(vec![authorized_plan]);
    let mock_memory = Box::new(MockMemoryStore::new());

    let planner = Planner::new(Box::new(mock_llm), mock_memory);

    let mut tool_registry = ToolRegistry::new();
    tool_registry.register(Box::new(FileReader::new()));

    // Generate plan
    let plan = planner
        .create_plan("Read /tmp/test.txt", &tool_registry.list_tools())
        .await
        .expect("Failed to create plan");

    // Create guardrail registry
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(FilePathGuardrail::new(vec![
        PathBuf::from("/tmp"),
    ])));
    guardrail_registry.register(Box::new(RateLimitGuardrail::new(10)));

    // Validation should succeed
    let validation_result = guardrail_registry.validate_all(&plan);
    assert!(
        validation_result.is_ok(),
        "Guardrails should allow valid plan"
    );

    // Now we can safely execute the plan
    // (execution will fail because /tmp/test.txt doesn't exist, but that's a tool error, not a guardrail error)
    let executor_memory = Box::new(MockMemoryStore::new());
    let mut executor = Executor::new(tool_registry, executor_memory);

    let execution_result = executor.execute_plan(plan).await;
    assert!(
        execution_result.is_ok(),
        "Execution should proceed after guardrail validation passes"
    );

    // The execution itself may fail due to file not existing, but that's expected
    // The important thing is that guardrails allowed it to proceed
}

#[tokio::test]
async fn test_file_path_guardrail_ignores_non_file_reader_tools() {
    // Create a guardrail that only allows /tmp
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(FilePathGuardrail::new(vec![
        PathBuf::from("/tmp"),
    ])));

    // Create a plan with calculator (not file_reader)
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({
                    "operation": "add",
                    "a": 10,
                    "b": 20
                }),
            )),
            Step::Response {
                text: "Result is 30".to_string(),
            },
        ],
        "Calculator operation".to_string(),
    );

    // Should pass because file path guardrail only checks file_reader
    let result = guardrail_registry.validate_all(&plan);
    assert!(
        result.is_ok(),
        "File path guardrail should ignore non-file_reader tools"
    );
}

#[tokio::test]
async fn test_rate_limit_guardrail_ignores_non_tool_steps() {
    // Create a guardrail with limit of 2 calls
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(RateLimitGuardrail::new(2)));

    // Create a plan with many steps but only 2 tool calls
    let plan = Plan::new(
        vec![
            Step::Reasoning {
                text: "Starting calculation".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 1, "b": 1}),
            )),
            Step::Reasoning {
                text: "First calculation done".to_string(),
            },
            Step::Reasoning {
                text: "Preparing second calculation".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "multiply", "a": 2, "b": 3}),
            )),
            Step::Reasoning {
                text: "Second calculation done".to_string(),
            },
            Step::Response {
                text: "All done".to_string(),
            },
        ],
        "Plan with many steps but few tool calls".to_string(),
    );

    // Should pass because only 2 tool calls despite 7 total steps
    let result = guardrail_registry.validate_all(&plan);
    assert!(
        result.is_ok(),
        "Rate limit should only count tool calls, not all steps"
    );
}

#[tokio::test]
async fn test_empty_guardrail_registry_allows_all_plans() {
    // Create an empty guardrail registry
    let guardrail_registry = GuardrailRegistry::new();

    // Create a plan that would violate guardrails if they were registered
    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"file_path": "/etc/passwd"}),
            )),
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 1, "b": 1}),
            )),
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 2, "b": 2}),
            )),
        ],
        "Plan with no guardrails".to_string(),
    );

    // Should pass because no guardrails are registered
    let result = guardrail_registry.validate_all(&plan);
    assert!(
        result.is_ok(),
        "Empty guardrail registry should allow all plans"
    );
}

#[tokio::test]
async fn test_guardrail_validation_with_complex_plan() {
    // Create registry with both guardrails
    let mut guardrail_registry = GuardrailRegistry::new();
    guardrail_registry.register(Box::new(FilePathGuardrail::new(vec![
        PathBuf::from("/tmp"),
        PathBuf::from("/home/user/safe"),
    ])));
    guardrail_registry.register(Box::new(RateLimitGuardrail::new(10)));

    // Create a complex plan with multiple tool types
    let plan = Plan::new(
        vec![
            Step::Reasoning {
                text: "Starting complex workflow".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 5, "b": 10}),
            )),
            Step::Reasoning {
                text: "Got 15, now reading file".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"file_path": "/tmp/data.txt"}),
            )),
            Step::Reasoning {
                text: "File read, doing more calculations".to_string(),
            },
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "multiply", "a": 15, "b": 2}),
            )),
            Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"file_path": "/home/user/safe/config.txt"}),
            )),
            Step::Response {
                text: "Workflow complete".to_string(),
            },
        ],
        "Complex workflow".to_string(),
    );

    // Should pass all guardrails
    let result = guardrail_registry.validate_all(&plan);
    assert!(
        result.is_ok(),
        "Complex plan should pass all guardrail checks"
    );
}

#[tokio::test]
async fn test_guardrail_error_messages_are_descriptive() {
    // Test file path guardrail error message
    let mut file_guardrail_registry = GuardrailRegistry::new();
    file_guardrail_registry.register(Box::new(FilePathGuardrail::new(vec![
        PathBuf::from("/tmp"),
    ])));

    let file_plan = Plan::new(
        vec![Step::ToolCall(ToolCall::new(
            "file_reader".to_string(),
            json!({"file_path": "/etc/shadow"}),
        ))],
        "Test".to_string(),
    );

    let file_error = file_guardrail_registry.validate_all(&file_plan).unwrap_err();
    let file_error_msg = file_error.to_string();
    assert!(
        file_error_msg.contains("/etc/shadow"),
        "Error should mention the specific path"
    );
    assert!(
        file_error_msg.contains("not allowed") || file_error_msg.contains("Allowed paths"),
        "Error should explain why it failed"
    );

    // Test rate limit guardrail error message
    let mut rate_guardrail_registry = GuardrailRegistry::new();
    rate_guardrail_registry.register(Box::new(RateLimitGuardrail::new(1)));

    let rate_plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 1, "b": 1}),
            )),
            Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 2, "b": 2}),
            )),
        ],
        "Test".to_string(),
    );

    let rate_error = rate_guardrail_registry.validate_all(&rate_plan).unwrap_err();
    let rate_error_msg = rate_error.to_string();
    assert!(
        rate_error_msg.contains("Rate limit") || rate_error_msg.contains("rate limit"),
        "Error should mention rate limit"
    );
    assert!(
        rate_error_msg.contains("2") || rate_error_msg.contains("tool calls"),
        "Error should mention the number of calls"
    );
}
