use async_trait::async_trait;
use agent_core::{AgentError, Result};
use serde_json::{json, Value};
use std::fs;
use crate::tool::Tool;

/// FileReader tool for reading file contents.
/// 
/// Reads the contents of a file from the filesystem and returns it as a string.
pub struct FileReader;

impl FileReader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileReader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileReader {
    fn name(&self) -> &str {
        "file_reader"
    }
    
    fn description(&self) -> &str {
        "Reads the contents of a file from the filesystem"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The path to the file to read"
                }
            },
            "required": ["file_path"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        // Extract file path parameter
        let file_path = params["file_path"]
            .as_str()
            .ok_or_else(|| AgentError::ToolExecution {
                tool_name: self.name().to_string(),
                reason: "Missing or invalid 'file_path' parameter".to_string(),
            })?;
        
        // Read file contents
        let contents = fs::read_to_string(file_path).map_err(|e| {
            let reason = match e.kind() {
                std::io::ErrorKind::NotFound => {
                    format!("File not found: {}", file_path)
                }
                std::io::ErrorKind::PermissionDenied => {
                    format!("Permission denied: {}", file_path)
                }
                _ => {
                    format!("Failed to read file {}: {}", file_path, e)
                }
            };
            
            AgentError::ToolExecution {
                tool_name: self.name().to_string(),
                reason,
            }
        })?;
        
        Ok(json!({
            "file_path": file_path,
            "contents": contents,
            "size": contents.len()
        }))
    }
}
