use crate::{PlanningContext, Rule};

/// Rule that limits the length of agent responses.
/// Adds a constraint to the planning context specifying the maximum word count.
pub struct ResponseLengthRule {
    max_words: usize,
}

impl ResponseLengthRule {
    /// Create a new response length rule with the specified maximum word count
    pub fn new(max_words: usize) -> Self {
        Self { max_words }
    }
}

impl Rule for ResponseLengthRule {
    fn name(&self) -> &str {
        "response_length"
    }

    fn priority(&self) -> u32 {
        100
    }

    fn apply(&self, context: &mut PlanningContext) {
        let constraint = format!("Keep responses under {} words", self.max_words);
        context.add_constraint(constraint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_length_rule_adds_constraint() {
        let rule = ResponseLengthRule::new(150);
        let mut context = PlanningContext::new("You are a helpful assistant.".to_string());

        // Initially no constraints
        assert_eq!(context.constraints.len(), 0);

        // Apply the rule
        rule.apply(&mut context);

        // Should have one constraint
        assert_eq!(context.constraints.len(), 1);
        assert_eq!(context.constraints[0], "Keep responses under 150 words");
    }

    #[test]
    fn test_response_length_rule_name() {
        let rule = ResponseLengthRule::new(100);
        assert_eq!(rule.name(), "response_length");
    }

    #[test]
    fn test_response_length_rule_priority() {
        let rule = ResponseLengthRule::new(100);
        assert_eq!(rule.priority(), 100);
    }

    #[test]
    fn test_response_length_rule_different_limits() {
        let rule1 = ResponseLengthRule::new(50);
        let rule2 = ResponseLengthRule::new(500);

        let mut context1 = PlanningContext::new("Test".to_string());
        let mut context2 = PlanningContext::new("Test".to_string());

        rule1.apply(&mut context1);
        rule2.apply(&mut context2);

        assert_eq!(context1.constraints[0], "Keep responses under 50 words");
        assert_eq!(context2.constraints[0], "Keep responses under 500 words");
    }
}
