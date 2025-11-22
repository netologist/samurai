use crate::tool::{Tool, ToolInfo};
use std::collections::HashMap;

/// Registry for managing available tools.
///
/// The ToolRegistry stores tools by name and provides methods for
/// registering, retrieving, and listing tools.
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Creates a new empty ToolRegistry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registers a tool in the registry.
    ///
    /// # Arguments
    /// * `tool` - The tool to register
    ///
    /// # Note
    /// If a tool with the same name already exists, it will be replaced.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// Retrieves a tool by name.
    ///
    /// # Arguments
    /// * `name` - The name of the tool to retrieve
    ///
    /// # Returns
    /// An optional reference to the tool if it exists
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|boxed| boxed.as_ref())
    }

    /// Lists all available tools with their information.
    ///
    /// # Returns
    /// A vector of ToolInfo containing name, description, and schema for each tool
    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools
            .values()
            .map(|tool| ToolInfo::from_tool(tool.as_ref()))
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Calculator, FileReader, WebSearchStub};

    #[test]
    fn test_registry_new() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.list_tools().len(), 0);
    }

    #[test]
    fn test_registry_register_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(Calculator::new()));

        assert_eq!(registry.list_tools().len(), 1);
        assert!(registry.get("calculator").is_some());
    }

    #[test]
    fn test_registry_register_multiple_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(Calculator::new()));
        registry.register(Box::new(FileReader::new()));
        registry.register(Box::new(WebSearchStub::new()));

        assert_eq!(registry.list_tools().len(), 3);
        assert!(registry.get("calculator").is_some());
        assert!(registry.get("file_reader").is_some());
        assert!(registry.get("web_search").is_some());
    }

    #[test]
    fn test_registry_get_existing_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(Calculator::new()));

        let tool = registry.get("calculator");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "calculator");
    }

    #[test]
    fn test_registry_get_nonexistent_tool() {
        let registry = ToolRegistry::new();
        assert!(registry.get("nonexistent_tool").is_none());
    }

    #[test]
    fn test_registry_replace_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(Calculator::new()));

        // Register another calculator (should replace the first)
        registry.register(Box::new(Calculator::new()));

        // Should still have only one tool
        assert_eq!(registry.list_tools().len(), 1);
    }

    #[test]
    fn test_registry_list_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(Calculator::new()));
        registry.register(Box::new(FileReader::new()));

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 2);

        // Check that tool info contains expected data
        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"calculator".to_string()));
        assert!(tool_names.contains(&"file_reader".to_string()));

        // Verify each tool has description and schema
        for tool in tools {
            assert!(!tool.description.is_empty());
            assert!(tool.parameters_schema.is_object());
        }
    }

    #[test]
    fn test_registry_default() {
        let registry = ToolRegistry::default();
        assert_eq!(registry.list_tools().len(), 0);
    }

    #[tokio::test]
    async fn test_registry_tool_execution() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(Calculator::new()));

        let tool = registry.get("calculator").unwrap();
        let params = serde_json::json!({
            "operation": "add",
            "a": 2.0,
            "b": 3.0
        });

        let result = tool.execute(params).await.unwrap();
        assert_eq!(result["result"], 5.0);
    }
}
