//! Agent健康检查器 - 负责Agent的健康监控和检查
//!
//! 这个组件负责监控Agent的健康状态，执行健康检查，并提供健康报告。

use crate::agent::modular::core::AgentCore;
use crate::agent::modular::state::AgentState;
use crate::agent::types::{AgentStatus, HealthStatus};
use crate::error::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;

/// Agent健康检查器
///
/// 负责监控Agent的健康状态，包括：
/// - 基础健康检查
/// - 性能指标监控
/// - 错误率监控
/// - 资源使用监控
/// - 健康报告生成
pub struct AgentHealth {
    /// Agent核心引用
    core: AgentCore,
    /// 健康状态
    health_status: Arc<RwLock<HealthStatus>>,
    /// 健康检查历史
    check_history: Arc<RwLock<Vec<HealthCheck>>>,
    /// 性能指标
    metrics: Arc<RwLock<HealthMetrics>>,
    /// 健康检查配置
    config: HealthConfig,
    /// 最后检查时间
    last_check: Arc<RwLock<Option<DateTime<Utc>>>>,
    /// 健康检查任务句柄
    check_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

/// 健康检查配置
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// 健康检查间隔（秒）
    pub check_interval_seconds: u64,
    /// 响应时间阈值（毫秒）
    pub response_time_threshold_ms: u64,
    /// 错误率阈值（0.0-1.0）
    pub error_rate_threshold: f64,
    /// 内存使用阈值（字节）
    pub memory_threshold_bytes: usize,
    /// CPU使用阈值（0.0-1.0）
    pub cpu_threshold: f64,
    /// 最大检查历史记录数
    pub max_history_records: usize,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30,
            response_time_threshold_ms: 5000,
            error_rate_threshold: 0.1,
            memory_threshold_bytes: 1024 * 1024 * 1024, // 1GB
            cpu_threshold: 0.8,
            max_history_records: 100,
        }
    }
}

/// 健康检查记录
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// 检查时间
    pub check_time: DateTime<Utc>,
    /// 健康状态
    pub status: HealthStatus,
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
    /// 错误消息
    pub error_message: Option<String>,
    /// 检查指标
    pub metrics: HealthMetrics,
}

/// 健康指标
#[derive(Debug, Clone, Default)]
pub struct HealthMetrics {
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
    /// 内存使用（字节）
    pub memory_usage_bytes: usize,
    /// CPU使用率（0.0-1.0）
    pub cpu_usage: f64,
    /// 消息队列长度
    pub message_queue_length: usize,
    /// 工具调用队列长度
    pub tool_queue_length: usize,
    /// 错误数量
    pub error_count: usize,
    /// 成功数量
    pub success_count: usize,
    /// 超时数量
    pub timeout_count: usize,
}

/// 健康报告
#[derive(Debug, Clone)]
pub struct HealthReport {
    /// Agent ID
    pub agent_id: String,
    /// Agent名称
    pub agent_name: String,
    /// 健康状态
    pub health_status: HealthStatus,
    /// 检查时间
    pub check_time: DateTime<Utc>,
    /// 运行时间（秒）
    pub uptime_seconds: u64,
    /// 指标
    pub metrics: HealthMetrics,
    /// 状态概要
    pub summary: HealthSummary,
    /// 建议
    pub recommendations: Vec<String>,
}

/// 健康概要
#[derive(Debug, Clone)]
pub struct HealthSummary {
    /// 总体健康评分（0-100）
    pub health_score: u8,
    /// 性能评分（0-100）
    pub performance_score: u8,
    /// 可靠性评分（0-100）
    pub reliability_score: u8,
    /// 资源使用评分（0-100）
    pub resource_score: u8,
    /// 主要问题
    pub issues: Vec<String>,
}

impl AgentHealth {
    /// 创建新的Agent健康检查器
    pub fn new(core: AgentCore) -> Result<Self> {
        Ok(Self {
            core,
            health_status: Arc::new(RwLock::new(HealthStatus::Healthy)),
            check_history: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(HealthMetrics::default())),
            config: HealthConfig::default(),
            last_check: Arc::new(RwLock::new(None)),
            check_handle: Arc::new(RwLock::new(None)),
        })
    }

    /// 执行健康检查
    pub async fn check_health(&self) -> Result<HealthCheck> {
        let start_time = Utc::now();

        // 执行检查
        let check_result = self.perform_health_check().await;

        // 计算响应时间
        let response_time_ms = (Utc::now() - start_time).num_milliseconds() as u64;

        // 记录检查结果
        let health_check = match check_result {
            Ok(metrics) => {
                let status = self.evaluate_health_status(&metrics);
                HealthCheck {
                    check_time: start_time,
                    status: status.clone(),
                    response_time_ms,
                    error_message: None,
                    metrics,
                }
            }
            Err(e) => HealthCheck {
                check_time: start_time,
                status: HealthStatus::Unhealthy,
                response_time_ms,
                error_message: Some(format!("Health check failed: {}", e)),
                metrics: HealthMetrics::default(),
            },
        };

        // 更新状态
        {
            let mut status = self.health_status.write().await;
            *status = health_check.status.clone();
        }

        // 更新指标
        {
            let mut metrics = self.metrics.write().await;
            *metrics = health_check.metrics.clone();
        }

        // 更新检查时间
        {
            let mut last_check = self.last_check.write().await;
            *last_check = Some(start_time);
        }

        // 添加到历史记录
        {
            let mut history = self.check_history.write().await;
            history.push(health_check.clone());

            // 限制历史记录数量
            if history.len() > self.config.max_history_records {
                history.remove(0);
            }
        }

        Ok(health_check)
    }

    /// 获取当前健康状态
    pub async fn health_status(&self) -> HealthStatus {
        self.health_status.read().await.clone()
    }

    /// 获取最新指标
    pub async fn current_metrics(&self) -> HealthMetrics {
        self.metrics.read().await.clone()
    }

    /// 获取健康检查历史
    pub async fn check_history(&self) -> Vec<HealthCheck> {
        self.check_history.read().await.clone()
    }

    /// 获取健康报告
    pub async fn get_health_report(&self, state: &AgentState) -> Result<HealthReport> {
        let health_status = self.health_status().await;
        let metrics = self.current_metrics().await;
        let last_check = self.last_check.read().await;

        let check_time = last_check.unwrap_or_else(Utc::now);
        let uptime_seconds = (check_time - state.created_at).num_seconds() as u64;

        let summary = self.calculate_health_summary(&metrics, state).await;
        let recommendations = self.generate_recommendations(&summary, &metrics).await;

        Ok(HealthReport {
            agent_id: self.core.id().to_string(),
            agent_name: self.core.name().to_string(),
            health_status,
            check_time,
            uptime_seconds,
            metrics,
            summary,
            recommendations,
        })
    }

    /// 启动健康检查监控
    pub async fn start_monitoring(&self) -> Result<()> {
        let health_status = self.health_status.clone();
        let config = self.config.clone();

        let check_handle = tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(config.check_interval_seconds));

            loop {
                interval.tick().await;

                // 执行健康检查
                if let Err(e) = self.check_health().await {
                    log::error!("Health check failed for agent {}: {}", self.core.id(), e);
                }

                // 检查是否需要自动恢复
                let current_status = health_status.read().await;
                if matches!(*current_status, HealthStatus::Unhealthy) {
                    log::warn!("Agent {} is unhealthy, attempting recovery", self.core.id());
                    // 这里可以添加自动恢复逻辑
                }
            }
        });

        let mut handle = self.check_handle.write().await;
        *handle = Some(check_handle);

        log::info!("Health monitoring started for agent {}", self.core.id());

        Ok(())
    }

    /// 停止健康检查监控
    pub async fn stop_monitoring(&self) {
        let mut handle = self.check_handle.write().await;
        if let Some(h) = handle.take() {
            h.abort();
        }

        log::info!("Health monitoring stopped for agent {}", self.core.id());
    }

    /// 设置配置
    pub fn set_config(&mut self, config: HealthConfig) {
        self.config = config;
    }

    /// 获取配置
    pub fn config(&self) -> &HealthConfig {
        &self.config
    }

    /// 执行健康检查逻辑
    async fn perform_health_check(&self) -> Result<HealthMetrics> {
        let mut metrics = HealthMetrics::default();

        // 检查响应时间
        metrics.response_time_ms = self.measure_response_time().await;

        // 检查内存使用
        metrics.memory_usage_bytes = self.measure_memory_usage().await;

        // 检查CPU使用率
        metrics.cpu_usage = self.measure_cpu_usage().await;

        // 检查队列长度（这里需要根据实际实现）
        metrics.message_queue_length = 0;
        metrics.tool_queue_length = 0;

        // 从Agent状态获取统计信息
        // 注意：这里需要传入AgentState作为参数

        Ok(metrics)
    }

    /// 测量响应时间
    async fn measure_response_time(&self) -> u64 {
        let start = std::time::Instant::now();

        // 模拟健康检查请求
        // 在实际实现中，这里可以发送一个测试请求
        tokio::time::sleep(Duration::from_millis(10)).await;

        start.elapsed().as_millis() as u64
    }

    /// 测量内存使用
    async fn measure_memory_usage(&self) -> usize {
        // 获取当前进程的内存使用情况
        if let Ok(memory_usage) = sys_info::mem_info() {
            memory_usage.total as usize
        } else {
            0
        }
    }

    /// 测量CPU使用率
    async fn measure_cpu_usage(&self) -> f64 {
        // 获取CPU使用率
        if let Ok(cpu_usage) = sys_info::cpu_info() {
            cpu_usage.brand_percent
        } else {
            0.0
        }
    }

    /// 评估健康状态
    fn evaluate_health_status(&self, metrics: &HealthMetrics) -> HealthStatus {
        let mut issues = Vec::new();

        // 检查响应时间
        if metrics.response_time_ms > self.config.response_time_threshold_ms {
            issues.push(format!(
                "Response time too high: {}ms",
                metrics.response_time_ms
            ));
        }

        // 检查内存使用
        if metrics.memory_usage_bytes > self.config.memory_threshold_bytes {
            issues.push(format!(
                "Memory usage too high: {} bytes",
                metrics.memory_usage_bytes
            ));
        }

        // 检查CPU使用率
        if metrics.cpu_usage > self.config.cpu_threshold {
            issues.push(format!(
                "CPU usage too high: {:.2}%",
                metrics.cpu_usage * 100.0
            ));
        }

        // 检查错误率
        let total_requests = metrics.success_count + metrics.error_count;
        if total_requests > 0 {
            let error_rate = metrics.error_count as f64 / total_requests as f64;
            if error_rate > self.config.error_rate_threshold {
                issues.push(format!("Error rate too high: {:.2}%", error_rate * 100.0));
            }
        }

        // 根据问题数量决定健康状态
        match issues.len() {
            0 => HealthStatus::Healthy,
            1 => HealthStatus::Degraded,
            _ => HealthStatus::Unhealthy,
        }
    }

    /// 计算健康概要
    async fn calculate_health_summary(
        &self,
        metrics: &HealthMetrics,
        state: &AgentState,
    ) -> HealthSummary {
        let mut issues = Vec::new();

        // 计算性能评分
        let performance_score =
            if metrics.response_time_ms <= self.config.response_time_threshold_ms {
                100
            } else {
                let ratio =
                    self.config.response_time_threshold_ms as f64 / metrics.response_time_ms as f64;
                (ratio * 100.0) as u8
            };

        // 计算可靠性评分
        let total_requests = metrics.success_count + metrics.error_count;
        let reliability_score = if total_requests == 0 {
            100
        } else {
            let success_rate = metrics.success_count as f64 / total_requests as f64;
            (success_rate * 100.0) as u8
        };

        // 计算资源使用评分
        let memory_ratio =
            metrics.memory_usage_bytes as f64 / self.config.memory_threshold_bytes as f64;
        let memory_score = if memory_ratio <= 1.0 {
            (1.0 - memory_ratio) * 100.0
        } else {
            0.0
        };

        let cpu_score = if metrics.cpu_usage <= self.config.cpu_threshold {
            (1.0 - metrics.cpu_usage) * 100.0
        } else {
            0.0
        };

        let resource_score = ((memory_score + cpu_score) / 2.0) as u8;

        // 计算总体健康评分
        let health_score = (performance_score + reliability_score + resource_score) / 3;

        // 收集问题
        if performance_score < 80 {
            issues.push("Performance issues detected".to_string());
        }

        if reliability_score < 80 {
            issues.push("Reliability issues detected".to_string());
        }

        if resource_score < 80 {
            issues.push("Resource usage issues detected".to_string());
        }

        HealthSummary {
            health_score,
            performance_score,
            reliability_score,
            resource_score,
            issues,
        }
    }

    /// 生成建议
    async fn generate_recommendations(
        &self,
        summary: &HealthSummary,
        metrics: &HealthMetrics,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if summary.performance_score < 80 {
            recommendations.push("Consider optimizing response time".to_string());
        }

        if summary.reliability_score < 80 {
            recommendations.push("Review error handling and retry mechanisms".to_string());
        }

        if summary.resource_score < 80 {
            if metrics.memory_usage_bytes > self.config.memory_threshold_bytes {
                recommendations.push(
                    "Consider increasing memory limits or optimizing memory usage".to_string(),
                );
            }

            if metrics.cpu_usage > self.config.cpu_threshold {
                recommendations
                    .push("Consider optimizing CPU usage or adding more resources".to_string());
            }
        }

        if recommendations.is_empty() {
            recommendations.push("System is operating normally".to_string());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_config_default() {
        let config = HealthConfig::default();
        assert_eq!(config.check_interval_seconds, 30);
        assert_eq!(config.response_time_threshold_ms, 5000);
        assert_eq!(config.error_rate_threshold, 0.1);
    }

    #[test]
    fn test_health_metrics_default() {
        let metrics = HealthMetrics::default();
        assert_eq!(metrics.response_time_ms, 0);
        assert_eq!(metrics.memory_usage_bytes, 0);
        assert_eq!(metrics.cpu_usage, 0.0);
    }

    #[tokio::test]
    async fn test_health_checker_creation() {
        let config = crate::agent::AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let health_checker = AgentHealth::new(core).unwrap();

        let status = health_checker.health_status().await;
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = crate::agent::AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let health_checker = AgentHealth::new(core).unwrap();

        let result = health_checker.check_health().await;
        assert!(result.is_ok());

        let check = result.unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
    }
}
