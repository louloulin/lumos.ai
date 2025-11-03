//! DAG-based Workflow Implementation
//!
//! 提供基于 DAG 的工作流实现，支持：
//! - 自动依赖分析
//! - 智能并行调度
//! - 步骤依赖管理
//! - 执行结果追踪

use super::dag_scheduler::{Dag, DagNode, DagScheduler};
use super::enhanced::{StepExecutor, WorkflowStep};
use super::{Workflow, WorkflowStatus};
use crate::agent::types::RuntimeContext;
use crate::{Error, Result};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// DAG-based workflow
pub struct DagWorkflow {
    /// Workflow ID
    id: String,
    /// Workflow description
    description: Option<String>,
    /// DAG scheduler
    scheduler: DagScheduler,
    /// DAG graph
    dag: Arc<RwLock<Dag>>,
    /// Workflow runs
    runs: Arc<RwLock<HashMap<String, DagWorkflowRun>>>,
}

/// DAG workflow run state
#[derive(Debug, Clone)]
pub struct DagWorkflowRun {
    /// Run ID
    pub id: String,
    /// Current status
    pub status: WorkflowStatus,
    /// Input data
    pub input: Value,
    /// Output data (all node results)
    pub output: Option<HashMap<String, Value>>,
    /// Error information
    pub error: Option<String>,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl DagWorkflow {
    /// Create a new DAG workflow
    pub fn new(id: String, description: Option<String>) -> Self {
        Self {
            id,
            description,
            scheduler: DagScheduler::new(num_cpus::get().max(4)),
            dag: Arc::new(RwLock::new(Dag::new())),
            runs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new DAG workflow with custom concurrency
    pub fn with_concurrency(
        id: String,
        description: Option<String>,
        max_concurrency: usize,
    ) -> Self {
        Self {
            id,
            description,
            scheduler: DagScheduler::new(max_concurrency),
            dag: Arc::new(RwLock::new(Dag::new())),
            runs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a node to the DAG
    pub async fn add_node(
        &self,
        id: String,
        name: String,
        dependencies: Vec<String>,
        step: WorkflowStep,
    ) -> Result<()> {
        let node = DagNode {
            id,
            name,
            dependencies,
            step,
        };

        let mut dag = self.dag.write().await;
        dag.add_node(node)?;

        Ok(())
    }

    /// Add a node without dependencies (root node)
    pub async fn add_root_node(&self, id: String, name: String, step: WorkflowStep) -> Result<()> {
        self.add_node(id, name, vec![], step).await
    }

    /// Validate the DAG (check for cycles)
    pub async fn validate(&self) -> Result<()> {
        let dag = self.dag.read().await;
        dag.detect_cycle()?;
        Ok(())
    }

    /// Get topological sort levels
    pub async fn get_execution_levels(&self) -> Result<Vec<Vec<String>>> {
        let dag = self.dag.read().await;
        dag.topological_sort()
    }

    /// Get node count
    pub async fn node_count(&self) -> usize {
        let dag = self.dag.read().await;
        dag.node_count()
    }

    /// Execute the workflow with DAG scheduler
    pub async fn execute_dag(&self, input: Value, context: &RuntimeContext) -> Result<Value> {
        let run_id = Uuid::new_v4().to_string();

        // Create run state
        let run = DagWorkflowRun {
            id: run_id.clone(),
            status: WorkflowStatus::Running,
            input: input.clone(),
            output: None,
            error: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Store run
        self.runs.write().await.insert(run_id.clone(), run);

        // Execute DAG
        let dag = self.dag.read().await;
        let result = self.scheduler.execute(&dag, input, context).await;

        // Update run state
        let mut runs = self.runs.write().await;
        if let Some(run) = runs.get_mut(&run_id) {
            match &result {
                Ok(output) => {
                    run.status = WorkflowStatus::Completed(json!(output));
                    run.output = Some(output.clone());
                }
                Err(e) => {
                    run.status = WorkflowStatus::Failed(e.to_string());
                    run.error = Some(e.to_string());
                }
            }
            run.updated_at = chrono::Utc::now();
        }

        // Return final result
        result.map(|output| json!(output))
    }

    /// Get run by ID
    pub async fn get_run(&self, run_id: &str) -> Option<DagWorkflowRun> {
        let runs = self.runs.read().await;
        runs.get(run_id).cloned()
    }

    /// List all runs
    pub async fn list_runs(&self) -> Vec<DagWorkflowRun> {
        let runs = self.runs.read().await;
        runs.values().cloned().collect()
    }
}

#[async_trait]
impl Workflow for DagWorkflow {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn execute(&self, input: Value, context: &RuntimeContext) -> Result<Value> {
        self.execute_dag(input, context).await
    }

    async fn execute_stream(
        &self,
        input: Value,
        context: &RuntimeContext,
    ) -> Result<Box<dyn futures::Stream<Item = Result<Value>> + Send + Unpin>> {
        // For DAG workflow, we can stream results as each level completes
        let dag = self.dag.read().await.clone();
        let levels = dag.topological_sort()?;
        let scheduler = self.scheduler.clone();
        let context = context.clone();

        let stream = stream::iter(levels)
            .then(move |level| {
                let dag = dag.clone();
                let scheduler = scheduler.clone();
                let input = input.clone();
                let context = context.clone();

                async move {
                    // Execute this level
                    let results: Arc<RwLock<HashMap<String, Value>>> = Arc::new(RwLock::new(HashMap::new()));
                    // This is a simplified version - in production we'd need proper level execution
                    Ok(json!({
                        "level": level,
                        "status": "completed"
                    }))
                }
            })
            .boxed();

        Ok(Box::new(stream))
    }

    async fn suspend(&self, run_id: &str) -> Result<()> {
        let mut runs = self.runs.write().await;
        if let Some(run) = runs.get_mut(run_id) {
            run.status = WorkflowStatus::Suspended;
            run.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(Error::Workflow(format!("Run not found: {}", run_id)))
        }
    }

    async fn resume(&self, run_id: &str, input: Option<Value>) -> Result<Value> {
        let run = {
            let runs = self.runs.read().await;
            runs.get(run_id).cloned()
        };

        if let Some(run) = run {
            let input = input.unwrap_or(run.input);
            let context = RuntimeContext::new();
            self.execute_dag(input, &context).await
        } else {
            Err(Error::Workflow(format!("Run not found: {}", run_id)))
        }
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

/// Builder for DAG workflow
pub struct DagWorkflowBuilder {
    id: String,
    description: Option<String>,
    max_concurrency: Option<usize>,
}

impl DagWorkflowBuilder {
    /// Create a new builder
    pub fn new(id: String) -> Self {
        Self {
            id,
            description: None,
            max_concurrency: None,
        }
    }

    /// Set description
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set max concurrency
    pub fn max_concurrency(mut self, max_concurrency: usize) -> Self {
        self.max_concurrency = Some(max_concurrency);
        self
    }

    /// Build the workflow
    pub fn build(self) -> DagWorkflow {
        if let Some(max_concurrency) = self.max_concurrency {
            DagWorkflow::with_concurrency(self.id, self.description, max_concurrency)
        } else {
            DagWorkflow::new(self.id, self.description)
        }
    }
}

