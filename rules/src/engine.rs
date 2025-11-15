use crate::{PlanningContext, Rule};

/// Engine that manages and applies multiple rules to a planning context.
/// Rules are applied in priority order (lower priority values first).
pub struct RuleEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleEngine {
    /// Create a new empty rule engine
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }
    
    /// Add a rule to the engine
    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }
    
    /// Apply all rules to the planning context in priority order.
    /// Rules with lower priority values are applied first.
    pub fn apply_all(&self, context: &mut PlanningContext) {
        // Collect references and sort by priority
        let mut sorted_rules: Vec<&Box<dyn Rule>> = self.rules.iter().collect();
        sorted_rules.sort_by_key(|r| r.priority());
        
        // Apply each rule in order
        for rule in sorted_rules {
            rule.apply(context);
        }
    }
    
    /// Get the number of rules in the engine
    pub fn len(&self) -> usize {
        self.rules.len()
    }
    
    /// Check if the engine has no rules
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}
