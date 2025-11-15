use agent_core::{AgentError, Result};
use planner::{Plan, Step};
use std::path::{Path, PathBuf};
use crate::Guardrail;

/// Guardrail that restricts file operations to allowed directories.
///
/// This guardrail validates that all file_reader tool calls in a plan
/// only access files within the configured allowed paths. This prevents
/// agents from reading sensitive files or accessing unauthorized directories.
///
/// # Example
///
/// ```rust,ignore
/// use guardrails::FilePathGuardrail;
/// use std::path::PathBuf;
///
/// let guardrail = FilePathGuardrail::new(vec![
///     PathBuf::from("/home/user/documents"),
///     PathBuf::from("/tmp"),
/// ]);
///
/// // This will pass validation if the plan only accesses files in allowed paths
/// guardrail.validate(&plan)?;
/// ```
pub struct FilePathGuardrail {
    allowed_paths: Vec<PathBuf>,
}

impl FilePathGuardrail {
    /// Creates a new FilePathGuardrail with the specified allowed paths.
    ///
    /// # Arguments
    ///
    /// * `allowed_paths` - Vector of paths that the agent is allowed to access
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let guardrail = FilePathGuardrail::new(vec![
    ///     PathBuf::from("/safe/directory"),
    /// ]);
    /// ```
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        Self { allowed_paths }
    }

    /// Extracts the file_path parameter from tool call parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - JSON parameters from the tool call
    ///
    /// # Returns
    ///
    /// The extracted file path as a PathBuf
    ///
    /// # Errors
    ///
    /// Returns an error if the file_path parameter is missing or invalid.
    fn extract_path(&self, parameters: &serde_json::Value) -> Result<PathBuf> {
        let path_str = parameters
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::GuardrailViolation(
                    "file_reader tool call missing 'file_path' parameter".to_string()
                )
            })?;

        Ok(PathBuf::from(path_str))
    }

    /// Checks if a path is within any of the allowed paths.
    ///
    /// This method canonicalizes both the target path and allowed paths
    /// to handle relative paths and symlinks correctly.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// True if the path is within an allowed directory, false otherwise.
    fn is_allowed(&self, path: &Path) -> bool {
        // Try to canonicalize the path, but if it fails (e.g., file doesn't exist yet),
        // use the path as-is for checking
        let target_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        for allowed in &self.allowed_paths {
            // Canonicalize allowed path
            let allowed_canonical = allowed
                .canonicalize()
                .unwrap_or_else(|_| allowed.clone());

            // Check if target path starts with allowed path
            if target_path.starts_with(&allowed_canonical) {
                return true;
            }

            // Also check non-canonicalized path in case of non-existent files
            if path.starts_with(allowed) {
                return true;
            }
        }

        false
    }
}

impl Guardrail for FilePathGuardrail {
    fn name(&self) -> &str {
        "file_path"
    }

    fn validate(&self, plan: &Plan) -> Result<()> {
        for step in &plan.steps {
            if let Step::ToolCall(tool_call) = step {
                // Only validate file_reader tool calls
                if tool_call.tool_name == "file_reader" {
                    let path = self.extract_path(&tool_call.parameters)?;

                    if !self.is_allowed(&path) {
                        return Err(AgentError::GuardrailViolation(format!(
                            "File path not allowed: {}. Allowed paths: {:?}",
                            path.display(),
                            self.allowed_paths
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use planner::ToolCall;
    use serde_json::json;

    #[test]
    fn test_allowed_path() {
        let guardrail = FilePathGuardrail::new(vec![PathBuf::from("/tmp")]);

        let plan = Plan::new(
            vec![Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"file_path": "/tmp/test.txt"}),
            ))],
            "Test plan".to_string(),
        );

        assert!(guardrail.validate(&plan).is_ok());
    }

    #[test]
    fn test_disallowed_path() {
        let guardrail = FilePathGuardrail::new(vec![PathBuf::from("/tmp")]);

        let plan = Plan::new(
            vec![Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"file_path": "/etc/passwd"}),
            ))],
            "Test plan".to_string(),
        );

        assert!(guardrail.validate(&plan).is_err());
    }

    #[test]
    fn test_non_file_reader_tool_ignored() {
        let guardrail = FilePathGuardrail::new(vec![PathBuf::from("/tmp")]);

        let plan = Plan::new(
            vec![Step::ToolCall(ToolCall::new(
                "calculator".to_string(),
                json!({"operation": "add", "a": 1, "b": 2}),
            ))],
            "Test plan".to_string(),
        );

        // Should pass because it's not a file_reader tool
        assert!(guardrail.validate(&plan).is_ok());
    }

    #[test]
    fn test_missing_file_path_parameter() {
        let guardrail = FilePathGuardrail::new(vec![PathBuf::from("/tmp")]);

        let plan = Plan::new(
            vec![Step::ToolCall(ToolCall::new(
                "file_reader".to_string(),
                json!({"wrong_param": "value"}),
            ))],
            "Test plan".to_string(),
        );

        assert!(guardrail.validate(&plan).is_err());
    }
}
