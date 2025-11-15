use async_trait::async_trait;
use agent_core::{AgentError, Result};
use serde_json::{json, Value};
use crate::tool::Tool;

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
