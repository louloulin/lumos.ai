//! Agent状态管理模块
//!
//! 提供完整的Agent生命周期管理、状态持久化和健康检查功能。

use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Agent状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    /// 初始化中
    Initializing,
    /// 就绪状态
    Ready,
    /// 运行中
    Running,
    /// 暂停状态
    Paused,
    /// 错误状态
    Error(String),
    /// 停止状态
    Stopped,
    /// 恢复中
    Recovering,
    /// 关闭中
    ShuttingDown,
}

/// Agent状态上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateContext {
    /// Agent ID
    pub agent_id: String,
    /// 当前状态
    pub state: AgentState,
    /// 状态变更时间
    pub timestamp: DateTime<Utc>,
    /// 状态持续时间（秒）
    pub duration_seconds: u64,
    /// 状态相关元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 健康状态
    pub health_status: HealthStatus,
    /// 性能指标
    pub performance_metrics: PerformanceMetrics,
}

/// 健康状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 警告
    Warning(String),
    /// 不健康
    Unhealthy(String),
    /// 未知
    Unknown,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 响应时间（毫秒）
    pub average_response_time_ms: f64,
    /// 请求总数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 错误请求数
    pub failed_requests: u64,
    /// 最后活跃时间
    pub last_active: DateTime<Utc>,
    /// 内存使用量（字节）
    pub memory_usage_bytes: u64,
    /// CPU使用率（百分比）
    pub cpu_usage_percent: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            average_response_time_ms: 0.0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            last_active: Utc::now(),
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
        }
    }
}

impl PerformanceMetrics {
    /// 更新请求指标
    pub fn update_request(&mut self, response_time_ms: f64, success: bool) {
        self.total_requests += 1;
        self.last_active = Utc::now();

        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        // 更新平均响应时间
        if self.total_requests == 1 {
            self.average_response_time_ms = response_time_ms;
        } else {
            self.average_response_time_ms = (self.average_response_time_ms
                * (self.total_requests - 1) as f64
                + response_time_ms)
                / self.total_requests as f64;
        }
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64 * 100.0
        }
    }
}

/// Agent状态持久化接口
#[async_trait]
pub trait AgentStatePersistence: Send + Sync {
    /// 保存Agent状态
    async fn save_state(&self, agent_id: &str, state: &AgentStateContext) -> Result<()>;

    /// 加载Agent状态
    async fn load_state(&self, agent_id: &str) -> Result<Option<AgentStateContext>>;

    /// 删除Agent状态
    async fn delete_state(&self, agent_id: &str) -> Result<()>;

    /// 列出所有Agent状态
    async fn list_states(&self) -> Result<Vec<String>>;
}

/// 内存中的状态持久化实现
pub struct InMemoryStatePersistence {
    states: Arc<RwLock<HashMap<String, AgentStateContext>>>,
}

impl InMemoryStatePersistence {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl AgentStatePersistence for InMemoryStatePersistence {
    async fn save_state(&self, agent_id: &str, state: &AgentStateContext) -> Result<()> {
        let mut states = self.states.write().await;
        states.insert(agent_id.to_string(), state.clone());
        Ok(())
    }

    async fn load_state(&self, agent_id: &str) -> Result<Option<AgentStateContext>> {
        let states = self.states.read().await;
        Ok(states.get(agent_id).cloned())
    }

    async fn delete_state(&self, agent_id: &str) -> Result<()> {
        let mut states = self.states.write().await;
        states.remove(agent_id);
        Ok(())
    }

    async fn list_states(&self) -> Result<Vec<String>> {
        let states = self.states.read().await;
        Ok(states.keys().cloned().collect())
    }
}

/// Agent状态管理器
pub struct AgentStateManager {
    agent_id: String,
    current_state: Arc<RwLock<AgentStateContext>>,
    persistence: Arc<dyn AgentStatePersistence>,
    health_check_interval: Duration,
    metrics_collection_interval: Duration,
}

impl AgentStateManager {
    /// 创建新的状态管理器
    pub fn new(
        agent_id: String,
        persistence: Arc<dyn AgentStatePersistence>,
        health_check_interval: Duration,
        metrics_collection_interval: Duration,
    ) -> Self {
        let initial_state = AgentStateContext {
            agent_id: agent_id.clone(),
            state: AgentState::Initializing,
            timestamp: Utc::now(),
            duration_seconds: 0,
            metadata: HashMap::new(),
            health_status: HealthStatus::Unknown,
            performance_metrics: PerformanceMetrics::default(),
        };

        Self {
            agent_id,
            current_state: Arc::new(RwLock::new(initial_state)),
            persistence,
            health_check_interval,
            metrics_collection_interval,
        }
    }

    /// 从持久化存储加载状态
    pub async fn load_from_persistence(&self) -> Result<()> {
        if let Some(loaded_state) = self.persistence.load_state(&self.agent_id).await? {
            let mut current_state = self.current_state.write().await;
            *current_state = loaded_state;
        }
        Ok(())
    }

    /// 获取当前状态
    pub async fn get_current_state(&self) -> AgentStateContext {
        let state = self.current_state.read().await;
        state.clone()
    }

    /// 转换状态
    pub async fn transition_to(
        &self,
        new_state: AgentState,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let mut current_state = self.current_state.write().await;
        let now = Utc::now();

        let duration = now
            .signed_duration_since(current_state.timestamp)
            .num_seconds();

        // 创建新的状态上下文
        let new_context = AgentStateContext {
            agent_id: self.agent_id.clone(),
            state: new_state.clone(),
            timestamp: now,
            duration_seconds: duration.max(0) as u64,
            metadata,
            health_status: current_state.health_status.clone(),
            performance_metrics: current_state.performance_metrics.clone(),
        };

        // 更新当前状态
        *current_state = new_context.clone();

        // 保存到持久化存储
        self.persistence
            .save_state(&self.agent_id, &new_context)
            .await?;

        Ok(())
    }

    /// 更新健康状态
    pub async fn update_health_status(&self, health_status: HealthStatus) -> Result<()> {
        let mut current_state = self.current_state.write().await;
        current_state.health_status = health_status;

        // 保存更新后的状态
        self.persistence
            .save_state(&self.agent_id, &*current_state)
            .await?;

        Ok(())
    }

    /// 更新性能指标
    pub async fn update_performance_metrics(
        &self,
        update_fn: impl FnOnce(&mut PerformanceMetrics),
    ) -> Result<()> {
        let mut current_state = self.current_state.write().await;
        update_fn(&mut current_state.performance_metrics);

        // 保存更新后的状态
        self.persistence
            .save_state(&self.agent_id, &*current_state)
            .await?;

        Ok(())
    }

    /// 检查Agent是否健康
    pub async fn is_healthy(&self) -> bool {
        let state = self.current_state.read().await;
        matches!(state.health_status, HealthStatus::Healthy)
    }

    /// 检查Agent是否处于运行状态
    pub async fn is_running(&self) -> bool {
        let state = self.current_state.read().await;
        matches!(state.state, AgentState::Running)
    }

    /// 获取状态持续时间
    pub async fn get_state_duration(&self) -> Duration {
        let state = self.current_state.read().await;
        Duration::from_secs(state.duration_seconds)
    }

    /// 执行健康检查
    pub async fn perform_health_check(&self) -> Result<HealthStatus> {
        let current_state = self.current_state.read().await;

        // 检查基础健康指标
        let metrics = &current_state.performance_metrics;

        // 检查错误率
        let error_rate = if metrics.total_requests > 0 {
            metrics.failed_requests as f64 / metrics.total_requests as f64
        } else {
            0.0
        };

        // 检查响应时间
        let response_time_ok = metrics.average_response_time_ms < 5000.0; // 5秒阈值

        // 检查最后活跃时间
        let now = Utc::now();
        let last_active_diff = now.signed_duration_since(metrics.last_active).num_seconds();
        let recent_active = last_active_diff < 300; // 5分钟内活跃

        // 确定健康状态
        let health_status = if error_rate > 0.5 {
            // 错误率超过50%
            HealthStatus::Unhealthy(format!("High error rate: {:.1}%", error_rate * 100.0))
        } else if !response_time_ok {
            HealthStatus::Warning(format!(
                "High response time: {:.1}ms",
                metrics.average_response_time_ms
            ))
        } else if !recent_active {
            HealthStatus::Warning("Agent inactive for too long".to_string())
        } else {
            HealthStatus::Healthy
        };

        // 更新健康状态
        self.update_health_status(health_status.clone()).await?;

        Ok(health_status)
    }

    /// 启动后台监控任务
    pub async fn start_monitoring(&self) -> Result<tokio::task::JoinHandle<()>> {
        let current_state = Arc::clone(&self.current_state);
        let health_check_interval = self.health_check_interval;
        let agent_id = self.agent_id.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(health_check_interval);

            loop {
                interval.tick().await;

                // 检查Agent状态
                let state = current_state.read().await;

                // 如果Agent处于错误状态，尝试恢复
                if matches!(state.state, AgentState::Error(_)) {
                    tracing::warn!("Agent {} is in error state, attempting recovery", agent_id);
                    // 这里可以实现恢复逻辑
                }

                // 执行健康检查
                // TODO: 实现健康检查逻辑，不依赖 self
                tracing::debug!("Health check for agent {}", agent_id);
            }
        });

        Ok(handle)
    }

    /// 重置Agent状态
    pub async fn reset(&self) -> Result<()> {
        self.transition_to(AgentState::Ready, HashMap::new())
            .await?;
        Ok(())
    }

    /// 停止Agent
    pub async fn shutdown(&self) -> Result<()> {
        self.transition_to(AgentState::ShuttingDown, HashMap::new())
            .await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.transition_to(AgentState::Stopped, HashMap::new())
            .await?;
        Ok(())
    }
}

/// Agent状态监听器接口
#[async_trait]
pub trait AgentStateListener: Send + Sync {
    /// 当状态变更时被调用
    async fn on_state_changed(
        &self,
        old_state: &AgentStateContext,
        new_state: &AgentStateContext,
    ) -> Result<()>;

    /// 当健康状态变更时被调用
    async fn on_health_changed(
        &self,
        agent_id: &str,
        old_health: &HealthStatus,
        new_health: &HealthStatus,
    ) -> Result<()>;
}

/// 状态变更监听器实现
pub struct StateChangeLogger;

#[async_trait]
impl AgentStateListener for StateChangeLogger {
    async fn on_state_changed(
        &self,
        old_state: &AgentStateContext,
        new_state: &AgentStateContext,
    ) -> Result<()> {
        tracing::info!(
            "Agent {} state changed: {:?} -> {:?} at {}",
            new_state.agent_id,
            old_state.state,
            new_state.state,
            new_state.timestamp
        );
        Ok(())
    }

    async fn on_health_changed(
        &self,
        agent_id: &str,
        old_health: &HealthStatus,
        new_health: &HealthStatus,
    ) -> Result<()> {
        if old_health != new_health {
            match new_health {
                HealthStatus::Unhealthy(msg) => {
                    tracing::error!("Agent {} became unhealthy: {}", agent_id, msg);
                }
                HealthStatus::Warning(msg) => {
                    tracing::warn!("Agent {} health warning: {}", agent_id, msg);
                }
                HealthStatus::Healthy => {
                    tracing::info!("Agent {} health restored", agent_id);
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_state_transitions() {
        let persistence = Arc::new(InMemoryStatePersistence::new());
        let manager = AgentStateManager::new(
            "test-agent".to_string(),
            persistence,
            Duration::from_secs(30),
            Duration::from_secs(60),
        );

        // 初始状态应该是Initializing
        let state = manager.get_current_state().await;
        assert_eq!(state.state, AgentState::Initializing);
        assert_eq!(state.agent_id, "test-agent");

        // 转换到Ready状态
        manager
            .transition_to(AgentState::Ready, HashMap::new())
            .await
            .unwrap();
        let state = manager.get_current_state().await;
        assert_eq!(state.state, AgentState::Ready);

        // 测试是否运行
        assert!(!manager.is_running().await);

        // 转换到Running状态
        manager
            .transition_to(AgentState::Running, HashMap::new())
            .await
            .unwrap();
        assert!(manager.is_running().await);
        assert!(manager.is_healthy().await);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let persistence = Arc::new(InMemoryStatePersistence::new());
        let manager = AgentStateManager::new(
            "test-agent".to_string(),
            persistence,
            Duration::from_secs(30),
            Duration::from_secs(60),
        );

        // 更新性能指标
        manager
            .update_performance_metrics(|metrics| {
                metrics.update_request(100.0, true);
                metrics.update_request(200.0, false);
                metrics.update_request(150.0, true);
            })
            .await
            .unwrap();

        let state = manager.get_current_state().await;
        let metrics = &state.performance_metrics;

        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.success_rate(), 66.66666666666667);
        assert_eq!(metrics.average_response_time_ms, 150.0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let persistence = Arc::new(InMemoryStatePersistence::new());
        let manager = AgentStateManager::new(
            "test-agent".to_string(),
            persistence,
            Duration::from_secs(30),
            Duration::from_secs(60),
        );

        // 添加一些性能数据
        manager
            .update_performance_metrics(|metrics| {
                metrics.update_request(6000.0, false); // 高响应时间
                metrics.update_request(100.0, true);
                metrics.update_request(200.0, false);
            })
            .await
            .unwrap();

        // 执行健康检查
        let health_status = manager.perform_health_check().await.unwrap();

        // 由于响应时间较高，应该是Warning状态
        assert!(matches!(health_status, HealthStatus::Warning(_)));

        let state = manager.get_current_state().await;
        assert_eq!(state.health_status, health_status);
    }

    #[tokio::test]
    async fn test_state_persistence() {
        let persistence = Arc::new(InMemoryStatePersistence::new());
        let manager1 = AgentStateManager::new(
            "persist-agent".to_string(),
            persistence.clone(),
            Duration::from_secs(30),
            Duration::from_secs(60),
        );

        // 设置状态
        manager1
            .transition_to(AgentState::Running, HashMap::new())
            .await
            .unwrap();

        // 创建新的管理器并加载状态
        let manager2 = AgentStateManager::new(
            "persist-agent".to_string(),
            persistence,
            Duration::from_secs(30),
            Duration::from_secs(60),
        );

        manager2.load_from_persistence().await.unwrap();
        let state = manager2.get_current_state().await;
        assert_eq!(state.state, AgentState::Running);
    }
}
