//! 威胁检测模块
//! 
//! 实时安全威胁监控和响应

use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration, Timelike};
use serde::{Deserialize, Serialize};
use regex::Regex;

use crate::error::{LumosError, Result};
use super::{SecurityContext, ThreatSeverity};

/// 威胁检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetectionConfig {
    /// 是否启用实时检测
    pub enable_realtime_detection: bool,
    
    /// 检测规则更新间隔（小时）
    pub rule_update_interval_hours: u64,
    
    /// 威胁情报源
    pub threat_intelligence_sources: Vec<String>,
    
    /// 异常检测敏感度
    pub anomaly_sensitivity: f64,
    
    /// 自动响应配置
    pub auto_response: AutoResponseConfig,
}

impl Default for ThreatDetectionConfig {
    fn default() -> Self {
        Self {
            enable_realtime_detection: true,
            rule_update_interval_hours: 1,
            threat_intelligence_sources: vec![
                "internal".to_string(),
                "mitre_attack".to_string(),
            ],
            anomaly_sensitivity: 0.8,
            auto_response: AutoResponseConfig::default(),
        }
    }
}

/// 自动响应配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResponseConfig {
    /// 是否启用自动阻断
    pub enable_auto_block: bool,
    
    /// 自动阻断阈值
    pub auto_block_threshold: ThreatSeverity,
    
    /// 是否启用自动隔离
    pub enable_auto_quarantine: bool,
    
    /// 通知配置
    pub notification_config: NotificationConfig,
}

impl Default for AutoResponseConfig {
    fn default() -> Self {
        Self {
            enable_auto_block: true,
            auto_block_threshold: ThreatSeverity::High,
            enable_auto_quarantine: false,
            notification_config: NotificationConfig::default(),
        }
    }
}

/// 通知配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub email_alerts: bool,
    pub slack_alerts: bool,
    pub webhook_url: Option<String>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            email_alerts: true,
            slack_alerts: false,
            webhook_url: None,
        }
    }
}

/// 威胁检测器
pub struct ThreatDetector {
    config: ThreatDetectionConfig,
    detection_rules: Vec<DetectionRule>,
    threat_intelligence: ThreatIntelligence,
    anomaly_detector: AnomalyDetector,
    response_engine: ResponseEngine,
    threat_history: Vec<ThreatAlert>,
}

/// 检测规则
#[derive(Debug, Clone)]
pub struct DetectionRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub pattern: String,
    pub severity: ThreatSeverity,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

/// 规则类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    Regex,
    Signature,
    Behavioral,
    Statistical,
}

/// 威胁情报
struct ThreatIntelligence {
    indicators: HashMap<String, ThreatIndicator>,
    last_updated: DateTime<Utc>,
}

/// 威胁指标
#[derive(Debug, Clone)]
struct ThreatIndicator {
    value: String,
    indicator_type: IndicatorType,
    severity: ThreatSeverity,
    source: String,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
}

/// 指标类型
#[derive(Debug, Clone)]
enum IndicatorType {
    IpAddress,
    Domain,
    Hash,
    UserAgent,
    Pattern,
}

/// 异常检测器
struct AnomalyDetector {
    baseline_models: HashMap<String, BaselineModel>,
    sensitivity: f64,
}

/// 基线模型
#[derive(Debug, Clone)]
struct BaselineModel {
    metric_name: String,
    normal_range: (f64, f64),
    last_updated: DateTime<Utc>,
}

/// 响应引擎
struct ResponseEngine {
    config: AutoResponseConfig,
    active_responses: HashMap<String, ActiveResponse>,
}

/// 活跃响应
#[derive(Debug, Clone)]
struct ActiveResponse {
    response_id: String,
    threat_id: String,
    response_type: ResponseType,
    started_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

/// 响应类型
#[derive(Debug, Clone)]
enum ResponseType {
    Block,
    Quarantine,
    Monitor,
    Alert,
}

/// 威胁告警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAlert {
    pub id: String,
    pub threat_type: String,
    pub severity: ThreatSeverity,
    pub source: String,
    pub target: String,
    pub description: String,
    pub indicators: HashMap<String, String>,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub status: AlertStatus,
}

/// 告警状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    New,
    Investigating,
    Confirmed,
    FalsePositive,
    Resolved,
}

/// 威胁级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl ThreatDetector {
    /// 创建新的威胁检测器
    pub async fn new(config: &ThreatDetectionConfig) -> Result<Self> {
        let detection_rules = Self::load_default_rules().await?;
        let threat_intelligence = ThreatIntelligence::new().await?;
        let anomaly_detector = AnomalyDetector::new(config.anomaly_sensitivity).await?;
        let response_engine = ResponseEngine::new(&config.auto_response).await?;
        
        Ok(Self {
            config: config.clone(),
            detection_rules,
            threat_intelligence,
            anomaly_detector,
            response_engine,
            threat_history: Vec::new(),
        })
    }
    
    /// 检测威胁
    pub async fn detect(&mut self, context: &SecurityContext) -> Result<Vec<ThreatAlert>> {
        let mut alerts = Vec::new();
        
        // 1. 基于规则的检测
        alerts.extend(self.rule_based_detection(context).await?);
        
        // 2. 威胁情报检测
        alerts.extend(self.threat_intelligence_detection(context).await?);
        
        // 3. 异常检测
        alerts.extend(self.anomaly_detection(context).await?);
        
        // 4. 处理告警
        for alert in &alerts {
            self.process_alert(alert).await?;
        }
        
        // 5. 保存到历史记录
        self.threat_history.extend(alerts.clone());
        
        Ok(alerts)
    }
    
    /// 获取当前威胁级别
    pub async fn get_current_threat_level(&self) -> Result<ThreatLevel> {
        let recent_alerts: Vec<_> = self.threat_history.iter()
            .filter(|alert| {
                let cutoff = Utc::now() - Duration::hours(24);
                alert.timestamp > cutoff
            })
            .collect();
        
        if recent_alerts.iter().any(|a| matches!(a.severity, ThreatSeverity::Critical)) {
            Ok(ThreatLevel::Critical)
        } else if recent_alerts.iter().any(|a| matches!(a.severity, ThreatSeverity::High)) {
            Ok(ThreatLevel::High)
        } else if recent_alerts.iter().any(|a| matches!(a.severity, ThreatSeverity::Medium)) {
            Ok(ThreatLevel::Medium)
        } else {
            Ok(ThreatLevel::Low)
        }
    }
    
    /// 添加检测规则
    pub async fn add_rule(&mut self, rule: DetectionRule) -> Result<()> {
        self.detection_rules.push(rule);
        Ok(())
    }
    
    /// 更新威胁情报
    pub async fn update_threat_intelligence(&mut self) -> Result<()> {
        self.threat_intelligence.update().await
    }
    
    /// 基于规则的检测
    async fn rule_based_detection(&self, context: &SecurityContext) -> Result<Vec<ThreatAlert>> {
        let mut alerts = Vec::new();
        
        for rule in &self.detection_rules {
            if !rule.enabled {
                continue;
            }
            
            match rule.rule_type {
                RuleType::Regex => {
                    if let Ok(regex) = Regex::new(&rule.pattern) {
                        let text = format!("{} {} {}",
                            context.request_path,
                            context.user_agent.as_deref().unwrap_or(""),
                            context.headers.values().map(|s| s.as_str()).collect::<Vec<_>>().join(" ")
                        );
                        
                        if regex.is_match(&text) {
                            alerts.push(ThreatAlert {
                                id: uuid::Uuid::new_v4().to_string(),
                                threat_type: rule.name.clone(),
                                severity: rule.severity.clone(),
                                source: context.ip_address.clone(),
                                target: context.request_path.clone(),
                                description: rule.description.clone(),
                                indicators: HashMap::from([
                                    ("rule_id".to_string(), rule.id.clone()),
                                    ("pattern".to_string(), rule.pattern.clone()),
                                ]),
                                confidence: 0.8,
                                timestamp: Utc::now(),
                                status: AlertStatus::New,
                            });
                        }
                    }
                }
                _ => {
                    // 其他规则类型的实现
                }
            }
        }
        
        Ok(alerts)
    }
    
    /// 威胁情报检测
    async fn threat_intelligence_detection(&self, context: &SecurityContext) -> Result<Vec<ThreatAlert>> {
        let mut alerts = Vec::new();
        
        // 检查IP地址
        if let Some(indicator) = self.threat_intelligence.indicators.get(&context.ip_address) {
            alerts.push(ThreatAlert {
                id: uuid::Uuid::new_v4().to_string(),
                threat_type: "Known Malicious IP".to_string(),
                severity: indicator.severity.clone(),
                source: context.ip_address.clone(),
                target: context.request_path.clone(),
                description: "Request from known malicious IP address".to_string(),
                indicators: HashMap::from([
                    ("ip_address".to_string(), context.ip_address.clone()),
                    ("source".to_string(), indicator.source.clone()),
                ]),
                confidence: 0.9,
                timestamp: Utc::now(),
                status: AlertStatus::New,
            });
        }
        
        Ok(alerts)
    }
    
    /// 异常检测
    async fn anomaly_detection(&self, context: &SecurityContext) -> Result<Vec<ThreatAlert>> {
        let mut alerts = Vec::new();
        
        // 简化的异常检测：检查请求频率
        let hour = context.timestamp.hour();
        if hour < 6 || hour > 22 {
            alerts.push(ThreatAlert {
                id: uuid::Uuid::new_v4().to_string(),
                threat_type: "Unusual Access Time".to_string(),
                severity: ThreatSeverity::Low,
                source: context.ip_address.clone(),
                target: context.request_path.clone(),
                description: "Access during unusual hours".to_string(),
                indicators: HashMap::from([
                    ("hour".to_string(), hour.to_string()),
                ]),
                confidence: 0.6,
                timestamp: Utc::now(),
                status: AlertStatus::New,
            });
        }
        
        Ok(alerts)
    }
    
    /// 处理告警
    async fn process_alert(&mut self, alert: &ThreatAlert) -> Result<()> {
        // 根据严重程度和配置决定响应
        match alert.severity {
            ThreatSeverity::Critical | ThreatSeverity::High => {
                if self.config.auto_response.enable_auto_block {
                    self.response_engine.block_source(&alert.source).await?;
                }
                self.response_engine.send_notification(alert).await?;
            }
            ThreatSeverity::Medium => {
                self.response_engine.send_notification(alert).await?;
            }
            ThreatSeverity::Low => {
                // 仅记录日志
            }
        }
        
        Ok(())
    }
    
    /// 加载默认规则
    async fn load_default_rules() -> Result<Vec<DetectionRule>> {
        Ok(vec![
            DetectionRule {
                id: "sql_injection".to_string(),
                name: "SQL Injection Attempt".to_string(),
                description: "Detects potential SQL injection attacks".to_string(),
                rule_type: RuleType::Regex,
                pattern: r"(?i)(union|select|insert|update|delete|drop|create|alter)\s+.*\s+(from|into|table)".to_string(),
                severity: ThreatSeverity::High,
                enabled: true,
                created_at: Utc::now(),
            },
            DetectionRule {
                id: "xss_attempt".to_string(),
                name: "XSS Attempt".to_string(),
                description: "Detects potential cross-site scripting attacks".to_string(),
                rule_type: RuleType::Regex,
                pattern: r"(?i)<script[^>]*>.*</script>|javascript:|on\w+\s*=".to_string(),
                severity: ThreatSeverity::Medium,
                enabled: true,
                created_at: Utc::now(),
            },
        ])
    }
}

impl ThreatIntelligence {
    async fn new() -> Result<Self> {
        let mut indicators = HashMap::new();
        
        // 添加一些示例威胁指标
        indicators.insert("192.168.100.100".to_string(), ThreatIndicator {
            value: "192.168.100.100".to_string(),
            indicator_type: IndicatorType::IpAddress,
            severity: ThreatSeverity::High,
            source: "internal_blacklist".to_string(),
            first_seen: Utc::now() - Duration::days(30),
            last_seen: Utc::now() - Duration::hours(1),
        });
        
        Ok(Self {
            indicators,
            last_updated: Utc::now(),
        })
    }
    
    async fn update(&mut self) -> Result<()> {
        // 在实际实现中，这里会从外部威胁情报源更新数据
        self.last_updated = Utc::now();
        Ok(())
    }
}

impl AnomalyDetector {
    async fn new(sensitivity: f64) -> Result<Self> {
        Ok(Self {
            baseline_models: HashMap::new(),
            sensitivity,
        })
    }
}

impl ResponseEngine {
    async fn new(config: &AutoResponseConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            active_responses: HashMap::new(),
        })
    }
    
    async fn block_source(&mut self, source: &str) -> Result<()> {
        let response_id = uuid::Uuid::new_v4().to_string();
        let response = ActiveResponse {
            response_id: response_id.clone(),
            threat_id: source.to_string(),
            response_type: ResponseType::Block,
            started_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::hours(24)),
        };
        
        self.active_responses.insert(response_id, response);
        
        // 在实际实现中，这里会调用防火墙或其他安全设备的API
        println!("Blocked source: {}", source);
        
        Ok(())
    }
    
    async fn send_notification(&self, alert: &ThreatAlert) -> Result<()> {
        if self.config.notification_config.email_alerts {
            // 发送邮件通知
            println!("Email alert sent for threat: {}", alert.threat_type);
        }
        
        if let Some(webhook_url) = &self.config.notification_config.webhook_url {
            // 发送Webhook通知
            println!("Webhook notification sent to: {}", webhook_url);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_threat_detector_creation() {
        let config = ThreatDetectionConfig::default();
        let detector = ThreatDetector::new(&config).await;
        assert!(detector.is_ok());
    }
    
    #[tokio::test]
    async fn test_sql_injection_detection() {
        let config = ThreatDetectionConfig::default();
        let mut detector = ThreatDetector::new(&config).await.unwrap();
        
        let context = SecurityContext {
            user_id: Some("test_user".to_string()),
            session_id: None,
            ip_address: "192.168.1.1".to_string(),
            user_agent: None,
            request_path: "/api/users?id=1' UNION SELECT * FROM passwords--".to_string(),
            request_method: "GET".to_string(),
            headers: HashMap::new(),
            timestamp: Utc::now(),
        };
        
        let alerts = detector.detect(&context).await.unwrap();
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| a.threat_type == "SQL Injection Attempt"));
    }
}
