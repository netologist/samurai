use crate::context::PlanningContext;

/// Trait for rules that customize agent behavior.
/// Rules can modify the planning context to influence how the agent operates.
pub trait Rule: Send + Sync {
    /// Returns the name of this rule
    fn name(&self) -> &str;
    
    /// Returns the priority of this rule.
    /// Rules with lower priority values are applied first.
    fn priority(&self) -> u32;
    
    /// Apply this rule to the planning context.
    /// Rules can modify the system prompt, add constraints, or set metadata.
    fn apply(&self, context: &mut PlanningContext);
}
