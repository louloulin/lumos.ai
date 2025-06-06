//! 企业级功能配置管理

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use crate::error::{EnterpriseError, Result};

/// 企业级功能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// 数据库连接URL
    pub database_url: String,
    
    /// Redis连接URL（可选）
    pub redis_url: Option<String>,
    
    /// 是否启用合规监控
    pub compliance_monitoring_enabled: bool,
    
    /// 是否启用安全审计
    pub security_auditing_enabled: bool,
    
    /// 是否启用成本跟踪
    pub cost_tracking_enabled: bool,
    
    /// 是否启用SLA监控
    pub sla_monitoring_enabled: bool,
    
    /// 是否启用异常检测
    pub anomaly_detection_enabled: bool,
    
    /// 是否启用多租户
    pub multi_tenant_enabled: bool,
    
    /// 监控配置
    pub monitoring: MonitoringConfig,
    
    /// 安全配置
    pub security: SecurityConfig,
    
    /// 合规配置
    pub compliance: ComplianceConfig,
    
    /// 告警配置
    pub alerting: AlertingConfig,
    
    /// 报告配置
    pub reporting: ReportingConfig,
    
    /// Prometheus端点
    pub prometheus_endpoint: Option<String>,
    
    /// Jaeger端点
    pub jaeger_endpoint: Option<String>,
    
    /// 数据保留策略
    pub data_retention: DataRetentionConfig,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 指标收集间隔（秒）
    pub metrics_collection_interval_seconds: u64,
    
    /// 实时监控更新间隔（秒）
    pub realtime_update_interval_seconds: u64,
    
    /// 性能监控阈值
    pub performance_thresholds: PerformanceThresholds,
    
    /// 是否启用详细追踪
    pub detailed_tracing_enabled: bool,
    
    /// 追踪采样率
    pub trace_sampling_rate: f64,
    
    /// 指标聚合窗口大小
    pub metrics_aggregation_window_minutes: u64,
}

/// 性能阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// 响应时间阈值（毫秒）
    pub response_time_ms: u64,
    
    /// CPU使用率阈值（百分比）
    pub cpu_usage_percent: f64,
    
    /// 内存使用率阈值（百分比）
    pub memory_usage_percent: f64,
    
    /// 错误率阈值（百分比）
    pub error_rate_percent: f64,
    
    /// 吞吐量阈值（请求/秒）
    pub throughput_rps: f64,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// JWT密钥
    pub jwt_secret: String,
    
    /// JWT过期时间（秒）
    pub jwt_expiration_seconds: u64,
    
    /// 是否启用双因素认证
    pub two_factor_auth_enabled: bool,
    
    /// 密码策略
    pub password_policy: PasswordPolicy,
    
    /// 会话超时时间（秒）
    pub session_timeout_seconds: u64,
    
    /// 是否启用IP白名单
    pub ip_whitelist_enabled: bool,
    
    /// IP白名单
    pub ip_whitelist: Vec<String>,
    
    /// 威胁检测配置
    pub threat_detection: ThreatDetectionConfig,
}

/// 密码策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// 最小长度
    pub min_length: usize,
    
    /// 是否需要大写字母
    pub require_uppercase: bool,
    
    /// 是否需要小写字母
    pub require_lowercase: bool,
    
    /// 是否需要数字
    pub require_numbers: bool,
    
    /// 是否需要特殊字符
    pub require_special_chars: bool,
    
    /// 密码历史记录数量
    pub password_history_count: usize,
}

/// 威胁检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetectionConfig {
    /// 是否启用异常登录检测
    pub anomalous_login_detection: bool,
    
    /// 是否启用暴力破解检测
    pub brute_force_detection: bool,
    
    /// 是否启用SQL注入检测
    pub sql_injection_detection: bool,
    
    /// 是否启用XSS检测
    pub xss_detection: bool,
    
    /// 威胁检测敏感度（0.0-1.0）
    pub detection_sensitivity: f64,
}

/// 合规配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// 启用的合规标准
    pub enabled_standards: Vec<ComplianceStandard>,
    
    /// 审计日志保留天数
    pub audit_log_retention_days: u32,
    
    /// 是否启用数据分类
    pub data_classification_enabled: bool,
    
    /// 是否启用数据脱敏
    pub data_masking_enabled: bool,
    
    /// 合规检查间隔（小时）
    pub compliance_check_interval_hours: u64,
    
    /// 合规报告生成间隔（天）
    pub compliance_report_interval_days: u32,
}

/// 合规标准
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComplianceStandard {
    /// SOC 2
    SOC2,
    /// GDPR
    GDPR,
    /// HIPAA
    HIPAA,
    /// PCI DSS
    PCIDSS,
    /// ISO 27001
    ISO27001,
    /// 自定义标准
    Custom(String),
}

/// 告警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    /// 是否启用邮件告警
    pub email_alerts_enabled: bool,
    
    /// 邮件服务器配置
    pub email_config: Option<EmailConfig>,
    
    /// 是否启用Slack告警
    pub slack_alerts_enabled: bool,
    
    /// Slack Webhook URL
    pub slack_webhook_url: Option<String>,
    
    /// 是否启用短信告警
    pub sms_alerts_enabled: bool,
    
    /// 短信服务配置
    pub sms_config: Option<SmsConfig>,
    
    /// 告警聚合窗口（分钟）
    pub alert_aggregation_window_minutes: u64,
    
    /// 告警升级策略
    pub escalation_policies: Vec<EscalationPolicy>,
}

/// 邮件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP服务器
    pub smtp_server: String,
    
    /// SMTP端口
    pub smtp_port: u16,
    
    /// 用户名
    pub username: String,
    
    /// 密码
    pub password: String,
    
    /// 发件人邮箱
    pub from_email: String,
    
    /// 是否使用TLS
    pub use_tls: bool,
}

/// 短信配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    /// 服务提供商
    pub provider: String,
    
    /// API密钥
    pub api_key: String,
    
    /// API密钥ID
    pub api_secret: String,
    
    /// 发送号码
    pub from_number: String,
}

/// 告警升级策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    /// 策略名称
    pub name: String,
    
    /// 升级步骤
    pub steps: Vec<EscalationStep>,
}

/// 升级步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationStep {
    /// 延迟时间（分钟）
    pub delay_minutes: u64,
    
    /// 通知渠道
    pub notification_channels: Vec<String>,
    
    /// 接收人
    pub recipients: Vec<String>,
}

/// 报告配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    /// 报告生成间隔（天）
    pub report_generation_interval_days: u32,
    
    /// 报告存储路径
    pub report_storage_path: String,
    
    /// 是否启用自动报告分发
    pub auto_distribution_enabled: bool,
    
    /// 报告分发列表
    pub distribution_list: Vec<String>,
    
    /// 报告格式
    pub report_formats: Vec<ReportFormat>,
    
    /// 报告模板
    pub report_templates: HashMap<String, String>,
}

/// 报告格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    /// PDF
    PDF,
    /// HTML
    HTML,
    /// JSON
    JSON,
    /// CSV
    CSV,
    /// Excel
    Excel,
}

/// 数据保留配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionConfig {
    /// 指标数据保留天数
    pub metrics_retention_days: u32,
    
    /// 日志数据保留天数
    pub logs_retention_days: u32,
    
    /// 追踪数据保留天数
    pub traces_retention_days: u32,
    
    /// 审计数据保留天数
    pub audit_retention_days: u32,
    
    /// 是否启用数据压缩
    pub data_compression_enabled: bool,
    
    /// 是否启用数据归档
    pub data_archiving_enabled: bool,
    
    /// 归档存储路径
    pub archive_storage_path: Option<String>,
}

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite://enterprise.db".to_string(),
            redis_url: None,
            compliance_monitoring_enabled: true,
            security_auditing_enabled: true,
            cost_tracking_enabled: true,
            sla_monitoring_enabled: true,
            anomaly_detection_enabled: true,
            multi_tenant_enabled: false,
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
            compliance: ComplianceConfig::default(),
            alerting: AlertingConfig::default(),
            reporting: ReportingConfig::default(),
            prometheus_endpoint: None,
            jaeger_endpoint: None,
            data_retention: DataRetentionConfig::default(),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_collection_interval_seconds: 30,
            realtime_update_interval_seconds: 5,
            performance_thresholds: PerformanceThresholds::default(),
            detailed_tracing_enabled: true,
            trace_sampling_rate: 0.1,
            metrics_aggregation_window_minutes: 5,
        }
    }
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            response_time_ms: 1000,
            cpu_usage_percent: 80.0,
            memory_usage_percent: 85.0,
            error_rate_percent: 5.0,
            throughput_rps: 100.0,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-secret-key-change-in-production".to_string(),
            jwt_expiration_seconds: 3600, // 1小时
            two_factor_auth_enabled: false,
            password_policy: PasswordPolicy::default(),
            session_timeout_seconds: 1800, // 30分钟
            ip_whitelist_enabled: false,
            ip_whitelist: Vec::new(),
            threat_detection: ThreatDetectionConfig::default(),
        }
    }
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            password_history_count: 5,
        }
    }
}

impl Default for ThreatDetectionConfig {
    fn default() -> Self {
        Self {
            anomalous_login_detection: true,
            brute_force_detection: true,
            sql_injection_detection: true,
            xss_detection: true,
            detection_sensitivity: 0.7,
        }
    }
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            enabled_standards: vec![ComplianceStandard::SOC2],
            audit_log_retention_days: 365,
            data_classification_enabled: true,
            data_masking_enabled: true,
            compliance_check_interval_hours: 24,
            compliance_report_interval_days: 30,
        }
    }
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            email_alerts_enabled: false,
            email_config: None,
            slack_alerts_enabled: false,
            slack_webhook_url: None,
            sms_alerts_enabled: false,
            sms_config: None,
            alert_aggregation_window_minutes: 5,
            escalation_policies: Vec::new(),
        }
    }
}

impl Default for ReportingConfig {
    fn default() -> Self {
        Self {
            report_generation_interval_days: 7,
            report_storage_path: "./reports".to_string(),
            auto_distribution_enabled: false,
            distribution_list: Vec::new(),
            report_formats: vec![ReportFormat::PDF, ReportFormat::HTML],
            report_templates: HashMap::new(),
        }
    }
}

impl Default for DataRetentionConfig {
    fn default() -> Self {
        Self {
            metrics_retention_days: 90,
            logs_retention_days: 30,
            traces_retention_days: 7,
            audit_retention_days: 365,
            data_compression_enabled: true,
            data_archiving_enabled: false,
            archive_storage_path: None,
        }
    }
}

impl EnterpriseConfig {
    /// 从文件加载配置
    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)
            .map_err(|e| EnterpriseError::config(format!("无法读取配置文件 {:?}: {}", path, e)))?;
        
        let config: Self = toml::from_str(&content)
            .map_err(|e| EnterpriseError::config(format!("配置文件格式错误: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// 保存配置到文件
    pub fn save_to_file(&self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();
        let content = toml::to_string_pretty(self)
            .map_err(|e| EnterpriseError::config(format!("序列化配置失败: {}", e)))?;
        
        std::fs::write(&path, content)
            .map_err(|e| EnterpriseError::config(format!("写入配置文件失败 {:?}: {}", path, e)))?;
        
        Ok(())
    }
    
    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        // 验证数据库URL
        if self.database_url.is_empty() {
            return Err(EnterpriseError::config("数据库URL不能为空"));
        }
        
        // 验证JWT密钥
        if self.security.jwt_secret.len() < 32 {
            return Err(EnterpriseError::config("JWT密钥长度至少32个字符"));
        }
        
        // 验证性能阈值
        let thresholds = &self.monitoring.performance_thresholds;
        if thresholds.cpu_usage_percent < 0.0 || thresholds.cpu_usage_percent > 100.0 {
            return Err(EnterpriseError::config("CPU使用率阈值必须在0-100之间"));
        }
        
        if thresholds.memory_usage_percent < 0.0 || thresholds.memory_usage_percent > 100.0 {
            return Err(EnterpriseError::config("内存使用率阈值必须在0-100之间"));
        }
        
        // 验证追踪采样率
        if self.monitoring.trace_sampling_rate < 0.0 || self.monitoring.trace_sampling_rate > 1.0 {
            return Err(EnterpriseError::config("追踪采样率必须在0.0-1.0之间"));
        }
        
        // 验证威胁检测敏感度
        let sensitivity = self.security.threat_detection.detection_sensitivity;
        if sensitivity < 0.0 || sensitivity > 1.0 {
            return Err(EnterpriseError::config("威胁检测敏感度必须在0.0-1.0之间"));
        }
        
        Ok(())
    }
    
    /// 获取完整的数据库URL
    pub fn get_database_url(&self) -> &str {
        &self.database_url
    }
    
    /// 获取Redis URL
    pub fn get_redis_url(&self) -> Option<&str> {
        self.redis_url.as_deref()
    }
    
    /// 检查是否启用Redis
    pub fn is_redis_enabled(&self) -> bool {
        self.redis_url.is_some()
    }
    
    /// 检查是否启用Prometheus
    pub fn is_prometheus_enabled(&self) -> bool {
        self.prometheus_endpoint.is_some()
    }
    
    /// 检查是否启用Jaeger
    pub fn is_jaeger_enabled(&self) -> bool {
        self.jaeger_endpoint.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = EnterpriseConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.compliance_monitoring_enabled);
        assert!(config.security_auditing_enabled);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = EnterpriseConfig::default();
        
        // 测试无效的数据库URL
        config.database_url = "".to_string();
        assert!(config.validate().is_err());
        
        // 测试无效的JWT密钥
        config.database_url = "sqlite://test.db".to_string();
        config.security.jwt_secret = "short".to_string();
        assert!(config.validate().is_err());
        
        // 测试无效的CPU阈值
        config.security.jwt_secret = "a-very-long-secret-key-for-testing-purposes".to_string();
        config.monitoring.performance_thresholds.cpu_usage_percent = 150.0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_file_operations() {
        let config = EnterpriseConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        // 保存配置
        assert!(config.save_to_file(temp_file.path()).is_ok());
        
        // 加载配置
        let loaded_config = EnterpriseConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.database_url, loaded_config.database_url);
        assert_eq!(config.compliance_monitoring_enabled, loaded_config.compliance_monitoring_enabled);
    }
}
