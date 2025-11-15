use std::collections::HashMap;

/// Context passed through the rule chain during planning.
/// Rules can modify the system prompt, add constraints, or set metadata.
#[derive(Debug, Clone)]
pub struct PlanningContext {
    /// The system prompt that will be sent to the LLM
    pub system_prompt: String,
    
    /// Additional constraints to guide agent behavior
    pub constraints: Vec<String>,
    
    /// Arbitrary metadata that rules can use to communicate
    pub metadata: HashMap<String, String>,
}

impl PlanningContext {
    /// Create a new planning context with the given system prompt
    pub fn new(system_prompt: String) -> Self {
        Self {
            system_prompt,
            constraints: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a constraint to the context
    pub fn add_constraint(&mut self, constraint: String) {
        self.constraints.push(constraint);
    }
    
    /// Set a metadata value
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Get a metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}
