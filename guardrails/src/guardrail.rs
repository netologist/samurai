use agent_core::Result;
use planner::Plan;

/// Trait for implementing safety guardrails that validate plans before execution.
///
/// Guardrails provide a safety layer that checks planned actions against
/// configured constraints and policies. They can prevent unauthorized file access,
/// enforce rate limits, validate parameters, or implement any custom safety logic.
///
/// # Example
///
/// ```rust,ignore
/// use guardrails::Guardrail;
/// use planner::Plan;
/// use agent_core::Result;
///
/// struct CustomGuardrail;
///
/// impl Guardrail for CustomGuardrail {
///     fn name(&self) -> &str {
///         "custom_guardrail"
///     }
///
///     fn validate(&self, plan: &Plan) -> Result<()> {
///         // Implement validation logic
///         Ok(())
///     }
/// }
/// ```
pub trait Guardrail: Send + Sync {
    /// Returns the name of this guardrail for identification and logging.
    fn name(&self) -> &str;

    /// Validates a plan against this guardrail's constraints.
    ///
    /// # Arguments
    ///
    /// * `plan` - The plan to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the plan passes validation
    /// * `Err(AgentError::GuardrailViolation)` if the plan violates this guardrail's constraints
    ///
    /// # Errors
    ///
    /// Returns an error if the plan violates the guardrail's safety constraints.
    fn validate(&self, plan: &Plan) -> Result<()>;
}
