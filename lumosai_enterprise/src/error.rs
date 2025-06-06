//! 企业级功能错误类型定义

use thiserror::Error;

/// 企业级功能错误类型
#[derive(Error, Debug)]
pub enum EnterpriseError {
    /// 数据库错误
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    
    /// Redis错误
    #[error("Redis错误: {0}")]
    Redis(#[from] redis::RedisError),
    
    /// HTTP错误
    #[error("HTTP错误: {0}")]
    Http(#[from] reqwest::Error),
    
    /// JSON序列化错误
    #[error("JSON序列化错误: {0}")]
    Json(#[from] serde_json::Error),
    
    /// IO错误
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    
    /// JWT错误
    #[error("JWT错误: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),
    
    /// 监控错误
    #[error("监控错误: {0}")]
    Monitoring(String),
    
    /// 安全错误
    #[error("安全错误: {0}")]
    Security(String),
    
    /// 合规错误
    #[error("合规错误: {0}")]
    Compliance(String),
    
    /// 多租户错误
    #[error("多租户错误: {0}")]
    MultiTenant(String),
    
    /// 成本跟踪错误
    #[error("成本跟踪错误: {0}")]
    CostTracking(String),
    
    /// SLA监控错误
    #[error("SLA监控错误: {0}")]
    SlaMonitoring(String),
    
    /// 事件管理错误
    #[error("事件管理错误: {0}")]
    IncidentManagement(String),
    
    /// 容量规划错误
    #[error("容量规划错误: {0}")]
    CapacityPlanning(String),
    
    /// 异常检测错误
    #[error("异常检测错误: {0}")]
    AnomalyDetection(String),
    
    /// 告警错误
    #[error("告警错误: {0}")]
    Alerting(String),
    
    /// 报告生成错误
    #[error("报告生成错误: {0}")]
    Reporting(String),
    
    /// 认证错误
    #[error("认证错误: {0}")]
    Authentication(String),
    
    /// 授权错误
    #[error("授权错误: {0}")]
    Authorization(String),
    
    /// 加密错误
    #[error("加密错误: {0}")]
    Encryption(String),
    
    /// 审计错误
    #[error("审计错误: {0}")]
    Audit(String),
    
    /// 威胁检测错误
    #[error("威胁检测错误: {0}")]
    ThreatDetection(String),
    
    /// 租户不存在
    #[error("租户不存在: {0}")]
    TenantNotFound(String),
    
    /// 资源不足
    #[error("资源不足: {0}")]
    InsufficientResources(String),
    
    /// 配额超限
    #[error("配额超限: {0}")]
    QuotaExceeded(String),
    
    /// 权限不足
    #[error("权限不足: {0}")]
    PermissionDenied(String),
    
    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
    
    /// 网络错误
    #[error("网络错误: {0}")]
    Network(String),
    
    /// 超时错误
    #[error("操作超时")]
    Timeout,
    
    /// 不支持的操作
    #[error("不支持的操作: {0}")]
    Unsupported(String),
    
    /// 验证错误
    #[error("验证错误: {0}")]
    Validation(String),
}

/// 企业级功能结果类型
pub type Result<T> = std::result::Result<T, EnterpriseError>;

impl From<&str> for EnterpriseError {
    fn from(msg: &str) -> Self {
        EnterpriseError::Internal(msg.to_string())
    }
}

impl From<String> for EnterpriseError {
    fn from(msg: String) -> Self {
        EnterpriseError::Internal(msg)
    }
}

/// 错误转换辅助函数
impl EnterpriseError {
    /// 创建配置错误
    pub fn config(msg: impl Into<String>) -> Self {
        EnterpriseError::Config(msg.into())
    }
    
    /// 创建监控错误
    pub fn monitoring(msg: impl Into<String>) -> Self {
        EnterpriseError::Monitoring(msg.into())
    }
    
    /// 创建安全错误
    pub fn security(msg: impl Into<String>) -> Self {
        EnterpriseError::Security(msg.into())
    }
    
    /// 创建合规错误
    pub fn compliance(msg: impl Into<String>) -> Self {
        EnterpriseError::Compliance(msg.into())
    }
    
    /// 创建多租户错误
    pub fn multi_tenant(msg: impl Into<String>) -> Self {
        EnterpriseError::MultiTenant(msg.into())
    }
    
    /// 创建认证错误
    pub fn authentication(msg: impl Into<String>) -> Self {
        EnterpriseError::Authentication(msg.into())
    }
    
    /// 创建授权错误
    pub fn authorization(msg: impl Into<String>) -> Self {
        EnterpriseError::Authorization(msg.into())
    }
    
    /// 创建权限错误
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        EnterpriseError::PermissionDenied(msg.into())
    }
    
    /// 创建验证错误
    pub fn validation(msg: impl Into<String>) -> Self {
        EnterpriseError::Validation(msg.into())
    }
    
    /// 检查是否为临时错误（可重试）
    pub fn is_temporary(&self) -> bool {
        matches!(
            self,
            EnterpriseError::Network(_) |
            EnterpriseError::Timeout |
            EnterpriseError::Http(_) |
            EnterpriseError::Redis(_) |
            EnterpriseError::Database(_)
        )
    }
    
    /// 检查是否为致命错误（不可重试）
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            EnterpriseError::Config(_) |
            EnterpriseError::Authentication(_) |
            EnterpriseError::Authorization(_) |
            EnterpriseError::PermissionDenied(_) |
            EnterpriseError::TenantNotFound(_) |
            EnterpriseError::Validation(_)
        )
    }
    
    /// 检查是否为安全相关错误
    pub fn is_security_related(&self) -> bool {
        matches!(
            self,
            EnterpriseError::Security(_) |
            EnterpriseError::Authentication(_) |
            EnterpriseError::Authorization(_) |
            EnterpriseError::PermissionDenied(_) |
            EnterpriseError::Encryption(_) |
            EnterpriseError::ThreatDetection(_)
        )
    }
    
    /// 检查是否为资源相关错误
    pub fn is_resource_related(&self) -> bool {
        matches!(
            self,
            EnterpriseError::InsufficientResources(_) |
            EnterpriseError::QuotaExceeded(_) |
            EnterpriseError::CapacityPlanning(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = EnterpriseError::config("测试配置错误");
        assert!(matches!(err, EnterpriseError::Config(_)));
        assert_eq!(err.to_string(), "配置错误: 测试配置错误");
    }
    
    #[test]
    fn test_error_classification() {
        let network_err = EnterpriseError::Network("连接失败".to_string());
        assert!(network_err.is_temporary());
        assert!(!network_err.is_fatal());
        assert!(!network_err.is_security_related());
        
        let auth_err = EnterpriseError::authentication("认证失败");
        assert!(!auth_err.is_temporary());
        assert!(auth_err.is_fatal());
        assert!(auth_err.is_security_related());
        
        let quota_err = EnterpriseError::QuotaExceeded("配额超限".to_string());
        assert!(!quota_err.is_temporary());
        assert!(!quota_err.is_security_related());
        assert!(quota_err.is_resource_related());
    }
    
    #[test]
    fn test_error_conversion() {
        let err: EnterpriseError = "测试错误".into();
        assert!(matches!(err, EnterpriseError::Internal(_)));
        
        let err: EnterpriseError = "测试错误".to_string().into();
        assert!(matches!(err, EnterpriseError::Internal(_)));
    }
}
