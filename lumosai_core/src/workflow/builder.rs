//! Workflow builder for creating workflows from configuration
//! 
//! This module provides a builder pattern for creating workflows,
//! inspired by Mastra's design but optimized for Rust.

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use crate::{Result, Error};
use crate::agent::trait_def::Agent;
use crate::config::{WorkflowConfig, WorkflowStepConfig};
use super::{EnhancedWorkflow, WorkflowStep, StepType, StepExecutor};

/// Builder for creating workflows from configuration
pub struct WorkflowBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    trigger: Option<String>,
    steps: Vec<WorkflowStepConfig>,
    agents: HashMap<String, Arc<dyn Agent>>,
    timeout: Option<u64>,
    max_retries: Option<u32>,
}

impl WorkflowBuilder {
    /// Create a new workflow builder
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            description: None,
            trigger: None,
            steps: Vec::new(),
            agents: HashMap::new(),
            timeout: None,
            max_retries: None,
        }
    }

    /// Create a workflow builder from configuration
    pub fn from_config(config: &WorkflowConfig) -> Self {
        let mut builder = Self::new()
            .id(&config.id)
            .name(&config.name);

        if let Some(description) = &config.description {
            builder = builder.description(description);
        }

        if let Some(trigger) = &config.trigger {
            builder = builder.trigger(trigger);
        }

        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }

        if let Some(max_retries) = config.max_retries {
            builder = builder.max_retries(max_retries);
        }

        // Add steps from config
        for step_config in &config.steps {
            builder = builder.add_step_config(step_config.clone());
        }

        builder
    }

    /// Set workflow ID
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    /// Set workflow name
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Set workflow description
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set workflow trigger
    pub fn trigger(mut self, trigger: &str) -> Self {
        self.trigger = Some(trigger.to_string());
        self
    }

    /// Set workflow timeout
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set max retries
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Add a step configuration
    pub fn add_step_config(mut self, step_config: WorkflowStepConfig) -> Self {
        self.steps.push(step_config);
        self
    }

    /// Register an agent for use in workflow steps
    pub fn register_agent(mut self, name: String, agent: Arc<dyn Agent>) -> Self {
        self.agents.insert(name, agent);
        self
    }

    /// Build the workflow
    pub fn build(self) -> Result<EnhancedWorkflow> {
        let id = self.id.ok_or_else(|| Error::Configuration("Workflow ID is required".to_string()))?;
        let name = self.name.ok_or_else(|| Error::Configuration("Workflow name is required".to_string()))?;

        // Convert step configurations to workflow steps
        let mut workflow_steps = Vec::new();
        for (index, step_config) in self.steps.iter().enumerate() {
            let step = self.create_workflow_step(step_config, index)?;
            workflow_steps.push(step);
        }

        if workflow_steps.is_empty() {
            return Err(Error::Configuration("Workflow must have at least one step".to_string()));
        }

        // Create the enhanced workflow
        let mut workflow = EnhancedWorkflow::new(id, name);
        
        if let Some(description) = self.description {
            workflow.set_description(description);
        }

        // Add steps to workflow
        for step in workflow_steps {
            workflow.add_step(step);
        }

        Ok(workflow)
    }

    /// Create a workflow step from configuration
    fn create_workflow_step(&self, config: &WorkflowStepConfig, index: usize) -> Result<WorkflowStep> {
        let step_id = config.id.clone().unwrap_or_else(|| format!("step_{}", index));
        
        // Determine step type based on configuration
        let step_type = if config.agent.is_some() {
            StepType::Agent
        } else if config.tool.is_some() {
            StepType::Tool
        } else if config.workflow.is_some() {
            StepType::Workflow
        } else {
            StepType::Custom
        };

        // Create step executor based on type
        let executor = self.create_step_executor(config, step_type.clone())?;

        let mut step = WorkflowStep::new(step_id, step_type, executor);

        // Set optional properties
        if let Some(name) = &config.name {
            step.set_name(name.clone());
        }

        if let Some(description) = &config.description {
            step.set_description(description.clone());
        }

        if let Some(condition) = &config.condition {
            step.set_condition(condition.clone());
        }

        if let Some(timeout) = config.timeout {
            step.set_timeout(timeout);
        }

        if let Some(retries) = config.retries {
            step.set_max_retries(retries);
        }

        Ok(step)
    }

    /// Create a step executor based on configuration
    fn create_step_executor(&self, config: &WorkflowStepConfig, step_type: StepType) -> Result<Box<dyn StepExecutor>> {
        match step_type {
            StepType::Agent => {
                let agent_name = config.agent.as_ref()
                    .ok_or_else(|| Error::Configuration("Agent name is required for agent step".to_string()))?;
                
                let agent = self.agents.get(agent_name)
                    .ok_or_else(|| Error::Configuration(format!("Agent '{}' not found", agent_name)))?
                    .clone();

                Ok(Box::new(AgentStepExecutor::new(agent)))
            },
            StepType::Tool => {
                let tool_name = config.tool.as_ref()
                    .ok_or_else(|| Error::Configuration("Tool name is required for tool step".to_string()))?;
                
                Ok(Box::new(ToolStepExecutor::new(tool_name.clone())))
            },
            StepType::Workflow => {
                let workflow_id = config.workflow.as_ref()
                    .ok_or_else(|| Error::Configuration("Workflow ID is required for workflow step".to_string()))?;
                
                Ok(Box::new(WorkflowStepExecutor::new(workflow_id.clone())))
            },
            StepType::Custom => {
                // For custom steps, we'll create a simple executor
                Ok(Box::new(CustomStepExecutor::new()))
            },
        }
    }
}

impl Default for WorkflowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent step executor
struct AgentStepExecutor {
    agent: Arc<dyn Agent>,
}

impl AgentStepExecutor {
    fn new(agent: Arc<dyn Agent>) -> Self {
        Self { agent }
    }
}

impl StepExecutor for AgentStepExecutor {
    fn execute(&self, input: Value) -> Result<Value> {
        // Convert input to message and execute agent
        let message = input.as_str().unwrap_or("").to_string();
        
        // For now, return a mock response
        // In a real implementation, this would call the agent
        Ok(serde_json::json!({
            "agent_response": format!("Agent processed: {}", message),
            "agent_name": self.agent.get_name()
        }))
    }
}

/// Tool step executor
struct ToolStepExecutor {
    tool_name: String,
}

impl ToolStepExecutor {
    fn new(tool_name: String) -> Self {
        Self { tool_name }
    }
}

impl StepExecutor for ToolStepExecutor {
    fn execute(&self, input: Value) -> Result<Value> {
        // For now, return a mock response
        // In a real implementation, this would execute the tool
        Ok(serde_json::json!({
            "tool_response": format!("Tool '{}' processed input", self.tool_name),
            "tool_name": self.tool_name
        }))
    }
}

/// Workflow step executor
struct WorkflowStepExecutor {
    workflow_id: String,
}

impl WorkflowStepExecutor {
    fn new(workflow_id: String) -> Self {
        Self { workflow_id }
    }
}

impl StepExecutor for WorkflowStepExecutor {
    fn execute(&self, input: Value) -> Result<Value> {
        // For now, return a mock response
        // In a real implementation, this would execute the sub-workflow
        Ok(serde_json::json!({
            "workflow_response": format!("Workflow '{}' processed input", self.workflow_id),
            "workflow_id": self.workflow_id
        }))
    }
}

/// Custom step executor
struct CustomStepExecutor;

impl CustomStepExecutor {
    fn new() -> Self {
        Self
    }
}

impl StepExecutor for CustomStepExecutor {
    fn execute(&self, input: Value) -> Result<Value> {
        // For custom steps, just pass through the input
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WorkflowStepConfig;

    #[test]
    fn test_workflow_builder() {
        let builder = WorkflowBuilder::new()
            .id("test_workflow")
            .name("Test Workflow")
            .description("A test workflow");

        // Can't build without steps
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_workflow_builder_with_step() {
        let step_config = WorkflowStepConfig {
            id: Some("step1".to_string()),
            name: Some("Test Step".to_string()),
            description: Some("A test step".to_string()),
            agent: Some("test_agent".to_string()),
            tool: None,
            workflow: None,
            condition: None,
            timeout: None,
            retries: None,
            input: None,
        };

        let builder = WorkflowBuilder::new()
            .id("test_workflow")
            .name("Test Workflow")
            .add_step_config(step_config);

        // Should still fail because agent is not registered
        assert!(builder.build().is_err());
    }
}
