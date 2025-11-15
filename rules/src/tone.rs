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
            Tone::Technical => "Use a technical tone with precise terminology. Be accurate and detailed.",
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
