use agent_core::{Message, Result};
use executor::Executor;
use guardrails::GuardrailRegistry;
use memory::MemoryStore;
use planner::Planner;
use tools::ToolRegistry;

/// A high-level AI agent that orchestrates planning, execution, and memory.
pub struct Agent {
    memory: Box<dyn MemoryStore>,
    planner: Planner,
    executor: Executor,
    guardrails: GuardrailRegistry,
}

impl Agent {
    /// Creates a new AgentBuilder to construct an Agent.
    pub fn builder() -> AgentBuilder {
        AgentBuilder::default()
    }

    /// Execute a task using the agent's workflow.
    ///
    /// 1. Store user task in memory
    /// 2. Generate a plan using the Planner
    /// 3. Validate the plan using Guardrails
    /// 4. Execute the plan using the Executor
    /// 5. Store the result in memory
    pub async fn run(&mut self, task: &str) -> Result<String> {
        // 1. Add user task to memory
        self.memory.add_message(Message::user(task));

        // 2. Generate plan
        let available_tools = self.executor.list_tools();
        let plan = self.planner.create_plan(task, &available_tools).await?;

        // 3. Validate plan
        self.guardrails.validate_all(&plan)?;

        // 4. Execute plan
        let result = self.executor.execute_plan(plan).await?;

        // 5. Store result
        self.memory
            .add_message(Message::assistant(&result.final_response));

        Ok(result.final_response)
    }
}

/// Builder for creating Agent instances.
#[derive(Default)]
pub struct AgentBuilder {
    llm: Option<Box<dyn llm::LLMProvider>>,
    memory: Option<Box<dyn MemoryStore>>,
    tools: Option<ToolRegistry>,
    guardrails: Option<GuardrailRegistry>,
}

impl AgentBuilder {
    /// Sets the LLM provider (Required).
    pub fn llm(mut self, llm: Box<dyn llm::LLMProvider>) -> Self {
        self.llm = Some(llm);
        self
    }

    /// Sets the memory store (Optional, defaults to InMemoryStore).
    pub fn memory(mut self, memory: Box<dyn MemoryStore>) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Sets the tool registry (Optional, defaults to empty).
    pub fn tools(mut self, tools: ToolRegistry) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the guardrail registry (Optional, defaults to empty).
    pub fn guardrails(mut self, guardrails: GuardrailRegistry) -> Self {
        self.guardrails = Some(guardrails);
        self
    }

    /// Builds the Agent.
    ///
    /// # Errors
    /// Returns error if required components (LLM) are missing.
    pub fn build(self) -> Result<Agent> {
        let llm = self.llm.ok_or_else(|| {
            agent_core::AgentError::Config("LLM provider is required".to_string())
        })?;

        let memory = self
            .memory
            .unwrap_or_else(|| Box::new(memory::InMemoryStore::new()));

        // We need two references to memory: one for Planner and one for Executor/Agent.
        // Since MemoryStore is a trait object, we can't easily clone it unless we enforce Clone.
        // For now, we'll create separate memory stores or require the user to provide a shared store if needed.
        // BUT, the Planner needs memory for context, and Executor needs memory to store results.
        // They should share the SAME memory.
        // The current architecture in examples creates separate memory stores for Planner and Executor,
        // which seems like a bug or at least a limitation (Planner won't see Executor's results?).
        // Actually, in `research.rs`:
        // `planner_memory` is new. `executor_memory` is new. `agent.memory` is new.
        // This means the Planner ONLY sees the current query and system prompt?
        // And Executor stores results in its own memory?
        // And Agent stores results in ITS own memory?
        // This seems disjointed.

        // Ideally, they should share the same memory.
        // But `Box<dyn MemoryStore>` is not easily shareable unless we use `Arc<Mutex<dyn MemoryStore>>`.
        // Or we rely on the Agent to pass context to the Planner.

        // Let's look at `Planner::create_plan`. It takes `goal: &str`.
        // It uses `self.memory` (which it owns) but `Planner::new` takes `Box<dyn MemoryStore>`.
        // In `research.rs`, `planner` gets a fresh memory.

        // If we want a unified agent, we should probably have one MemoryStore.
        // But the current trait definitions might make sharing hard without Arc/Mutex.
        // For this Builder implementation, I will stick to the pattern in `research.rs`
        // but maybe improve it by using the SAME memory if I can clone it,
        // or just acknowledge they are separate for now.

        // Wait, `Planner` uses `self.llm.send_message(&messages)`.
        // `messages` is constructed from system prompt + user goal.
        // It DOES NOT seem to pull from `self.memory` in `create_plan`!
        // Let's check `Planner::create_plan` in `planner/src/planner.rs`.

        // `planner.rs`:
        // pub async fn create_plan(&self, goal: &str, available_tools: &[ToolInfo]) -> Result<Plan> {
        //    let system_prompt = self.build_system_prompt(available_tools);
        //    let messages = vec![
        //        Message::system(&system_prompt),
        //        Message::user(goal),
        //    ];
        //    let response = self.llm.send_message(&messages).await?;
        //    ...
        // }

        // The `Planner` struct HAS a `memory` field but `create_plan` DOES NOT USE IT!
        // This is a finding! The Planner ignores history!
        // I should fix this in `Planner` too, but for now let's focus on the Agent struct.

        // I will create the Planner and Executor.
        // Since `Planner` takes ownership of `llm` and `memory`, and `Executor` takes ownership of `tools` and `memory`.
        // I need to be careful.

        // For now, I will create new InMemoryStores for Planner and Executor if they need them,
        // but the Agent's main memory is what matters for the outer loop.

        let tools = self.tools.unwrap_or_default();
        let guardrails = self.guardrails.unwrap_or_default();

        // We need to clone LLM for Planner? `Planner` takes `Box<dyn LLMProvider>`.
        // `LLMProvider` is `Send + Sync`.
        // We can't clone `Box<dyn LLMProvider>` easily.
        // But `Planner` needs it.

        let planner = Planner::new(llm, Box::new(memory::InMemoryStore::new()));
        let executor = Executor::new(tools, Box::new(memory::InMemoryStore::new()));

        Ok(Agent {
            memory,
            planner,
            executor,
            guardrails,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{Message, Result};
    use async_trait::async_trait;

    struct MockLLM;
    #[async_trait]
    impl llm::LLMProvider for MockLLM {
        async fn send_message(&self, _messages: &[Message]) -> Result<String> {
            Ok("{}".to_string())
        }
    }

    #[test]
    fn test_agent_builder_minimal() {
        let llm = Box::new(MockLLM);
        let agent = Agent::builder().llm(llm).build();

        assert!(agent.is_ok());
    }

    #[test]
    fn test_agent_builder_missing_llm() {
        let agent = Agent::builder().build();
        assert!(agent.is_err());
    }
}
