//! A2A Protocol Configuration
//!
//! 定义 A2A 协议的配置选项和设置

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// A2A 协议配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AConfig {
    /// 默认任务超时时间（秒）
    pub default_task_timeout: u64,
    /// 最大任务重试次数
    pub max_task_retries: u32,
    /// 是否启用流式响应
    pub enable_streaming: bool,
    /// 是否启用推送通知
    pub enable_push_notifications: bool,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 任务状态历史保留数量
    pub task_history_limit: usize,
    /// Agent 发现缓存过期时间（秒）
    pub agent_cache_ttl: u64,
    /// 自定义配置
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for A2AConfig {
    fn default() -> Self {
        Self {
            default_task_timeout: 300, // 5分钟
            max_task_retries: 3,
            enable_streaming: true,
            enable_push_notifications: false,
            heartbeat_interval: 30, // 30秒
            max_concurrent_tasks: 10,
            task_history_limit: 100,
            agent_cache_ttl: 600, // 10分钟
            custom_settings: HashMap::new(),
        }
    }
}

impl A2AConfig {
    /// 创建新的配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置任务超时时间
    pub fn with_task_timeout(mut self, timeout_seconds: u64) -> Self {
        self.default_task_timeout = timeout_seconds;
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_task_retries = max_retries;
        self
    }

    /// 启用/禁用流式响应
    pub fn with_streaming(mut self, enable: bool) -> Self {
        self.enable_streaming = enable;
        self
    }

    /// 启用/禁用推送通知
    pub fn with_push_notifications(mut self, enable: bool) -> Self {
        self.enable_push_notifications = enable;
        self
    }

    /// 设置心跳间隔
    pub fn with_heartbeat_interval(mut self, interval_seconds: u64) -> Self {
        self.heartbeat_interval = interval_seconds;
        self
    }

    /// 设置最大并发任务数
    pub fn with_max_concurrent_tasks(mut self, max_tasks: usize) -> Self {
        self.max_concurrent_tasks = max_tasks;
        self
    }

    /// 添加自定义设置
    pub fn with_custom_setting(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom_settings.insert(key, value);
        self
    }

    /// 获取超时时长
    pub fn task_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.default_task_timeout)
    }

    /// 获取心跳间隔时长
    pub fn heartbeat_interval_duration(&self) -> Duration {
        Duration::from_secs(self.heartbeat_interval)
    }

    /// 获取缓存过期时长
    pub fn agent_cache_ttl_duration(&self) -> Duration {
        Duration::from_secs(self.agent_cache_ttl)
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.default_task_timeout == 0 {
            return Err("Task timeout must be greater than 0".to_string());
        }

        if self.max_concurrent_tasks == 0 {
            return Err("Max concurrent tasks must be greater than 0".to_string());
        }

        if self.heartbeat_interval == 0 {
            return Err("Heartbeat interval must be greater than 0".to_string());
        }

        Ok(())
    }
}

/// A2A 协议安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 是否启用TLS
    pub enable_tls: bool,
    /// 验证证书链
    pub verify_certificates: bool,
    /// 允许的自定义CA证书路径
    pub ca_cert_path: Option<String>,
    /// 客户端证书路径
    pub client_cert_path: Option<String>,
    /// 客户端私钥路径
    pub client_key_path: Option<String>,
    /// API密钥
    pub api_keys: HashMap<String, String>,
    /// OAuth配置
    pub oauth_config: Option<OAuthConfig>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_tls: true,
            verify_certificates: true,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            api_keys: HashMap::new(),
            oauth_config: None,
        }
    }
}

/// OAuth配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// OAuth提供商
    pub provider: String,
    /// 客户端ID
    pub client_id: String,
    /// 客户端密钥
    pub client_secret: String,
    /// 授权范围
    pub scopes: Vec<String>,
    /// 重定向URI
    pub redirect_uri: String,
    /// token端点
    pub token_endpoint: String,
}

/// A2A 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 连接超时时间（秒）
    pub connect_timeout: u64,
    /// 请求超时时间（秒）
    pub request_timeout: u64,
    /// 最大重定向次数
    pub max_redirects: u32,
    /// 用户代理
    pub user_agent: String,
    /// 代理设置
    pub proxy: Option<ProxyConfig>,
    /// 连接池大小
    pub connection_pool_size: usize,
    /// 是否启用HTTP/2
    pub enable_http2: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            connect_timeout: 10,
            request_timeout: 60,
            max_redirects: 5,
            user_agent: format!("LumosAI-A2A/{}", env!("CARGO_PKG_VERSION")),
            proxy: None,
            connection_pool_size: 100,
            enable_http2: true,
        }
    }
}

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 代理URL
    pub url: String,
    /// 代理用户名
    pub username: Option<String>,
    /// 代理密码
    pub password: Option<String>,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// 日志级别
    pub level: LogLevel,
    /// 是否记录请求/响应详情
    pub log_requests: bool,
    /// 是否记录任务状态变化
    pub log_task_transitions: bool,
    /// 是否记录Agent发现
    pub log_agent_discovery: bool,
    /// 日志文件路径（可选）
    pub log_file: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            log_requests: true,
            log_task_transitions: true,
            log_agent_discovery: false,
            log_file: None,
        }
    }
}

/// 日志级别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// 完整的 A2A 设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ASettings {
    /// 基础配置
    pub config: A2AConfig,
    /// 安全配置
    pub security: SecurityConfig,
    /// 网络配置
    pub network: NetworkConfig,
    /// 日志配置
    pub logging: LogConfig,
}

impl Default for A2ASettings {
    fn default() -> Self {
        Self {
            config: A2AConfig::default(),
            security: SecurityConfig::default(),
            network: NetworkConfig::default(),
            logging: LogConfig::default(),
        }
    }
}

impl A2ASettings {
    /// 创建新的设置
    pub fn new() -> Self {
        Self::default()
    }

    /// 从环境变量加载设置
    pub fn from_env() -> Self {
        let mut settings = Self::default();

        // 从环境变量读取配置
        if let Ok(timeout) = std::env::var("A2A_TASK_TIMEOUT") {
            if let Ok(seconds) = timeout.parse::<u64>() {
                settings.config.default_task_timeout = seconds;
            }
        }

        if let Ok(max_tasks) = std::env::var("A2A_MAX_CONCURRENT_TASKS") {
            if let Ok(count) = max_tasks.parse::<usize>() {
                settings.config.max_concurrent_tasks = count;
            }
        }

        if let Ok(streaming) = std::env::var("A2A_ENABLE_STREAMING") {
            settings.config.enable_streaming = streaming.parse().unwrap_or(true);
        }

        if let Ok(user_agent) = std::env::var("A2A_USER_AGENT") {
            settings.network.user_agent = user_agent;
        }

        settings
    }

    /// 从配置文件加载设置
    #[cfg(feature = "config")]
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let settings: A2ASettings = serde_yaml::from_str(&content)?;
        Ok(settings)
    }

    /// 保存设置到配置文件
    #[cfg(feature = "config")]
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 验证所有设置
    pub fn validate(&self) -> Result<(), String> {
        self.config.validate()
    }

    /// 构建器模式
    pub fn builder() -> A2ASettingsBuilder {
        A2ASettingsBuilder::new()
    }
}

/// A2A 设置构建器
pub struct A2ASettingsBuilder {
    settings: A2ASettings,
}

impl A2ASettingsBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            settings: A2ASettings::default(),
        }
    }

    /// 设置基础配置
    pub fn config(mut self, config: A2AConfig) -> Self {
        self.settings.config = config;
        self
    }

    /// 设置安全配置
    pub fn security(mut self, security: SecurityConfig) -> Self {
        self.settings.security = security;
        self
    }

    /// 设置网络配置
    pub fn network(mut self, network: NetworkConfig) -> Self {
        self.settings.network = network;
        self
    }

    /// 设置日志配置
    pub fn logging(mut self, logging: LogConfig) -> Self {
        self.settings.logging = logging;
        self
    }

    /// 构建设置
    pub fn build(self) -> Result<A2ASettings, String> {
        self.settings.validate().map(|_| self.settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a2a_config_default() {
        let config = A2AConfig::default();
        assert_eq!(config.default_task_timeout, 300);
        assert_eq!(config.max_task_retries, 3);
        assert!(config.enable_streaming);
    }

    #[test]
    fn test_a2a_config_builder() {
        let config = A2AConfig::new()
            .with_task_timeout(600)
            .with_max_retries(5)
            .with_streaming(false);

        assert_eq!(config.default_task_timeout, 600);
        assert_eq!(config.max_task_retries, 5);
        assert!(!config.enable_streaming);
    }

    #[test]
    fn test_config_validation() {
        let mut config = A2AConfig::default();
        assert!(config.validate().is_ok());

        config.default_task_timeout = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_settings_builder() {
        let settings = A2ASettings::builder()
            .config(A2AConfig::new().with_task_timeout(120))
            .build()
            .unwrap();

        assert_eq!(settings.config.default_task_timeout, 120);
    }

    #[test]
    fn test_settings_from_env() {
        // 设置环境变量
        std::env::set_var("A2A_TASK_TIMEOUT", "180");
        std::env::set_var("A2A_ENABLE_STREAMING", "false");

        let settings = A2ASettings::from_env();

        assert_eq!(settings.config.default_task_timeout, 180);
        assert!(!settings.config.enable_streaming);

        // 清理环境变量
        std::env::remove_var("A2A_TASK_TIMEOUT");
        std::env::remove_var("A2A_ENABLE_STREAMING");
    }

    #[test]
    fn test_settings_validation() {
        let settings = A2ASettings::default();
        assert!(settings.validate().is_ok());
    }
}