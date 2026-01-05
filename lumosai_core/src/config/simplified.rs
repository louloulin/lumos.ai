//! Simplified Configuration System for LumosAI
//!
//! 简化后的配置系统，提供智能默认值和易用的API
//! 大幅减少配置复杂性，提升开发体验

use crate::config::types::*;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 简化配置模式枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigMode {
    /// 开发模式 - 最小配置，最大便利性
    Development,

    /// 生产模式 - 完整配置，最大性能
    Production,

    /// 测试模式 - 隔离配置，可重现性
    Testing,
}

/// 简化配置构建器 - 流式API
#[derive(Debug)]
pub struct LumosConfigBuilder {
    mode: ConfigMode,
    project_name: Option<String>,
    llm_provider: Option<String>,
    database_url: Option<String>,
    enable_cache: bool,
    enable_monitoring: bool,
    custom_settings: HashMap<String, serde_json::Value>,
}

impl LumosConfigBuilder {
    /// 创建开发模式配置构建器
    pub fn development() -> Self {
        Self {
            mode: ConfigMode::Development,
            project_name: None,
            llm_provider: None,
            database_url: None,
            enable_cache: true,
            enable_monitoring: false,
            custom_settings: HashMap::new(),
        }
    }

    /// 创建生产模式配置构建器
    pub fn production() -> Self {
        Self {
            mode: ConfigMode::Production,
            project_name: None,
            llm_provider: None,
            database_url: None,
            enable_cache: true,
            enable_monitoring: true,
            custom_settings: HashMap::new(),
        }
    }

    /// 创建测试模式配置构建器
    pub fn testing() -> Self {
        Self {
            mode: ConfigMode::Testing,
            project_name: Some("test-project".to_string()),
            llm_provider: Some("mock".to_string()),
            database_url: Some("sqlite::memory:".to_string()),
            enable_cache: false,
            enable_monitoring: false,
            custom_settings: HashMap::new(),
        }
    }

    /// 设置项目名称
    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }

    /// 设置LLM提供商
    pub fn llm_provider(mut self, provider: impl Into<String>) -> Self {
        self.llm_provider = Some(provider.into());
        self
    }

    /// 设置数据库URL
    pub fn database_url(mut self, url: impl Into<String>) -> Self {
        self.database_url = Some(url.into());
        self
    }

    /// 启用/禁用缓存
    pub fn cache(mut self, enabled: bool) -> Self {
        self.enable_cache = enabled;
        self
    }

    /// 启用/禁用监控
    pub fn monitoring(mut self, enabled: bool) -> Self {
        self.enable_monitoring = enabled;
        self
    }

    /// 添加自定义设置
    pub fn with_custom<K: Into<String>, V: Into<serde_json::Value>>(mut self, key: K, value: V) -> Self {
        self.custom_settings.insert(key.into(), value.into());
        self
    }

    /// 构建完整配置
    pub fn build(self) -> Result<LumosaiConfig> {
        let config = match self.mode {
            ConfigMode::Development => self.build_development_config(),
            ConfigMode::Production => self.build_production_config(),
            ConfigMode::Testing => self.build_testing_config(),
        };

        // 应用自定义设置覆盖
        let mut final_config = config;
        self.apply_custom_settings(&mut final_config);

        Ok(final_config)
    }

    /// 构建开发模式配置
    fn build_development_config(self) -> LumosaiConfig {
        LumosaiConfig {
            project: self.project_name.map(|name| ProjectConfig {
                name,
                version: "0.1.0".to_string(),
                description: Some("Development project".to_string()),
                authors: vec![],
                license: None,
                repository: None,
                homepage: None,
            }),

            agents: AgentsConfig {
                defaults: AgentDefaultsConfig {
                    model: "gpt-3.5-turbo".to_string(),
                    temperature: 0.7,
                    max_tokens: 1000,
                    timeout: 30,
                    enable_function_calling: true,
                    memory_enabled: true,
                    streaming: false,
                    instructions_template: None,
                    tools: vec![],
                    custom_parameters: HashMap::new(),
                },
                agents: vec![],
            },

            llm: LlmConfig {
                default_provider: self.llm_provider.unwrap_or_else(|| "openai".to_string()),
                providers: {
                    let mut providers = HashMap::new();
                    providers.insert("openai".to_string(), ProviderConfig {
                        provider_type: "openai".to_string(),
                        connection: ConnectionConfig {
                            base_url: "https://api.openai.com/v1".to_string(),
                            timeout: 30,
                            retry_attempts: 3,
                            retry_delay: 1,
                            rate_limit: Some(RateLimitConfig {
                                requests_per_minute: 60,
                                burst_limit: 10,
                            }),
                            proxy: None,
                        },
                        api_key: ApiKeyConfig {
                            key_source: "env".to_string(),
                            key_name: "OPENAI_API_KEY".to_string(),
                            rotation: None,
                        },
                        models: vec!["gpt-3.5-turbo".to_string(), "gpt-4".to_string()],
                        parameters: HashMap::new(),
                    });
                    providers
                },
                routing: Default::default(),
                fallback: Default::default(),
            },

            tools: ToolsConfig {
                builtin: BuiltinToolsConfig {
                    enabled: vec!["web_search".to_string(), "calculator".to_string()],
                    disabled: vec![],
                    config: HashMap::new(),
                },
                custom: vec![],
            },

            storage: StorageConfig {
                backend: StorageBackendConfig::FileSystem {
                    path: "./data".to_string(),
                    compression: true,
                },
                vector: VectorStorageConfig {
                    backend: "qdrant".to_string(),
                    connection: "http://localhost:6334".to_string(),
                    collection: "lumosai_dev".to_string(),
                    dimension: 384,
                    index: IndexConfig {
                        index_type: "hnsw".to_string(),
                        parameters: HashMap::new(),
                    },
                },
                cache: CacheConfig {
                    enabled: self.enable_cache,
                    backend: "memory".to_string(),
                    ttl: 3600,
                    max_size: 1000,
                    compression: false,
                },
            },

            monitoring: MonitoringConfig {
                enabled: self.enable_monitoring,
                metrics: Default::default(),
                tracing: Default::default(),
                alerts: Default::default(),
            },

            security: SecurityConfig {
                authentication: Default::default(),
                authorization: Default::default(),
                encryption: Default::default(),
                audit: Default::default(),
            },

            cache: CacheConfig {
                enabled: self.enable_cache,
                backend: "memory".to_string(),
                ttl: 3600,
                max_size: 1000,
                compression: false,
            },

            network: NetworkConfig {
                timeout: 30,
                retry: Default::default(),
                proxy: None,
                tls: Default::default(),
            },

            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                outputs: vec!["stdout".to_string()],
                filters: vec![],
            },
        }
    }

    /// 构建生产模式配置
    fn build_production_config(self) -> LumosaiConfig {
        let mut dev_config = self.build_development_config();

        // 生产模式优化
        dev_config.agents.defaults.model = "gpt-4".to_string();
        dev_config.agents.defaults.max_tokens = 4000;

        if let StorageBackendConfig::FileSystem { ref mut path, .. } = dev_config.storage.backend {
            *path = "/data/lumosai".to_string();
        }

        dev_config.storage.vector.connection = "http://qdrant:6334".to_string();
        dev_config.monitoring.enabled = true;

        dev_config.cache.enabled = true;
        dev_config.cache.backend = "redis".to_string();
        dev_config.cache.max_size = 10000;

        dev_config.logging.level = "warn".to_string();
        dev_config.logging.outputs = vec!["stdout".to_string(), "file".to_string()];

        dev_config
    }

    /// 构建测试模式配置
    fn build_testing_config(self) -> LumosaiConfig {
        let mut dev_config = self.build_development_config();

        // 测试模式优化
        dev_config.agents.defaults.model = "mock".to_string();
        dev_config.storage.backend = StorageBackendConfig::InMemory;
        dev_config.cache.enabled = false;
        dev_config.monitoring.enabled = false;

        dev_config.logging.level = "error".to_string();

        dev_config
    }

    /// 应用自定义设置
    fn apply_custom_settings(&self, config: &mut LumosaiConfig) {
        // 这里可以实现自定义设置的覆盖逻辑
        // 暂时留空，实际实现会根据custom_settings覆盖相应配置
        for (key, value) in &self.custom_settings {
            match key.as_str() {
                "llm.model" => {
                    if let serde_json::Value::String(model) = value {
                        config.agents.defaults.model = model.clone();
                    }
                }
                "llm.temperature" => {
                    if let serde_json::Value::Number(temp) = value {
                        if let Some(temp_f64) = temp.as_f64() {
                            config.agents.defaults.temperature = temp_f64;
                        }
                    }
                }
                // 可以继续添加更多自定义设置
                _ => {}
            }
        }
    }
}

/// 快速配置函数 - 一行代码创建常用配置
pub mod quick_config {
    use super::*;

    /// 创建开发环境配置
    pub fn development() -> Result<LumosaiConfig> {
        LumosConfigBuilder::development().build()
    }

    /// 创建生产环境配置
    pub fn production() -> Result<LumosaiConfig> {
        LumosConfigBuilder::production().build()
    }

    /// 创建测试环境配置
    pub fn testing() -> Result<LumosaiConfig> {
        LumosConfigBuilder::testing().build()
    }

    /// 从环境变量自动配置
    pub fn from_env() -> Result<LumosaiConfig> {
        let mut builder = LumosConfigBuilder::development();

        if let Ok(project_name) = std::env::var("LUMOSAI_PROJECT_NAME") {
            builder = builder.project_name(project_name);
        }

        if let Ok(llm_provider) = std::env::var("LUMOSAI_LLM_PROVIDER") {
            builder = builder.llm_provider(llm_provider);
        }

        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            builder = builder.database_url(database_url);
        }

        if let Ok(cache_enabled) = std::env::var("LUMOSAI_CACHE_ENABLED") {
            builder = builder.cache(cache_enabled.parse().unwrap_or(true));
        }

        if let Ok(monitoring_enabled) = std::env::var("LUMOSAI_MONITORING_ENABLED") {
            builder = builder.monitoring(monitoring_enabled.parse().unwrap_or(false));
        }

        builder.build()
    }

    /// 从TOML文件加载配置，支持简化语法
    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<LumosaiConfig> {
        let content = std::fs::read_to_string(path)?;
        from_toml_string(&content)
    }

    /// 从TOML字符串加载配置，支持简化语法
    pub fn from_toml_string(content: &str) -> Result<LumosaiConfig> {
        // 首先尝试解析简化格式
        if let Ok(simplified) = toml::from_str::<SimplifiedConfigToml>(content) {
            return simplified.to_full_config();
        }

        // 回退到完整格式
        let config: LumosaiConfig = toml::from_str(content)?;
        Ok(config)
    }
}

/// 简化TOML配置格式
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimplifiedConfigToml {
    /// 模式：development/production/testing
    mode: Option<String>,

    /// 项目名称
    project: Option<String>,

    /// LLM提供商
    llm_provider: Option<String>,

    /// 数据库URL
    database_url: Option<String>,

    /// 启用缓存
    cache: Option<bool>,

    /// 启用监控
    monitoring: Option<bool>,

    /// 自定义设置
    #[serde(flatten)]
    custom: HashMap<String, serde_json::Value>,
}

impl SimplifiedConfigToml {
    fn to_full_config(self) -> Result<LumosaiConfig> {
        let mut builder = match self.mode.as_deref() {
            Some("production") => LumosConfigBuilder::production(),
            Some("testing") => LumosConfigBuilder::testing(),
            _ => LumosConfigBuilder::development(),
        };

        if let Some(project) = self.project {
            builder = builder.project_name(project);
        }

        if let Some(llm_provider) = self.llm_provider {
            builder = builder.llm_provider(llm_provider);
        }

        if let Some(database_url) = self.database_url {
            builder = builder.database_url(database_url);
        }

        if let Some(cache) = self.cache {
            builder = builder.cache(cache);
        }

        if let Some(monitoring) = self.monitoring {
            builder = builder.monitoring(monitoring);
        }

        // 应用自定义设置
        for (key, value) in self.custom {
            builder = builder.with_custom(key, value);
        }

        builder.build()
    }
}

/// 配置验证和优化工具
pub struct ConfigOptimizer;

impl ConfigOptimizer {
    /// 验证配置完整性和一致性
    pub fn validate_config(config: &LumosaiConfig) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // 检查必需的配置项
        if config.agents.defaults.model.is_empty() {
            issues.push(ValidationIssue {
                level: IssueLevel::Error,
                message: "Agent default model cannot be empty".to_string(),
                field: "agents.defaults.model".to_string(),
                suggestion: "Set a valid LLM model name".to_string(),
            });
        }

        // 检查LLM提供商配置
        if !config.llm.providers.contains_key(&config.llm.default_provider) {
            issues.push(ValidationIssue {
                level: IssueLevel::Error,
                message: format!("Default LLM provider '{}' not configured", config.llm.default_provider),
                field: "llm.default_provider".to_string(),
                suggestion: "Add the provider to llm.providers or change default_provider".to_string(),
            });
        }

        // 检查存储配置
        match &config.storage.backend {
            StorageBackendConfig::FileSystem { path, .. } => {
                if !Path::new(path).exists() {
                    issues.push(ValidationIssue {
                        level: IssueLevel::Warning,
                        message: format!("Storage path '{}' does not exist", path),
                        field: "storage.backend.path".to_string(),
                        suggestion: "Create the directory or update the path".to_string(),
                    });
                }
            }
            _ => {}
        }

        // 检查缓存配置一致性
        if config.cache.enabled && config.storage.cache.enabled {
            issues.push(ValidationIssue {
                level: IssueLevel::Warning,
                message: "Both global cache and storage cache are enabled".to_string(),
                field: "cache.enabled".to_string(),
                suggestion: "Consider disabling one to avoid duplication".to_string(),
            });
        }

        Ok(issues)
    }

    /// 优化配置性能
    pub fn optimize_for_performance(config: &mut LumosaiConfig) {
        // 生产模式优化
        if matches!(config.monitoring.enabled, true) {
            // 减少监控频率以提高性能
            config.monitoring.metrics.collection_interval = 60; // 1分钟
        }

        // 缓存优化
        if config.cache.enabled {
            config.cache.ttl = 7200; // 2小时TTL
            config.cache.max_size = 5000; // 增加缓存大小
        }

        // LLM优化
        for provider in config.llm.providers.values_mut() {
            if let Some(rate_limit) = &mut provider.connection.rate_limit {
                rate_limit.requests_per_minute = 120; // 提高速率限制
            }
        }
    }

    /// 优化配置内存使用
    pub fn optimize_for_memory(config: &mut LumosaiConfig) {
        // 减少缓存大小
        config.cache.max_size = 500;
        config.storage.cache.max_size = 500;

        // 减少监控数据保留时间
        config.monitoring.metrics.retention_days = 7;

        // 减少日志级别以减少内存使用
        if config.logging.level == "debug" {
            config.logging.level = "info".to_string();
        }
    }
}

/// 配置验证问题
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub level: IssueLevel,
    pub message: String,
    pub field: String,
    pub suggestion: String,
}

/// 问题级别
#[derive(Debug, Clone)]
pub enum IssueLevel {
    Error,
    Warning,
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_config_builder() -> Result<()> {
        let config = LumosConfigBuilder::development()
            .project_name("test-project")
            .llm_provider("openai")
            .cache(true)
            .monitoring(false)
            .build()?;

        assert_eq!(config.project.as_ref().unwrap().name, "test-project");
        assert_eq!(config.llm.default_provider, "openai");
        assert!(config.cache.enabled);
        assert!(!config.monitoring.enabled);

        Ok(())
    }

    #[test]
    fn test_production_config_builder() -> Result<()> {
        let config = LumosConfigBuilder::production()
            .project_name("prod-project")
            .build()?;

        assert_eq!(config.project.as_ref().unwrap().name, "prod-project");
        assert!(config.monitoring.enabled);
        assert_eq!(config.agents.defaults.model, "gpt-4");

        Ok(())
    }

    #[test]
    fn test_testing_config_builder() -> Result<()> {
        let config = LumosConfigBuilder::testing().build()?;

        assert_eq!(config.project.as_ref().unwrap().name, "test-project");
        assert_eq!(config.llm.default_provider, "mock");
        assert!(!config.cache.enabled);
        assert!(!config.monitoring.enabled);

        Ok(())
    }

    #[test]
    fn test_custom_settings() -> Result<()> {
        let config = LumosConfigBuilder::development()
            .with_custom("llm.model", "gpt-4-turbo")
            .with_custom("llm.temperature", 0.5)
            .build()?;

        assert_eq!(config.agents.defaults.model, "gpt-4-turbo");
        assert_eq!(config.agents.defaults.temperature, 0.5);

        Ok(())
    }

    #[test]
    fn test_config_validation() -> Result<()> {
        let config = LumosConfigBuilder::development().build()?;
        let issues = ConfigOptimizer::validate_config(&config)?;

        // 开发配置应该没有错误
        let errors = issues.iter().filter(|i| matches!(i.level, IssueLevel::Error)).count();
        assert_eq!(errors, 0);

        Ok(())
    }

    #[test]
    fn test_quick_config_functions() -> Result<()> {
        let dev_config = quick_config::development()?;
        assert_eq!(dev_config.llm.default_provider, "openai");

        let prod_config = quick_config::production()?;
        assert!(prod_config.monitoring.enabled);

        let test_config = quick_config::testing()?;
        assert_eq!(test_config.llm.default_provider, "mock");

        Ok(())
    }

    #[test]
    fn test_simplified_toml_parsing() -> Result<()> {
        let toml_content = r#"
            mode = "development"
            project = "my-project"
            llm_provider = "openai"
            cache = true
            monitoring = false
        "#;

        let config = quick_config::from_toml_string(toml_content)?;
        assert_eq!(config.project.as_ref().unwrap().name, "my-project");
        assert_eq!(config.llm.default_provider, "openai");

        Ok(())
    }
}



