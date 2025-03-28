//! 工作流模块，用于执行由步骤组成的工作流

mod types;
mod step;
mod workflow;
#[cfg(test)]
mod tests;

// 重新导出公共项
pub use types::{
    Step, StepContext, StepResult, StepStatus, StepCondition,
    WorkflowRunResult, WorkflowState, RetryConfig
};
pub use step::{BasicStep, StepBuilder, StepConfig};
pub use workflow::{Workflow as WorkflowImpl, WorkflowInstance, resume_workflow}; 

// 创建一个Workflow trait
pub trait Workflow: Send + Sync {
    /// 获取工作流ID
    fn id(&self) -> &str;
    
    /// 获取工作流名称
    fn name(&self) -> &str;
    
    /// 创建一个工作流实例
    fn create_run(&self, trigger_data: serde_json::Value) -> WorkflowInstance;
} 