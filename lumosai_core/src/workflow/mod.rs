//! 工作流模块，用于执行由步骤组成的工作流
//!
//! 基于Mastra的设计，提供强大的工作流编排能力
#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(non_camel_case_types, ambiguous_glob_reexports, hidden_glob_reexports)]
#![allow(unexpected_cfgs, unused_assignments)]

mod step;
mod tests;
mod types;
mod workflow;
pub mod basic;
pub mod enhanced;
pub mod execution_engine;

use async_trait::async_trait;
use serde_json::Value;
use crate::{Result, Error};
use crate::agent::types::RuntimeContext;

/// 工作流trait，定义工作流的核心接口
#[async_trait]
pub trait Workflow: Send + Sync {
    /// 获取工作流ID
    fn id(&self) -> &str;

    /// 获取工作流描述
    fn description(&self) -> Option<&str>;

    /// 执行工作流
    async fn execute(&self, input: Value, context: &RuntimeContext) -> Result<Value>;

    /// 流式执行工作流
    async fn execute_stream(&self, input: Value, context: &RuntimeContext) -> Result<Box<dyn futures::Stream<Item = Result<Value>> + Send + Unpin>>;

    /// 暂停工作流执行
    async fn suspend(&self, run_id: &str) -> Result<()>;

    /// 恢复工作流执行
    async fn resume(&self, run_id: &str, input: Option<Value>) -> Result<Value>;

    /// 获取工作流状态
    async fn get_status(&self, run_id: &str) -> Result<WorkflowStatus>;
}

/// 工作流状态
#[derive(Debug, Clone)]
pub enum WorkflowStatus {
    /// 运行中
    Running,
    /// 已完成
    Completed(Value),
    /// 已失败
    Failed(String),
    /// 已暂停
    Suspended,
    /// 未找到
    NotFound,
}

// 重新导出公共项
pub use step::{BasicStep, StepBuilder, StepConfig};
pub use workflow::{Workflow as WorkflowImpl, WorkflowInstance, resume_workflow};
pub use types::{Step, StepContext, StepStatus, RetryConfig, WorkflowRunResult, WorkflowState};
pub use enhanced::{EnhancedWorkflow, WorkflowStep, StepFlowEntry, StepExecutor, StepType};
pub use execution_engine::{ExecutionEngine, DefaultExecutionEngine, ExecutionMetrics};
