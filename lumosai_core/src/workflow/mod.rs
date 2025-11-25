//! Workflow module for orchestrating multi-step processes
//!
//! This module provides powerful workflow orchestration capabilities,
//! inspired by Mastra's design, supporting sequential, parallel, and
//! DAG-based execution patterns.
//!
//! # Overview
//!
//! The workflow module consists of:
//!
//! - **Workflow Trait** - Core interface for all workflows
//! - **DAG Workflow** - Directed Acyclic Graph workflow execution
//! - **Enhanced Workflow** - Advanced workflow with parallel execution
//! - **Execution Engine** - Pluggable execution backends
//! - **Step System** - Reusable workflow steps
//! - **Retry & Error Handling** - Automatic retry and error recovery
//!
//! # Quick Start
//!
//! ## Creating a Simple Workflow
//!
//! ```rust
//! use lumosai_core::workflow::{DagWorkflowBuilder, WorkflowStep};
//! use lumosai_core::agent::types::RuntimeContext;
//! use serde_json::json;
//!
//! # async fn example() -> lumosai_core::Result<()> {
//! // Create a DAG workflow
//! let workflow = DagWorkflowBuilder::new("my-workflow".to_string())
//!     .description("My first workflow".to_string())
//!     .max_concurrency(4)
//!     .build();
//!
//! // Execute the workflow
//! let context = RuntimeContext::new();
//! let result = workflow.execute(json!({"input": "data"}), &context).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Adding Steps to a Workflow
//!
//! ```rust
//! use lumosai_core::workflow::{DagWorkflow, WorkflowStep};
//!
//! # async fn example(workflow: &DagWorkflow) -> lumosai_core::Result<()> {
//! // Create a step
//! let step = WorkflowStep::new("step1".to_string(), "First step".to_string());
//!
//! // Add root node (no dependencies)
//! workflow.add_root_node("step1".to_string(), "First step".to_string(), step).await?;
//!
//! // Add dependent node
//! let step2 = WorkflowStep::new("step2".to_string(), "Second step".to_string());
//! workflow.add_node(
//!     "step2".to_string(),
//!     "Second step".to_string(),
//!     vec!["step1".to_string()], // Depends on step1
//!     step2,
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Parallel Execution
//!
//! ```rust
//! use lumosai_core::workflow::{EnhancedWorkflow, WorkflowStep, StepFlowEntry};
//!
//! # fn example() {
//! let mut workflow = EnhancedWorkflow::new(
//!     "parallel-workflow".to_string(),
//!     Some("Workflow with parallel steps".to_string()),
//! );
//!
//! // Add parallel steps
//! let steps = vec![
//!     StepFlowEntry::Step {
//!         step: WorkflowStep::new("step1".to_string(), "Step 1".to_string()),
//!     },
//!     StepFlowEntry::Step {
//!         step: WorkflowStep::new("step2".to_string(), "Step 2".to_string()),
//!     },
//! ];
//!
//! workflow.add_parallel(steps, Some(2)); // Max 2 concurrent steps
//! # }
//! ```
//!
//! # Workflow Types
//!
//! ## DAG Workflow
//!
//! Directed Acyclic Graph workflow for complex dependencies:
//! - Automatic topological sorting
//! - Parallel execution of independent steps
//! - Cycle detection
//! - Configurable concurrency
//!
//! ## Enhanced Workflow
//!
//! Advanced workflow with rich features:
//! - Sequential and parallel execution
//! - Conditional branching
//! - Input/output schema validation
//! - Retry configuration
//! - Streaming execution
//!
//! # Advanced Features
//!
//! ## Retry Configuration
//!
//! ```rust
//! use lumosai_core::workflow::RetryConfig;
//!
//! let retry_config = RetryConfig {
//!     max_retries: 3,
//!     initial_delay_ms: 1000,
//!     max_delay_ms: 10000,
//!     backoff_multiplier: 2.0,
//! };
//! ```
//!
//! ## Workflow Status Tracking
//!
//! ```rust
//! use lumosai_core::workflow::{Workflow, WorkflowStatus};
//!
//! # async fn example(workflow: &dyn Workflow) -> lumosai_core::Result<()> {
//! let status = workflow.get_status("run-123").await?;
//!
//! match status {
//!     WorkflowStatus::Running => println!("Workflow is running"),
//!     WorkflowStatus::Completed(result) => println!("Completed: {:?}", result),
//!     WorkflowStatus::Failed(error) => println!("Failed: {}", error),
//!     WorkflowStatus::Suspended => println!("Workflow is suspended"),
//!     WorkflowStatus::NotFound => println!("Workflow not found"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # See Also
//!
//! - [`Workflow`] - Core workflow trait
//! - [`DagWorkflow`] - DAG workflow implementation
//! - [`DagWorkflowBuilder`] - Builder for DAG workflows
//! - [`EnhancedWorkflow`] - Enhanced workflow implementation
//! - [`WorkflowStep`] - Workflow step definition
//! - [`ExecutionEngine`] - Execution engine trait

#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(non_camel_case_types, ambiguous_glob_reexports, hidden_glob_reexports)]
#![allow(unexpected_cfgs, unused_assignments)]

pub mod basic;
pub mod dag_scheduler;
pub mod dag_workflow;
pub mod enhanced;
pub mod execution_engine;
mod real_api_tests;
mod step;
mod tests;
mod types;
mod workflow;

use crate::agent::types::RuntimeContext;
use crate::{Error, Result};
use async_trait::async_trait;
use serde_json::Value;

/// Core trait for workflow implementations
///
/// The `Workflow` trait defines the fundamental interface for all workflow
/// types in LumosAI. It provides methods for executing workflows, managing
/// workflow state, and handling workflow lifecycle.
///
/// # Overview
///
/// Workflows orchestrate multiple steps into cohesive processes:
/// - Execute steps in sequence or parallel
/// - Handle dependencies between steps
/// - Support streaming execution
/// - Manage workflow state (running, completed, failed, suspended)
/// - Enable pause/resume functionality
///
/// # Implementations
///
/// LumosAI provides several workflow implementations:
///
/// - [`DagWorkflow`] - DAG-based workflow with automatic dependency resolution
/// - [`EnhancedWorkflow`] - Advanced workflow with parallel execution
/// - [`BasicWorkflow`] - Simple sequential workflow
///
/// # Examples
///
/// ## Basic Workflow Execution
///
/// ```rust
/// use lumosai_core::workflow::Workflow;
/// use lumosai_core::agent::types::RuntimeContext;
/// use serde_json::json;
///
/// # async fn example(workflow: &dyn Workflow) -> lumosai_core::Result<()> {
/// let context = RuntimeContext::new();
/// let input = json!({"task": "process data"});
///
/// let result = workflow.execute(input, &context).await?;
/// println!("Workflow result: {:?}", result);
/// # Ok(())
/// # }
/// ```
///
/// ## Streaming Execution
///
/// ```rust
/// use lumosai_core::workflow::Workflow;
/// use lumosai_core::agent::types::RuntimeContext;
/// use serde_json::json;
/// use futures::StreamExt;
///
/// # async fn example(workflow: &dyn Workflow) -> lumosai_core::Result<()> {
/// let context = RuntimeContext::new();
/// let input = json!({"task": "stream data"});
///
/// let mut stream = workflow.execute_stream(input, &context).await?;
///
/// while let Some(result) = stream.next().await {
///     match result {
///         Ok(value) => println!("Step result: {:?}", value),
///         Err(e) => eprintln!("Step error: {}", e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Pause and Resume
///
/// ```rust
/// use lumosai_core::workflow::Workflow;
/// use serde_json::json;
///
/// # async fn example(workflow: &dyn Workflow) -> lumosai_core::Result<()> {
/// // Suspend a running workflow
/// workflow.suspend("run-123").await?;
///
/// // Resume later with optional new input
/// let result = workflow.resume("run-123", Some(json!({"new": "data"}))).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Thread Safety
///
/// All workflow implementations must be `Send + Sync`.
///
/// # See Also
///
/// - [`DagWorkflow`] - DAG workflow implementation
/// - [`EnhancedWorkflow`] - Enhanced workflow implementation
/// - [`WorkflowStatus`] - Workflow status enum
/// - [`RuntimeContext`] - Execution context
#[async_trait]
pub trait Workflow: Send + Sync {
    /// Returns the unique identifier of the workflow
    ///
    /// # Returns
    ///
    /// A string identifier for the workflow.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::workflow::Workflow;
    ///
    /// # fn example(workflow: &dyn Workflow) {
    /// println!("Workflow ID: {}", workflow.id());
    /// # }
    /// ```
    fn id(&self) -> &str;

    /// Returns the description of the workflow
    ///
    /// # Returns
    ///
    /// An optional description string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::workflow::Workflow;
    ///
    /// # fn example(workflow: &dyn Workflow) {
    /// if let Some(desc) = workflow.description() {
    ///     println!("Description: {}", desc);
    /// }
    /// # }
    /// ```
    fn description(&self) -> Option<&str>;

    /// Executes the workflow with the given input
    ///
    /// This method runs the workflow to completion, executing all steps
    /// according to their dependencies and configuration.
    ///
    /// # Arguments
    ///
    /// * `input` - Input data for the workflow (JSON value)
    /// * `context` - Runtime context for execution
    ///
    /// # Returns
    ///
    /// The final output of the workflow as a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any step fails and cannot be retried
    /// - The workflow contains cycles (for DAG workflows)
    /// - Input validation fails
    /// - Timeout is exceeded
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::workflow::Workflow;
    /// use lumosai_core::agent::types::RuntimeContext;
    /// use serde_json::json;
    ///
    /// # async fn example(workflow: &dyn Workflow) -> lumosai_core::Result<()> {
    /// let context = RuntimeContext::new();
    /// let input = json!({
    ///     "document": "Process this document",
    ///     "options": {"format": "pdf"}
    /// });
    ///
    /// let result = workflow.execute(input, &context).await?;
    /// println!("Result: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    async fn execute(&self, input: Value, context: &RuntimeContext) -> Result<Value>;

    /// Executes the workflow with streaming output
    ///
    /// This method returns a stream of intermediate results as each step
    /// completes, enabling real-time progress monitoring.
    ///
    /// # Arguments
    ///
    /// * `input` - Input data for the workflow
    /// * `context` - Runtime context for execution
    ///
    /// # Returns
    ///
    /// A stream of step results. Each item is a `Result<Value>`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workflow cannot be started
    /// - Streaming is not supported by the workflow type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::workflow::Workflow;
    /// use lumosai_core::agent::types::RuntimeContext;
    /// use serde_json::json;
    /// use futures::StreamExt;
    ///
    /// # async fn example(workflow: &dyn Workflow) -> lumosai_core::Result<()> {
    /// let context = RuntimeContext::new();
    /// let mut stream = workflow.execute_stream(json!({}), &context).await?;
    ///
    /// while let Some(result) = stream.next().await {
    ///     match result {
    ///         Ok(value) => println!("Step completed: {:?}", value),
    ///         Err(e) => eprintln!("Step failed: {}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn execute_stream(
        &self,
        input: Value,
        context: &RuntimeContext,
    ) -> Result<Box<dyn futures::Stream<Item = Result<Value>> + Send + Unpin>>;

    /// Suspends a running workflow
    ///
    /// Pauses the workflow execution, allowing it to be resumed later.
    ///
    /// # Arguments
    ///
    /// * `run_id` - The unique identifier of the workflow run
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workflow run is not found
    /// - The workflow is not in a running state
    /// - Suspension is not supported
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::workflow::Workflow;
    ///
    /// # async fn example(workflow: &dyn Workflow) -> lumosai_core::Result<()> {
    /// workflow.suspend("run-123").await?;
    /// println!("Workflow suspended");
    /// # Ok(())
    /// # }
    /// ```
    async fn suspend(&self, run_id: &str) -> Result<()>;

    /// Resumes a suspended workflow
    ///
    /// Continues execution of a previously suspended workflow, optionally
    /// with new input data.
    ///
    /// # Arguments
    ///
    /// * `run_id` - The unique identifier of the workflow run
    /// * `input` - Optional new input data to use when resuming
    ///
    /// # Returns
    ///
    /// The final output of the workflow.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workflow run is not found
    /// - The workflow is not in a suspended state
    /// - Resume fails due to invalid state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::workflow::Workflow;
    /// use serde_json::json;
    ///
    /// # async fn example(workflow: &dyn Workflow) -> lumosai_core::Result<()> {
    /// // Resume with original input
    /// let result = workflow.resume("run-123", None).await?;
    ///
    /// // Resume with new input
    /// let result = workflow.resume("run-456", Some(json!({"new": "data"}))).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn resume(&self, run_id: &str, input: Option<Value>) -> Result<Value>;

    /// Retrieves the status of a workflow run
    ///
    /// # Arguments
    ///
    /// * `run_id` - The unique identifier of the workflow run
    ///
    /// # Returns
    ///
    /// The current status of the workflow run.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::workflow::{Workflow, WorkflowStatus};
    ///
    /// # async fn example(workflow: &dyn Workflow) -> lumosai_core::Result<()> {
    /// let status = workflow.get_status("run-123").await?;
    ///
    /// match status {
    ///     WorkflowStatus::Running => println!("Still running"),
    ///     WorkflowStatus::Completed(result) => println!("Done: {:?}", result),
    ///     WorkflowStatus::Failed(error) => println!("Failed: {}", error),
    ///     WorkflowStatus::Suspended => println!("Paused"),
    ///     WorkflowStatus::NotFound => println!("Not found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn get_status(&self, run_id: &str) -> Result<WorkflowStatus>;
}

/// Workflow execution status
///
/// Represents the current state of a workflow run.
///
/// # Examples
///
/// ```rust
/// use lumosai_core::workflow::WorkflowStatus;
/// use serde_json::json;
///
/// let status = WorkflowStatus::Completed(json!({"result": "success"}));
///
/// match status {
///     WorkflowStatus::Running => println!("In progress"),
///     WorkflowStatus::Completed(value) => println!("Done: {:?}", value),
///     WorkflowStatus::Failed(error) => println!("Error: {}", error),
///     WorkflowStatus::Suspended => println!("Paused"),
///     WorkflowStatus::NotFound => println!("Not found"),
/// }
/// ```
#[derive(Debug, Clone)]
pub enum WorkflowStatus {
    /// Workflow is currently running
    Running,

    /// Workflow completed successfully
    ///
    /// Contains the final output value.
    Completed(Value),

    /// Workflow failed with an error
    ///
    /// Contains the error message.
    Failed(String),

    /// Workflow is suspended (paused)
    Suspended,

    /// Workflow run was not found
    NotFound,
}

// 重新导出公共项
pub use basic::BasicWorkflow;
pub use dag_scheduler::{Dag, DagNode, DagScheduler};
pub use dag_workflow::{DagWorkflow, DagWorkflowBuilder, DagWorkflowRun};
pub use workflow::WorkflowBuilder;
pub use enhanced::{EnhancedWorkflow, StepExecutor, StepFlowEntry, StepType, WorkflowStep};
pub use execution_engine::{DefaultExecutionEngine, ExecutionEngine, ExecutionMetrics};
pub use step::{BasicStep, StepBuilder, StepConfig};
pub use types::{RetryConfig, Step, StepContext, StepStatus, WorkflowRunResult, WorkflowState};
pub use workflow::{resume_workflow, Workflow as WorkflowImpl, WorkflowInstance};
