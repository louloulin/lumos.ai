//! Configuration types and traits for LumosAI
//!
//! Provides type-safe configuration structures and validation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use validator::Validate;

/// Trait for configuration types that can be loaded and validated
pub trait ConfigType: for<'de> Deserialize<'de> + Serialize + Validate + Send + Sync {
    /// Get the configuration schema version
    fn schema_version() -> &'static str {
        "1.0"
    }

    /// Get default configuration name/namespace
    fn config_name() -> &'static str;

    /// Validate configuration after loading
    fn validate_custom(&self) -> crate::Result<()> {
        Ok(())
    }
}

/// Main LumosAI configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LumosaiConfig {
    /// Application metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectConfig>,

    /// Agent configuration
    #[serde(default)]
    pub agents: AgentsConfig,

    /// LLM provider configuration
    #[serde(default)]
    pub llm: LlmConfig,

    /// Tool configuration
    #[serde(default)]
    pub tools: ToolsConfig,

    /// Storage configuration
    #[serde(default)]
    pub storage: StorageConfig,

    /// Monitoring and telemetry configuration
    #[serde(default)]
    pub monitoring: MonitoringConfig,

    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,

    /// Network configuration
    #[serde(default)]
    pub network: NetworkConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl ConfigType for LumosaiConfig {
    fn config_name() -> &'static str {
        "lumosai"
    }
}

impl Default for LumosaiConfig {
    fn default() -> Self {
        Self {
            project: None,
            agents: AgentsConfig::default(),
            llm: LlmConfig::default(),
            tools: ToolsConfig::default(),
            storage: StorageConfig::default(),
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
            cache: CacheConfig::default(),
            network: NetworkConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

/// Project metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProjectConfig {
    /// Project name
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    /// Project version
    pub version: String,

    /// Project description
    #[validate(length(max = 500))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Project author
    #[validate(length(max = 100))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Project homepage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentsConfig {
    /// Default agent settings
    #[serde(default)]
    pub defaults: AgentDefaultsConfig,

    /// Specific agent configurations
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub agents: HashMap<String, AgentConfig>,
}

/// Default agent settings
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AgentDefaultsConfig {
    /// Default LLM model
    #[validate(length(min = 1))]
    #[serde(default = "default_model")]
    pub model: String,

    /// Default temperature
    #[validate(range(min = 0.0, max = 2.0))]
    #[serde(default = "default_temperature")]
    pub temperature: f64,

    /// Default max tokens
    #[validate(range(min = 1))]
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Default timeout in seconds
    #[validate(range(min = 1))]
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_model() -> String {
    "gpt-3.5-turbo".to_string()
}

fn default_temperature() -> f64 {
    0.7
}

fn default_max_tokens() -> u32 {
    1000
}

fn default_timeout() -> u64 {
    30
}

impl Default for AgentDefaultsConfig {
    fn default() -> Self {
        Self {
            model: default_model(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            timeout: default_timeout(),
        }
    }
}

/// Individual agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AgentConfig {
    /// Agent name
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    /// Agent instructions/prompt
    #[validate(length(min = 1, max = 10000))]
    pub instructions: String,

    /// LLM model (overrides default)
    #[validate(length(min = 1))]
    #[serde(default = "default_model")]
    pub model: String,

    /// Temperature (overrides default)
    #[validate(range(min = 0.0, max = 2.0))]
    #[serde(default = "default_temperature")]
    pub temperature: f64,

    /// Max tokens (overrides default)
    #[validate(range(min = 1))]
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Available tools for this agent
    #[serde(default)]
    pub tools: Vec<String>,

    /// Enable function calling
    #[serde(default = "default_function_calling")]
    pub enable_function_calling: bool,

    /// Timeout in seconds
    #[validate(range(min = 1))]
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_function_calling() -> bool {
    true
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmConfig {
    /// Default provider
    #[serde(default = "default_provider")]
    pub default_provider: String,

    /// Provider-specific configurations
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub providers: HashMap<String, ProviderConfig>,

    /// API key configuration
    #[serde(default)]
    pub api_keys: ApiKeyConfig,

    /// Rate limiting
    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,
}

fn default_provider() -> String {
    "openai".to_string()
}

/// Individual LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProviderConfig {
    /// Provider name
    #[validate(length(min = 1))]
    pub name: String,

    /// API base URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// API version
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Region/location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Custom headers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,

    /// Connection settings
    #[serde(default)]
    pub connection: ConnectionConfig,
}

/// Connection configuration for external services
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectionConfig {
    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub timeout: u64,

    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Retry delay in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,

    /// Keep-alive setting
    #[serde(default)]
    pub keep_alive: bool,
}

fn default_connect_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    1000
}

/// API key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Environment variable names for API keys
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env_vars: HashMap<String, String>,

    /// Encrypted API keys storage
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub encrypted: HashMap<String, String>,

    /// Key rotation settings
    #[serde(default)]
    pub rotation: Option<KeyRotationConfig>,
}

/// Key rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationConfig {
    /// Enable automatic rotation
    #[serde(default)]
    pub enabled: bool,

    /// Rotation interval in hours
    #[serde(default)]
    pub interval_hours: u32,

    /// Rotation webhook URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RateLimitConfig {
    /// Requests per minute
    #[validate(range(min = 1))]
    pub requests_per_minute: u32,

    /// Tokens per minute
    #[validate(range(min = 1))]
    pub tokens_per_minute: u32,

    /// Concurrent requests
    #[validate(range(min = 1))]
    pub concurrent_requests: u32,
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsConfig {
    /// Built-in tools configuration
    #[serde(default)]
    pub builtin: BuiltinToolsConfig,

    /// Custom tools
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, CustomToolConfig>,
}

/// Built-in tools configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuiltinToolsConfig {
    /// Enable web tools (HTTP, scraping, etc.)
    #[serde(default = "default_web_tools")]
    pub web: bool,

    /// Enable file system tools
    #[serde(default = "default_file_tools")]
    pub file_system: bool,

    /// Enable database tools
    #[serde(default)]
    pub database: bool,

    /// Enable data processing tools
    #[serde(default = "default_data_tools")]
    pub data_processing: bool,
}

fn default_web_tools() -> bool {
    true
}

fn default_file_tools() -> bool {
    true
}

fn default_data_tools() -> bool {
    true
}

/// Custom tool configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CustomToolConfig {
    /// Tool name
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    /// Tool description
    #[validate(length(min = 1, max = 500))]
    pub description: String,

    /// Tool type (function, command, etc.)
    #[validate(length(min = 1))]
    pub tool_type: String,

    /// Tool configuration
    #[serde(default)]
    pub config: serde_json::Value,

    /// Enable this tool
    #[serde(default = "default_tool_enabled")]
    pub enabled: bool,
}

fn default_tool_enabled() -> bool {
    true
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageConfig {
    /// Default storage backend
    #[serde(default = "default_storage_backend")]
    pub default_backend: String,

    /// Storage backend configurations
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub backends: HashMap<String, StorageBackendConfig>,

    /// Vector storage configuration
    #[serde(default)]
    pub vector: VectorStorageConfig,
}

fn default_storage_backend() -> String {
    "memory".to_string()
}

/// Individual storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct StorageBackendConfig {
    /// Backend type (memory, file, redis, etc.)
    #[validate(length(min = 1))]
    pub backend_type: String,

    /// Connection string or path
    #[validate(length(min = 1))]
    pub connection: String,

    /// Additional backend-specific settings
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

/// Vector storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorStorageConfig {
    /// Embedding provider
    #[serde(default = "default_embedding_provider")]
    pub embedding_provider: String,

    /// Default vector dimension
    #[serde(default = "default_vector_dimension")]
    pub dimension: usize,

    /// Similarity metric
    #[serde(default = "default_similarity_metric")]
    pub similarity_metric: String,

    /// Index settings
    #[serde(default)]
    pub index: IndexConfig,
}

fn default_embedding_provider() -> String {
    "openai".to_string()
}

fn default_vector_dimension() -> usize {
    1536
}

fn default_similarity_metric() -> String {
    "cosine".to_string()
}

/// Vector index configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexConfig {
    /// Index type (flat, hnsw, ivf, etc.)
    #[serde(default = "default_index_type")]
    pub index_type: String,

    /// Index parameters
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

fn default_index_type() -> String {
    "flat".to_string()
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringConfig {
    /// Enable monitoring
    #[serde(default)]
    pub enabled: bool,

    /// Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,

    /// Tracing configuration
    #[serde(default)]
    pub tracing: TracingConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Health check configuration
    #[serde(default)]
    pub health_check: HealthCheckConfig,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    #[serde(default)]
    pub enabled: bool,

    /// Metrics backend (prometheus, statsd, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,

    /// Export interval in seconds
    #[serde(default = "default_metrics_interval")]
    pub export_interval: u64,

    /// Metrics endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

fn default_metrics_interval() -> u64 {
    60
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable distributed tracing
    #[serde(default)]
    pub enabled: bool,

    /// Sampling rate (0.0 to 1.0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling_rate: Option<f64>,

    /// Tracing backend (jaeger, zipkin, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,

    /// Service name for tracing
    #[serde(default = "default_service_name")]
    pub service_name: String,
}

fn default_service_name() -> String {
    "lumosai".to_string()
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    #[serde(default = "default_health_check")]
    pub enabled: bool,

    /// Health check interval in seconds
    #[serde(default = "default_health_interval")]
    pub interval: u64,

    /// Health check timeout in seconds
    #[serde(default = "default_health_timeout")]
    pub timeout: u64,

    /// Health check endpoint path
    #[serde(default = "default_health_path")]
    pub path: String,
}

fn default_health_check() -> bool {
    true
}

fn default_health_interval() -> u64 {
    30
}

fn default_health_timeout() -> u64 {
    5
}

fn default_health_path() -> String {
    "/health".to_string()
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,

    /// Authorization configuration
    #[serde(default)]
    pub authorization: AuthorizationConfig,

    /// Encryption configuration
    #[serde(default)]
    pub encryption: EncryptionConfig,

    /// Audit configuration
    #[serde(default)]
    pub audit: AuditConfig,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    #[serde(default)]
    pub enabled: bool,

    /// Auth type (jwt, oauth2, api-key, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_type: Option<String>,

    /// JWT configuration
    #[serde(default)]
    pub jwt: JwtConfig,

    /// OAuth2 configuration
    #[serde(default)]
    pub oauth2: OAuth2Config,
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Secret key (can be file path or direct value)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,

    /// Token expiration time in seconds
    #[serde(default = "default_jwt_expiration")]
    pub expiration: u64,

    /// Refresh token expiration in seconds
    #[serde(default = "default_refresh_expiration")]
    pub refresh_expiration: u64,
}

fn default_jwt_expiration() -> u64 {
    3600 // 1 hour
}

fn default_refresh_expiration() -> u64 {
    86400 // 24 hours
}

/// OAuth2 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// Client ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// Client secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// Authorization URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_url: Option<String>,

    /// Token URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_url: Option<String>,
}

/// Authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    /// Enable authorization
    #[serde(default)]
    pub enabled: bool,

    /// Authorization type (rbac, abac, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authz_type: Option<String>,

    /// Default permissions
    #[serde(default)]
    pub default_permissions: Vec<String>,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption
    #[serde(default)]
    pub enabled: bool,

    /// Encryption algorithm
    #[serde(default = "default_encryption_algorithm")]
    pub algorithm: String,

    /// Key management
    #[serde(default)]
    pub key_management: KeyManagementConfig,
}

fn default_encryption_algorithm() -> String {
    "aes-256-gcm".to_string()
}

/// Key management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagementConfig {
    /// Key source (file, vault, env, etc.)
    #[serde(default = "default_key_source")]
    pub source: String,

    /// Key rotation interval in hours
    #[serde(default)]
    pub rotation_interval: u32,
}

fn default_key_source() -> String {
    "env".to_string()
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    #[serde(default)]
    pub enabled: bool,

    /// Audit log destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,

    /// Events to audit
    #[serde(default)]
    pub events: Vec<String>,

    /// Retention period in days
    #[serde(default = "default_audit_retention")]
    pub retention_days: u32,
}

fn default_audit_retention() -> u32 {
    90
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheConfig {
    /// Enable caching
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,

    /// Cache backend (memory, redis, etc.)
    #[serde(default = "default_cache_backend")]
    pub backend: String,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl: u64,

    /// Maximum cache size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<usize>,
}

fn default_cache_enabled() -> bool {
    true
}

fn default_cache_backend() -> String {
    "memory".to_string()
}

fn default_cache_ttl() -> u64 {
    300 // 5 minutes
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkConfig {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,

    /// Client configuration
    #[serde(default)]
    pub client: ClientConfig,

    /// Proxy configuration
    #[serde(default)]
    pub proxy: ProxyConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host
    #[serde(default = "default_server_host")]
    pub host: String,

    /// Server port
    #[serde(default = "default_server_port")]
    pub port: u16,

    /// Enable HTTPS
    #[serde(default)]
    pub https: bool,

    /// SSL/TLS configuration
    #[serde(default)]
    pub tls: TlsConfig,
}

fn default_server_host() -> String {
    "0.0.0.0".to_string()
}

fn default_server_port() -> u16 {
    8080
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cert_file: Option<PathBuf>,

    /// Private key file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_file: Option<PathBuf>,

    /// CA certificate file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_file: Option<PathBuf>,
}

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Request timeout in seconds
    #[serde(default = "default_client_timeout")]
    pub timeout: u64,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,

    /// Keep-alive duration in seconds
    #[serde(default = "default_keep_alive")]
    pub keep_alive: u64,
}

fn default_client_timeout() -> u64 {
    30
}

fn default_pool_size() -> usize {
    10
}

fn default_keep_alive() -> u64 {
    60
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Enable proxy
    #[serde(default)]
    pub enabled: bool,

    /// Proxy URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Proxy authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<ProxyAuthConfig>,
}

/// Proxy authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuthConfig {
    /// Username
    pub username: String,

    /// Password
    pub password: String,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format (json, text, etc.)
    #[serde(default = "default_log_format")]
    pub format: String,

    /// Log output destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    /// Enable colored output
    #[serde(default)]
    pub colored: bool,

    /// Log file rotation
    #[serde(default)]
    pub rotation: LogRotationConfig,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Enable log rotation
    #[serde(default)]
    pub enabled: bool,

    /// Maximum file size in MB
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,

    /// Maximum number of backup files
    #[serde(default = "default_max_files")]
    pub max_files: u32,
}

fn default_max_file_size() -> u64 {
    100 // 100MB
}

fn default_max_files() -> u32 {
    5
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            env_vars: HashMap::new(),
            encrypted: HashMap::new(),
            rotation: None,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: None,
            export_interval: default_metrics_interval(),
            endpoint: None,
        }
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sampling_rate: None,
            backend: None,
            service_name: default_service_name(),
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: default_health_check(),
            interval: default_health_interval(),
            timeout: default_health_timeout(),
            path: default_health_path(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auth_type: None,
            jwt: JwtConfig::default(),
            oauth2: OAuth2Config::default(),
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: None,
            expiration: default_jwt_expiration(),
            refresh_expiration: default_refresh_expiration(),
        }
    }
}

impl Default for OAuth2Config {
    fn default() -> Self {
        Self {
            client_id: None,
            client_secret: None,
            auth_url: None,
            token_url: None,
        }
    }
}

impl Default for AuthorizationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            authz_type: None,
            default_permissions: Vec::new(),
        }
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: default_encryption_algorithm(),
            key_management: KeyManagementConfig::default(),
        }
    }
}

impl Default for KeyManagementConfig {
    fn default() -> Self {
        Self {
            source: default_key_source(),
            rotation_interval: 0,
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            destination: None,
            events: Vec::new(),
            retention_days: default_audit_retention(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_server_host(),
            port: default_server_port(),
            https: false,
            tls: TlsConfig::default(),
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            cert_file: None,
            key_file: None,
            ca_file: None,
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: default_client_timeout(),
            pool_size: default_pool_size(),
            keep_alive: default_keep_alive(),
        }
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: None,
            auth: None,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            output: None,
            colored: true,
            rotation: LogRotationConfig::default(),
        }
    }
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_file_size: default_max_file_size(),
            max_files: default_max_files(),
        }
    }
}

// Environment variable constants
pub const ENV_OPENAI_API_KEY: &str = "OPENAI_API_KEY";
pub const ENV_ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";
pub const ENV_DEEPSEEK_API_KEY: &str = "DEEPSEEK_API_KEY";
pub const ENV_ZHIPU_API_KEY: &str = "ZHIPU_API_KEY";
pub const ENV_QWEN_API_KEY: &str = "QWEN_API_KEY";
pub const ENV_LUMOSAI_LOG_LEVEL: &str = "LUMOSAI_LOG_LEVEL";
pub const ENV_LUMOSAI_CONFIG_PATH: &str = "LUMOSAI_CONFIG_PATH";
pub const ENV_LUMOSAI_CACHE_DIR: &str = "LUMOSAI_CACHE_DIR";

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_lumosai_config_validation() {
        let config = LumosaiConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_agent_config_validation() {
        let agent = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant".to_string(),
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
            tools: vec![],
            enable_function_calling: true,
            timeout: 30,
        };

        assert!(agent.validate().is_ok());
    }

    #[test]
    fn test_project_config_validation() {
        let project = ProjectConfig {
            name: "test-project".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test project".to_string()),
            author: Some("Test Author".to_string()),
            homepage: Some("https://example.com".to_string()),
        };

        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_invalid_temperature() {
        let agent = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are helpful".to_string(),
            model: "gpt-4".to_string(),
            temperature: 3.0, // Invalid: > 2.0
            max_tokens: 1000,
            tools: vec![],
            enable_function_calling: true,
            timeout: 30,
        };

        assert!(agent.validate().is_err());
    }
}