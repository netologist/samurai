use async_trait::async_trait;
use agent_core::{AgentError, Result};
use serde_json::{json, Value};
use crate::tool::Tool;

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
        let operation = params["operation"]
            .as_str()
            .ok_or_else(|| AgentError::ToolExecution {
                tool_name: self.name().to_string(),
                reason: "Missing or invalid 'operation' parameter".to_string(),
            })?;
        
        let a = params["a"]
            .as_f64()
            .ok_or_else(|| AgentError::ToolExecution {
                tool_name: self.name().to_string(),
                reason: "Missing or invalid 'a' parameter".to_string(),
            })?;
        
        let b = params["b"]
            .as_f64()
            .ok_or_else(|| AgentError::ToolExecution {
                tool_name: self.name().to_string(),
                reason: "Missing or invalid 'b' parameter".to_string(),
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
                return Err(AgentError::ToolExecution {
                    tool_name: self.name().to_string(),
                    reason: format!("Unknown operation: {}", operation),
                });
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
