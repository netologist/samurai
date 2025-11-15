use agent_core::{AgentError, Result};
use planner::{Plan, Step};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::Guardrail;

/// Guardrail that enforces rate limits on tool calls.
///
/// This guardrail tracks the number of tool calls over time and prevents
/// plans from executing if they would exceed the configured rate limit.
/// This helps prevent excessive API usage and protects against runaway agents.
///
/// # Example
///
/// ```rust,ignore
/// use guardrails::RateLimitGuardrail;
///
/// // Allow maximum 100 tool calls per minute
/// let guardrail = RateLimitGuardrail::new(100);
///
/// // This will fail if the plan would exceed the rate limit
/// guardrail.validate(&plan)?;
/// ```
pub struct RateLimitGuardrail {
    max_calls_per_minute: usize,
    call_history: Arc<Mutex<Vec<Instant>>>,
}

impl RateLimitGuardrail {
    /// Creates a new RateLimitGuardrail with the specified limit.
    ///
    /// # Arguments
    ///
    /// * `max_calls_per_minute` - Maximum number of tool calls allowed per minute
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Allow up to 50 tool calls per minute
    /// let guardrail = RateLimitGuardrail::new(50);
    /// ```
    pub fn new(max_calls_per_minute: usize) -> Self {
        Self {
            max_calls_per_minute,
            call_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Counts the number of ToolCall steps in a plan.
    ///
    /// # Arguments
    ///
    /// * `plan` - The plan to analyze
    ///
    /// # Returns
    ///
    /// The number of tool call steps in the plan.
    fn count_tool_calls(&self, plan: &Plan) -> usize {
        plan.steps
            .iter()
            .filter(|step| matches!(step, Step::ToolCall(_)))
            .count()
    }

    /// Cleans up old call records that are outside the time window.
    ///
    /// Removes all call timestamps that are older than one minute.
    fn cleanup_old_calls(&self, history: &mut Vec<Instant>) {
        let one_minute_ago = Instant::now() - Duration::from_secs(60);
        history.retain(|&timestamp| timestamp > one_minute_ago);
    }

    /// Records new tool calls in the history.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of tool calls to record
    fn record_calls(&self, count: usize) {
        let mut history = self.call_history.lock().unwrap();
        let now = Instant::now();
        for _ in 0..count {
            history.push(now);
        }
    }

    /// Gets the current number of calls in the last minute.
    ///
    /// # Returns
    ///
    /// The number of tool calls made in the last 60 seconds.
    fn current_call_count(&self) -> usize {
        let mut history = self.call_history.lock().unwrap();
        self.cleanup_old_calls(&mut history);
        history.len()
    }
}

impl Guardrail for RateLimitGuardrail {
    fn name(&self) -> &str {
        "rate_limit"
    }

    fn validate(&self, plan: &Plan) -> Result<()> {
        let tool_calls_in_plan = self.count_tool_calls(plan);
        
        // Get current call count and clean up old entries
        let mut history = self.call_history.lock().unwrap();
        self.cleanup_old_calls(&mut history);
        let current_calls = history.len();
        
        // Check if adding these calls would exceed the limit
        let total_calls = current_calls + tool_calls_in_plan;
        
        if total_calls > self.max_calls_per_minute {
            return Err(AgentError::GuardrailViolation(format!(
                "Rate limit exceeded: plan contains {} tool calls, but only {} calls remaining in current minute (limit: {} per minute, current: {})",
                tool_calls_in_plan,
                self.max_calls_per_minute.saturating_sub(current_calls),
                self.max_calls_per_minute,
                current_calls
            )));
        }

        // Record the calls from this plan
        let now = Instant::now();
        for _ in 0..tool_calls_in_plan {
            history.push(now);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use planner::ToolCall;
    use serde_json::json;

    #[test]
    fn test_within_rate_limit() {
        let guardrail = RateLimitGuardrail::new(10);

        let plan = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "add", "a": 1, "b": 2}),
                )),
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "multiply", "a": 3, "b": 4}),
                )),
            ],
            "Test plan".to_string(),
        );

        // Should pass with 2 calls when limit is 10
        assert!(guardrail.validate(&plan).is_ok());
    }

    #[test]
    fn test_exceeds_rate_limit() {
        let guardrail = RateLimitGuardrail::new(2);

        let plan = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "add", "a": 1, "b": 2}),
                )),
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "multiply", "a": 3, "b": 4}),
                )),
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "subtract", "a": 5, "b": 1}),
                )),
            ],
            "Test plan".to_string(),
        );

        // Should fail with 3 calls when limit is 2
        assert!(guardrail.validate(&plan).is_err());
    }

    #[test]
    fn test_non_tool_call_steps_ignored() {
        let guardrail = RateLimitGuardrail::new(1);

        let plan = Plan::new(
            vec![
                Step::Reasoning {
                    text: "Thinking about the problem".to_string(),
                },
                Step::ToolCall(ToolCall::new(
                    "calculator".to_string(),
                    json!({"operation": "add", "a": 1, "b": 2}),
                )),
                Step::Response {
                    text: "The answer is 3".to_string(),
                },
            ],
            "Test plan".to_string(),
        );

        // Should pass because only 1 tool call, even though there are 3 steps total
        assert!(guardrail.validate(&plan).is_ok());
    }

    #[test]
    fn test_count_tool_calls() {
        let guardrail = RateLimitGuardrail::new(100);

        let plan = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new("tool1".to_string(), json!({}))),
                Step::Reasoning {
                    text: "thinking".to_string(),
                },
                Step::ToolCall(ToolCall::new("tool2".to_string(), json!({}))),
                Step::Response {
                    text: "response".to_string(),
                },
                Step::ToolCall(ToolCall::new("tool3".to_string(), json!({}))),
            ],
            "Test plan".to_string(),
        );

        assert_eq!(guardrail.count_tool_calls(&plan), 3);
    }

    #[test]
    fn test_multiple_validations_accumulate() {
        let guardrail = RateLimitGuardrail::new(5);

        // First plan with 2 calls - should pass
        let plan1 = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new("tool1".to_string(), json!({}))),
                Step::ToolCall(ToolCall::new("tool2".to_string(), json!({}))),
            ],
            "Plan 1".to_string(),
        );
        assert!(guardrail.validate(&plan1).is_ok());

        // Second plan with 2 calls - should pass (total 4)
        let plan2 = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new("tool3".to_string(), json!({}))),
                Step::ToolCall(ToolCall::new("tool4".to_string(), json!({}))),
            ],
            "Plan 2".to_string(),
        );
        assert!(guardrail.validate(&plan2).is_ok());

        // Third plan with 2 calls - should fail (would be 6 total, limit is 5)
        let plan3 = Plan::new(
            vec![
                Step::ToolCall(ToolCall::new("tool5".to_string(), json!({}))),
                Step::ToolCall(ToolCall::new("tool6".to_string(), json!({}))),
            ],
            "Plan 3".to_string(),
        );
        assert!(guardrail.validate(&plan3).is_err());
    }
}
