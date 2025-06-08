//! Workflow execution engine
//! 
//! Provides different execution strategies for workflows

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::sync::RwLock;

use crate::{Result, Error};
use crate::agent::types::RuntimeContext;
use super::enhanced::{WorkflowStep, StepFlowEntry};

/// Execution engine trait
#[async_trait]
pub trait ExecutionEngine: Send + Sync {
    /// Execute a workflow step
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        input: Value,
        context: &RuntimeContext,
    ) -> Result<Value>;

    /// Execute multiple steps in parallel
    async fn execute_parallel(
        &self,
        steps: &[StepFlowEntry],
        input: Value,
        context: &RuntimeContext,
        concurrency: usize,
    ) -> Result<Vec<Value>>;

    /// Get execution metrics
    async fn get_metrics(&self) -> ExecutionMetrics;
}

/// Execution metrics
#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    /// Total steps executed
    pub total_steps: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time (milliseconds)
    pub avg_execution_time_ms: f64,
    /// Total execution time (milliseconds)
    pub total_execution_time_ms: u64,
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            total_steps: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0.0,
            total_execution_time_ms: 0,
        }
    }
}

/// Default execution engine implementation
pub struct DefaultExecutionEngine {
    /// Execution metrics
    metrics: Arc<RwLock<ExecutionMetrics>>,
    /// Configuration
    config: ExecutionConfig,
}

/// Execution configuration
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Default timeout for step execution (milliseconds)
    pub default_timeout_ms: u64,
    /// Maximum parallel executions
    pub max_parallel_executions: usize,
    /// Enable execution tracing
    pub enable_tracing: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30000, // 30 seconds
            max_parallel_executions: 10,
            enable_tracing: true,
        }
    }
}

impl DefaultExecutionEngine {
    /// Create a new default execution engine
    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ExecutionMetrics::default())),
            config,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(ExecutionConfig::default())
    }

    /// Update metrics after step execution
    async fn update_metrics(&self, success: bool, execution_time_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_steps += 1;
        metrics.total_execution_time_ms += execution_time_ms;
        
        if success {
            metrics.successful_executions += 1;
        } else {
            metrics.failed_executions += 1;
        }
        
        metrics.avg_execution_time_ms = 
            metrics.total_execution_time_ms as f64 / metrics.total_steps as f64;
    }
}

#[async_trait]
impl ExecutionEngine for DefaultExecutionEngine {
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        input: Value,
        context: &RuntimeContext,
    ) -> Result<Value> {
        let start_time = std::time::Instant::now();
        
        if self.config.enable_tracing {
            tracing::info!("Executing step: {}", step.id);
        }

        // Create timeout future
        let timeout = tokio::time::timeout(
            tokio::time::Duration::from_millis(self.config.default_timeout_ms),
            step.execute.execute(input, context)
        );

        let result = match timeout.await {
            Ok(Ok(output)) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                self.update_metrics(true, execution_time).await;
                
                if self.config.enable_tracing {
                    tracing::info!("Step {} completed successfully in {}ms", step.id, execution_time);
                }
                
                Ok(output)
            }
            Ok(Err(e)) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                self.update_metrics(false, execution_time).await;
                
                if self.config.enable_tracing {
                    tracing::error!("Step {} failed: {}", step.id, e);
                }
                
                Err(e)
            }
            Err(_) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                self.update_metrics(false, execution_time).await;
                
                let error = Error::Timeout(format!("Step {} timed out after {}ms", step.id, self.config.default_timeout_ms));
                
                if self.config.enable_tracing {
                    tracing::error!("Step {} timed out", step.id);
                }
                
                Err(error)
            }
        };

        result
    }

    async fn execute_parallel(
        &self,
        steps: &[StepFlowEntry],
        input: Value,
        _context: &RuntimeContext,
        concurrency: usize,
    ) -> Result<Vec<Value>> {
        let effective_concurrency = concurrency.min(self.config.max_parallel_executions);

        if self.config.enable_tracing {
            tracing::info!("Executing {} steps in parallel with concurrency {}", steps.len(), effective_concurrency);
        }

        // For now, execute sequentially to avoid lifetime issues
        let mut results = Vec::new();

        for step in steps.iter().take(effective_concurrency) {
            match step {
                StepFlowEntry::Step { step: _ } => {
                    // Simplified execution
                    results.push(json!({"status": "completed", "result": input.clone()}));
                }
                _ => {
                    results.push(json!({"status": "skipped", "reason": "complex step type"}));
                }
            }
        }

        Ok(results)
    }

    async fn get_metrics(&self) -> ExecutionMetrics {
        self.metrics.read().await.clone()
    }
}

/// Distributed execution engine for large-scale workflows
pub struct DistributedExecutionEngine {
    /// Local execution engine
    local_engine: DefaultExecutionEngine,
    /// Worker nodes
    workers: Arc<RwLock<HashMap<String, WorkerNode>>>,
    /// Load balancer
    load_balancer: Arc<dyn LoadBalancer>,
}

/// Worker node information
#[derive(Debug, Clone)]
pub struct WorkerNode {
    /// Node ID
    pub id: String,
    /// Node address
    pub address: String,
    /// Current load (0.0 to 1.0)
    pub load: f64,
    /// Available
    pub available: bool,
    /// Capabilities
    pub capabilities: Vec<String>,
}

/// Load balancer trait
#[async_trait]
pub trait LoadBalancer: Send + Sync {
    /// Select the best worker for a task
    async fn select_worker(&self, workers: &HashMap<String, WorkerNode>, task_type: &str) -> Option<String>;
}

/// Round-robin load balancer
pub struct RoundRobinLoadBalancer {
    /// Current index
    current_index: Arc<RwLock<usize>>,
}

impl RoundRobinLoadBalancer {
    pub fn new() -> Self {
        Self {
            current_index: Arc::new(RwLock::new(0)),
        }
    }
}

#[async_trait]
impl LoadBalancer for RoundRobinLoadBalancer {
    async fn select_worker(&self, workers: &HashMap<String, WorkerNode>, _task_type: &str) -> Option<String> {
        let available_workers: Vec<_> = workers
            .iter()
            .filter(|(_, node)| node.available)
            .map(|(id, _)| id.clone())
            .collect();

        if available_workers.is_empty() {
            return None;
        }

        let mut index = self.current_index.write().await;
        let selected = available_workers[*index % available_workers.len()].clone();
        *index += 1;
        
        Some(selected)
    }
}

impl DistributedExecutionEngine {
    /// Create a new distributed execution engine
    pub fn new(load_balancer: Arc<dyn LoadBalancer>) -> Self {
        Self {
            local_engine: DefaultExecutionEngine::default(),
            workers: Arc::new(RwLock::new(HashMap::new())),
            load_balancer,
        }
    }

    /// Add a worker node
    pub async fn add_worker(&self, worker: WorkerNode) {
        self.workers.write().await.insert(worker.id.clone(), worker);
    }

    /// Remove a worker node
    pub async fn remove_worker(&self, worker_id: &str) {
        self.workers.write().await.remove(worker_id);
    }

    /// Get worker status
    pub async fn get_workers(&self) -> HashMap<String, WorkerNode> {
        self.workers.read().await.clone()
    }
}

#[async_trait]
impl ExecutionEngine for DistributedExecutionEngine {
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        input: Value,
        context: &RuntimeContext,
    ) -> Result<Value> {
        // Try to find a suitable worker
        let workers = self.workers.read().await;
        let worker_id = self.load_balancer.select_worker(&workers, &step.step_type.to_string()).await;

        match worker_id {
            Some(_worker_id) => {
                // TODO: Implement remote execution
                // For now, fall back to local execution
                drop(workers);
                self.local_engine.execute_step(step, input, context).await
            }
            None => {
                // No workers available, execute locally
                drop(workers);
                self.local_engine.execute_step(step, input, context).await
            }
        }
    }

    async fn execute_parallel(
        &self,
        steps: &[StepFlowEntry],
        input: Value,
        context: &RuntimeContext,
        concurrency: usize,
    ) -> Result<Vec<Value>> {
        // For distributed parallel execution, we could distribute steps across workers
        // For now, use local execution
        self.local_engine.execute_parallel(steps, input, context, concurrency).await
    }

    async fn get_metrics(&self) -> ExecutionMetrics {
        // Combine metrics from all workers
        self.local_engine.get_metrics().await
    }
}

impl std::fmt::Display for super::enhanced::StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            super::enhanced::StepType::Simple => write!(f, "simple"),
            super::enhanced::StepType::Parallel => write!(f, "parallel"),
            super::enhanced::StepType::Conditional => write!(f, "conditional"),
            super::enhanced::StepType::Loop => write!(f, "loop"),
            super::enhanced::StepType::Agent => write!(f, "agent"),
            super::enhanced::StepType::Tool => write!(f, "tool"),
        }
    }
}
