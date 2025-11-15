use async_trait::async_trait;
use agent_core::Result;
use serde_json::Value;

/// Trait defining the interface for tools that agents can use.
/// 
/// Tools are external capabilities that agents can invoke to perform actions
/// beyond text generation, such as calculations, file operations, or web searches.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Returns the unique name of this tool.
    fn name(&self) -> &str;
    
    /// Returns a human-readable description of what this tool does.
    fn description(&self) -> &str;
    
    /// Returns a JSON Schema describing the parameters this tool accepts.
    fn parameters_schema(&self) -> Value;
    
    /// Executes the tool with the given parameters.
    /// 
    /// # Arguments
    /// * `params` - JSON value containing the tool parameters
    /// 
    /// # Returns
    /// A JSON value containing the tool's result
    async fn execute(&self, params: Value) -> Result<Value>;
}

/// Information about a tool for display and planning purposes.
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters_schema: Value,
}

impl ToolInfo {
    pub fn from_tool(tool: &dyn Tool) -> Self {
        Self {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            parameters_schema: tool.parameters_schema(),
        }
    }
}
