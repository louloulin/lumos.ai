//! 云服务适配器模块
//! 
//! 提供对主要云服务提供商的集成支持，包括：
//! - AWS (Amazon Web Services)
//! - Azure (Microsoft Azure)
//! - GCP (Google Cloud Platform)
//! 
//! 这些适配器简化了在不同云平台上部署和运行LumosAI应用的过程。

pub mod aws;
pub mod azure;
pub mod gcp;

pub use aws::AwsAdapter;
pub use azure::AzureAdapter;
pub use gcp::GcpAdapter;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Result;

/// 云服务适配器trait
#[async_trait]
pub trait CloudAdapter: Send + Sync {
    /// 获取适配器名称
    fn name(&self) -> &str;

    /// 获取支持的服务列表
    fn supported_services(&self) -> Vec<CloudService>;

    /// 部署应用
    async fn deploy_application(&self, config: &DeploymentConfig) -> Result<DeploymentResult>;

    /// 获取部署状态
    async fn get_deployment_status(&self, deployment_id: &str) -> Result<DeploymentStatus>;

    /// 更新部署
    async fn update_deployment(&self, deployment_id: &str, config: &DeploymentConfig) -> Result<DeploymentResult>;

    /// 删除部署
    async fn delete_deployment(&self, deployment_id: &str) -> Result<()>;

    /// 获取服务日志
    async fn get_logs(&self, deployment_id: &str, options: &LogOptions) -> Result<Vec<LogEntry>>;

    /// 获取服务指标
    async fn get_metrics(&self, deployment_id: &str, options: &MetricsOptions) -> Result<MetricsData>;

    /// 配置自动扩缩容
    async fn configure_autoscaling(&self, deployment_id: &str, config: &AutoscalingConfig) -> Result<()>;

    /// 配置负载均衡
    async fn configure_load_balancer(&self, deployment_id: &str, config: &LoadBalancerConfig) -> Result<()>;
}

/// 云服务类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudService {
    /// 容器服务
    Container,
    /// 函数计算
    Function,
    /// 数据库
    Database,
    /// 对象存储
    Storage,
    /// 消息队列
    MessageQueue,
    /// 缓存服务
    Cache,
    /// 监控服务
    Monitoring,
    /// 日志服务
    Logging,
}

/// 部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// 应用名称
    pub name: String,
    /// 应用版本
    pub version: String,
    /// 容器镜像
    pub image: String,
    /// 环境变量
    pub environment: HashMap<String, String>,
    /// 资源配置
    pub resources: ResourceConfig,
    /// 网络配置
    pub network: NetworkConfig,
    /// 存储配置
    pub storage: Option<StorageConfig>,
    /// 扩缩容配置
    pub autoscaling: Option<AutoscalingConfig>,
    /// 健康检查配置
    pub health_check: Option<HealthCheckConfig>,
}

/// 资源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// CPU核心数
    pub cpu: f32,
    /// 内存大小(MB)
    pub memory: u32,
    /// 存储大小(GB)
    pub storage: Option<u32>,
    /// GPU数量
    pub gpu: Option<u32>,
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 端口映射
    pub ports: Vec<PortMapping>,
    /// 是否公开访问
    pub public: bool,
    /// 域名配置
    pub domain: Option<String>,
    /// SSL配置
    pub ssl: Option<SslConfig>,
}

/// 端口映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    /// 容器端口
    pub container_port: u16,
    /// 主机端口
    pub host_port: Option<u16>,
    /// 协议类型
    pub protocol: String,
}

/// SSL配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    /// 证书ARN或ID
    pub certificate_id: String,
    /// 是否强制HTTPS
    pub force_https: bool,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 存储类型
    pub storage_type: StorageType,
    /// 存储大小(GB)
    pub size: u32,
    /// 挂载路径
    pub mount_path: String,
}

/// 存储类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    /// 临时存储
    Ephemeral,
    /// 持久化存储
    Persistent,
    /// 共享存储
    Shared,
}

/// 自动扩缩容配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoscalingConfig {
    /// 最小实例数
    pub min_instances: u32,
    /// 最大实例数
    pub max_instances: u32,
    /// CPU使用率阈值
    pub cpu_threshold: f32,
    /// 内存使用率阈值
    pub memory_threshold: f32,
    /// 扩容冷却时间(秒)
    pub scale_up_cooldown: u32,
    /// 缩容冷却时间(秒)
    pub scale_down_cooldown: u32,
}

/// 负载均衡配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// 负载均衡类型
    pub lb_type: LoadBalancerType,
    /// 健康检查路径
    pub health_check_path: String,
    /// 会话保持
    pub sticky_sessions: bool,
}

/// 负载均衡类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerType {
    /// 应用负载均衡
    Application,
    /// 网络负载均衡
    Network,
    /// 经典负载均衡
    Classic,
}

/// 健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// 检查路径
    pub path: String,
    /// 检查间隔(秒)
    pub interval: u32,
    /// 超时时间(秒)
    pub timeout: u32,
    /// 健康阈值
    pub healthy_threshold: u32,
    /// 不健康阈值
    pub unhealthy_threshold: u32,
}

/// 部署结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    /// 部署ID
    pub deployment_id: String,
    /// 部署状态
    pub status: DeploymentStatus,
    /// 访问URL
    pub url: Option<String>,
    /// 部署信息
    pub metadata: HashMap<String, String>,
}

/// 部署状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    /// 部署中
    Deploying,
    /// 运行中
    Running,
    /// 已停止
    Stopped,
    /// 失败
    Failed,
    /// 更新中
    Updating,
}

/// 日志选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOptions {
    /// 开始时间
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 结束时间
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 日志级别过滤
    pub level: Option<String>,
    /// 最大条数
    pub limit: Option<u32>,
    /// 是否跟踪实时日志
    pub follow: bool,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 日志级别
    pub level: String,
    /// 日志消息
    pub message: String,
    /// 来源
    pub source: String,
    /// 额外字段
    pub fields: HashMap<String, String>,
}

/// 指标选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsOptions {
    /// 开始时间
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// 结束时间
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// 指标名称
    pub metrics: Vec<String>,
    /// 聚合间隔(秒)
    pub interval: u32,
}

/// 指标数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    /// 指标点
    pub data_points: Vec<MetricPoint>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 指标点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 指标名称
    pub metric_name: String,
    /// 指标值
    pub value: f64,
    /// 标签
    pub labels: HashMap<String, String>,
}

/// 创建云适配器的便利函数
pub fn create_aws_adapter() -> Result<AwsAdapter> {
    AwsAdapter::from_env()
}

pub fn create_azure_adapter() -> Result<AzureAdapter> {
    AzureAdapter::from_env()
}

pub fn create_gcp_adapter() -> Result<GcpAdapter> {
    GcpAdapter::from_env()
}

/// 根据云提供商名称创建适配器
pub fn create_adapter(provider: &str) -> Result<Box<dyn CloudAdapter>> {
    match provider.to_lowercase().as_str() {
        "aws" => Ok(Box::new(create_aws_adapter()?)),
        "azure" => Ok(Box::new(create_azure_adapter()?)),
        "gcp" | "google" => Ok(Box::new(create_gcp_adapter()?)),
        _ => Err(crate::error::LumosError::ConfigError {
            message: format!("Unsupported cloud provider: {}", provider),
        }),
    }
}
