use agent_core::Result;
use planner::Plan;
use crate::Guardrail;

/// Registry for managing multiple guardrails.
///
/// The GuardrailRegistry stores a collection of guardrails and provides
/// a method to validate plans against all registered guardrails. Validation
/// stops at the first violation, ensuring that plans are only executed if
/// they pass all safety checks.
///
/// # Example
///
/// ```rust,ignore
/// use guardrails::{GuardrailRegistry, FilePathGuardrail, RateLimitGuardrail};
///
/// let mut registry = GuardrailRegistry::new();
///
/// // Register multiple guardrails
/// registry.register(Box::new(FilePathGuardrail::new(allowed_paths)));
/// registry.register(Box::new(RateLimitGuardrail::new(100)));
///
/// // Validate a plan - stops at first violation
/// match registry.validate_all(&plan) {
///     Ok(()) => println!("Plan is safe to execute"),
///     Err(e) => println!("Plan violates guardrail: {}", e),
/// }
/// ```
pub struct GuardrailRegistry {
    guardrails: Vec<Box<dyn Guardrail>>,
}

impl GuardrailRegistry {
    /// Creates a new empty guardrail registry.
    pub fn new() -> Self {
        Self {
            guardrails: Vec::new(),
        }
    }

    /// Registers a new guardrail in the registry.
    ///
    /// # Arguments
    ///
    /// * `guardrail` - The guardrail to register
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut registry = GuardrailRegistry::new();
    /// registry.register(Box::new(MyCustomGuardrail::new()));
    /// ```
    pub fn register(&mut self, guardrail: Box<dyn Guardrail>) {
        self.guardrails.push(guardrail);
    }

    /// Validates a plan against all registered guardrails.
    ///
    /// This method checks the plan against each guardrail in the order they
    /// were registered. If any guardrail fails validation, the method returns
    /// immediately with the error, and subsequent guardrails are not checked.
    ///
    /// # Arguments
    ///
    /// * `plan` - The plan to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the plan passes all guardrails
    /// * `Err(AgentError::GuardrailViolation)` if any guardrail fails
    ///
    /// # Errors
    ///
    /// Returns an error on the first guardrail violation encountered.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let registry = GuardrailRegistry::new();
    /// // ... register guardrails ...
    ///
    /// match registry.validate_all(&plan) {
    ///     Ok(()) => {
    ///         // Safe to execute plan
    ///         executor.execute_plan(plan).await?;
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Cannot execute plan: {}", e);
    ///     }
    /// }
    /// ```
    pub fn validate_all(&self, plan: &Plan) -> Result<()> {
        for guardrail in &self.guardrails {
            guardrail.validate(plan)?;
        }
        Ok(())
    }

    /// Returns the number of registered guardrails.
    pub fn len(&self) -> usize {
        self.guardrails.len()
    }

    /// Returns true if no guardrails are registered.
    pub fn is_empty(&self) -> bool {
        self.guardrails.is_empty()
    }
}

impl Default for GuardrailRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::AgentError;
    use planner::{Step, ToolCall};
    use serde_json::json;
    use std::path::PathBuf;
    use crate::{FilePathGuardrail, RateLimitGuardrail};

    #[test]
    fn test_empty_registry_passes_validation() {
        let registry = GuardrailRegistry::new();
        
        let plan = Plan::new(
            vec![Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 1, "b": 2}),
            ))],
            "Test plan".to_string(),
        );

        // Empty registry should pass any plan
        assert!(registry.validate_all(&plan).is_ok());
    }

    #[test]
    fn test_registry_validates_all_guardrails() {
        let mut registry = GuardrailRegistry::new();

        // Register file path guardrail
        let file_guardrail = FilePathGuardrail::new(vec![PathBuf::from("/tmp")]);
        registry.register(Box::new(file_guardrail));

        // Register rate limit guardrail
        let rate_guardrail = RateLimitGuardrail::new(10);
        registry.register(Box::new(rate_guardrail));

        // Create a plan that passes both guardrails
        let plan = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new(
                    "file_reader".to_string(),
                    json!({"file_path": "/tmp/test.txt"}),
                )),
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "add", "a": 1, "b": 2}),
                )),
            ],
            "Test plan".to_string(),
        );

        // Should pass both guardrails
        assert!(registry.validate_all(&plan).is_ok());
    }

    #[test]
    fn test_validation_stops_on_first_violation() {
        let mut registry = GuardrailRegistry::new();

        // Register file path guardrail (will fail)
        let file_guardrail = FilePathGuardrail::new(vec![PathBuf::from("/tmp")]);
        registry.register(Box::new(file_guardrail));

        // Register rate limit guardrail (would also fail if checked)
        let rate_guardrail = RateLimitGuardrail::new(1);
        registry.register(Box::new(rate_guardrail));

        // Create a plan that violates file path guardrail
        let plan = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new(
                    "file_reader".to_string(),
                    json!({"file_path": "/etc/passwd"}),
                )),
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "add", "a": 1, "b": 2}),
                )),
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "multiply", "a": 3, "b": 4}),
                )),
            ],
            "Test plan".to_string(),
        );

        // Should fail on file path guardrail (first one)
        let result = registry.validate_all(&plan);
        assert!(result.is_err());
        
        // Verify it's a file path violation, not rate limit
        match result {
            Err(AgentError::GuardrailViolation(msg)) => {
                assert!(msg.contains("File path not allowed"));
                assert!(msg.contains("/etc/passwd"));
            }
            _ => panic!("Expected GuardrailViolation error"),
        }
    }

    #[test]
    fn test_validation_fails_on_second_guardrail() {
        let mut registry = GuardrailRegistry::new();

        // Register file path guardrail (will pass)
        let file_guardrail = FilePathGuardrail::new(vec![PathBuf::from("/tmp")]);
        registry.register(Box::new(file_guardrail));

        // Register rate limit guardrail (will fail)
        let rate_guardrail = RateLimitGuardrail::new(1);
        registry.register(Box::new(rate_guardrail));

        // Create a plan that passes file path but fails rate limit
        let plan = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new(
                    "file_reader".to_string(),
                    json!({"file_path": "/tmp/test.txt"}),
                )),
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "add", "a": 1, "b": 2}),
                )),
            ],
            "Test plan".to_string(),
        );

        // Should fail on rate limit guardrail (second one)
        let result = registry.validate_all(&plan);
        assert!(result.is_err());
        
        // Verify it's a rate limit violation
        match result {
            Err(AgentError::GuardrailViolation(msg)) => {
                assert!(msg.contains("Rate limit exceeded"));
            }
            _ => panic!("Expected GuardrailViolation error"),
        }
    }

    #[test]
    fn test_registry_len_and_is_empty() {
        let mut registry = GuardrailRegistry::new();
        
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());

        registry.register(Box::new(RateLimitGuardrail::new(10)));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        registry.register(Box::new(FilePathGuardrail::new(vec![PathBuf::from("/tmp")])));
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_multiple_guardrails_all_pass() {
        let mut registry = GuardrailRegistry::new();

        // Register multiple guardrails
        registry.register(Box::new(FilePathGuardrail::new(vec![
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
        ])));
        registry.register(Box::new(RateLimitGuardrail::new(100)));

        // Create a plan that passes all guardrails
        let plan = Plan::new(
            vec![
                Step::Reasoning {
                    text: "Planning the task".to_string(),
                },
                Step::ToolCall(ToolCall::new(
                    "file_reader".to_string(),
                    json!({"file_path": "/tmp/data.txt"}),
                )),
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "add", "a": 5, "b": 10}),
                )),
                Step::Response {
                    text: "Task completed".to_string(),
                },
            ],
            "Multi-step plan".to_string(),
        );

        // Should pass all guardrails
        assert!(registry.validate_all(&plan).is_ok());
    }
}
