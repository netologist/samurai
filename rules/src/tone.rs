use crate::{PlanningContext, Rule};

/// Tone styles that can be applied to agent responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Formal, professional tone
    Formal,
    /// Casual, conversational tone
    Casual,
    /// Technical, precise tone with domain-specific terminology
    Technical,
}

impl Tone {
    /// Get the guidance text for this tone
    fn guidance(&self) -> &str {
        match self {
            Tone::Formal => "Use a formal, professional tone. Be polite and respectful.",
            Tone::Casual => "Use a casual, conversational tone. Be friendly and approachable.",
            Tone::Technical => {
                "Use a technical tone with precise terminology. Be accurate and detailed."
            }
        }
    }
}

/// Rule that modifies the system prompt to guide response tone.
pub struct ToneRule {
    tone: Tone,
}

impl ToneRule {
    /// Create a new tone rule with the specified tone
    pub fn new(tone: Tone) -> Self {
        Self { tone }
    }
}

impl Rule for ToneRule {
    fn name(&self) -> &str {
        "tone"
    }

    fn priority(&self) -> u32 {
        50
    }

    fn apply(&self, context: &mut PlanningContext) {
        // Append tone guidance to the system prompt
        let guidance = self.tone.guidance();
        context.system_prompt.push_str("\n\n");
        context.system_prompt.push_str(guidance);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_rule_modifies_system_prompt_formal() {
        let rule = ToneRule::new(Tone::Formal);
        let mut context = PlanningContext::new("You are a helpful assistant.".to_string());

        let original_prompt = context.system_prompt.clone();

        // Apply the rule
        rule.apply(&mut context);

        // System prompt should be modified
        assert_ne!(context.system_prompt, original_prompt);
        assert!(context.system_prompt.contains("formal, professional tone"));
        assert!(context.system_prompt.contains("Be polite and respectful"));
    }

    #[test]
    fn test_tone_rule_modifies_system_prompt_casual() {
        let rule = ToneRule::new(Tone::Casual);
        let mut context = PlanningContext::new("You are a helpful assistant.".to_string());

        rule.apply(&mut context);

        assert!(context
            .system_prompt
            .contains("casual, conversational tone"));
        assert!(context.system_prompt.contains("friendly and approachable"));
    }

    #[test]
    fn test_tone_rule_modifies_system_prompt_technical() {
        let rule = ToneRule::new(Tone::Technical);
        let mut context = PlanningContext::new("You are a helpful assistant.".to_string());

        rule.apply(&mut context);

        assert!(context.system_prompt.contains("technical tone"));
        assert!(context.system_prompt.contains("precise terminology"));
    }

    #[test]
    fn test_tone_rule_name() {
        let rule = ToneRule::new(Tone::Formal);
        assert_eq!(rule.name(), "tone");
    }

    #[test]
    fn test_tone_rule_priority() {
        let rule = ToneRule::new(Tone::Formal);
        assert_eq!(rule.priority(), 50);
    }

    #[test]
    fn test_tone_rule_preserves_original_prompt() {
        let rule = ToneRule::new(Tone::Formal);
        let original = "You are a helpful assistant.";
        let mut context = PlanningContext::new(original.to_string());

        rule.apply(&mut context);

        // Original prompt should still be present
        assert!(context.system_prompt.starts_with(original));
    }
}
