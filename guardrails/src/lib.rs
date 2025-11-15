//! Guardrails crate for the AI Agent Framework
//!
//! This crate provides safety guardrails that validate plans before execution.
//! Guardrails act as a safety layer to prevent agents from performing unauthorized
//! or dangerous actions.
//!
//! # Core Concepts
//!
//! - **Guardrail**: A trait for implementing validation logic that checks plans
//! - **GuardrailRegistry**: Manages multiple guardrails and validates plans against all of them
//! - **FilePathGuardrail**: Restricts file operations to allowed directories
//! - **RateLimitGuardrail**: Enforces limits on API calls per time period
//!
//! # Architecture
//!
//! The guardrails system follows a simple but powerful pattern:
//!
//! 1. **Define Guardrails**: Implement the `Guardrail` trait to create custom validation logic
//! 2. **Register Guardrails**: Add guardrails to a `GuardrailRegistry`
//! 3. **Validate Plans**: Call `validate_all()` before executing plans
//! 4. **Handle Violations**: Catch `GuardrailViolation` errors and prevent execution
//!
//! # Built-in Guardrails
//!
//! ## FilePathGuardrail
//!
//! Restricts file operations to specific directories. This prevents agents from
//! accessing sensitive files like `/etc/passwd` or user home directories.
//!
//! ```rust,ignore
//! use guardrails::FilePathGuardrail;
//! use std::path::PathBuf;
//!
//! let guardrail = FilePathGuardrail::new(vec![
//!     PathBuf::from("/tmp"),
//!     PathBuf::from("/home/user/documents"),
//! ]);
//! ```
//!
//! ## RateLimitGuardrail
//!
//! Enforces a maximum number of tool calls per minute. This prevents runaway
//! agents from making excessive API calls or consuming too many resources.
//!
//! ```rust,ignore
//! use guardrails::RateLimitGuardrail;
//!
//! // Allow maximum 100 tool calls per minute
//! let guardrail = RateLimitGuardrail::new(100);
//! ```
//!
//! # Complete Example
//!
//! ```rust,ignore
//! use guardrails::{GuardrailRegistry, FilePathGuardrail, RateLimitGuardrail};
//! use std::path::PathBuf;
//!
//! // Create registry
//! let mut registry = GuardrailRegistry::new();
//!
//! // Add file path restriction
//! let file_guardrail = FilePathGuardrail::new(vec![
//!     PathBuf::from("/safe/directory"),
//! ]);
//! registry.register(Box::new(file_guardrail));
//!
//! // Add rate limiting
//! let rate_guardrail = RateLimitGuardrail::new(50);
//! registry.register(Box::new(rate_guardrail));
//!
//! // Validate a plan before execution
//! match registry.validate_all(&plan) {
//!     Ok(()) => {
//!         // Safe to execute
//!         executor.execute_plan(plan).await?;
//!     }
//!     Err(e) => {
//!         eprintln!("Guardrail violation: {}", e);
//!         // Do not execute the plan
//!     }
//! }
//! ```
//!
//! # Creating Custom Guardrails
//!
//! You can create custom guardrails by implementing the `Guardrail` trait:
//!
//! ```rust,ignore
//! use guardrails::Guardrail;
//! use planner::Plan;
//! use agent_core::{Result, AgentError};
//!
//! struct CustomGuardrail {
//!     max_steps: usize,
//! }
//!
//! impl Guardrail for CustomGuardrail {
//!     fn name(&self) -> &str {
//!         "custom_guardrail"
//!     }
//!
//!     fn validate(&self, plan: &Plan) -> Result<()> {
//!         if plan.steps.len() > self.max_steps {
//!             return Err(AgentError::GuardrailViolation(
//!                 format!("Plan has too many steps: {}", plan.steps.len())
//!             ));
//!         }
//!         Ok(())
//!     }
//! }
//! ```

mod guardrail;
mod registry;
mod file_path;
mod rate_limit;

pub use guardrail::Guardrail;
pub use registry::GuardrailRegistry;
pub use file_path::FilePathGuardrail;
pub use rate_limit::RateLimitGuardrail;
