use crate::tool::Tool;
use agent_core::{AgentError, Result};
use async_trait::async_trait;
use serde_json::{Value, json};

/// WebSearchStub tool that returns mock search results.
///
/// This is a demonstration tool that simulates web search functionality
/// by returning hardcoded mock results. In a production system, this would
/// integrate with a real search API.
pub struct WebSearchStub;

impl WebSearchStub {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WebSearchStub {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for WebSearchStub {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Searches the web for information (mock implementation for demonstration)"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        // Extract query parameter
        let query = params["query"]
            .as_str()
            .ok_or_else(|| AgentError::ToolExecution {
                tool_name: self.name().to_string(),
                reason: "Missing or invalid 'query' parameter".to_string(),
            })?;

        // Return mock search results
        let mock_results = vec![
            json!({
                "title": format!("Result 1 for '{}'", query),
                "url": "https://example.com/result1",
                "snippet": "This is a mock search result demonstrating the web search tool functionality."
            }),
            json!({
                "title": format!("Result 2 for '{}'", query),
                "url": "https://example.com/result2",
                "snippet": "Another mock result showing how search results would be structured."
            }),
            json!({
                "title": format!("Result 3 for '{}'", query),
                "url": "https://example.com/result3",
                "snippet": "A third mock result to demonstrate multiple search results."
            }),
        ];

        Ok(json!({
            "query": query,
            "results": mock_results,
            "total_results": 3,
            "note": "These are mock results for demonstration purposes"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_web_search_returns_results() {
        let search = WebSearchStub::new();
        let params = json!({
            "query": "rust programming"
        });

        let result = search.execute(params).await.unwrap();
        assert_eq!(result["query"], "rust programming");
        assert_eq!(result["total_results"], 3);
        assert!(result["results"].is_array());
    }

    #[tokio::test]
    async fn test_web_search_result_format() {
        let search = WebSearchStub::new();
        let params = json!({
            "query": "test query"
        });

        let result = search.execute(params).await.unwrap();
        let results = result["results"].as_array().unwrap();

        assert_eq!(results.len(), 3);

        // Check first result has expected fields
        let first_result = &results[0];
        assert!(first_result["title"].is_string());
        assert!(first_result["url"].is_string());
        assert!(first_result["snippet"].is_string());

        // Verify title contains the query
        let title = first_result["title"].as_str().unwrap();
        assert!(title.contains("test query"));
    }

    #[tokio::test]
    async fn test_web_search_missing_parameter() {
        let search = WebSearchStub::new();
        let params = json!({});

        let result = search.execute(params).await;
        assert!(result.is_err());

        if let Err(AgentError::ToolExecution { tool_name, reason }) = result {
            assert_eq!(tool_name, "web_search");
            assert!(reason.contains("query"));
        } else {
            panic!("Expected ToolExecution error");
        }
    }

    #[tokio::test]
    async fn test_web_search_invalid_parameter_type() {
        let search = WebSearchStub::new();
        let params = json!({
            "query": 123  // Should be a string
        });

        let result = search.execute(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_web_search_empty_query() {
        let search = WebSearchStub::new();
        let params = json!({
            "query": ""
        });

        // Should still work with empty query
        let result = search.execute(params).await.unwrap();
        assert_eq!(result["query"], "");
        assert_eq!(result["total_results"], 3);
    }

    #[test]
    fn test_web_search_name() {
        let search = WebSearchStub::new();
        assert_eq!(search.name(), "web_search");
    }

    #[test]
    fn test_web_search_description() {
        let search = WebSearchStub::new();
        assert!(!search.description().is_empty());
    }

    #[test]
    fn test_web_search_parameters_schema() {
        let search = WebSearchStub::new();
        let schema = search.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["query"].is_object());
        assert!(schema["required"].is_array());
    }
}
