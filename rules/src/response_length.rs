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
