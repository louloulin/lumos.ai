//! # Multi-Agent 编排系统 (Orchestration System)
//!
//! LumosAI 的高级多 Agent 编排和协调框架。
//!
//! ## 核心功能
//!
//! - **MultiAgentCoordinator** - 核心协调器
//! - **Orchestration Patterns** - 多种编排模式
//! - **Agent Communication** - Agent 间通信协议
//! - **Task Routing** - 智能任务路由
//! - **Crew Management** - Agent 团队管理
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use lumosai_core::orchestration::{MultiAgentCoordinator, OrchestrationPattern};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 创建协调器
//! let coordinator = MultiAgentCoordinator::new(OrchestrationPattern::Hierarchical);
//!
//! // 注册 Agent
//! coordinator.register_agent("manager", manager_agent).await?;
//! coordinator.register_agent("worker1", worker_agent).await?;
//!
//! // 执行任务
//! let result = coordinator.execute("Process this data").await?;
//! # Ok(())
//! # }
//! ```

pub mod coordinator;
pub mod patterns;
pub mod communication;
pub mod router;
pub mod crew;
pub mod agent_registry;
pub mod task_queue;

// ✅ Phase 2: Week 5-7 导出核心类型
pub use coordinator::{MultiAgentCoordinator, CoordinatorConfig};
pub use patterns::{OrchestrationPattern, PatternExecutor, HierarchicalPattern, FlatPattern, PipelinePattern, GraphPattern};
pub use communication::{AgentMessage, MessageBus, MessageType, Priority};
pub use router::{TaskRouter, RoutingStrategy, LoadBalancingStrategy};
pub use crew::{CrewManager, CrewConfig, AgentRole};
pub use agent_registry::{AgentRegistry, AgentInfo, AgentCapabilities};
pub use task_queue::{TaskQueue, Task, TaskStatus, TaskPriority};

use thiserror::Error;

/// 编排错误
#[derive(Error, Debug)]
pub enum OrchestrationError {
    #[error("Agent 未注册: {0}")]
    AgentNotRegistered(String),

    #[error("Agent 执行失败: {0}")]
    AgentExecutionFailed(String),

    #[error("任务路由失败: {0}")]
    RoutingFailed(String),

    #[error("通信错误: {0}")]
    CommunicationError(String),

    #[error("超时: {0}")]
    Timeout(String),

    #[error("无效配置: {0}")]
    InvalidConfig(String),
}

/// 编排结果
#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    /// 是否成功
    pub success: bool,
    /// 最终输出
    pub output: String,
    /// 执行的 Agent 列表
    pub agents_involved: Vec<String>,
    /// 总耗时 (毫秒)
    pub duration_ms: u64,
    /// 中间结果
    pub intermediate_results: Vec<IntermediateResult>,
    /// 错误信息 (如果失败)
    pub error: Option<String>,
}

/// 中间结果
#[derive(Debug, Clone)]
pub struct IntermediateResult {
    /// Agent 名称
    pub agent_name: String,
    /// 结果内容
    pub content: String,
    /// 时间戳
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = OrchestrationError::AgentNotRegistered("test_agent".to_string());
        assert_eq!(err.to_string(), "Agent 未注册: test_agent");
    }

    #[test]
    fn test_orchestration_result() {
        let result = OrchestrationResult {
            success: true,
            output: "Final result".to_string(),
            agents_involved: vec!["agent1".to_string(), "agent2".to_string()],
            duration_ms: 1000,
            intermediate_results: vec![],
            error: None,
        };

        assert!(result.success);
        assert_eq!(result.agents_involved.len(), 2);
    }
}
