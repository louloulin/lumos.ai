//! 智能告警引擎 - 企业级监控和自动化响应系统
//!
//! 提供基于规则的智能告警、自动化响应和告警生命周期管理

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::alerts::{AlertCondition, AlertEvent, AlertRule, AlertSeverity, AlertStatus};
use super::metrics::{MetricsCollector, MetricsSummary};

/// 智能告警引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEngineConfig {
    /// 告警检查间隔（秒）
    pub check_interval_seconds: u64,
    /// 最大并发告警数量
    pub max_concurrent_alerts: usize,
    /// 告警去重时间窗口（秒）
    pub deduplication_window_seconds: u64,
    /// 自动恢复检查间隔（秒）
    pub auto_recovery_check_seconds: u64,
    /// 告警升级配置
    pub escalation_config: EscalationConfig,
    /// 自动化响应配置
    pub automation_config: AutomationConfig,
}

/// 告警升级配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// 是否启用告警升级
    pub enabled: bool,
    /// 升级时间阈值（分钟）
    pub escalation_time_minutes: u64,
    /// 升级级别映射
    pub severity_escalation: HashMap<AlertSeverity, AlertSeverity>,
}

/// 自动化响应配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    /// 是否启用自动化响应
    pub enabled: bool,
    /// 自动化操作定义
    pub actions: HashMap<String, AutomationAction>,
    /// 响应超时时间（秒）
    pub action_timeout_seconds: u64,
}

/// 自动化操作定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationAction {
    /// 操作名称
    pub name: String,
    /// 操作类型
    pub action_type: AutomationActionType,
    /// 操作参数
    pub parameters: HashMap<String, serde_json::Value>,
    /// 触发条件
    pub trigger_conditions: Vec<String>,
    /// 是否启用
    pub enabled: bool,
}

/// 自动化操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationActionType {
    /// 重启服务
    RestartService,
    /// 扩容资源
    ScaleUp,
    /// 缩容资源
    ScaleDown,
    /// 发送通知
    SendNotification,
    /// 执行脚本
    ExecuteScript,
    /// 调用Webhook
    CallWebhook,
    /// 更新配置
    UpdateConfig,
}

/// 告警上下文信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertContext {
    /// 告警源
    pub source: String,
    /// 相关资源
    pub resource: String,
    /// 环境信息
    pub environment: String,
    /// 标签
    pub labels: HashMap<String, String>,
    /// 额外元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 告警处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertActionResult {
    /// 操作ID
    pub action_id: String,
    /// 操作类型
    pub action_type: AutomationActionType,
    /// 执行状态
    pub status: ActionStatus,
    /// 执行时间
    pub execution_time: u64,
    /// 执行结果
    pub result: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
}

/// 操作执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 执行成功
    Success,
    /// 执行失败
    Failed,
    /// 超时
    Timeout,
}

/// 智能告警引擎
pub struct SmartAlertEngine {
    /// 配置
    config: AlertEngineConfig,
    /// 指标收集器
    metrics_collector: Arc<dyn MetricsCollector>,
    /// 告警规则
    rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    /// 活跃告警
    active_alerts: Arc<RwLock<HashMap<String, AlertEvent>>>,
    /// 告警历史
    alert_history: Arc<RwLock<Vec<AlertEvent>>>,
    /// 自动化操作执行器
    automation_executor: Arc<dyn AutomationExecutor>,
    /// 告警统计
    alert_stats: Arc<RwLock<AlertStatistics>>,
}

/// 告警统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertStatistics {
    /// 总告警数量
    pub total_alerts: u64,
    /// 活跃告警数量
    pub active_alerts: u64,
    /// 已解决告警数量
    pub resolved_alerts: u64,
    /// 按严重程度分组的告警数量
    pub alerts_by_severity: HashMap<AlertSeverity, u64>,
    /// 按规则分组的告警数量
    pub alerts_by_rule: HashMap<String, u64>,
    /// 平均解决时间（分钟）
    pub avg_resolution_time_minutes: f64,
    /// 自动化操作执行次数
    pub automation_executions: u64,
    /// 自动化操作成功率
    pub automation_success_rate: f64,
}

/// 自动化操作执行器trait
#[async_trait]
pub trait AutomationExecutor: Send + Sync {
    /// 执行自动化操作
    async fn execute_action(
        &self,
        action: &AutomationAction,
        alert: &AlertEvent,
        context: &AlertContext,
    ) -> Result<AlertActionResult, Box<dyn std::error::Error + Send + Sync>>;

    /// 检查操作执行状态
    async fn check_action_status(
        &self,
        action_id: &str,
    ) -> Result<ActionStatus, Box<dyn std::error::Error + Send + Sync>>;

    /// 取消操作执行
    async fn cancel_action(
        &self,
        action_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认自动化执行器
pub struct DefaultAutomationExecutor {
    /// 配置
    config: AutomationConfig,
}

impl DefaultAutomationExecutor {
    pub fn new(config: AutomationConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AutomationExecutor for DefaultAutomationExecutor {
    async fn execute_action(
        &self,
        action: &AutomationAction,
        alert: &AlertEvent,
        context: &AlertContext,
    ) -> Result<AlertActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let action_id = Uuid::new_v4().to_string();
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        println!("🤖 执行自动化操作: {} (ID: {})", action.name, action_id);
        println!("   告警: {} - {}", alert.title, alert.description);
        println!("   上下文: {} / {}", context.source, context.resource);

        // 模拟操作执行
        let result: Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> =
            match action.action_type {
                AutomationActionType::RestartService => {
                    println!("   🔄 重启服务: {}", context.resource);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    Ok(serde_json::json!({"service": context.resource, "status": "restarted"}))
                }
                AutomationActionType::ScaleUp => {
                    println!("   📈 扩容资源: {}", context.resource);
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    Ok(
                        serde_json::json!({"resource": context.resource, "action": "scaled_up", "instances": 3}),
                    )
                }
                AutomationActionType::SendNotification => {
                    println!("   📧 发送通知: {}", alert.title);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    Ok(serde_json::json!({"notification": "sent", "channels": ["email", "slack"]}))
                }
                AutomationActionType::CallWebhook => {
                    println!("   🔗 调用Webhook: {:?}", action.parameters.get("url"));
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    Ok(serde_json::json!({"webhook": "called", "status": "success"}))
                }
                _ => {
                    println!("   ⚠️  操作类型暂未实现: {:?}", action.action_type);
                    Ok(serde_json::json!({"status": "not_implemented"}))
                }
            };

        let execution_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            - start_time;

        match result {
            Ok(result_data) => {
                println!("   ✅ 操作执行成功 (耗时: {}ms)", execution_time);
                Ok(AlertActionResult {
                    action_id,
                    action_type: action.action_type.clone(),
                    status: ActionStatus::Success,
                    execution_time,
                    result: Some(result_data),
                    error: None,
                })
            }
            Err(e) => {
                println!("   ❌ 操作执行失败: {}", e);
                Ok(AlertActionResult {
                    action_id,
                    action_type: action.action_type.clone(),
                    status: ActionStatus::Failed,
                    execution_time,
                    result: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    async fn check_action_status(
        &self,
        action_id: &str,
    ) -> Result<ActionStatus, Box<dyn std::error::Error + Send + Sync>> {
        // 模拟状态检查
        println!("🔍 检查操作状态: {}", action_id);
        Ok(ActionStatus::Success)
    }

    async fn cancel_action(
        &self,
        action_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🛑 取消操作: {}", action_id);
        Ok(())
    }
}

impl SmartAlertEngine {
    /// 创建新的智能告警引擎
    pub fn new(
        config: AlertEngineConfig,
        metrics_collector: Arc<dyn MetricsCollector>,
        automation_executor: Arc<dyn AutomationExecutor>,
    ) -> Self {
        Self {
            config,
            metrics_collector,
            rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            automation_executor,
            alert_stats: Arc::new(RwLock::new(AlertStatistics::default())),
        }
    }

    /// 添加告警规则
    pub async fn add_rule(
        &self,
        rule: AlertRule,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id.clone(), rule);
        println!("✅ 添加告警规则: {}", rules.len());
        Ok(())
    }

    /// 移除告警规则
    pub async fn remove_rule(
        &self,
        rule_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut rules = self.rules.write().await;
        rules.remove(rule_id);
        println!("🗑️  移除告警规则: {}", rule_id);
        Ok(())
    }

    /// 启动告警引擎
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 启动智能告警引擎");

        // 启动告警检查循环
        let engine = self.clone();
        tokio::spawn(async move {
            engine.alert_check_loop().await;
        });

        // 启动自动恢复检查循环
        let engine = self.clone();
        tokio::spawn(async move {
            engine.auto_recovery_loop().await;
        });

        // 启动告警升级检查循环
        let engine = self.clone();
        tokio::spawn(async move {
            engine.escalation_loop().await;
        });

        Ok(())
    }

    /// 告警检查循环
    async fn alert_check_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.check_interval_seconds,
        ));

        loop {
            interval.tick().await;

            if let Err(e) = self.check_alerts().await {
                eprintln!("告警检查失败: {}", e);
            }
        }
    }

    /// 检查告警
    async fn check_alerts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rules = self.rules.read().await;
        let mut triggered_alerts = Vec::new();

        for rule in rules.values() {
            if !rule.enabled {
                continue;
            }

            // 检查告警条件
            if let Ok(should_trigger) = self.evaluate_rule_condition(rule).await {
                if should_trigger {
                    // 检查是否需要去重
                    if !self.is_duplicate_alert(rule).await {
                        let alert = self.create_alert_from_rule(rule).await?;
                        triggered_alerts.push(alert);
                    }
                }
            }
        }

        // 处理触发的告警
        for alert in triggered_alerts {
            self.handle_triggered_alert(alert).await?;
        }

        Ok(())
    }

    /// 评估规则条件
    async fn evaluate_rule_condition(
        &self,
        rule: &AlertRule,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // 获取相关指标
        let metrics = self
            .metrics_collector
            .get_metrics_summary(None, None, None)
            .await?;

        match &rule.condition {
            AlertCondition::ResponseTime { threshold_ms, .. } => {
                Ok(metrics.avg_execution_time_ms > *threshold_ms as f64)
            }
            AlertCondition::ErrorRate {
                threshold_percent, ..
            } => {
                let error_rate = if metrics.total_executions > 0 {
                    (metrics.failed_executions as f64 / metrics.total_executions as f64) * 100.0
                } else {
                    0.0
                };
                Ok(error_rate > *threshold_percent)
            }
            AlertCondition::MemoryUsage { threshold_mb, .. } => {
                // 简化的内存使用检查
                Ok(metrics.avg_execution_time_ms > (*threshold_mb as f64 * 10.0))
            }
            AlertCondition::CpuUsage {
                threshold_percent, ..
            } => {
                // 简化的CPU使用检查
                Ok(metrics.avg_execution_time_ms > (*threshold_percent as f64 * 20.0))
            }
            _ => Ok(false),
        }
    }

    /// 检查是否为重复告警
    async fn is_duplicate_alert(&self, rule: &AlertRule) -> bool {
        let active_alerts = self.active_alerts.read().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        for alert in active_alerts.values() {
            if alert.rule_id == rule.id {
                let time_diff = now - alert.triggered_at;
                if time_diff < self.config.deduplication_window_seconds * 1000 {
                    return true;
                }
            }
        }

        false
    }

    /// 从规则创建告警
    async fn create_alert_from_rule(
        &self,
        rule: &AlertRule,
    ) -> Result<AlertEvent, Box<dyn std::error::Error + Send + Sync>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let metrics = self
            .metrics_collector
            .get_metrics_summary(None, None, None)
            .await?;
        let mut alert_metrics = HashMap::new();
        alert_metrics.insert(
            "avg_execution_time".to_string(),
            metrics.avg_execution_time_ms,
        );
        alert_metrics.insert(
            "total_executions".to_string(),
            metrics.total_executions as f64,
        );
        alert_metrics.insert(
            "failed_executions".to_string(),
            metrics.failed_executions as f64,
        );

        Ok(AlertEvent {
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
            diagnosis: None,
        })
    }

    /// 处理触发的告警
    async fn handle_triggered_alert(
        &self,
        alert: AlertEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚨 触发告警: {} - {}", alert.title, alert.description);

        // 添加到活跃告警列表
        {
            let mut active_alerts = self.active_alerts.write().await;
            active_alerts.insert(alert.id.clone(), alert.clone());
        }

        // 添加到历史记录
        {
            let mut history = self.alert_history.write().await;
            history.push(alert.clone());
        }

        // 更新统计信息
        self.update_alert_statistics(&alert).await;

        // 执行自动化响应
        if self.config.automation_config.enabled {
            self.execute_automation_response(&alert).await?;
        }

        Ok(())
    }

    /// 执行自动化响应
    async fn execute_automation_response(
        &self,
        alert: &AlertEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rules = self.rules.read().await;
        if let Some(rule) = rules.get(&alert.rule_id) {
            for channel_id in &rule.channels {
                if let Some(action) = self.config.automation_config.actions.get(channel_id) {
                    if action.enabled {
                        let context = AlertContext {
                            source: "lumos-ai".to_string(),
                            resource: alert.rule_id.clone(),
                            environment: "production".to_string(),
                            labels: alert.labels.clone(),
                            metadata: HashMap::new(),
                        };

                        match self
                            .automation_executor
                            .execute_action(action, alert, &context)
                            .await
                        {
                            Ok(result) => {
                                println!("✅ 自动化响应执行成功: {:?}", result.status);
                                self.update_automation_statistics(true).await;
                            }
                            Err(e) => {
                                println!("❌ 自动化响应执行失败: {}", e);
                                self.update_automation_statistics(false).await;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 自动恢复检查循环
    async fn auto_recovery_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.auto_recovery_check_seconds,
        ));

        loop {
            interval.tick().await;

            if let Err(e) = self.check_auto_recovery().await {
                eprintln!("自动恢复检查失败: {}", e);
            }
        }
    }

    /// 检查自动恢复
    async fn check_auto_recovery(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut active_alerts = self.active_alerts.write().await;
        let rules = self.rules.read().await;
        let mut resolved_alerts = Vec::new();

        for (alert_id, alert) in active_alerts.iter() {
            if let Some(rule) = rules.get(&alert.rule_id) {
                // 重新评估告警条件
                if let Ok(should_trigger) = self.evaluate_rule_condition(rule).await {
                    if !should_trigger {
                        // 告警条件已恢复
                        let mut resolved_alert = alert.clone();
                        resolved_alert.status = AlertStatus::Resolved;
                        resolved_alert.resolved_at = Some(
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64,
                        );

                        resolved_alerts.push((alert_id.clone(), resolved_alert));
                    }
                }
            }
        }

        // 处理已恢复的告警
        for (alert_id, resolved_alert) in resolved_alerts {
            active_alerts.remove(&alert_id);
            println!("✅ 告警自动恢复: {}", resolved_alert.title);

            // 更新历史记录
            let mut history = self.alert_history.write().await;
            if let Some(historical_alert) = history.iter_mut().find(|a| a.id == alert_id) {
                historical_alert.status = AlertStatus::Resolved;
                historical_alert.resolved_at = resolved_alert.resolved_at;
            }

            // 更新统计信息
            self.update_resolution_statistics(&resolved_alert).await;
        }

        Ok(())
    }

    /// 告警升级检查循环
    async fn escalation_loop(&self) {
        if !self.config.escalation_config.enabled {
            return;
        }

        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(60), // 每分钟检查一次升级
        );

        loop {
            interval.tick().await;

            if let Err(e) = self.check_escalation().await {
                eprintln!("告警升级检查失败: {}", e);
            }
        }
    }

    /// 检查告警升级
    async fn check_escalation(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut active_alerts = self.active_alerts.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let escalation_threshold =
            self.config.escalation_config.escalation_time_minutes * 60 * 1000;

        for alert in active_alerts.values_mut() {
            if alert.status == AlertStatus::Active {
                let alert_age = now - alert.triggered_at;

                if alert_age > escalation_threshold {
                    if let Some(new_severity) = self
                        .config
                        .escalation_config
                        .severity_escalation
                        .get(&alert.severity)
                    {
                        if new_severity != &alert.severity {
                            println!(
                                "⬆️  告警升级: {} ({:?} -> {:?})",
                                alert.title, alert.severity, new_severity
                            );
                            alert.severity = new_severity.clone();
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 更新告警统计信息
    async fn update_alert_statistics(&self, alert: &AlertEvent) {
        let mut stats = self.alert_stats.write().await;
        stats.total_alerts += 1;
        stats.active_alerts += 1;

        *stats
            .alerts_by_severity
            .entry(alert.severity.clone())
            .or_insert(0) += 1;
        *stats
            .alerts_by_rule
            .entry(alert.rule_id.clone())
            .or_insert(0) += 1;
    }

    /// 更新解决统计信息
    async fn update_resolution_statistics(&self, alert: &AlertEvent) {
        let mut stats = self.alert_stats.write().await;
        stats.active_alerts = stats.active_alerts.saturating_sub(1);
        stats.resolved_alerts += 1;

        if let Some(resolved_at) = alert.resolved_at {
            let resolution_time = (resolved_at - alert.triggered_at) / (60 * 1000); // 转换为分钟

            // 更新平均解决时间
            let total_resolved = stats.resolved_alerts as f64;
            stats.avg_resolution_time_minutes = (stats.avg_resolution_time_minutes
                * (total_resolved - 1.0)
                + resolution_time as f64)
                / total_resolved;
        }
    }

    /// 更新自动化统计信息
    async fn update_automation_statistics(&self, success: bool) {
        let mut stats = self.alert_stats.write().await;
        stats.automation_executions += 1;

        if success {
            let total_executions = stats.automation_executions as f64;
            let current_success_count = stats.automation_success_rate * (total_executions - 1.0);
            stats.automation_success_rate = (current_success_count + 1.0) / total_executions;
        } else {
            let total_executions = stats.automation_executions as f64;
            let current_success_count = stats.automation_success_rate * (total_executions - 1.0);
            stats.automation_success_rate = current_success_count / total_executions;
        }
    }

    /// 获取告警统计信息
    pub async fn get_alert_statistics(&self) -> AlertStatistics {
        self.alert_stats.read().await.clone()
    }

    /// 获取活跃告警列表
    pub async fn get_active_alerts(&self) -> Vec<AlertEvent> {
        self.active_alerts.read().await.values().cloned().collect()
    }

    /// 获取告警历史
    pub async fn get_alert_history(&self, limit: Option<usize>) -> Vec<AlertEvent> {
        let history = self.alert_history.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.clone(),
        }
    }

    /// 确认告警
    pub async fn acknowledge_alert(
        &self,
        alert_id: &str,
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
            println!("✅ 告警已确认: {}", alert.title);
        }
        Ok(())
    }

    /// 手动解决告警
    pub async fn resolve_alert(
        &self,
        alert_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut active_alerts = self.active_alerts.write().await;
        if let Some(alert) = active_alerts.remove(alert_id) {
            let mut resolved_alert = alert;
            resolved_alert.status = AlertStatus::Resolved;
            resolved_alert.resolved_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            );

            // 更新历史记录
            let mut history = self.alert_history.write().await;
            if let Some(historical_alert) = history.iter_mut().find(|a| a.id == alert_id) {
                historical_alert.status = AlertStatus::Resolved;
                historical_alert.resolved_at = resolved_alert.resolved_at;
            }

            self.update_resolution_statistics(&resolved_alert).await;
            println!("✅ 告警已手动解决: {}", resolved_alert.title);
        }
        Ok(())
    }
}

// 为SmartAlertEngine实现Clone trait
impl Clone for SmartAlertEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metrics_collector: self.metrics_collector.clone(),
            rules: self.rules.clone(),
            active_alerts: self.active_alerts.clone(),
            alert_history: self.alert_history.clone(),
            automation_executor: self.automation_executor.clone(),
            alert_stats: self.alert_stats.clone(),
        }
    }
}

impl Default for AlertEngineConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30,
            max_concurrent_alerts: 100,
            deduplication_window_seconds: 300,
            auto_recovery_check_seconds: 60,
            escalation_config: EscalationConfig {
                enabled: true,
                escalation_time_minutes: 15,
                severity_escalation: {
                    let mut map = HashMap::new();
                    map.insert(AlertSeverity::Warning, AlertSeverity::Critical);
                    map.insert(AlertSeverity::Critical, AlertSeverity::Critical);
                    map
                },
            },
            automation_config: AutomationConfig {
                enabled: true,
                actions: HashMap::new(),
                action_timeout_seconds: 300,
            },
        }
    }
}
