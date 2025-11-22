use thiserror::Error;

/// Common error type for the AI agent framework
#[derive(Error, Debug)]
pub enum AgentError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// LLM provider error
    #[error("LLM provider error: {0}")]
    LLMProvider(String),

    /// Tool execution failed
    #[error("Tool execution failed: {tool_name} - {reason}")]
    ToolExecution {
        /// Name of the tool that failed
        tool_name: String,
        /// Reason for the failure
        reason: String,
    },

    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Guardrail violation
    #[error("Guardrail violation: {0}")]
    GuardrailViolation(String),

    /// Memory error
    #[error("Memory error: {0}")]
    Memory(String),

    /// Planning error
    #[error("Planning error: {0}")]
    Planning(String),

    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type alias for the AI agent framework
pub type Result<T> = std::result::Result<T, AgentError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let err = AgentError::Config("missing api key".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing api key");
    }

    #[test]
    fn test_tool_execution_error() {
        let err = AgentError::ToolExecution {
            tool_name: "calculator".to_string(),
            reason: "invalid parameters".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Tool execution failed: calculator - invalid parameters"
        );
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let agent_err: AgentError = io_err.into();
        assert!(matches!(agent_err, AgentError::Io(_)));
    }

    #[test]
    fn test_serde_error_conversion() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let agent_err: AgentError = json_err.into();
        assert!(matches!(agent_err, AgentError::Serialization(_)));
    }
}
