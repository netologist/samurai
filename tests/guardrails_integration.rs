//! Integration tests for guardrails
//
// These tests verify that guardrails correctly validate plans and prevent
// the execution of unauthorized or dangerous actions.

use agent_core::{AgentError, Result};
use guardrails::{FilePathGuardrail, Guardrail, GuardrailRegistry, RateLimitGuardrail};
use planner::{Plan, Step, ToolCall};
use serde_json::json;
use std::path::PathBuf;

#[test]
fn test_file_path_guardrail_allowed() {
    let allowed_paths = vec![PathBuf::from("/tmp"), PathBuf::from("/safe")];
    let guardrail = FilePathGuardrail::new(allowed_paths);

    let plan = Plan::new(
        vec![
            Step::ToolCall(ToolCall {
                tool_name: "file_reader".to_string(),
                parameters: json!({ "path": "/safe/file.txt" }),
            }),
            Step::ToolCall(ToolCall {
                tool_name: "file_writer".to_string(),
                parameters: json!({ "path": "/tmp/file.txt" }),
            }),
        ],
        "Test plan with allowed paths".to_string(),
    );

    let result = guardrail.validate(&plan);
    assert!(result.is_ok());
}

#[test]
fn test_file_path_guardrail_denied() {
    let allowed_paths = vec![PathBuf::from("/tmp")];
    let guardrail = FilePathGuardrail::new(allowed_paths);

    let plan = Plan::new(
        vec![Step::ToolCall(ToolCall {
            tool_name: "file_reader".to_string(),
            parameters: json!({ "path": "/etc/passwd" }),
        })],
        "Test plan with denied path".to_string(),
    );

    let result = guardrail.validate(&plan);
    assert!(result.is_err());

    match result {
        Err(AgentError::GuardrailViolation(msg)) => {
            assert!(msg.contains("Access to path '/etc/passwd' is not allowed"));
        }
        _ => panic!("Expected GuardrailViolation error"),
    }
}

#[test]
fn test_file_path_guardrail_no_file_tools() {
    let allowed_paths = vec![PathBuf::from("/tmp")];
    let guardrail = FilePathGuardrail::new(allowed_paths);

    let plan = Plan::new(
        vec![Step::ToolCall(ToolCall {
            tool_name: "calculator".to_string(),
            parameters: json!({ "a": 1, "b": 2 }),
        })],
        "Test plan with no file tools".to_string(),
    );

    let result = guardrail.validate(&plan);
    assert!(result.is_ok());
}

#[test]
fn test_rate_limit_guardrail_within_limit() {
    let mut guardrail = RateLimitGuardrail::new(10); // 10 calls per minute

    for _ in 0..10 {
        let plan = Plan::new(
            vec![Step::ToolCall(ToolCall {
                tool_name: "calculator".to_string(),
                parameters: json!({ "a": 1, "b": 2 }),
            })],
            "Test plan".to_string(),
        );
        let result = guardrail.validate(&plan);
        assert!(result.is_ok());
    }
}

#[test]
fn test_rate_limit_guardrail_exceed_limit() {
    let mut guardrail = RateLimitGuardrail::new(5); // 5 calls per minute

    for _ in 0..5 {
        let plan = Plan::new(
            vec![Step::ToolCall(ToolCall {
                tool_name: "calculator".to_string(),
                parameters: json!({ "a": 1, "b": 2 }),
            })],
            "Test plan".to_string(),
        );
        assert!(guardrail.validate(&plan).is_ok());
    }

    let plan = Plan::new(
        vec![Step::ToolCall(ToolCall {
            tool_name: "calculator".to_string(),
            parameters: json!({ "a": 1, "b": 2 }),
        })],
        "Test plan that exceeds limit".to_string(),
    );
    let result = guardrail.validate(&plan);
    assert!(result.is_err());

    match result {
        Err(AgentError::GuardrailViolation(msg)) => {
            assert!(msg.contains("Rate limit exceeded"));
        }
        _ => panic!("Expected GuardrailViolation error"),
    }
}

#[test]
fn test_guardrail_registry() {
    let allowed_paths = vec![PathBuf::from("/safe")];
    let mut registry = GuardrailRegistry::new();
    registry.register(Box::new(FilePathGuardrail::new(allowed_paths)));

    let valid_plan = Plan::new(
        vec![Step::ToolCall(ToolCall {
            tool_name: "file_reader".to_string(),
            parameters: json!({ "path": "/safe/file.txt" }),
        })],
        "Valid plan".to_string(),
    );

    let invalid_plan = Plan::new(
        vec![Step::ToolCall(ToolCall {
            tool_name: "file_reader".to_string(),
            parameters: json!({ "path": "/etc/passwd" }),
        })],
        "Invalid plan".to_string(),
    );

    assert!(registry.validate_all(&valid_plan).is_ok());
    assert!(registry.validate_all(&invalid_plan).is_err());
}