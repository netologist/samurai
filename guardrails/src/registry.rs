use agent_core::Result;
use planner::Plan;
use crate::Guardrail;

/// Registry for managing multiple guardrails.
///
/// The GuardrailRegistry stores a collection of guardrails and provides
/// a method to validate plans against all registered guardrails. Validation
/// stops at the first violation, ensuring that plans are only executed if
/// they pass all safety checks.
///
/// # Example
///
/// ```rust,ignore
/// use guardrails::{GuardrailRegistry, FilePathGuardrail, RateLimitGuardrail};
///
/// let mut registry = GuardrailRegistry::new();
///
/// // Register multiple guardrails
/// registry.register(Box::new(FilePathGuardrail::new(allowed_paths)));
/// registry.register(Box::new(RateLimitGuardrail::new(100)));
///
/// // Validate a plan - stops at first violation
/// match registry.validate_all(&plan) {
///     Ok(()) => println!("Plan is safe to execute"),
///     Err(e) => println!("Plan violates guardrail: {}", e),
/// }
/// ```
pub struct GuardrailRegistry {
    guardrails: Vec<Box<dyn Guardrail>>,
}

impl GuardrailRegistry {
    /// Creates a new empty guardrail registry.
    pub fn new() -> Self {
        Self {
            guardrails: Vec::new(),
        }
    }

    /// Registers a new guardrail in the registry.
    ///
    /// # Arguments
    ///
    /// * `guardrail` - The guardrail to register
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut registry = GuardrailRegistry::new();
    /// registry.register(Box::new(MyCustomGuardrail::new()));
    /// ```
    pub fn register(&mut self, guardrail: Box<dyn Guardrail>) {
        self.guardrails.push(guardrail);
    }

    /// Validates a plan against all registered guardrails.
    ///
    /// This method checks the plan against each guardrail in the order they
    /// were registered. If any guardrail fails validation, the method returns
    /// immediately with the error, and subsequent guardrails are not checked.
    ///
    /// # Arguments
    ///
    /// * `plan` - The plan to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the plan passes all guardrails
    /// * `Err(AgentError::GuardrailViolation)` if any guardrail fails
    ///
    /// # Errors
    ///
    /// Returns an error on the first guardrail violation encountered.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let registry = GuardrailRegistry::new();
    /// // ... register guardrails ...
    ///
    /// match registry.validate_all(&plan) {
    ///     Ok(()) => {
    ///         // Safe to execute plan
    ///         executor.execute_plan(plan).await?;
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Cannot execute plan: {}", e);
    ///     }
    /// }
    /// ```
    pub fn validate_all(&self, plan: &Plan) -> Result<()> {
        for guardrail in &self.guardrails {
            guardrail.validate(plan)?;
        }
        Ok(())
    }

    /// Returns the number of registered guardrails.
    pub fn len(&self) -> usize {
        self.guardrails.len()
    }

    /// Returns true if no guardrails are registered.
    pub fn is_empty(&self) -> bool {
        self.guardrails.is_empty()
    }
}

impl Default for GuardrailRegistry {
    fn default() -> Self {
        Self::new()
    }
}
