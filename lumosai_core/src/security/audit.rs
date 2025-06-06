//! 审计日志模块
//! 
//! 完整的操作记录和追踪

use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{LumosError, Result};
use super::SecurityEvent;

/// 审计配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// 是否启用审计
    pub enabled: bool,
    
    /// 日志保留天数
    pub retention_days: u32,
    
    /// 存储后端
    pub storage_backend: StorageBackend,
    
    /// 是否启用实时流
    pub enable_realtime_stream: bool,
    
    /// 审计级别
    pub audit_level: AuditLevel,
    
    /// 敏感字段掩码
    pub sensitive_fields: Vec<String>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 365,
            storage_backend: StorageBackend::File {
                path: "./audit_logs".to_string(),
            },
            enable_realtime_stream: true,
            audit_level: AuditLevel::Standard,
            sensitive_fields: vec![
                "password".to_string(),
                "token".to_string(),
                "secret".to_string(),
                "key".to_string(),
            ],
        }
    }
}

/// 存储后端
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    File { path: String },
    Database { connection_string: String },
    ElasticSearch { endpoint: String },
    S3 { bucket: String, region: String },
}

/// 审计级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLevel {
    Minimal,    // 仅记录关键操作
    Standard,   // 记录大部分操作
    Detailed,   // 记录所有操作
    Debug,      // 包含调试信息
}

/// 审计日志记录器
pub struct AuditLogger {
    config: AuditConfig,
    storage: Box<dyn AuditStorage>,
    event_processor: EventProcessor,
    realtime_stream: Option<RealtimeStream>,
}

/// 审计存储trait
#[async_trait]
pub trait AuditStorage: Send + Sync {
    async fn store_event(&mut self, event: &AuditEvent) -> Result<()>;
    async fn query_events(&self, query: &AuditQuery) -> Result<Vec<AuditEvent>>;
    async fn cleanup_old_events(&mut self, cutoff_date: DateTime<Utc>) -> Result<usize>;
}

/// 事件处理器
struct EventProcessor {
    sensitive_fields: Vec<String>,
    audit_level: AuditLevel,
}

/// 实时流
struct RealtimeStream {
    subscribers: Vec<Box<dyn AuditSubscriber>>,
}

/// 审计订阅者trait
#[async_trait]
pub trait AuditSubscriber: Send + Sync {
    async fn on_audit_event(&mut self, event: &AuditEvent) -> Result<()>;
}

/// 审计事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub event_type: AuditEventType,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub resource: String,
    pub action: String,
    pub outcome: AuditOutcome,
    pub details: HashMap<String, serde_json::Value>,
    pub risk_score: Option<f64>,
    pub compliance_tags: Vec<String>,
}

/// 审计事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    SystemAccess,
    ConfigurationChange,
    SecurityEvent,
    ComplianceEvent,
}

/// 审计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOutcome {
    Success,
    Failure,
    Partial,
    Unknown,
}

/// 审计查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub user_id: Option<String>,
    pub event_type: Option<AuditEventType>,
    pub resource: Option<String>,
    pub outcome: Option<AuditOutcome>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// 文件存储实现
struct FileStorage {
    base_path: String,
}

impl AuditLogger {
    /// 创建新的审计日志记录器
    pub async fn new(config: &AuditConfig) -> Result<Self> {
        let storage: Box<dyn AuditStorage> = match &config.storage_backend {
            StorageBackend::File { path } => {
                Box::new(FileStorage::new(path).await?)
            }
            _ => {
                return Err(LumosError::SecurityError(
                    "Unsupported storage backend".to_string()
                ));
            }
        };
        
        let event_processor = EventProcessor::new(
            config.sensitive_fields.clone(),
            config.audit_level.clone(),
        );
        
        let realtime_stream = if config.enable_realtime_stream {
            Some(RealtimeStream::new())
        } else {
            None
        };
        
        Ok(Self {
            config: config.clone(),
            storage,
            event_processor,
            realtime_stream,
        })
    }
    
    /// 记录安全事件
    pub async fn log_event(&mut self, security_event: SecurityEvent) -> Result<()> {
        let audit_event = self.convert_security_event(security_event)?;
        self.log_audit_event(audit_event).await
    }
    
    /// 记录审计事件
    pub async fn log_audit_event(&mut self, mut event: AuditEvent) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // 处理事件（掩码敏感字段等）
        self.event_processor.process_event(&mut event)?;
        
        // 存储事件
        self.storage.store_event(&event).await?;
        
        // 实时流推送
        if let Some(stream) = &mut self.realtime_stream {
            stream.publish_event(&event).await?;
        }
        
        Ok(())
    }
    
    /// 查询审计事件
    pub async fn query_events(&self, query: &AuditQuery) -> Result<Vec<AuditEvent>> {
        self.storage.query_events(query).await
    }
    
    /// 清理过期事件
    pub async fn cleanup_expired_events(&mut self) -> Result<usize> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        self.storage.cleanup_old_events(cutoff_date).await
    }
    
    /// 添加实时订阅者
    pub async fn add_subscriber(&mut self, subscriber: Box<dyn AuditSubscriber>) -> Result<()> {
        if let Some(stream) = &mut self.realtime_stream {
            stream.add_subscriber(subscriber);
        }
        Ok(())
    }
    
    /// 生成合规报告
    pub async fn generate_compliance_report(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        compliance_standard: &str,
    ) -> Result<ComplianceReport> {
        let query = AuditQuery {
            start_time: Some(start_time),
            end_time: Some(end_time),
            user_id: None,
            event_type: None,
            resource: None,
            outcome: None,
            limit: None,
            offset: None,
        };
        
        let events = self.query_events(&query).await?;
        let filtered_events: Vec<_> = events.into_iter()
            .filter(|e| e.compliance_tags.contains(&compliance_standard.to_string()))
            .collect();
        
        Ok(ComplianceReport {
            standard: compliance_standard.to_string(),
            period_start: start_time,
            period_end: end_time,
            total_events: filtered_events.len(),
            success_events: filtered_events.iter().filter(|e| matches!(e.outcome, AuditOutcome::Success)).count(),
            failure_events: filtered_events.iter().filter(|e| matches!(e.outcome, AuditOutcome::Failure)).count(),
            events: filtered_events,
            generated_at: Utc::now(),
        })
    }
    
    /// 转换安全事件为审计事件
    fn convert_security_event(&self, security_event: SecurityEvent) -> Result<AuditEvent> {
        let (event_type, user_id, ip_address, resource, action, outcome, details, timestamp) = match security_event {
            SecurityEvent::LoginAttempt { user_id, success, ip_address, timestamp } => {
                let outcome = if success { AuditOutcome::Success } else { AuditOutcome::Failure };
                let details = HashMap::from([
                    ("login_attempt".to_string(), serde_json::Value::Bool(true)),
                ]);
                (AuditEventType::Authentication, Some(user_id), ip_address, "login".to_string(), "authenticate".to_string(), outcome, details, timestamp)
            }
            SecurityEvent::PermissionCheck { user_id, resource, action, granted, timestamp } => {
                let outcome = if granted { AuditOutcome::Success } else { AuditOutcome::Failure };
                let details = HashMap::from([
                    ("permission_check".to_string(), serde_json::Value::Bool(true)),
                ]);
                (AuditEventType::Authorization, Some(user_id), "unknown".to_string(), resource, action, outcome, details, timestamp)
            }
            SecurityEvent::DataAccess { user_id, resource_type, resource_id, action, timestamp } => {
                let details = HashMap::from([
                    ("resource_type".to_string(), serde_json::Value::String(resource_type)),
                    ("resource_id".to_string(), serde_json::Value::String(resource_id)),
                ]);
                (AuditEventType::DataAccess, Some(user_id), "unknown".to_string(), "data".to_string(), action, AuditOutcome::Success, details, timestamp)
            }
            SecurityEvent::ThreatDetected { threat_type, severity, source, details, timestamp } => {
                let mut audit_details = HashMap::new();
                audit_details.insert("threat_type".to_string(), serde_json::Value::String(threat_type));
                audit_details.insert("severity".to_string(), serde_json::to_value(severity)?);
                for (k, v) in details {
                    audit_details.insert(k, serde_json::Value::String(v));
                }
                (AuditEventType::SecurityEvent, None, source, "threat_detection".to_string(), "detect".to_string(), AuditOutcome::Success, audit_details, timestamp)
            }
            SecurityEvent::ComplianceViolation { standard, rule, severity, details, timestamp } => {
                let audit_details = HashMap::from([
                    ("standard".to_string(), serde_json::Value::String(standard)),
                    ("rule".to_string(), serde_json::Value::String(rule)),
                    ("severity".to_string(), serde_json::to_value(severity)?),
                    ("details".to_string(), serde_json::Value::String(details)),
                ]);
                (AuditEventType::ComplianceEvent, None, "system".to_string(), "compliance".to_string(), "check".to_string(), AuditOutcome::Failure, audit_details, timestamp)
            }
        };
        
        Ok(AuditEvent {
            id: Uuid::new_v4().to_string(),
            event_type,
            timestamp,
            user_id,
            session_id: None,
            ip_address,
            user_agent: None,
            resource,
            action,
            outcome,
            details,
            risk_score: None,
            compliance_tags: vec!["SOC2".to_string(), "GDPR".to_string()],
        })
    }
}

/// 合规报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub standard: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: usize,
    pub success_events: usize,
    pub failure_events: usize,
    pub events: Vec<AuditEvent>,
    pub generated_at: DateTime<Utc>,
}

impl EventProcessor {
    fn new(sensitive_fields: Vec<String>, audit_level: AuditLevel) -> Self {
        Self {
            sensitive_fields,
            audit_level,
        }
    }
    
    fn process_event(&self, event: &mut AuditEvent) -> Result<()> {
        // 掩码敏感字段
        for field in &self.sensitive_fields {
            if let Some(value) = event.details.get_mut(field) {
                *value = serde_json::Value::String("***MASKED***".to_string());
            }
        }
        
        // 根据审计级别过滤详细信息
        match self.audit_level {
            AuditLevel::Minimal => {
                event.details.clear();
            }
            AuditLevel::Standard => {
                // 保留重要字段
            }
            AuditLevel::Detailed | AuditLevel::Debug => {
                // 保留所有字段
            }
        }
        
        Ok(())
    }
}

impl RealtimeStream {
    fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }
    
    fn add_subscriber(&mut self, subscriber: Box<dyn AuditSubscriber>) {
        self.subscribers.push(subscriber);
    }
    
    async fn publish_event(&mut self, event: &AuditEvent) -> Result<()> {
        for subscriber in &mut self.subscribers {
            subscriber.on_audit_event(event).await?;
        }
        Ok(())
    }
}

impl FileStorage {
    async fn new(base_path: &str) -> Result<Self> {
        // 创建目录
        std::fs::create_dir_all(base_path)
            .map_err(|e| LumosError::SecurityError(format!("Failed to create audit directory: {}", e)))?;
        
        Ok(Self {
            base_path: base_path.to_string(),
        })
    }
}

#[async_trait]
impl AuditStorage for FileStorage {
    async fn store_event(&mut self, event: &AuditEvent) -> Result<()> {
        let date = event.timestamp.format("%Y-%m-%d").to_string();
        let file_path = format!("{}/audit-{}.jsonl", self.base_path, date);
        
        let event_json = serde_json::to_string(event)
            .map_err(|e| LumosError::SecurityError(format!("Failed to serialize audit event: {}", e)))?;
        
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| LumosError::SecurityError(format!("Failed to open audit file: {}", e)))?;
        
        writeln!(file, "{}", event_json)
            .map_err(|e| LumosError::SecurityError(format!("Failed to write audit event: {}", e)))?;
        
        Ok(())
    }
    
    async fn query_events(&self, _query: &AuditQuery) -> Result<Vec<AuditEvent>> {
        // 简化实现：返回空结果
        // 在实际实现中，这里会解析文件并根据查询条件过滤
        Ok(Vec::new())
    }
    
    async fn cleanup_old_events(&mut self, cutoff_date: DateTime<Utc>) -> Result<usize> {
        // 简化实现：返回0
        // 在实际实现中，这里会删除过期的日志文件
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_audit_logger_creation() {
        let config = AuditConfig::default();
        let logger = AuditLogger::new(&config).await;
        assert!(logger.is_ok());
    }
    
    #[tokio::test]
    async fn test_log_security_event() {
        let config = AuditConfig::default();
        let mut logger = AuditLogger::new(&config).await.unwrap();
        
        let security_event = SecurityEvent::LoginAttempt {
            user_id: "test_user".to_string(),
            success: true,
            ip_address: "192.168.1.1".to_string(),
            timestamp: Utc::now(),
        };
        
        let result = logger.log_event(security_event).await;
        assert!(result.is_ok());
    }
}
