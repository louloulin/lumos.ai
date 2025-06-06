//! 云原生部署错误处理
//! 
//! 提供统一的错误类型和处理机制

use std::fmt;
use thiserror::Error;

/// 云原生部署错误类型
#[derive(Error, Debug)]
pub enum CloudError {
    /// Kubernetes相关错误
    #[error("Kubernetes error: {0}")]
    KubernetesConnection(String),
    
    #[error("Kubernetes deployment error: {0}")]
    KubernetesDeployment(String),
    
    #[error("Kubernetes not configured")]
    KubernetesNotConfigured,
    
    /// Docker相关错误
    #[error("Docker error: {0}")]
    DockerConnection(String),
    
    #[error("Docker deployment error: {0}")]
    DockerDeployment(String),
    
    #[error("Docker not configured")]
    DockerNotConfigured,
    
    /// 云提供商错误
    #[error("Cloud provider '{0}' not configured")]
    CloudProviderNotConfigured(String),
    
    #[error("AWS error: {0}")]
    AwsError(String),
    
    #[error("Azure error: {0}")]
    AzureError(String),
    
    #[error("GCP error: {0}")]
    GcpError(String),
    
    /// 配置错误
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Invalid deployment config: {field}: {message}")]
    InvalidDeploymentConfig { field: String, message: String },
    
    /// 网络错误
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Service discovery error: {0}")]
    ServiceDiscovery(String),
    
    #[error("Load balancer error: {0}")]
    LoadBalancer(String),
    
    /// 存储错误
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Volume mount error: {0}")]
    VolumeMount(String),
    
    /// 安全错误
    #[error("Security error: {0}")]
    Security(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    /// 监控错误
    #[error("Monitoring error: {0}")]
    Monitoring(String),
    
    #[error("Metrics collection error: {0}")]
    MetricsCollection(String),
    
    /// 自动扩容错误
    #[error("Autoscaling error: {0}")]
    Autoscaling(String),
    
    #[error("Resource scaling error: {0}")]
    ResourceScaling(String),
    
    /// 资源错误
    #[error("Resource not found: {resource_type} '{name}'")]
    ResourceNotFound { resource_type: String, name: String },
    
    #[error("Resource conflict: {0}")]
    ResourceConflict(String),
    
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),
    
    /// 部署错误
    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),
    
    #[error("Deployment timeout: operation timed out after {timeout_seconds}s")]
    DeploymentTimeout { timeout_seconds: u64 },
    
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
    
    /// 健康检查错误
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
    
    #[error("Readiness probe failed: {0}")]
    ReadinessProbe(String),
    
    #[error("Liveness probe failed: {0}")]
    LivenessProbe(String),
    
    /// 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// IO错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// 其他错误
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, CloudError>;

/// 错误上下文
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// 错误代码
    pub error_code: String,
    
    /// 错误消息
    pub message: String,
    
    /// 详细信息
    pub details: Option<String>,
    
    /// 建议操作
    pub suggestions: Vec<String>,
    
    /// 错误分类
    pub category: ErrorCategory,
    
    /// 严重程度
    pub severity: ErrorSeverity,
    
    /// 是否可重试
    pub retryable: bool,
    
    /// 相关文档链接
    pub documentation_url: Option<String>,
}

/// 错误分类
#[derive(Debug, Clone)]
pub enum ErrorCategory {
    /// 基础设施错误
    Infrastructure,
    /// 配置错误
    Configuration,
    /// 网络错误
    Network,
    /// 安全错误
    Security,
    /// 资源错误
    Resource,
    /// 部署错误
    Deployment,
    /// 监控错误
    Monitoring,
}

/// 错误严重程度
#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 严重
    Critical,
}

impl CloudError {
    /// 获取错误上下文
    pub fn context(&self) -> ErrorContext {
        match self {
            Self::KubernetesConnection(msg) => ErrorContext {
                error_code: "KUBERNETES_CONNECTION".to_string(),
                message: msg.clone(),
                details: None,
                suggestions: vec![
                    "检查kubeconfig文件是否正确配置".to_string(),
                    "验证Kubernetes集群是否可访问".to_string(),
                    "确认网络连接正常".to_string(),
                ],
                category: ErrorCategory::Infrastructure,
                severity: ErrorSeverity::High,
                retryable: true,
                documentation_url: Some("https://docs.lumosai.com/deployment/kubernetes".to_string()),
            },
            Self::KubernetesDeployment(msg) => ErrorContext {
                error_code: "KUBERNETES_DEPLOYMENT".to_string(),
                message: msg.clone(),
                details: None,
                suggestions: vec![
                    "检查部署配置是否正确".to_string(),
                    "验证资源配额是否充足".to_string(),
                    "查看Pod日志获取详细错误信息".to_string(),
                ],
                category: ErrorCategory::Deployment,
                severity: ErrorSeverity::High,
                retryable: true,
                documentation_url: Some("https://docs.lumosai.com/troubleshooting/kubernetes".to_string()),
            },
            Self::DockerConnection(msg) => ErrorContext {
                error_code: "DOCKER_CONNECTION".to_string(),
                message: msg.clone(),
                details: None,
                suggestions: vec![
                    "确认Docker守护进程正在运行".to_string(),
                    "检查Docker socket权限".to_string(),
                    "验证Docker版本兼容性".to_string(),
                ],
                category: ErrorCategory::Infrastructure,
                severity: ErrorSeverity::High,
                retryable: true,
                documentation_url: Some("https://docs.lumosai.com/deployment/docker".to_string()),
            },
            Self::InvalidDeploymentConfig { field, message } => ErrorContext {
                error_code: "INVALID_DEPLOYMENT_CONFIG".to_string(),
                message: format!("配置字段 '{}': {}", field, message),
                details: None,
                suggestions: vec![
                    format!("检查配置字段 '{}' 的值", field),
                    "参考配置文档确认正确格式".to_string(),
                    "使用配置验证工具检查配置".to_string(),
                ],
                category: ErrorCategory::Configuration,
                severity: ErrorSeverity::Medium,
                retryable: false,
                documentation_url: Some("https://docs.lumosai.com/configuration/deployment".to_string()),
            },
            Self::ResourceNotFound { resource_type, name } => ErrorContext {
                error_code: "RESOURCE_NOT_FOUND".to_string(),
                message: format!("资源 {} '{}' 未找到", resource_type, name),
                details: None,
                suggestions: vec![
                    format!("确认资源 '{}' 是否存在", name),
                    "检查命名空间是否正确".to_string(),
                    "验证资源名称拼写".to_string(),
                ],
                category: ErrorCategory::Resource,
                severity: ErrorSeverity::Medium,
                retryable: false,
                documentation_url: Some("https://docs.lumosai.com/resources".to_string()),
            },
            Self::DeploymentTimeout { timeout_seconds } => ErrorContext {
                error_code: "DEPLOYMENT_TIMEOUT".to_string(),
                message: format!("部署超时 ({}秒)", timeout_seconds),
                details: None,
                suggestions: vec![
                    "增加部署超时时间".to_string(),
                    "检查资源是否充足".to_string(),
                    "查看部署日志确认问题".to_string(),
                ],
                category: ErrorCategory::Deployment,
                severity: ErrorSeverity::High,
                retryable: true,
                documentation_url: Some("https://docs.lumosai.com/troubleshooting/timeouts".to_string()),
            },
            Self::InsufficientResources(msg) => ErrorContext {
                error_code: "INSUFFICIENT_RESOURCES".to_string(),
                message: msg.clone(),
                details: None,
                suggestions: vec![
                    "增加集群资源容量".to_string(),
                    "调整资源请求和限制".to_string(),
                    "启用自动扩容".to_string(),
                ],
                category: ErrorCategory::Resource,
                severity: ErrorSeverity::High,
                retryable: true,
                documentation_url: Some("https://docs.lumosai.com/scaling/resources".to_string()),
            },
            _ => ErrorContext {
                error_code: "UNKNOWN_ERROR".to_string(),
                message: self.to_string(),
                details: None,
                suggestions: vec!["联系技术支持".to_string()],
                category: ErrorCategory::Infrastructure,
                severity: ErrorSeverity::High,
                retryable: false,
                documentation_url: Some("https://docs.lumosai.com/support".to_string()),
            },
        }
    }
    
    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::KubernetesConnection(_) => "KUBERNETES_CONNECTION",
            Self::KubernetesDeployment(_) => "KUBERNETES_DEPLOYMENT",
            Self::KubernetesNotConfigured => "KUBERNETES_NOT_CONFIGURED",
            Self::DockerConnection(_) => "DOCKER_CONNECTION",
            Self::DockerDeployment(_) => "DOCKER_DEPLOYMENT",
            Self::DockerNotConfigured => "DOCKER_NOT_CONFIGURED",
            Self::CloudProviderNotConfigured(_) => "CLOUD_PROVIDER_NOT_CONFIGURED",
            Self::AwsError(_) => "AWS_ERROR",
            Self::AzureError(_) => "AZURE_ERROR",
            Self::GcpError(_) => "GCP_ERROR",
            Self::Configuration(_) => "CONFIGURATION_ERROR",
            Self::InvalidDeploymentConfig { .. } => "INVALID_DEPLOYMENT_CONFIG",
            Self::Network(_) => "NETWORK_ERROR",
            Self::ServiceDiscovery(_) => "SERVICE_DISCOVERY_ERROR",
            Self::LoadBalancer(_) => "LOAD_BALANCER_ERROR",
            Self::Storage(_) => "STORAGE_ERROR",
            Self::VolumeMount(_) => "VOLUME_MOUNT_ERROR",
            Self::Security(_) => "SECURITY_ERROR",
            Self::Authentication(_) => "AUTHENTICATION_ERROR",
            Self::Authorization(_) => "AUTHORIZATION_ERROR",
            Self::Monitoring(_) => "MONITORING_ERROR",
            Self::MetricsCollection(_) => "METRICS_COLLECTION_ERROR",
            Self::Autoscaling(_) => "AUTOSCALING_ERROR",
            Self::ResourceScaling(_) => "RESOURCE_SCALING_ERROR",
            Self::ResourceNotFound { .. } => "RESOURCE_NOT_FOUND",
            Self::ResourceConflict(_) => "RESOURCE_CONFLICT",
            Self::InsufficientResources(_) => "INSUFFICIENT_RESOURCES",
            Self::DeploymentFailed(_) => "DEPLOYMENT_FAILED",
            Self::DeploymentTimeout { .. } => "DEPLOYMENT_TIMEOUT",
            Self::RollbackFailed(_) => "ROLLBACK_FAILED",
            Self::HealthCheckFailed(_) => "HEALTH_CHECK_FAILED",
            Self::ReadinessProbe(_) => "READINESS_PROBE_FAILED",
            Self::LivenessProbe(_) => "LIVENESS_PROBE_FAILED",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::Io(_) => "IO_ERROR",
            Self::Unknown(_) => "UNKNOWN_ERROR",
        }
    }
    
    /// 是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::KubernetesConnection(_) |
            Self::DockerConnection(_) |
            Self::Network(_) |
            Self::DeploymentTimeout { .. } |
            Self::InsufficientResources(_) |
            Self::HealthCheckFailed(_) |
            Self::ReadinessProbe(_) |
            Self::LivenessProbe(_)
        )
    }
}

// 从其他错误类型转换
impl From<serde_json::Error> for CloudError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<serde_yaml::Error> for CloudError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}
