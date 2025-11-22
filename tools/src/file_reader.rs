use crate::tool::Tool;
use agent_core::{AgentError, Result};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::fs;

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_file_reader_existing_file() {
        // Create a temporary file with test content
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_content = "Hello, this is test content!";
        temp_file.write_all(test_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let file_path = temp_file.path().to_str().unwrap();

        let reader = FileReader::new();
        let params = json!({
            "file_path": file_path
        });

        let result = reader.execute(params).await.unwrap();
        assert_eq!(result["contents"], test_content);
        assert_eq!(result["file_path"], file_path);
        assert_eq!(result["size"], test_content.len());
    }

    #[tokio::test]
    async fn test_file_reader_nonexistent_file() {
        let reader = FileReader::new();
        let params = json!({
            "file_path": "/nonexistent/path/to/file.txt"
        });

        let result = reader.execute(params).await;
        assert!(result.is_err());

        if let Err(AgentError::ToolExecution { tool_name, reason }) = result {
            assert_eq!(tool_name, "file_reader");
            assert!(reason.contains("File not found") || reason.contains("not found"));
        } else {
            panic!("Expected ToolExecution error");
        }
    }

    #[tokio::test]
    async fn test_file_reader_empty_file() {
        // Create an empty temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        let reader = FileReader::new();
        let params = json!({
            "file_path": file_path
        });

        let result = reader.execute(params).await.unwrap();
        assert_eq!(result["contents"], "");
        assert_eq!(result["size"], 0);
    }

    #[tokio::test]
    async fn test_file_reader_missing_parameter() {
        let reader = FileReader::new();
        let params = json!({});

        let result = reader.execute(params).await;
        assert!(result.is_err());

        if let Err(AgentError::ToolExecution { tool_name, reason }) = result {
            assert_eq!(tool_name, "file_reader");
            assert!(reason.contains("file_path"));
        } else {
            panic!("Expected ToolExecution error");
        }
    }

    #[tokio::test]
    async fn test_file_reader_invalid_parameter_type() {
        let reader = FileReader::new();
        let params = json!({
            "file_path": 123  // Should be a string
        });

        let result = reader.execute(params).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_file_reader_name() {
        let reader = FileReader::new();
        assert_eq!(reader.name(), "file_reader");
    }

    #[test]
    fn test_file_reader_description() {
        let reader = FileReader::new();
        assert!(!reader.description().is_empty());
    }

    #[test]
    fn test_file_reader_parameters_schema() {
        let reader = FileReader::new();
        let schema = reader.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["file_path"].is_object());
        assert!(schema["required"].is_array());
    }
}
