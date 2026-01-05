//! # Agent 推理系统 (Reasoning System)
//!
//! LumosAI 的高级推理能力,包括 ReAct、CoT 和自我反思。
//!
//! ## 核心功能
//!
//! - **ReAct** - Reasoning + Acting 循环
//! - **CoT** - Chain of Thought 推理
//! - **Self-Reflection** - 自我反思和改进
//! - **Planning** - 任务规划和分解
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use lumosai_core::reasoning::{ReActAgent, ReActConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let agent = ReActAgent::new(ReActConfig::default())
//!     .with_max_iterations(10)
//!     .with_tools(vec![calculator, search]);
//!
//! let result = agent.execute("What is 2+2? Then verify.").await?;
//! # Ok(())
//! # }
//! ```

pub mod react;
pub mod cot;
pub mod planning;
pub mod reflection;
pub mod trace;

// ✅ Phase 2: Week 8-10 导出核心类型
pub use react::{ReActAgent, ReActConfig, ReActStep, ReActResult};
pub use cot::{ChainOfThought, CoTConfig, CoTStep, ThoughtProcess};
pub use planning::{Planner, PlanningConfig, TaskPlan, SubTask, TaskDependency};
pub use reflection::{ReflectionAgent, ReflectionConfig, ReflectionResult};
pub use trace::{ReasoningTrace, TraceEvent, TraceVisualization};

use thiserror::Error;

/// 推理错误
#[derive(Error, Debug)]
pub enum ReasoningError {
    #[error("推理步骤失败: {0}")]
    StepFailed(String),

    #[error("工具执行错误: {0}")]
    ToolExecutionError(String),

    #[error("超出最大迭代次数")]
    MaxIterationsExceeded,

    #[error("规划失败: {0}")]
    PlanningFailed(String),

    #[error("反思失败: {0}")]
    ReflectionFailed(String),

    #[error("无效配置: {0}")]
    InvalidConfig(String),
}

/// 推理步骤
#[derive(Debug, Clone)]
pub struct ReasoningStep {
    /// 步骤 ID
    pub id: String,
    /// 步骤类型
    pub step_type: StepType,
    /// 步骤内容
    pub content: String,
    /// 时间戳
    pub timestamp: u64,
    /// 子步骤
    pub sub_steps: Vec<ReasoningStep>,
}

/// 步骤类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepType {
    /// 思考
    Thought,
    /// 行动
    Action,
    /// 观察
    Observation,
    /// 反思
    Reflection,
    /// 规划
    Planning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ReasoningError::MaxIterationsExceeded;
        assert_eq!(err.to_string(), "超出最大迭代次数");
    }

    #[test]
    fn test_reasoning_step() {
        let step = ReasoningStep {
            id: "step_1".to_string(),
            step_type: StepType::Thought,
            content: "I need to calculate 2+2".to_string(),
            timestamp: 123456,
            sub_steps: vec![],
        };

        assert_eq!(step.id, "step_1");
        assert_eq!(step.step_type, StepType::Thought);
    }
}
