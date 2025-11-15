//! Rules engine for customizing AI agent behavior.
//!
//! This crate provides a flexible rules system that allows customization of agent behavior
//! through priority-based rules that modify the planning context. Rules can:
//! - Modify system prompts to guide LLM behavior
//! - Add constraints to limit or shape responses
//! - Set metadata for communication between rules
//!
//! # Example
//!
//! ```
//! use rules::{RuleEngine, PlanningContext, ResponseLengthRule, ToneRule, Tone};
//!
//! let mut engine = RuleEngine::new();
//! engine.add_rule(Box::new(ToneRule::new(Tone::Formal)));
//! engine.add_rule(Box::new(ResponseLengthRule::new(100)));
//!
//! let mut context = PlanningContext::new("You are a helpful assistant.".to_string());
//! engine.apply_all(&mut context);
//!
//! // Context now has tone guidance in system prompt and length constraint
//! assert!(!context.constraints.is_empty());
//! ```

mod context;
mod engine;
mod response_length;
mod rule;
mod tone;

pub use context::PlanningContext;
pub use engine::RuleEngine;
pub use response_length::ResponseLengthRule;
pub use rule::Rule;
pub use tone::{Tone, ToneRule};
