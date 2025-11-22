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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ResponseLengthRule, Tone, ToneRule};

    #[test]
    fn test_rule_engine_applies_rules_in_priority_order() {
        let mut engine = RuleEngine::new();

        // Add rules in reverse priority order (ToneRule has priority 50, ResponseLengthRule has 100)
        engine.add_rule(Box::new(ResponseLengthRule::new(100)));
        engine.add_rule(Box::new(ToneRule::new(Tone::Formal)));

        let mut context = PlanningContext::new("You are a helpful assistant.".to_string());
        engine.apply_all(&mut context);

        // ToneRule (priority 50) should be applied first, modifying the system prompt
        // ResponseLengthRule (priority 100) should be applied second, adding a constraint

        // Check that tone guidance was added to system prompt
        assert!(context.system_prompt.contains("formal, professional tone"));

        // Check that length constraint was added
        assert_eq!(context.constraints.len(), 1);
        assert_eq!(context.constraints[0], "Keep responses under 100 words");
    }

    #[test]
    fn test_rule_engine_multiple_rules_same_context() {
        let mut engine = RuleEngine::new();

        // Add multiple rules
        engine.add_rule(Box::new(ToneRule::new(Tone::Technical)));
        engine.add_rule(Box::new(ResponseLengthRule::new(200)));

        let mut context = PlanningContext::new("Base prompt.".to_string());
        engine.apply_all(&mut context);

        // Both rules should have been applied
        assert!(context.system_prompt.contains("technical tone"));
        assert_eq!(context.constraints.len(), 1);
        assert!(context.constraints[0].contains("200 words"));
    }

    #[test]
    fn test_rule_engine_empty() {
        let engine = RuleEngine::new();
        assert!(engine.is_empty());
        assert_eq!(engine.len(), 0);
    }

    #[test]
    fn test_rule_engine_add_rule() {
        let mut engine = RuleEngine::new();

        engine.add_rule(Box::new(ToneRule::new(Tone::Casual)));
        assert_eq!(engine.len(), 1);
        assert!(!engine.is_empty());

        engine.add_rule(Box::new(ResponseLengthRule::new(150)));
        assert_eq!(engine.len(), 2);
    }

    #[test]
    fn test_rule_engine_apply_all_empty_engine() {
        let engine = RuleEngine::new();
        let mut context = PlanningContext::new("Test prompt.".to_string());

        // Should not panic or modify context
        engine.apply_all(&mut context);

        assert_eq!(context.system_prompt, "Test prompt.");
        assert_eq!(context.constraints.len(), 0);
    }

    #[test]
    fn test_rule_engine_priority_ordering_with_three_rules() {
        // Create a custom rule with priority 25 to test ordering
        struct EarlyRule;
        impl Rule for EarlyRule {
            fn name(&self) -> &str {
                "early"
            }
            fn priority(&self) -> u32 {
                25
            }
            fn apply(&self, context: &mut PlanningContext) {
                context.set_metadata("order".to_string(), "early".to_string());
            }
        }

        struct MiddleRule;
        impl Rule for MiddleRule {
            fn name(&self) -> &str {
                "middle"
            }
            fn priority(&self) -> u32 {
                50
            }
            fn apply(&self, context: &mut PlanningContext) {
                let current = context
                    .get_metadata("order")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                context.set_metadata("order".to_string(), format!("{},middle", current));
            }
        }

        struct LateRule;
        impl Rule for LateRule {
            fn name(&self) -> &str {
                "late"
            }
            fn priority(&self) -> u32 {
                100
            }
            fn apply(&self, context: &mut PlanningContext) {
                let current = context
                    .get_metadata("order")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                context.set_metadata("order".to_string(), format!("{},late", current));
            }
        }

        let mut engine = RuleEngine::new();

        // Add rules in random order
        engine.add_rule(Box::new(LateRule));
        engine.add_rule(Box::new(EarlyRule));
        engine.add_rule(Box::new(MiddleRule));

        let mut context = PlanningContext::new("Test".to_string());
        engine.apply_all(&mut context);

        // Should be applied in priority order: early (25), middle (50), late (100)
        assert_eq!(context.get_metadata("order").unwrap(), "early,middle,late");
    }

    #[test]
    fn test_rule_engine_default() {
        let engine = RuleEngine::default();
        assert!(engine.is_empty());
    }
}
