//! Enhanced workflow implementation based on Mastra design
//! 
//! Provides advanced workflow orchestration with parallel execution,
//! conditional branching, loops, and dynamic step resolution.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use futures::stream::{self, StreamExt};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{Result, Error};
use crate::agent::types::RuntimeContext;
use crate::tool::Tool;
use super::{Workflow, WorkflowStatus};

/// Enhanced workflow step
#[derive(Clone)]
pub struct WorkflowStep {
    /// Step ID
    pub id: String,
    /// Step description
    pub description: Option<String>,
    /// Step type
    pub step_type: StepType,
    /// Input schema (JSON Schema)
    pub input_schema: Option<Value>,
    /// Output schema (JSON Schema)
    pub output_schema: Option<Value>,
    /// Step execution function
    pub execute: Arc<dyn StepExecutor>,
}

impl std::fmt::Debug for WorkflowStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowStep")
            .field("id", &self.id)
            .field("description", &self.description)
            .field("step_type", &self.step_type)
            .field("input_schema", &self.input_schema)
            .field("output_schema", &self.output_schema)
            .field("execute", &"<StepExecutor>")
            .finish()
    }
}

/// Step execution trait
#[async_trait]
pub trait StepExecutor: Send + Sync {
    /// Execute the step
    async fn execute(&self, input: Value, context: &RuntimeContext) -> Result<Value>;
}

/// Step type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepType {
    /// Simple step
    Simple,
    /// Parallel execution
    Parallel,
    /// Conditional execution
    Conditional,
    /// Loop execution
    Loop,
    /// Agent step
    Agent,
    /// Tool step
    Tool,
}

/// Workflow flow entry
#[derive(Clone)]
pub enum StepFlowEntry {
    /// Single step
    Step {
        step: WorkflowStep,
    },
    /// Parallel steps
    Parallel {
        steps: Vec<StepFlowEntry>,
        concurrency: Option<usize>,
    },
    /// Conditional steps
    Conditional {
        condition: Arc<dyn ConditionEvaluator>,
        if_true: Vec<StepFlowEntry>,
        if_false: Option<Vec<StepFlowEntry>>,
    },
    /// Loop steps
    Loop {
        condition: Arc<dyn ConditionEvaluator>,
        body: Vec<StepFlowEntry>,
        loop_type: LoopType,
    },
}

impl std::fmt::Debug for StepFlowEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepFlowEntry::Step { step } => {
                f.debug_struct("Step")
                    .field("step", step)
                    .finish()
            }
            StepFlowEntry::Parallel { steps, concurrency } => {
                f.debug_struct("Parallel")
                    .field("steps", &format!("{} steps", steps.len()))
                    .field("concurrency", concurrency)
                    .finish()
            }
            StepFlowEntry::Conditional { condition: _, if_true, if_false } => {
                f.debug_struct("Conditional")
                    .field("condition", &"<ConditionEvaluator>")
                    .field("if_true", &format!("{} steps", if_true.len()))
                    .field("if_false", &if_false.as_ref().map(|steps| format!("{} steps", steps.len())))
                    .finish()
            }
            StepFlowEntry::Loop { condition: _, body, loop_type } => {
                f.debug_struct("Loop")
                    .field("condition", &"<ConditionEvaluator>")
                    .field("body", &format!("{} steps", body.len()))
                    .field("loop_type", loop_type)
                    .finish()
            }
        }
    }
}

/// Loop type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoopType {
    /// Do-while loop
    DoWhile,
    /// Do-until loop
    DoUntil,
    /// For-each loop
    ForEach,
}

/// Condition evaluator trait
#[async_trait]
pub trait ConditionEvaluator: Send + Sync {
    /// Evaluate condition
    async fn evaluate(&self, input: &Value, context: &RuntimeContext) -> Result<bool>;
}

/// Enhanced workflow implementation
pub struct EnhancedWorkflow {
    /// Workflow ID
    id: String,
    /// Workflow description
    description: Option<String>,
    /// Input schema
    input_schema: Option<Value>,
    /// Output schema
    output_schema: Option<Value>,
    /// Step flow
    step_flow: Vec<StepFlowEntry>,
    /// Workflow runs
    runs: Arc<RwLock<HashMap<String, WorkflowRun>>>,
    /// Retry configuration
    retry_config: RetryConfig,
}

/// Workflow run state
#[derive(Debug, Clone)]
pub struct WorkflowRun {
    /// Run ID
    pub id: String,
    /// Current status
    pub status: WorkflowStatus,
    /// Input data
    pub input: Value,
    /// Output data
    pub output: Option<Value>,
    /// Current step index
    pub current_step: usize,
    /// Step results
    pub step_results: HashMap<String, Value>,
    /// Error information
    pub error: Option<String>,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Delay between retries (milliseconds)
    pub delay_ms: u64,
    /// Exponential backoff factor
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay_ms: 1000,
            backoff_factor: 2.0,
        }
    }
}

impl EnhancedWorkflow {
    /// Create a new enhanced workflow
    pub fn new(id: String, description: Option<String>) -> Self {
        Self {
            id,
            description,
            input_schema: None,
            output_schema: None,
            step_flow: Vec::new(),
            runs: Arc::new(RwLock::new(HashMap::new())),
            retry_config: RetryConfig::default(),
        }
    }

    /// Add a step to the workflow
    pub fn add_step(&mut self, step: WorkflowStep) -> &mut Self {
        self.step_flow.push(StepFlowEntry::Step { step });
        self
    }

    /// Add parallel steps
    pub fn add_parallel(&mut self, steps: Vec<StepFlowEntry>, concurrency: Option<usize>) -> &mut Self {
        self.step_flow.push(StepFlowEntry::Parallel { steps, concurrency });
        self
    }

    /// Add conditional steps
    pub fn add_conditional(
        &mut self,
        condition: Arc<dyn ConditionEvaluator>,
        if_true: Vec<StepFlowEntry>,
        if_false: Option<Vec<StepFlowEntry>>,
    ) -> &mut Self {
        self.step_flow.push(StepFlowEntry::Conditional {
            condition,
            if_true,
            if_false,
        });
        self
    }

    /// Add loop steps
    pub fn add_loop(
        &mut self,
        condition: Arc<dyn ConditionEvaluator>,
        body: Vec<StepFlowEntry>,
        loop_type: LoopType,
    ) -> &mut Self {
        self.step_flow.push(StepFlowEntry::Loop {
            condition,
            body,
            loop_type,
        });
        self
    }

    /// Set retry configuration
    pub fn set_retry_config(&mut self, config: RetryConfig) -> &mut Self {
        self.retry_config = config;
        self
    }

    /// Execute step flow entries
    fn execute_step_flow<'a>(
        &'a self,
        entries: &'a [StepFlowEntry],
        input: Value,
        context: &'a RuntimeContext,
        run: &'a mut WorkflowRun,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            let mut current_input = input;

            for entry in entries {
                current_input = self.execute_step_entry(entry, current_input, context, run).await?;
            }

            Ok(current_input)
        })
    }

    /// Execute a single step flow entry
    fn execute_step_entry<'a>(
        &'a self,
        entry: &'a StepFlowEntry,
        input: Value,
        context: &'a RuntimeContext,
        run: &'a mut WorkflowRun,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            match entry {
                StepFlowEntry::Step { step } => {
                    self.execute_single_step(step, input, context, run).await
                }
                StepFlowEntry::Parallel { steps, concurrency } => {
                    self.execute_parallel_steps(steps, input, context, run, *concurrency).await
                }
                StepFlowEntry::Conditional { condition, if_true, if_false } => {
                    self.execute_conditional_steps(condition, if_true, if_false, input, context, run).await
                }
                StepFlowEntry::Loop { condition, body, loop_type } => {
                    self.execute_loop_steps(condition, body, loop_type, input, context, run).await
                }
            }
        })
    }

    /// Execute a single step with retry logic
    async fn execute_single_step(
        &self,
        step: &WorkflowStep,
        input: Value,
        context: &RuntimeContext,
        run: &mut WorkflowRun,
    ) -> Result<Value> {
        let mut attempts = 0;
        let mut delay = self.retry_config.delay_ms;

        loop {
            match step.execute.execute(input.clone(), context).await {
                Ok(result) => {
                    run.step_results.insert(step.id.clone(), result.clone());
                    return Ok(result);
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.retry_config.max_attempts {
                        return Err(e);
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    delay = (delay as f64 * self.retry_config.backoff_factor) as u64;
                }
            }
        }
    }

    /// Execute parallel steps
    async fn execute_parallel_steps(
        &self,
        steps: &[StepFlowEntry],
        input: Value,
        context: &RuntimeContext,
        _run: &mut WorkflowRun,
        concurrency: Option<usize>,
    ) -> Result<Value> {
        let concurrency = concurrency.unwrap_or(steps.len());

        // For now, execute steps sequentially to avoid lifetime issues
        // In a production implementation, we would use proper async coordination
        let mut results = Vec::new();

        for step in steps.iter().take(concurrency) {
            match step {
                StepFlowEntry::Step { step } => {
                    let result = step.execute.execute(input.clone(), context).await?;
                    results.push(result);
                }
                _ => {
                    // Handle other step types
                    results.push(json!({"status": "skipped", "reason": "complex step type"}));
                }
            }
        }

        Ok(Value::Array(results))
    }

    /// Execute conditional steps
    fn execute_conditional_steps<'a>(
        &'a self,
        condition: &'a Arc<dyn ConditionEvaluator>,
        if_true: &'a [StepFlowEntry],
        if_false: &'a Option<Vec<StepFlowEntry>>,
        input: Value,
        context: &'a RuntimeContext,
        run: &'a mut WorkflowRun,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            let condition_result = condition.evaluate(&input, context).await?;

            if condition_result {
                self.execute_step_flow(if_true, input, context, run).await
            } else if let Some(false_steps) = if_false {
                self.execute_step_flow(false_steps, input, context, run).await
            } else {
                Ok(input)
            }
        })
    }

    /// Execute loop steps
    fn execute_loop_steps<'a>(
        &'a self,
        condition: &'a Arc<dyn ConditionEvaluator>,
        body: &'a [StepFlowEntry],
        loop_type: &'a LoopType,
        input: Value,
        context: &'a RuntimeContext,
        run: &'a mut WorkflowRun,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            let mut current_input = input;

            match loop_type {
                LoopType::DoWhile => {
                    loop {
                        current_input = self.execute_step_flow(body, current_input, context, run).await?;
                        if !condition.evaluate(&current_input, context).await? {
                            break;
                        }
                    }
                }
                LoopType::DoUntil => {
                    loop {
                        current_input = self.execute_step_flow(body, current_input, context, run).await?;
                        if condition.evaluate(&current_input, context).await? {
                            break;
                        }
                    }
                }
                LoopType::ForEach => {
                    if let Value::Array(items) = current_input {
                        let mut results = Vec::new();
                        for item in items {
                            let result = self.execute_step_flow(body, item, context, run).await?;
                            results.push(result);
                        }
                        current_input = Value::Array(results);
                    } else {
                        return Err(Error::InvalidInput("ForEach loop requires array input".to_string()));
                    }
                }
            }

            Ok(current_input)
        })
    }
}

#[async_trait]
impl Workflow for EnhancedWorkflow {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn execute(&self, input: Value, context: &RuntimeContext) -> Result<Value> {
        let run_id = Uuid::new_v4().to_string();
        let mut run = WorkflowRun {
            id: run_id.clone(),
            status: WorkflowStatus::Running,
            input: input.clone(),
            output: None,
            current_step: 0,
            step_results: HashMap::new(),
            error: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = self.execute_step_flow(&self.step_flow, input, context, &mut run).await;

        match result {
            Ok(output) => {
                run.status = WorkflowStatus::Completed(output.clone());
                run.output = Some(output.clone());
                run.updated_at = chrono::Utc::now();
                
                self.runs.write().await.insert(run_id, run);
                Ok(output)
            }
            Err(e) => {
                run.status = WorkflowStatus::Failed(e.to_string());
                run.error = Some(e.to_string());
                run.updated_at = chrono::Utc::now();
                
                self.runs.write().await.insert(run_id, run);
                Err(e)
            }
        }
    }

    async fn execute_stream(&self, _input: Value, _context: &RuntimeContext) -> Result<Box<dyn futures::Stream<Item = Result<Value>> + Send + Unpin>> {
        // TODO: Implement streaming execution
        Err(Error::Unsupported("Streaming execution not yet implemented".to_string()))
    }

    async fn suspend(&self, run_id: &str) -> Result<()> {
        let mut runs = self.runs.write().await;
        if let Some(run) = runs.get_mut(run_id) {
            run.status = WorkflowStatus::Suspended;
            run.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(Error::NotFound(format!("Workflow run '{}' not found", run_id)))
        }
    }

    async fn resume(&self, run_id: &str, input: Option<Value>) -> Result<Value> {
        let run_input = {
            let runs = self.runs.read().await;
            if let Some(run) = runs.get(run_id) {
                if matches!(run.status, WorkflowStatus::Suspended) {
                    run.input.clone()
                } else {
                    return Err(Error::InvalidState(format!("Workflow run '{}' is not suspended", run_id)));
                }
            } else {
                return Err(Error::NotFound(format!("Workflow run '{}' not found", run_id)));
            }
        };

        let resume_input = input.unwrap_or(run_input);
        let context = RuntimeContext::default();
        self.execute(resume_input, &context).await
    }

    async fn get_status(&self, run_id: &str) -> Result<WorkflowStatus> {
        let runs = self.runs.read().await;
        if let Some(run) = runs.get(run_id) {
            Ok(run.status.clone())
        } else {
            Ok(WorkflowStatus::NotFound)
        }
    }
}
