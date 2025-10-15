//! 企业级告警系统
//!
//! 提供智能告警、异常检测、自动化诊断等高级监控功能

use crate::telemetry::metrics::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

/// 告警级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AlertSeverity {
    /// 信息级别
    Info,
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 严重错误
    Critical,
}

/// 告警状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    /// 活跃状态
    Active,
    /// 已确认
    Acknowledged,
    /// 已解决
    Resolved,
    /// 已抑制
    Suppressed,
}

/// 告警条件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// 响应时间阈值
    ResponseTime {
        threshold_ms: u64,
        window_minutes: u32,
        percentile: f64,
    },
    /// 错误率阈值
    ErrorRate {
        threshold_percent: f64,
        window_minutes: u32,
        min_requests: u32,
    },
    /// 内存使用阈值
    MemoryUsage {
        threshold_mb: f64,
        window_minutes: u32,
    },
    /// CPU使用阈值
    CpuUsage {
        threshold_percent: f64,
        window_minutes: u32,
    },
    /// 自定义指标阈值
    CustomMetric {
        metric_name: String,
        threshold: f64,
        comparison: ComparisonOperator,
        window_minutes: u32,
    },
    /// 异常检测
    AnomalyDetection {
        metric_name: String,
        sensitivity: f64,
        window_minutes: u32,
    },
}

/// 比较操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// 规则ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 规则描述
    pub description: String,
    /// 告警条件
    pub condition: AlertCondition,
    /// 告警级别
    pub severity: AlertSeverity,
    /// 是否启用
    pub enabled: bool,
    /// 冷却时间
    pub cooldown_duration: Duration,
    /// 通知渠道
    pub channels: Vec<String>,
    /// 标签
    pub labels: HashMap<String, String>,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
}

/// 告警事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    /// 告警ID
    pub id: String,
    /// 规则ID
    pub rule_id: String,
    /// 告警标题
    pub title: String,
    /// 告警描述
    pub description: String,
    /// 告警级别
    pub severity: AlertSeverity,
    /// 告警状态
    pub status: AlertStatus,
    /// 触发时间
    pub triggered_at: u64,
    /// 确认时间
    pub acknowledged_at: Option<u64>,
    /// 解决时间
    pub resolved_at: Option<u64>,
    /// 相关指标
    pub metrics: HashMap<String, f64>,
    /// 标签
    pub labels: HashMap<String, String>,
    /// 诊断信息
    pub diagnosis: Option<DiagnosisInfo>,
}

/// 诊断信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisInfo {
    /// 可能原因
    pub possible_causes: Vec<String>,
    /// 建议操作
    pub recommended_actions: Vec<String>,
    /// 相关日志
    pub related_logs: Vec<String>,
    /// 影响评估
    pub impact_assessment: String,
    /// 自动修复建议
    pub auto_fix_suggestions: Vec<AutoFixSuggestion>,
}

/// 自动修复建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFixSuggestion {
    /// 修复类型
    pub fix_type: String,
    /// 修复描述
    pub description: String,
    /// 修复命令
    pub command: Option<String>,
    /// 风险级别
    pub risk_level: String,
    /// 预期效果
    pub expected_outcome: String,
}

/// 告警通道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertChannel {
    /// 通道ID
    pub id: String,
    /// 通道名称
    pub name: String,
    /// 通道类型
    pub channel_type: AlertChannelType,
    /// 配置信息
    pub config: serde_json::Value,
    /// 是否启用
    pub enabled: bool,
}

/// 告警通道类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannelType {
    /// 邮件通知
    Email,
    /// Slack通知
    Slack,
    /// 钉钉通知
    DingTalk,
    /// 企业微信通知
    WeChat,
    /// Webhook通知
    Webhook,
    /// SMS短信通知
    SMS,
}

/// 告警管理器trait
#[async_trait]
pub trait AlertManager: Send + Sync {
    /// 添加告警规则
    async fn add_rule(
        &self,
        rule: AlertRule,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 删除告警规则
    async fn remove_rule(
        &self,
        rule_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 更新告警规则
    async fn update_rule(
        &self,
        rule: AlertRule,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 获取所有告警规则
    async fn get_rules(&self) -> Result<Vec<AlertRule>, Box<dyn std::error::Error + Send + Sync>>;

    /// 评估告警条件
    async fn evaluate_conditions(
        &self,
        metrics: &MetricsSummary,
    ) -> Result<Vec<AlertEvent>, Box<dyn std::error::Error + Send + Sync>>;

    /// 发送告警通知
    async fn send_alert(
        &self,
        alert: &AlertEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 确认告警
    async fn acknowledge_alert(
        &self,
        alert_id: &str,
        user: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 解决告警
    async fn resolve_alert(
        &self,
        alert_id: &str,
        user: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 获取活跃告警
    async fn get_active_alerts(
        &self,
    ) -> Result<Vec<AlertEvent>, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取告警历史
    async fn get_alert_history(
        &self,
        from: Option<u64>,
        to: Option<u64>,
    ) -> Result<Vec<AlertEvent>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 内存告警管理器实现
#[derive(Debug)]
pub struct InMemoryAlertManager {
    /// 告警规则
    rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    /// 活跃告警
    active_alerts: Arc<RwLock<HashMap<String, AlertEvent>>>,
    /// 告警历史
    alert_history: Arc<RwLock<VecDeque<AlertEvent>>>,
    /// 告警通道
    channels: Arc<RwLock<HashMap<String, AlertChannel>>>,
    /// 最大历史记录数
    max_history_size: usize,
}

impl InMemoryAlertManager {
    /// 创建新的内存告警管理器
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            max_history_size: 10000,
        }
    }

    /// 添加告警通道
    pub async fn add_channel(
        &self,
        channel: AlertChannel,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut channels = self.channels.write().await;
        channels.insert(channel.id.clone(), channel);
        Ok(())
    }

    /// 生成诊断信息
    async fn generate_diagnosis(
        &self,
        condition: &AlertCondition,
        _metrics: &HashMap<String, f64>,
    ) -> DiagnosisInfo {
        match condition {
            AlertCondition::ResponseTime { threshold_ms, .. } => DiagnosisInfo {
                possible_causes: vec![
                    "数据库查询缓慢".to_string(),
                    "网络延迟增加".to_string(),
                    "CPU资源不足".to_string(),
                    "内存不足导致GC频繁".to_string(),
                ],
                recommended_actions: vec![
                    "检查数据库性能".to_string(),
                    "优化查询语句".to_string(),
                    "增加服务器资源".to_string(),
                    "检查网络连接".to_string(),
                ],
                related_logs: vec![],
                impact_assessment: format!("响应时间超过{}ms，可能影响用户体验", threshold_ms),
                auto_fix_suggestions: vec![AutoFixSuggestion {
                    fix_type: "缓存优化".to_string(),
                    description: "启用查询缓存以减少数据库负载".to_string(),
                    command: Some("redis-cli config set maxmemory-policy allkeys-lru".to_string()),
                    risk_level: "低".to_string(),
                    expected_outcome: "响应时间减少20-30%".to_string(),
                }],
            },
            AlertCondition::ErrorRate {
                threshold_percent, ..
            } => DiagnosisInfo {
                possible_causes: vec![
                    "代码错误或异常".to_string(),
                    "外部服务不可用".to_string(),
                    "配置错误".to_string(),
                    "资源耗尽".to_string(),
                ],
                recommended_actions: vec![
                    "检查错误日志".to_string(),
                    "验证外部服务状态".to_string(),
                    "检查配置文件".to_string(),
                    "监控资源使用情况".to_string(),
                ],
                related_logs: vec![],
                impact_assessment: format!(
                    "错误率达到{:.1}%，严重影响服务可用性",
                    threshold_percent
                ),
                auto_fix_suggestions: vec![AutoFixSuggestion {
                    fix_type: "服务重启".to_string(),
                    description: "重启相关服务以清除临时错误状态".to_string(),
                    command: Some("systemctl restart lumos-agent".to_string()),
                    risk_level: "中".to_string(),
                    expected_outcome: "清除临时错误状态，恢复正常服务".to_string(),
                }],
            },
            _ => DiagnosisInfo {
                possible_causes: vec!["需要进一步分析".to_string()],
                recommended_actions: vec!["联系技术支持".to_string()],
                related_logs: vec![],
                impact_assessment: "影响程度待评估".to_string(),
                auto_fix_suggestions: vec![],
            },
        }
    }
}

#[async_trait]
impl AlertManager for InMemoryAlertManager {
    async fn add_rule(
        &self,
        rule: AlertRule,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    async fn remove_rule(
        &self,
        rule_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut rules = self.rules.write().await;
        rules.remove(rule_id);
        Ok(())
    }

    async fn update_rule(
        &self,
        rule: AlertRule,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    async fn get_rules(&self) -> Result<Vec<AlertRule>, Box<dyn std::error::Error + Send + Sync>> {
        let rules = self.rules.read().await;
        Ok(rules.values().cloned().collect())
    }

    async fn evaluate_conditions(
        &self,
        metrics: &MetricsSummary,
    ) -> Result<Vec<AlertEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let rules = self.rules.read().await;
        let mut triggered_alerts = Vec::new();

        for rule in rules.values() {
            if !rule.enabled {
                continue;
            }

            let should_trigger = match &rule.condition {
                AlertCondition::ResponseTime { threshold_ms, .. } => {
                    metrics.avg_execution_time_ms > *threshold_ms as f64
                }
                AlertCondition::ErrorRate {
                    threshold_percent, ..
                } => {
                    let error_rate = if metrics.total_executions > 0 {
                        (metrics.failed_executions as f64 / metrics.total_executions as f64) * 100.0
                    } else {
                        0.0
                    };
                    error_rate > *threshold_percent
                }
                AlertCondition::MemoryUsage { threshold_mb, .. } => {
                    // 暂时使用平均执行时间作为内存使用的代理指标
                    metrics.avg_execution_time_ms > (*threshold_mb as f64 * 10.0)
                }
                AlertCondition::CpuUsage {
                    threshold_percent, ..
                } => {
                    // 暂时使用平均执行时间作为CPU使用的代理指标
                    metrics.avg_execution_time_ms > (*threshold_percent as f64 * 20.0)
                }
                _ => false, // 其他条件暂时不实现
            };

            if should_trigger {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                let mut alert_metrics = HashMap::new();
                alert_metrics.insert(
                    "avg_execution_time_ms".to_string(),
                    metrics.avg_execution_time_ms,
                );

                let success_rate = if metrics.total_executions > 0 {
                    metrics.successful_executions as f64 / metrics.total_executions as f64
                } else {
                    0.0
                };
                let error_rate = if metrics.total_executions > 0 {
                    metrics.failed_executions as f64 / metrics.total_executions as f64
                } else {
                    0.0
                };

                alert_metrics.insert("success_rate".to_string(), success_rate);
                alert_metrics.insert("error_rate".to_string(), error_rate);

                let diagnosis = self
                    .generate_diagnosis(&rule.condition, &alert_metrics)
                    .await;

                let alert = AlertEvent {
                    id: Uuid::new_v4().to_string(),
                    rule_id: rule.id.clone(),
                    title: format!("告警: {}", rule.name),
                    description: rule.description.clone(),
                    severity: rule.severity.clone(),
                    status: AlertStatus::Active,
                    triggered_at: now,
                    acknowledged_at: None,
                    resolved_at: None,
                    metrics: alert_metrics,
                    labels: rule.labels.clone(),
                    diagnosis: Some(diagnosis),
                };

                triggered_alerts.push(alert);
            }
        }

        // 将新触发的告警添加到活跃告警列表
        let mut active_alerts = self.active_alerts.write().await;
        for alert in &triggered_alerts {
            active_alerts.insert(alert.id.clone(), alert.clone());
        }

        Ok(triggered_alerts)
    }

    async fn send_alert(
        &self,
        alert: &AlertEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 这里实现告警发送逻辑
        println!("🚨 发送告警: {} - {}", alert.title, alert.description);

        // 模拟发送到不同通道
        let channels = self.channels.read().await;
        for channel_id in &self
            .rules
            .read()
            .await
            .get(&alert.rule_id)
            .unwrap()
            .channels
        {
            if let Some(channel) = channels.get(channel_id) {
                if channel.enabled {
                    match channel.channel_type {
                        AlertChannelType::Email => {
                            println!("📧 发送邮件告警到: {}", channel.name);
                        }
                        AlertChannelType::Slack => {
                            println!("💬 发送Slack告警到: {}", channel.name);
                        }
                        AlertChannelType::Webhook => {
                            println!("🔗 发送Webhook告警到: {}", channel.name);
                        }
                        _ => {
                            println!("📱 发送告警到 {:?}: {}", channel.channel_type, channel.name);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn acknowledge_alert(
        &self,
        alert_id: &str,
        user: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut active_alerts = self.active_alerts.write().await;
        if let Some(alert) = active_alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Acknowledged;
            alert.acknowledged_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            );
            println!("✅ 告警 {} 已被用户 {} 确认", alert_id, user);
        }
        Ok(())
    }

    async fn resolve_alert(
        &self,
        alert_id: &str,
        user: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut active_alerts = self.active_alerts.write().await;
        if let Some(mut alert) = active_alerts.remove(alert_id) {
            alert.status = AlertStatus::Resolved;
            alert.resolved_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            );

            // 添加到历史记录
            let mut history = self.alert_history.write().await;
            history.push_back(alert);

            // 限制历史记录大小
            while history.len() > self.max_history_size {
                history.pop_front();
            }

            println!("🔧 告警 {} 已被用户 {} 解决", alert_id, user);
        }
        Ok(())
    }

    async fn get_active_alerts(
        &self,
    ) -> Result<Vec<AlertEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let active_alerts = self.active_alerts.read().await;
        Ok(active_alerts.values().cloned().collect())
    }

    async fn get_alert_history(
        &self,
        from: Option<u64>,
        to: Option<u64>,
    ) -> Result<Vec<AlertEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let history = self.alert_history.read().await;
        let mut filtered_history: Vec<AlertEvent> = history
            .iter()
            .filter(|alert| {
                if let Some(from_time) = from {
                    if alert.triggered_at < from_time {
                        return false;
                    }
                }
                if let Some(to_time) = to {
                    if alert.triggered_at > to_time {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // 按时间倒序排列
        filtered_history.sort_by(|a, b| b.triggered_at.cmp(&a.triggered_at));

        Ok(filtered_history)
    }
}

impl Default for InMemoryAlertManager {
    fn default() -> Self {
        Self::new()
    }
}
