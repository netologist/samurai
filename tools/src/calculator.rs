use crate::tool::Tool;
use agent_core::{AgentError, Result};
use async_trait::async_trait;
use serde_json::{Value, json};

/// Calculator tool for performing basic arithmetic operations.
///
/// Supports addition, subtraction, multiplication, and division.
pub struct Calculator;

impl Calculator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for Calculator {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Performs arithmetic operations (add, subtract, multiply, divide)"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "The arithmetic operation to perform"
                },
                "a": {
                    "type": "number",
                    "description": "The first operand"
                },
                "b": {
                    "type": "number",
                    "description": "The second operand"
                }
            },
            "required": ["operation", "a", "b"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        // Extract parameters
        let operation = params["operation"].as_str().ok_or_else(|| {
            AgentError::InvalidParameter("Missing or invalid 'operation' parameter".to_string())
        })?;

        let a = params["a"].as_f64().ok_or_else(|| {
            AgentError::InvalidParameter("Missing or invalid 'a' parameter".to_string())
        })?;

        let b = params["b"].as_f64().ok_or_else(|| {
            AgentError::InvalidParameter("Missing or invalid 'b' parameter".to_string())
        })?;

        // Perform calculation
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(AgentError::ToolExecution {
                        tool_name: self.name().to_string(),
                        reason: "Division by zero".to_string(),
                    });
                }
                a / b
            }
            _ => {
                return Err(AgentError::InvalidParameter(format!(
                    "Unknown operation: {}",
                    operation
                )));
            }
        };

        Ok(json!({
            "result": result,
            "operation": operation,
            "a": a,
            "b": b
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_calculator_addition() {
        let calc = Calculator::new();
        let params = json!({
            "operation": "add",
            "a": 5.0,
            "b": 3.0
        });

        let result = calc.execute(params).await.unwrap();
        assert_eq!(result["result"], 8.0);
        assert_eq!(result["operation"], "add");
    }

    #[tokio::test]
    async fn test_calculator_subtraction() {
        let calc = Calculator::new();
        let params = json!({
            "operation": "subtract",
            "a": 10.0,
            "b": 4.0
        });

        let result = calc.execute(params).await.unwrap();
        assert_eq!(result["result"], 6.0);
    }

    #[tokio::test]
    async fn test_calculator_multiplication() {
        let calc = Calculator::new();
        let params = json!({
            "operation": "multiply",
            "a": 6.0,
            "b": 7.0
        });

        let result = calc.execute(params).await.unwrap();
        assert_eq!(result["result"], 42.0);
    }

    #[tokio::test]
    async fn test_calculator_division() {
        let calc = Calculator::new();
        let params = json!({
            "operation": "divide",
            "a": 20.0,
            "b": 4.0
        });

        let result = calc.execute(params).await.unwrap();
        assert_eq!(result["result"], 5.0);
    }

    #[tokio::test]
    async fn test_calculator_division_by_zero() {
        let calc = Calculator::new();
        let params = json!({
            "operation": "divide",
            "a": 10.0,
            "b": 0.0
        });

        let result = calc.execute(params).await;
        assert!(result.is_err());

        if let Err(AgentError::ToolExecution { tool_name, reason }) = result {
            assert_eq!(tool_name, "calculator");
            assert!(reason.contains("Division by zero"));
        } else {
            panic!("Expected ToolExecution error");
        }
    }

    #[tokio::test]
    async fn test_calculator_invalid_operation() {
        let calc = Calculator::new();
        let params = json!({
            "operation": "modulo",
            "a": 10.0,
            "b": 3.0
        });

        let result = calc.execute(params).await;
        assert!(result.is_err());

        if let Err(AgentError::InvalidParameter(reason)) = result {
            assert!(reason.contains("Unknown operation"));
        } else {
            panic!("Expected InvalidParameter error");
        }
    }

    #[tokio::test]
    async fn test_calculator_missing_parameter() {
        let calc = Calculator::new();
        let params = json!({
            "operation": "add",
            "a": 5.0
            // Missing 'b' parameter
        });

        let result = calc.execute(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_calculator_invalid_parameter_type() {
        let calc = Calculator::new();
        let params = json!({
            "operation": "add",
            "a": "not a number",
            "b": 3.0
        });

        let result = calc.execute(params).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_calculator_name() {
        let calc = Calculator::new();
        assert_eq!(calc.name(), "calculator");
    }

    #[test]
    fn test_calculator_description() {
        let calc = Calculator::new();
        assert!(!calc.description().is_empty());
    }

    #[test]
    fn test_calculator_parameters_schema() {
        let calc = Calculator::new();
        let schema = calc.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["operation"].is_object());
        assert!(schema["properties"]["a"].is_object());
        assert!(schema["properties"]["b"].is_object());
        assert!(schema["required"].is_array());
    }
}
