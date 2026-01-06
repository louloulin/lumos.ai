//! Agent核心模块 - 重构后的核心组件
//!
//! 这个模块包含了重构后的Agent核心组件，将原来的BasicAgent拆分为多个专门的组件。

pub mod capability;
pub mod core;
pub mod executor;
pub mod health;
pub mod lifecycle;
pub mod state;

// 重新导出主要类型
pub use capability::AgentCapability;
pub use core::AgentCore;
pub use executor::AgentExecutor;
pub use health::AgentHealth;
pub use lifecycle::AgentLifecycle;
pub use state::AgentState;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent管理器 - 统一管理Agent的生命周期和状态
pub struct AgentManager {
    /// Agent核心实例
    core: AgentCore,
    /// Agent执行器
    executor: AgentExecutor,
    /// Agent生命周期管理器
    lifecycle: AgentLifecycle,
    /// Agent状态管理器
    state: Arc<RwLock<AgentState>>,
    /// Agent能力管理器
    capability: AgentCapability,
    /// Agent健康检查器
    health: AgentHealth,
}

impl AgentManager {
    /// 创建新的Agent管理器
    pub async fn new(config: crate::agent::AgentConfig) -> crate::error::Result<Self> {
        let core = AgentCore::new(config.clone())?;
        let state = Arc::new(RwLock::new(AgentState::new()));
        let executor = AgentExecutor::new(core.clone(), state.clone()).await?;
        let lifecycle = AgentLifecycle::new(core.clone())?;
        let capability = AgentCapability::new(core.clone())?;
        let health = AgentHealth::new(core.clone())?;

        Ok(Self {
            core,
            executor,
            lifecycle,
            state,
            capability,
            health,
        })
    }

    /// 启动Agent
    pub async fn start(&mut self) -> crate::error::Result<()> {
        self.lifecycle.start().await?;
        self.health.start_monitoring().await?;
        Ok(())
    }

    /// 停止Agent
    pub async fn stop(&mut self) -> crate::error::Result<()> {
        self.lifecycle.stop().await?;
        // Health monitoring will be stopped automatically when the agent stops
        Ok(())
    }

    /// 获取Agent核心引用
    pub fn core(&self) -> &AgentCore {
        &self.core
    }

    /// 获取Agent执行器引用
    pub fn executor(&self) -> &AgentExecutor {
        &self.executor
    }

    /// 获取Agent生命周期管理器引用
    pub fn lifecycle(&self) -> &AgentLifecycle {
        &self.lifecycle
    }

    /// 获取Agent状态
    pub async fn state(&self) -> crate::agent::types::AgentState {
        self.state.read().await.clone()
    }

    /// 获取Agent能力管理器引用
    pub fn capability(&self) -> &AgentCapability {
        &self.capability
    }

    /// 获取Agent健康检查器引用
    pub fn health(&self) -> &AgentHealth {
        &self.health
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentConfig;

    #[tokio::test]
    async fn test_agent_manager_creation() {
        let config = AgentConfig::default();
        let result = AgentManager::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_agent_manager_lifecycle() {
        let config = AgentConfig::default();
        let mut manager = AgentManager::new(config).await.unwrap();

        // 测试启动
        assert!(manager.start().await.is_ok());

        // 测试停止
        assert!(manager.stop().await.is_ok());
    }
}
