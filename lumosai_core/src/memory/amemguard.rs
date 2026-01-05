//! A-MemGuard Security Framework - 基于A-MemGuard研究的主动防御框架
//!
//! A-MemGuard的核心思想：主动防御AI Agent记忆污染攻击
//! 实现记忆隔离、共识验证、异常检测和安全审计

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::error::{Error, Result};

/// A-MemGuard安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMemGuardConfig {
    /// 启用双重写入
    pub enable_dual_write: bool,

    /// 共识阈值 (0.0-1.0)
    pub consensus_threshold: f32,

    /// 异常检测灵敏度
    pub anomaly_sensitivity: f32,

    /// 隔离级别
    pub isolation_level: IsolationLevel,

    /// 审计日志级别
    pub audit_log_level: AuditLogLevel,

    /// 自动恢复策略
    pub auto_recovery_policy: AutoRecoveryPolicy,
}

/// 隔离级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// 轻量级隔离 - 仅基础验证
    Lightweight,

    /// 中等隔离 - 双重写入 + 共识验证
    Medium,

    /// 严格隔离 - 完整安全栈
    Strict,
}

/// 审计日志级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLogLevel {
    /// 仅记录安全事件
    SecurityOnly,

    /// 记录所有访问
    AllAccess,

    /// 详细调试信息
    Debug,
}

/// 自动恢复策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoRecoveryPolicy {
    /// 不自动恢复
    None,

    /// 仅隔离受污染记忆
    Isolate,

    /// 尝试修复并恢复
    RepairAndRecover,

    /// 完全重置受影响组件
    FullReset,
}

/// 安全审计事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditEvent {
    pub event_id: String,
    pub event_type: SecurityEventType,
    pub timestamp: DateTime<Utc>,
    pub severity: SecuritySeverity,
    pub description: String,
    pub actor: String,
    pub target: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub resolution: Option<String>,
}

/// 安全事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    /// 记忆污染尝试
    MemoryTampering,

    /// 共识验证失败
    ConsensusFailure,

    /// 异常行为检测
    AnomalyDetected,

    /// 隔离机制激活
    IsolationActivated,

    /// 自动恢复执行
    AutoRecovery,

    /// 安全策略违规
    PolicyViolation,
}

/// 安全严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// A-MemGuard安全trait
#[async_trait]
pub trait AMemGuardSecurity: Send + Sync {
    /// 安全存储记忆
    async fn secure_store(&self, content: &str, metadata: &HashMap<String, String>) -> Result<String>;

    /// 安全检索记忆
    async fn secure_retrieve(&self, query: &str) -> Result<Vec<SecureMemoryEntry>>;

    /// 验证记忆一致性
    async fn verify_consistency(&self) -> Result<ConsistencyReport>;

    /// 检测异常行为
    async fn detect_anomalies(&self) -> Result<Vec<AnomalyReport>>;

    /// 隔离受污染记忆
    async fn isolate_memory(&self, memory_ids: &[String]) -> Result<IsolationReport>;

    /// 获取安全统计
    async fn get_security_stats(&self) -> Result<SecurityStats>;
}

/// 安全记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureMemoryEntry {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub security_hash: String,
    pub consensus_score: f32,
    pub isolation_status: IsolationStatus,
    pub created_at: DateTime<Utc>,
    pub last_verified: DateTime<Utc>,
}

/// 隔离状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationStatus {
    /// 正常访问
    Normal,

    /// 隔离中
    Isolated,

    /// 已修复
    Repaired,

    /// 已销毁
    Destroyed,
}

/// 一致性报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyReport {
    pub total_memories: usize,
    pub consistent_count: usize,
    pub inconsistent_count: usize,
    pub consensus_score: f32,
    pub check_timestamp: DateTime<Utc>,
    pub inconsistencies: Vec<Inconsistency>,
}

/// 不一致项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inconsistency {
    pub memory_id: String,
    pub issue_type: InconsistencyType,
    pub severity: SecuritySeverity,
    pub description: String,
}

/// 不一致类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InconsistencyType {
    ContentMismatch,
    HashMismatch,
    ConsensusFailure,
    TamperingDetected,
}

/// 异常报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub anomaly_id: String,
    pub anomaly_type: AnomalyType,
    pub confidence: f32,
    pub affected_memories: Vec<String>,
    pub description: String,
    pub recommended_action: String,
}

/// 异常类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    UnusualAccessPattern,
    ContentDrift,
    ConsensusDeviation,
    IsolationBreach,
}

/// 隔离报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationReport {
    pub isolated_count: usize,
    pub failed_isolations: Vec<String>,
    pub isolation_timestamp: DateTime<Utc>,
}

/// 安全统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub total_memories: usize,
    pub isolated_memories: usize,
    pub anomalies_detected: usize,
    pub consensus_failures: usize,
    pub security_events: usize,
    pub overall_security_score: f32,
    pub last_security_check: DateTime<Utc>,
}

/// A-MemGuard安全实现
pub struct AMemGuard {
    config: AMemGuardConfig,
    primary_memory: Arc<dyn SecureMemoryBackend>,
    backup_memory: Arc<dyn SecureMemoryBackend>,
    consensus_verifier: Arc<ConsensusVerifier>,
    anomaly_detector: Arc<AnomalyDetector>,
    audit_logger: Arc<AuditLogger>,
    isolation_manager: Arc<IsolationManager>,
    stats: Arc<RwLock<SecurityStats>>,
}

/// 安全记忆后端trait
#[async_trait]
pub trait SecureMemoryBackend: Send + Sync {
    async fn store(&self, entry: &SecureMemoryEntry) -> Result<()>;
    async fn retrieve(&self, query: &str) -> Result<Vec<SecureMemoryEntry>>;
    async fn delete(&self, memory_id: &str) -> Result<()>;
    async fn get_hash(&self, memory_id: &str) -> Result<String>;
}

/// 共识验证器
pub struct ConsensusVerifier {
    threshold: f32,
}

impl ConsensusVerifier {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }

    pub async fn verify_entry(&self, primary: &SecureMemoryEntry, backup: &SecureMemoryEntry) -> Result<f32> {
        // 计算共识分数
        let mut score = 1.0;

        // 内容一致性检查
        if primary.content != backup.content {
            score *= 0.5;
        }

        // 哈希一致性检查
        if primary.security_hash != backup.security_hash {
            score *= 0.3;
        }

        // 元数据一致性检查
        if primary.metadata != backup.metadata {
            score *= 0.8;
        }

        Ok(score)
    }

    pub async fn verify_consensus(&self, entries: &[SecureMemoryEntry]) -> Result<f32> {
        if entries.is_empty() {
            return Ok(1.0);
        }

        let mut total_score = 0.0;
        let mut comparisons = 0;

        // 两两比较计算共识分数
        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                let score = self.verify_entry(&entries[i], &entries[j]).await?;
                total_score += score;
                comparisons += 1;
            }
        }

        if comparisons == 0 {
            Ok(1.0)
        } else {
            Ok(total_score / comparisons as f32)
        }
    }
}

/// 异常检测器
pub struct AnomalyDetector {
    sensitivity: f32,
    baseline_stats: Arc<RwLock<HashMap<String, f64>>>,
}

impl AnomalyDetector {
    pub fn new(sensitivity: f32) -> Self {
        Self {
            sensitivity,
            baseline_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn detect_anomalies(&self, recent_entries: &[SecureMemoryEntry]) -> Result<Vec<AnomalyReport>> {
        let mut reports = Vec::new();
        let baseline = self.baseline_stats.read().await;

        // 检测异常访问模式
        if let Some(anomaly) = self.detect_access_pattern_anomaly(recent_entries, &baseline).await? {
            reports.push(anomaly);
        }

        // 检测内容漂移
        if let Some(anomaly) = self.detect_content_drift(recent_entries).await? {
            reports.push(anomaly);
        }

        // 检测共识偏差
        if let Some(anomaly) = self.detect_consensus_anomaly(recent_entries).await? {
            reports.push(anomaly);
        }

        Ok(reports)
    }

    async fn detect_access_pattern_anomaly(
        &self,
        entries: &[SecureMemoryEntry],
        _baseline: &HashMap<String, f64>,
    ) -> Result<Option<AnomalyReport>> {
        // 简单的时间模式异常检测
        let timestamps: Vec<_> = entries.iter().map(|e| e.created_at).collect();

        // 计算时间间隔的标准差
        if timestamps.len() >= 3 {
            let intervals: Vec<f64> = timestamps.windows(2)
                .map(|w| (w[1] - w[0]).num_seconds() as f64)
                .collect();

            let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
            let variance = intervals.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / intervals.len() as f64;
            let std_dev = variance.sqrt();

            // 如果标准差过大，认为存在异常访问模式
            if std_dev > mean * self.sensitivity as f64 {
                return Ok(Some(AnomalyReport {
                    anomaly_id: format!("access_pattern_{}", uuid::Uuid::new_v4()),
                    anomaly_type: AnomalyType::UnusualAccessPattern,
                    confidence: 0.8,
                    affected_memories: entries.iter().map(|e| e.id.clone()).collect(),
                    description: format!("Unusual access pattern detected (std_dev: {:.2}, mean: {:.2})", std_dev, mean),
                    recommended_action: "Review access logs and consider rate limiting".to_string(),
                }));
            }
        }

        Ok(None)
    }

    async fn detect_content_drift(&self, _entries: &[SecureMemoryEntry]) -> Result<Option<AnomalyReport>> {
        // 内容漂移检测的简化实现
        // 在实际实现中，这里会使用更复杂的NLP技术
        Ok(None)
    }

    async fn detect_consensus_anomaly(&self, entries: &[SecureMemoryEntry]) -> Result<Option<AnomalyReport>> {
        let low_consensus_count = entries.iter()
            .filter(|e| e.consensus_score < self.sensitivity)
            .count();

        if low_consensus_count > entries.len() / 3 {
            return Ok(Some(AnomalyReport {
                anomaly_id: format!("consensus_{}", uuid::Uuid::new_v4()),
                anomaly_type: AnomalyType::ConsensusDeviation,
                confidence: 0.9,
                affected_memories: entries.iter().map(|e| e.id.clone()).collect(),
                description: format!("{} out of {} memories have low consensus scores", low_consensus_count, entries.len()),
                recommended_action: "Verify memory integrity and check for tampering".to_string(),
            }));
        }

        Ok(None)
    }
}

/// 审计日志记录器
pub struct AuditLogger {
    events: Arc<RwLock<Vec<SecurityAuditEvent>>>,
    log_level: AuditLogLevel,
}

impl AuditLogger {
    pub fn new(log_level: AuditLogLevel) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            log_level,
        }
    }

    pub async fn log_event(&self, event: SecurityAuditEvent) -> Result<()> {
        let should_log = match (&self.log_level, &event.severity) {
            (AuditLogLevel::SecurityOnly, SecuritySeverity::Low) => false,
            _ => true,
        };

        if should_log {
            let mut events = self.events.write().await;
            events.push(event);
        }

        Ok(())
    }

    pub async fn get_events(&self, since: Option<DateTime<Utc>>) -> Result<Vec<SecurityAuditEvent>> {
        let events = self.events.read().await;
        let filtered: Vec<_> = if let Some(timestamp) = since {
            events.iter().filter(|e| e.timestamp >= timestamp).cloned().collect()
        } else {
            events.clone()
        };

        Ok(filtered)
    }
}

/// 隔离管理器
pub struct IsolationManager {
    isolated_memories: Arc<RwLock<HashSet<String>>>,
}

impl IsolationManager {
    pub fn new() -> Self {
        Self {
            isolated_memories: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn isolate_memory(&self, memory_ids: &[String]) -> Result<()> {
        let mut isolated = self.isolated_memories.write().await;
        for id in memory_ids {
            isolated.insert(id.clone());
        }
        Ok(())
    }

    pub async fn is_isolated(&self, memory_id: &str) -> Result<bool> {
        let isolated = self.isolated_memories.read().await;
        Ok(isolated.contains(memory_id))
    }

    pub async fn release_isolation(&self, memory_id: &str) -> Result<()> {
        let mut isolated = self.isolated_memories.write().await;
        isolated.remove(memory_id);
        Ok(())
    }

    pub async fn get_isolated_count(&self) -> Result<usize> {
        let isolated = self.isolated_memories.read().await;
        Ok(isolated.len())
    }
}

impl AMemGuard {
    pub fn new(
        config: AMemGuardConfig,
        primary_memory: Arc<dyn SecureMemoryBackend>,
        backup_memory: Arc<dyn SecureMemoryBackend>,
    ) -> Self {
        Self {
            config: config.clone(),
            primary_memory,
            backup_memory,
            consensus_verifier: Arc::new(ConsensusVerifier::new(config.consensus_threshold)),
            anomaly_detector: Arc::new(AnomalyDetector::new(config.anomaly_sensitivity)),
            audit_logger: Arc::new(AuditLogger::new(config.audit_log_level)),
            isolation_manager: Arc::new(IsolationManager::new()),
            stats: Arc::new(RwLock::new(SecurityStats {
                total_memories: 0,
                isolated_memories: 0,
                anomalies_detected: 0,
                consensus_failures: 0,
                security_events: 0,
                overall_security_score: 1.0,
                last_security_check: Utc::now(),
            })),
        }
    }
}

#[async_trait]
impl AMemGuardSecurity for AMemGuard {
    async fn secure_store(&self, content: &str, metadata: &HashMap<String, String>) -> Result<String> {
        let memory_id = format!("mem_secure_{}", uuid::Uuid::new_v4());

        // 计算安全哈希
        let security_hash = self.calculate_security_hash(content, metadata);

        // 创建安全记忆条目
        let entry = SecureMemoryEntry {
            id: memory_id.clone(),
            content: content.to_string(),
            metadata: metadata.clone(),
            security_hash,
            consensus_score: 1.0,
            isolation_status: IsolationStatus::Normal,
            created_at: Utc::now(),
            last_verified: Utc::now(),
        };

        // 双重写入（如果启用）
        if self.config.enable_dual_write {
            self.primary_memory.store(&entry).await?;
            self.backup_memory.store(&entry).await?;
        } else {
            self.primary_memory.store(&entry).await?;
        }

        // 记录审计事件
        self.audit_logger.log_event(SecurityAuditEvent {
            event_id: format!("store_{}", uuid::Uuid::new_v4()),
            event_type: SecurityEventType::PolicyViolation, // 应该根据具体情况判断
            timestamp: Utc::now(),
            severity: SecuritySeverity::Low,
            description: format!("Secure memory stored: {}", memory_id),
            actor: "system".to_string(),
            target: memory_id.clone(),
            metadata: HashMap::new(),
            resolution: None,
        }).await?;

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.total_memories += 1;
        stats.last_security_check = Utc::now();

        Ok(memory_id)
    }

    async fn secure_retrieve(&self, query: &str) -> Result<Vec<SecureMemoryEntry>> {
        // 从主存储检索
        let mut results = self.primary_memory.retrieve(query).await?;

        // 检查隔离状态
        results.retain(|entry| {
            // 在实际实现中，这里会异步检查隔离状态
            matches!(entry.isolation_status, IsolationStatus::Normal)
        });

        // 记录审计事件
        self.audit_logger.log_event(SecurityAuditEvent {
            event_id: format!("retrieve_{}", uuid::Uuid::new_v4()),
            event_type: SecurityEventType::PolicyViolation,
            timestamp: Utc::now(),
            severity: SecuritySeverity::Low,
            description: format!("Secure memory retrieved: {} results", results.len()),
            actor: "system".to_string(),
            target: query.to_string(),
            metadata: HashMap::new(),
            resolution: None,
        }).await?;

        Ok(results)
    }

    async fn verify_consistency(&self) -> Result<ConsistencyReport> {
        // 获取所有记忆条目
        let primary_entries = self.primary_memory.retrieve("").await?;
        let backup_entries = if self.config.enable_dual_write {
            self.backup_memory.retrieve("").await?
        } else {
            vec![]
        };

        let mut inconsistencies = Vec::new();

        // 比较主备存储的一致性
        for primary in &primary_entries {
            if let Some(backup) = backup_entries.iter().find(|b| b.id == primary.id) {
                let consensus_score = self.consensus_verifier.verify_entry(primary, backup).await?;

                if consensus_score < self.config.consensus_threshold {
                    inconsistencies.push(Inconsistency {
                        memory_id: primary.id.clone(),
                        issue_type: InconsistencyType::ConsensusFailure,
                        severity: SecuritySeverity::High,
                        description: format!("Consensus score too low: {:.2}", consensus_score),
                    });
                }
            } else if self.config.enable_dual_write {
                inconsistencies.push(Inconsistency {
                    memory_id: primary.id.clone(),
                    issue_type: InconsistencyType::ContentMismatch,
                    severity: SecuritySeverity::Critical,
                    description: "Memory missing in backup storage".to_string(),
                });
            }
        }

        let consistent_count = primary_entries.len() - inconsistencies.len();
        let consensus_score = if primary_entries.is_empty() {
            1.0
        } else {
            consistent_count as f32 / primary_entries.len() as f32
        };

        let inconsistencies_count = inconsistencies.len();

        let report = ConsistencyReport {
            total_memories: primary_entries.len(),
            consistent_count,
            inconsistent_count: inconsistencies_count,
            consensus_score,
            check_timestamp: Utc::now(),
            inconsistencies,
        };

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.consensus_failures = inconsistencies_count;
        stats.overall_security_score = consensus_score;
        stats.last_security_check = Utc::now();

        Ok(report)
    }

    async fn detect_anomalies(&self) -> Result<Vec<AnomalyReport>> {
        // 获取最近的记忆条目
        let recent_entries = self.primary_memory.retrieve("").await?;

        // 限制为最近1000个条目以提高性能
        let recent_entries = recent_entries.into_iter()
            .rev()
            .take(1000)
            .collect::<Vec<_>>();

        let anomalies = self.anomaly_detector.detect_anomalies(&recent_entries).await?;

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.anomalies_detected = anomalies.len();

        Ok(anomalies)
    }

    async fn isolate_memory(&self, memory_ids: &[String]) -> Result<IsolationReport> {
        // 隔离记忆条目
        self.isolation_manager.isolate_memory(memory_ids).await?;

        // 记录审计事件
        for memory_id in memory_ids {
            self.audit_logger.log_event(SecurityAuditEvent {
                event_id: format!("isolate_{}", uuid::Uuid::new_v4()),
                event_type: SecurityEventType::IsolationActivated,
                timestamp: Utc::now(),
                severity: SecuritySeverity::Medium,
                description: format!("Memory isolated: {}", memory_id),
                actor: "system".to_string(),
                target: memory_id.clone(),
                metadata: HashMap::new(),
                resolution: Some("Memory isolated for security reasons".to_string()),
            }).await?;
        }

        let report = IsolationReport {
            isolated_count: memory_ids.len(),
            failed_isolations: vec![], // 简化实现
            isolation_timestamp: Utc::now(),
        };

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.isolated_memories += memory_ids.len();

        Ok(report)
    }

    async fn get_security_stats(&self) -> Result<SecurityStats> {
        let mut stats = self.stats.read().await.clone();

        // 获取最新的隔离计数
        stats.isolated_memories = self.isolation_manager.get_isolated_count().await?;

        // 获取安全事件计数
        let events = self.audit_logger.get_events(None).await?;
        stats.security_events = events.len();

        Ok(stats)
    }
}

impl AMemGuard {
    /// 计算安全哈希
    fn calculate_security_hash(&self, content: &str, metadata: &HashMap<String, String>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);

        // 手动哈希metadata
        for (key, value) in metadata.iter() {
            key.hash(&mut hasher);
            value.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }
}

/// 内存安全后端实现
pub struct InMemorySecureBackend {
    memories: Arc<RwLock<HashMap<String, SecureMemoryEntry>>>,
}

impl InMemorySecureBackend {
    pub fn new() -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SecureMemoryBackend for InMemorySecureBackend {
    async fn store(&self, entry: &SecureMemoryEntry) -> Result<()> {
        let mut memories = self.memories.write().await;
        memories.insert(entry.id.clone(), entry.clone());
        Ok(())
    }

    async fn retrieve(&self, _query: &str) -> Result<Vec<SecureMemoryEntry>> {
        let memories = self.memories.read().await;
        // 简化实现：返回所有记忆
        Ok(memories.values().cloned().collect())
    }

    async fn delete(&self, memory_id: &str) -> Result<()> {
        let mut memories = self.memories.write().await;
        memories.remove(memory_id);
        Ok(())
    }

    async fn get_hash(&self, memory_id: &str) -> Result<String> {
        let memories = self.memories.read().await;
        memories.get(memory_id)
            .map(|entry| entry.security_hash.clone())
            .ok_or_else(|| Error::NotFound(format!("Memory not found: {}", memory_id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_hash_calculation() {
        let backend = InMemorySecureBackend::new();
        let amemguard = AMemGuard::new(
            AMemGuardConfig {
                enable_dual_write: false,
                consensus_threshold: 0.8,
                anomaly_sensitivity: 0.7,
                isolation_level: IsolationLevel::Medium,
                audit_log_level: AuditLogLevel::AllAccess,
                auto_recovery_policy: AutoRecoveryPolicy::Isolate,
            },
            Arc::new(backend),
            Arc::new(InMemorySecureBackend::new()),
        );

        let content = "test content";
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());

        let hash1 = amemguard.calculate_security_hash(content, &metadata);
        let hash2 = amemguard.calculate_security_hash(content, &metadata);

        // 相同的输入应该产生相同的哈希
        assert_eq!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_secure_memory_operations() -> Result<()> {
        let primary = Arc::new(InMemorySecureBackend::new());
        let backup = Arc::new(InMemorySecureBackend::new());

        let amemguard = AMemGuard::new(
            AMemGuardConfig {
                enable_dual_write: true,
                consensus_threshold: 0.8,
                anomaly_sensitivity: 0.7,
                isolation_level: IsolationLevel::Medium,
                audit_log_level: AuditLogLevel::AllAccess,
                auto_recovery_policy: AutoRecoveryPolicy::Isolate,
            },
            primary,
            backup,
        );

        // 测试安全存储
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), "test".to_string());

        let memory_id = amemguard.secure_store("Test memory content", &metadata).await?;
        assert!(!memory_id.is_empty());

        // 测试安全检索
        let results = amemguard.secure_retrieve("").await?;
        assert!(!results.is_empty());

        let entry = results.iter().find(|e| e.id == memory_id).unwrap();
        assert_eq!(entry.content, "Test memory content");
        assert_eq!(entry.metadata, metadata);

        // 测试一致性验证
        let report = amemguard.verify_consistency().await?;
        assert_eq!(report.total_memories, 1);
        assert_eq!(report.consistent_count, 1);
        assert!(report.consensus_score >= 0.8);

        Ok(())
    }

    #[tokio::test]
    async fn test_isolation_mechanism() -> Result<()> {
        let primary = Arc::new(InMemorySecureBackend::new());
        let backup = Arc::new(InMemorySecureBackend::new());

        let amemguard = AMemGuard::new(
            AMemGuardConfig {
                enable_dual_write: false,
                consensus_threshold: 0.8,
                anomaly_sensitivity: 0.7,
                isolation_level: IsolationLevel::Strict,
                audit_log_level: AuditLogLevel::SecurityOnly,
                auto_recovery_policy: AutoRecoveryPolicy::Isolate,
            },
            primary,
            backup,
        );

        // 存储记忆
        let memory_id = amemguard.secure_store("Test content", &HashMap::new()).await?;

        // 隔离记忆
        let isolation_report = amemguard.isolate_memory(&[memory_id.clone()]).await?;
        assert_eq!(isolation_report.isolated_count, 1);

        // 验证统计更新
        let stats = amemguard.get_security_stats().await?;
        assert_eq!(stats.isolated_memories, 1);

        Ok(())
    }
}
