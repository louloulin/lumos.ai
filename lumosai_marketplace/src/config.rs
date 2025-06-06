//! 工具市场配置管理

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{MarketplaceError, Result};

/// 工具市场配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    /// 数据库连接URL
    pub database_url: String,
    
    /// Redis连接URL（可选）
    pub redis_url: Option<String>,
    
    /// 搜索索引路径
    pub search_index_path: String,
    
    /// 工具存储路径
    pub tool_storage_path: String,
    
    /// 是否启用安全扫描
    pub security_scanning_enabled: bool,
    
    /// 是否启用分析功能
    pub analytics_enabled: bool,
    
    /// 最大工具包大小（字节）
    pub max_package_size: u64,
    
    /// 允许的文件类型
    pub allowed_file_types: Vec<String>,
    
    /// API配置
    pub api: ApiConfig,
    
    /// 安全配置
    pub security: SecurityConfig,
    
    /// 缓存配置
    pub cache: CacheConfig,
    
    /// 搜索配置
    pub search: SearchConfig,
}

/// API配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// 监听地址
    pub host: String,
    
    /// 监听端口
    pub port: u16,
    
    /// 是否启用CORS
    pub cors_enabled: bool,
    
    /// 允许的源
    pub allowed_origins: Vec<String>,
    
    /// API密钥（可选）
    pub api_key: Option<String>,
    
    /// 请求速率限制
    pub rate_limit: RateLimitConfig,
}

/// 速率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 每分钟最大请求数
    pub requests_per_minute: u32,
    
    /// 每小时最大请求数
    pub requests_per_hour: u32,
    
    /// 每天最大请求数
    pub requests_per_day: u32,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 是否启用代码扫描
    pub code_scanning_enabled: bool,
    
    /// 是否启用依赖扫描
    pub dependency_scanning_enabled: bool,
    
    /// 是否启用恶意软件扫描
    pub malware_scanning_enabled: bool,
    
    /// 扫描超时时间（秒）
    pub scan_timeout_seconds: u64,
    
    /// 允许的许可证类型
    pub allowed_licenses: Vec<String>,
    
    /// 禁止的关键词
    pub forbidden_keywords: Vec<String>,
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 是否启用缓存
    pub enabled: bool,
    
    /// 缓存TTL（秒）
    pub ttl_seconds: u64,
    
    /// 最大缓存大小（条目数）
    pub max_entries: usize,
    
    /// 缓存预热
    pub preload_popular_tools: bool,
}

/// 搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// 索引更新间隔（秒）
    pub index_update_interval_seconds: u64,
    
    /// 最大搜索结果数
    pub max_search_results: usize,
    
    /// 是否启用模糊搜索
    pub fuzzy_search_enabled: bool,
    
    /// 模糊搜索阈值
    pub fuzzy_threshold: f64,
    
    /// 是否启用语义搜索
    pub semantic_search_enabled: bool,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite://marketplace.db".to_string(),
            redis_url: None,
            search_index_path: "./search_index".to_string(),
            tool_storage_path: "./tool_storage".to_string(),
            security_scanning_enabled: true,
            analytics_enabled: true,
            max_package_size: 100 * 1024 * 1024, // 100MB
            allowed_file_types: vec![
                "rs".to_string(),
                "toml".to_string(),
                "md".to_string(),
                "txt".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
            ],
            api: ApiConfig::default(),
            security: SecurityConfig::default(),
            cache: CacheConfig::default(),
            search: SearchConfig::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_enabled: true,
            allowed_origins: vec!["*".to_string()],
            api_key: None,
            rate_limit: RateLimitConfig::default(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            requests_per_day: 10000,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            code_scanning_enabled: true,
            dependency_scanning_enabled: true,
            malware_scanning_enabled: true,
            scan_timeout_seconds: 300, // 5分钟
            allowed_licenses: vec![
                "MIT".to_string(),
                "Apache-2.0".to_string(),
                "BSD-3-Clause".to_string(),
                "ISC".to_string(),
                "GPL-3.0".to_string(),
                "LGPL-3.0".to_string(),
            ],
            forbidden_keywords: vec![
                "eval".to_string(),
                "exec".to_string(),
                "system".to_string(),
                "shell".to_string(),
                "unsafe".to_string(),
            ],
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: 3600, // 1小时
            max_entries: 10000,
            preload_popular_tools: true,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            index_update_interval_seconds: 300, // 5分钟
            max_search_results: 100,
            fuzzy_search_enabled: true,
            fuzzy_threshold: 0.8,
            semantic_search_enabled: false, // 需要额外的ML模型
        }
    }
}

impl MarketplaceConfig {
    /// 从文件加载配置
    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)
            .map_err(|e| MarketplaceError::config(format!("无法读取配置文件 {:?}: {}", path, e)))?;
        
        let config: Self = toml::from_str(&content)
            .map_err(|e| MarketplaceError::config(format!("配置文件格式错误: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// 保存配置到文件
    pub fn save_to_file(&self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();
        let content = toml::to_string_pretty(self)
            .map_err(|e| MarketplaceError::config(format!("序列化配置失败: {}", e)))?;
        
        std::fs::write(&path, content)
            .map_err(|e| MarketplaceError::config(format!("写入配置文件失败 {:?}: {}", path, e)))?;
        
        Ok(())
    }
    
    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        // 验证数据库URL
        if self.database_url.is_empty() {
            return Err(MarketplaceError::config("数据库URL不能为空"));
        }
        
        // 验证端口范围
        if self.api.port == 0 {
            return Err(MarketplaceError::config("API端口不能为0"));
        }
        
        // 验证包大小限制
        if self.max_package_size == 0 {
            return Err(MarketplaceError::config("最大包大小不能为0"));
        }
        
        // 验证搜索配置
        if self.search.max_search_results == 0 {
            return Err(MarketplaceError::config("最大搜索结果数不能为0"));
        }
        
        if self.search.fuzzy_threshold < 0.0 || self.search.fuzzy_threshold > 1.0 {
            return Err(MarketplaceError::config("模糊搜索阈值必须在0.0-1.0之间"));
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = MarketplaceConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.api.port, 8080);
        assert!(config.security_scanning_enabled);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = MarketplaceConfig::default();
        
        // 测试无效的数据库URL
        config.database_url = "".to_string();
        assert!(config.validate().is_err());
        
        // 测试无效的端口
        config.database_url = "sqlite://test.db".to_string();
        config.api.port = 0;
        assert!(config.validate().is_err());
        
        // 测试无效的模糊搜索阈值
        config.api.port = 8080;
        config.search.fuzzy_threshold = 1.5;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_file_operations() {
        let config = MarketplaceConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        // 保存配置
        assert!(config.save_to_file(temp_file.path()).is_ok());
        
        // 加载配置
        let loaded_config = MarketplaceConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.api.port, loaded_config.api.port);
        assert_eq!(config.database_url, loaded_config.database_url);
    }
}
