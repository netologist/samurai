use std::collections::HashMap;
use crate::tool::{Tool, ToolInfo};

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
