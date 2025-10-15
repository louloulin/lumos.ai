//! Agent生命周期管理器 - 负责Agent的生命周期管理
//!
//! 这个组件负责管理Agent的创建、启动、停止、销毁等生命周期操作。

use crate::agent::modular::core::AgentCore;
use crate::agent::types::{AgentStatus, LifecycleState};
use crate::error::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{Notify, RwLock};

/// Agent生命周期管理器
///
/// 负责管理Agent的完整生命周期，包括：
/// - 创建和初始化
/// - 启动和停止
/// - 暂停和恢复
/// - 销毁和清理
/// - 健康检查
pub struct AgentLifecycle {
    /// Agent核心引用
    core: Arc<AgentCore>,
    /// 当前生命周期状态
    state: Arc<RwLock<LifecycleState>>,
    /// 启动时间
    start_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    /// 停止时间
    stop_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    /// 健康检查间隔
    health_check_interval: std::time::Duration,
    /// 健康检查通知器
    health_check_notify: Arc<Notify>,
    /// 健康检查任务句柄
    health_check_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// 关闭信号
    shutdown_signal: Arc<Notify>,
    /// 配置
    config: LifecycleConfig,
}

/// 生命周期配置
#[derive(Debug, Clone)]
pub struct LifecycleConfig {
    /// 健康检查间隔（秒）
    pub health_check_interval_seconds: u64,
    /// 启动超时时间（秒）
    pub startup_timeout_seconds: u64,
    /// 停止超时时间（秒）
    pub shutdown_timeout_seconds: u64,
    /// 是否启用自动重启
    pub auto_restart: bool,
    /// 最大重启次数
    pub max_restart_attempts: usize,
    /// 重启延迟（秒）
    pub restart_delay_seconds: u64,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            health_check_interval_seconds: 30,
            startup_timeout_seconds: 60,
            shutdown_timeout_seconds: 30,
            auto_restart: true,
            max_restart_attempts: 3,
            restart_delay_seconds: 5,
        }
    }
}

impl AgentLifecycle {
    /// 创建新的Agent生命周期管理器
    pub fn new(core: AgentCore) -> Result<Self> {
        Ok(Self {
            core: Arc::new(core),
            state: Arc::new(RwLock::new(LifecycleState::Created)),
            start_time: Arc::new(RwLock::new(None)),
            stop_time: Arc::new(RwLock::new(None)),
            health_check_interval: std::time::Duration::from_secs(
                LifecycleConfig::default().health_check_interval_seconds,
            ),
            health_check_notify: Arc::new(Notify::new()),
            health_check_handle: Arc::new(RwLock::new(None)),
            shutdown_signal: Arc::new(Notify::new()),
            config: LifecycleConfig::default(),
        })
    }

    /// 启动Agent
    pub async fn start(&self) -> Result<()> {
        let mut state = self.state.write().await;

        match *state {
            LifecycleState::Created | LifecycleState::Stopped => {
                // 开始启动流程
                *state = LifecycleState::Starting;
                drop(state);

                // 执行启动逻辑
                if let Err(e) = self.do_start().await {
                    let mut state = self.state.write().await;
                    *state = LifecycleState::Failed(format!("Startup failed: {}", e));
                    return Err(e);
                }

                // 启动健康检查
                self.start_health_check().await?;

                Ok(())
            }
            LifecycleState::Starting => Err(crate::error::Error::Lifecycle {
                message: "Agent is already starting".to_string(),
            }),
            LifecycleState::Running => Err(crate::error::Error::Lifecycle {
                message: "Agent is already running".to_string(),
            }),
            LifecycleState::Stopping => Err(crate::error::Error::Lifecycle {
                message: "Agent is stopping".to_string(),
            }),
            LifecycleState::Failed(_) => Err(crate::error::Error::Lifecycle {
                message: "Agent is in failed state".to_string(),
            }),
        }
    }

    /// 停止Agent
    pub async fn stop(&self) -> Result<()> {
        let mut state = self.state.write().await;

        match *state {
            LifecycleState::Running => {
                // 开始停止流程
                *state = LifecycleState::Stopping;
                drop(state);

                // 执行停止逻辑
                if let Err(e) = self.do_stop().await {
                    let mut state = self.state.write().await;
                    *state = LifecycleState::Failed(format!("Stop failed: {}", e));
                    return Err(e);
                }

                // 停止健康检查
                self.stop_health_check().await;

                Ok(())
            }
            LifecycleState::Created => Err(crate::error::Error::Lifecycle {
                message: "Agent is not started".to_string(),
            }),
            LifecycleState::Starting => Err(crate::error::Error::Lifecycle {
                message: "Agent is starting".to_string(),
            }),
            LifecycleState::Stopped => Err(crate::error::Error::Lifecycle {
                message: "Agent is already stopped".to_string(),
            }),
            LifecycleState::Stopping => Err(crate::error::Error::Lifecycle {
                message: "Agent is already stopping".to_string(),
            }),
            LifecycleState::Failed(_) => Err(crate::error::Error::Lifecycle {
                message: "Agent is in failed state".to_string(),
            }),
        }
    }

    /// 重启Agent
    pub async fn restart(&self) -> Result<()> {
        self.stop().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(
            self.config.restart_delay_seconds,
        ))
        .await;
        self.start().await
    }

    /// 暂停Agent
    pub async fn pause(&self) -> Result<()> {
        let mut state = self.state.write().await;

        match *state {
            LifecycleState::Running => {
                *state = LifecycleState::Paused;
                Ok(())
            }
            LifecycleState::Paused => Err(crate::error::Error::Lifecycle {
                message: "Agent is already paused".to_string(),
            }),
            _ => Err(crate::error::Error::Lifecycle {
                message: "Agent must be running to pause".to_string(),
            }),
        }
    }

    /// 恢复Agent
    pub async fn resume(&self) -> Result<()> {
        let mut state = self.state.write().await;

        match *state {
            LifecycleState::Paused => {
                *state = LifecycleState::Running;
                Ok(())
            }
            LifecycleState::Running => Err(crate::error::Error::Lifecycle {
                message: "Agent is already running".to_string(),
            }),
            _ => Err(crate::error::Error::Lifecycle {
                message: "Agent must be paused to resume".to_string(),
            }),
        }
    }

    /// 获取当前状态
    pub async fn state(&self) -> LifecycleState {
        self.state.read().await.clone()
    }

    /// 检查是否正在运行
    pub async fn is_running(&self) -> bool {
        matches!(*self.state.read().await, LifecycleState::Running)
    }

    /// 检查是否已停止
    pub async fn is_stopped(&self) -> bool {
        matches!(*self.state.read().await, LifecycleState::Stopped)
    }

    /// 获取运行时间
    pub async fn uptime(&self) -> std::time::Duration {
        let start_time = self.start_time.read().await;
        if let Some(start) = *start_time {
            let now = Utc::now();
            now.signed_duration_since(start)
                .to_std()
                .unwrap_or_default()
        } else {
            std::time::Duration::from_secs(0)
        }
    }

    /// 获取启动时间
    pub async fn start_time(&self) -> Option<DateTime<Utc>> {
        *self.start_time.read().await
    }

    /// 获取停止时间
    pub async fn stop_time(&self) -> Option<DateTime<Utc>> {
        *self.stop_time.read().await
    }

    /// 设置配置
    pub fn set_config(&mut self, config: LifecycleConfig) {
        self.config = config;
        self.health_check_interval =
            std::time::Duration::from_secs(config.health_check_interval_seconds);
    }

    /// 获取配置
    pub fn config(&self) -> &LifecycleConfig {
        &self.config
    }

    /// 执行启动逻辑
    async fn do_start(&self) -> Result<()> {
        let now = Utc::now();

        // 设置启动时间
        let mut start_time = self.start_time.write().await;
        *start_time = Some(now);
        drop(start_time);

        // 清空停止时间
        let mut stop_time = self.stop_time.write().await;
        *stop_time = None;
        drop(stop_time);

        // 更新状态
        let mut state = self.state.write().await;
        *state = LifecycleState::Running;

        log::info!("Agent {} started at {}", self.core.id(), now);

        Ok(())
    }

    /// 执行停止逻辑
    async fn do_stop(&self) -> Result<()> {
        let now = Utc::now();

        // 设置停止时间
        let mut stop_time = self.stop_time.write().await;
        *stop_time = Some(now);
        drop(stop_time);

        // 更新状态
        let mut state = self.state.write().await;
        *state = LifecycleState::Stopped;

        log::info!("Agent {} stopped at {}", self.core.id(), now);

        Ok(())
    }

    /// 启动健康检查
    async fn start_health_check(&self) -> Result<()> {
        let state = self.state.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let health_check_notify = self.health_check_notify.clone();
        let health_check_interval = self.health_check_interval;

        let health_check_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(health_check_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // 执行健康检查
                        health_check_notify.notify_one();
                    }
                    _ = shutdown_signal.notified() => {
                        break;
                    }
                }
            }
        });

        let mut handle = self.health_check_handle.write().await;
        *handle = Some(health_check_handle);

        Ok(())
    }

    /// 停止健康检查
    async fn stop_health_check(&self) {
        // 发送关闭信号
        self.shutdown_signal.notify_one();

        // 等待健康检查任务结束
        let mut handle = self.health_check_handle.write().await;
        if let Some(h) = handle.take() {
            let _ = h.await;
        }
    }

    /// 启动健康检查监控
    pub async fn start_monitoring(&self) -> Result<()> {
        self.start_health_check().await
    }

    /// 停止健康检查监控
    pub async fn stop_monitoring(&self) {
        self.stop_health_check().await;
    }

    /// 获取生命周期信息
    pub async fn get_lifecycle_info(&self) -> LifecycleInfo {
        let state = self.state.read().await;
        let start_time = self.start_time.read().await;
        let stop_time = self.stop_time.read().await;

        LifecycleInfo {
            agent_id: self.core.id().to_string(),
            agent_name: self.core.name().to_string(),
            state: state.clone(),
            start_time: *start_time,
            stop_time: *stop_time,
            uptime: if start_time.is_some() {
                self.uptime().await.as_secs()
            } else {
                0
            },
            config: self.config.clone(),
        }
    }
}

/// 生命周期信息
#[derive(Debug, Clone)]
pub struct LifecycleInfo {
    /// Agent ID
    pub agent_id: String,
    /// Agent名称
    pub agent_name: String,
    /// 当前状态
    pub state: LifecycleState,
    /// 启动时间
    pub start_time: Option<DateTime<Utc>>,
    /// 停止时间
    pub stop_time: Option<DateTime<Utc>>,
    /// 运行时间（秒）
    pub uptime: u64,
    /// 配置
    pub config: LifecycleConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lifecycle_creation() {
        let config = crate::agent::AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let lifecycle = AgentLifecycle::new(core).unwrap();

        assert_eq!(lifecycle.state().await, LifecycleState::Created);
        assert!(!lifecycle.is_running().await);
    }

    #[tokio::test]
    async fn test_lifecycle_start_stop() {
        let config = crate::agent::AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let lifecycle = AgentLifecycle::new(core).unwrap();

        // 测试启动
        assert!(lifecycle.start().await.is_ok());
        assert!(lifecycle.is_running().await);
        assert!(lifecycle.start_time().await.is_some());

        // 测试停止
        assert!(lifecycle.stop().await.is_ok());
        assert!(lifecycle.is_stopped().await);
        assert!(lifecycle.stop_time().await.is_some());
    }

    #[tokio::test]
    async fn test_lifecycle_pause_resume() {
        let config = crate::agent::AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let lifecycle = AgentLifecycle::new(core).unwrap();

        // 启动
        assert!(lifecycle.start().await.is_ok());

        // 暂停
        assert!(lifecycle.pause().await.is_ok());
        assert_eq!(lifecycle.state().await, LifecycleState::Paused);

        // 恢复
        assert!(lifecycle.resume().await.is_ok());
        assert!(lifecycle.is_running().await);
    }

    #[tokio::test]
    async fn test_lifecycle_restart() {
        let config = crate::agent::AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let lifecycle = AgentLifecycle::new(core).unwrap();

        // 启动
        assert!(lifecycle.start().await.is_ok());
        let start_time1 = lifecycle.start_time().await;

        // 重启
        assert!(lifecycle.restart().await.is_ok());
        let start_time2 = lifecycle.start_time().await;

        // 验证重启后时间不同
        assert!(start_time1.is_some());
        assert!(start_time2.is_some());
        assert_ne!(start_time1, start_time2);
    }

    #[tokio::test]
    async fn test_lifecycle_uptime() {
        let config = crate::agent::AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let lifecycle = AgentLifecycle::new(core).unwrap();

        // 未启动时运行时间为0
        assert_eq!(lifecycle.uptime().await.as_secs(), 0);

        // 启动后运行时间应该大于0
        assert!(lifecycle.start().await.is_ok());
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        assert!(lifecycle.uptime().await.as_secs() > 0);
    }
}
