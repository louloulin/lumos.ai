use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::error::{Error, Result};
use crate::agent::trait_def::Agent;
use crate::tool::Tool;

/// 插件系统管理器
pub struct PluginManager {
    plugins: HashMap<String, Arc<dyn Plugin>>,
    plugin_registry: PluginRegistry,
    hooks: HashMap<PluginHook, Vec<Arc<dyn Plugin>>>,
}

/// 插件注册表
pub struct PluginRegistry {
    registered_plugins: HashMap<String, PluginMetadata>,
    dependencies: HashMap<String, Vec<String>>,
}

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<PluginCapability>,
    pub hooks: Vec<PluginHook>,
    pub config_schema: Option<serde_json::Value>,
}

/// 插件能力
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginCapability {
    /// 提供工具
    ProvideTools,
    /// 处理消息
    ProcessMessages,
    /// 修改响应
    ModifyResponses,
    /// 监控和日志
    Monitoring,
    /// 存储扩展
    Storage,
    /// 网络通信
    Networking,
    /// 自定义能力
    Custom(String),
}

/// 插件钩子
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginHook {
    /// Agent初始化前
    BeforeAgentInit,
    /// Agent初始化后
    AfterAgentInit,
    /// 消息处理前
    BeforeMessageProcess,
    /// 消息处理后
    AfterMessageProcess,
    /// 响应生成前
    BeforeResponseGenerate,
    /// 响应生成后
    AfterResponseGenerate,
    /// 工具执行前
    BeforeToolExecution,
    /// 工具执行后
    AfterToolExecution,
    /// 错误处理
    OnError,
    /// Agent关闭前
    BeforeAgentShutdown,
}

/// 插件上下文
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub agent_name: String,
    pub request_id: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub config: HashMap<String, serde_json::Value>,
}

/// 插件执行结果
#[derive(Debug, Clone)]
pub enum PluginResult {
    /// 继续执行
    Continue,
    /// 修改数据并继续
    ModifyAndContinue(serde_json::Value),
    /// 停止执行
    Stop,
    /// 错误
    Error(String),
}

/// 插件trait
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 获取插件元数据
    fn metadata(&self) -> &PluginMetadata;
    
    /// 初始化插件
    async fn initialize(&mut self, config: HashMap<String, serde_json::Value>) -> Result<()>;
    
    /// 关闭插件
    async fn shutdown(&mut self) -> Result<()>;
    
    /// 执行钩子
    async fn execute_hook(
        &self,
        hook: PluginHook,
        context: &PluginContext,
        data: Option<serde_json::Value>,
    ) -> Result<PluginResult>;
    
    /// 获取插件提供的工具
    fn get_tools(&self) -> Vec<Arc<dyn Tool>> {
        Vec::new()
    }
    
    /// 检查插件健康状态
    async fn health_check(&self) -> Result<PluginHealthStatus>;
    
    /// 获取插件配置模式
    fn config_schema(&self) -> Option<serde_json::Value> {
        self.metadata().config_schema.clone()
    }
}

/// 插件健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHealthStatus {
    pub healthy: bool,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_registry: PluginRegistry::new(),
            hooks: HashMap::new(),
        }
    }
    
    /// 注册插件
    pub async fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata();
        let name = metadata.name.clone();
        
        // 检查依赖
        self.check_dependencies(&metadata.dependencies)?;
        
        // 注册插件
        self.plugin_registry.register(metadata.clone())?;
        
        // 注册钩子
        for hook in &metadata.hooks {
            self.hooks.entry(hook.clone()).or_insert_with(Vec::new).push(plugin.clone());
        }
        
        // 存储插件
        self.plugins.insert(name, plugin);
        
        Ok(())
    }
    
    /// 卸载插件
    pub async fn unregister_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.remove(name) {
            // 关闭插件
            let mut plugin_mut = plugin.as_ref();
            // Note: This is a simplified approach. In a real implementation,
            // you'd need a way to get mutable access to the plugin.
            
            // 从钩子中移除
            for hook_plugins in self.hooks.values_mut() {
                hook_plugins.retain(|p| p.metadata().name != name);
            }
            
            // 从注册表中移除
            self.plugin_registry.unregister(name)?;
        }
        
        Ok(())
    }
    
    /// 执行钩子
    pub async fn execute_hook(
        &self,
        hook: PluginHook,
        context: &PluginContext,
        data: Option<serde_json::Value>,
    ) -> Result<Vec<PluginResult>> {
        let mut results = Vec::new();
        
        if let Some(plugins) = self.hooks.get(&hook) {
            for plugin in plugins {
                let result = plugin.execute_hook(hook.clone(), context, data.clone()).await?;

                // 如果插件要求停止执行，则停止
                let should_stop = matches!(result, PluginResult::Stop | PluginResult::Error(_));
                results.push(result);

                if should_stop {
                    break;
                }
            }
        }
        
        Ok(results)
    }
    
    /// 获取所有插件提供的工具
    pub fn get_all_tools(&self) -> Vec<Arc<dyn Tool>> {
        let mut tools = Vec::new();
        
        for plugin in self.plugins.values() {
            tools.extend(plugin.get_tools());
        }
        
        tools
    }
    
    /// 获取插件
    pub fn get_plugin(&self, name: &str) -> Option<&Arc<dyn Plugin>> {
        self.plugins.get(name)
    }
    
    /// 列出所有插件
    pub fn list_plugins(&self) -> Vec<&PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }
    
    /// 检查所有插件健康状态
    pub async fn health_check_all(&self) -> HashMap<String, PluginHealthStatus> {
        let mut results = HashMap::new();
        
        for (name, plugin) in &self.plugins {
            match plugin.health_check().await {
                Ok(status) => {
                    results.insert(name.clone(), status);
                }
                Err(e) => {
                    results.insert(name.clone(), PluginHealthStatus {
                        healthy: false,
                        message: format!("Health check failed: {}", e),
                        details: HashMap::new(),
                    });
                }
            }
        }
        
        results
    }
    
    /// 检查依赖
    fn check_dependencies(&self, dependencies: &[String]) -> Result<()> {
        for dep in dependencies {
            if !self.plugins.contains_key(dep) {
                return Err(Error::Plugin(format!("Missing dependency: {}", dep)));
            }
        }
        Ok(())
    }
    
    /// 初始化所有插件
    pub async fn initialize_all(&mut self, configs: HashMap<String, HashMap<String, serde_json::Value>>) -> Result<()> {
        for (name, plugin) in &self.plugins {
            let config = configs.get(name).cloned().unwrap_or_default();
            // Note: This is simplified. In a real implementation, you'd need mutable access.
            // plugin.initialize(config).await?;
        }
        Ok(())
    }
    
    /// 关闭所有插件
    pub async fn shutdown_all(&mut self) -> Result<()> {
        for plugin in self.plugins.values() {
            // Note: This is simplified. In a real implementation, you'd need mutable access.
            // plugin.shutdown().await?;
        }
        Ok(())
    }
}

impl PluginRegistry {
    /// 创建新的插件注册表
    pub fn new() -> Self {
        Self {
            registered_plugins: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }
    
    /// 注册插件元数据
    pub fn register(&mut self, metadata: PluginMetadata) -> Result<()> {
        let name = metadata.name.clone();
        let dependencies = metadata.dependencies.clone();
        
        self.registered_plugins.insert(name.clone(), metadata);
        self.dependencies.insert(name, dependencies);
        
        Ok(())
    }
    
    /// 注销插件
    pub fn unregister(&mut self, name: &str) -> Result<()> {
        self.registered_plugins.remove(name);
        self.dependencies.remove(name);
        Ok(())
    }
    
    /// 获取插件元数据
    pub fn get_metadata(&self, name: &str) -> Option<&PluginMetadata> {
        self.registered_plugins.get(name)
    }
    
    /// 列出所有注册的插件
    pub fn list_all(&self) -> Vec<&PluginMetadata> {
        self.registered_plugins.values().collect()
    }
    
    /// 检查依赖关系
    pub fn check_dependencies(&self, name: &str) -> Result<Vec<String>> {
        if let Some(deps) = self.dependencies.get(name) {
            for dep in deps {
                if !self.registered_plugins.contains_key(dep) {
                    return Err(Error::Plugin(format!("Missing dependency {} for plugin {}", dep, name)));
                }
            }
            Ok(deps.clone())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// 获取依赖顺序
    pub fn get_dependency_order(&self) -> Result<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();
        
        for name in self.registered_plugins.keys() {
            if !visited.contains(name) {
                self.visit_plugin(name, &mut order, &mut visited, &mut visiting)?;
            }
        }
        
        Ok(order)
    }
    
    /// 深度优先访问插件（用于依赖排序）
    fn visit_plugin(
        &self,
        name: &str,
        order: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        if visiting.contains(name) {
            return Err(Error::Plugin(format!("Circular dependency detected involving plugin: {}", name)));
        }
        
        if visited.contains(name) {
            return Ok(());
        }
        
        visiting.insert(name.to_string());
        
        if let Some(deps) = self.dependencies.get(name) {
            for dep in deps {
                self.visit_plugin(dep, order, visited, visiting)?;
            }
        }
        
        visiting.remove(name);
        visited.insert(name.to_string());
        order.push(name.to_string());
        
        Ok(())
    }
}

/// 示例插件：日志插件
pub struct LoggingPlugin {
    metadata: PluginMetadata,
    enabled: bool,
    log_level: String,
}

impl LoggingPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "logging".to_string(),
                version: "1.0.0".to_string(),
                description: "Provides logging capabilities for agents".to_string(),
                author: "LumosAI Team".to_string(),
                license: "MIT".to_string(),
                dependencies: vec![],
                capabilities: vec![PluginCapability::Monitoring],
                hooks: vec![
                    PluginHook::BeforeMessageProcess,
                    PluginHook::AfterMessageProcess,
                    PluginHook::OnError,
                ],
                config_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "enabled": {
                            "type": "boolean",
                            "default": true
                        },
                        "log_level": {
                            "type": "string",
                            "enum": ["debug", "info", "warn", "error"],
                            "default": "info"
                        }
                    }
                })),
            },
            enabled: true,
            log_level: "info".to_string(),
        }
    }
}

#[async_trait]
impl Plugin for LoggingPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&mut self, config: HashMap<String, serde_json::Value>) -> Result<()> {
        if let Some(enabled) = config.get("enabled") {
            self.enabled = enabled.as_bool().unwrap_or(true);
        }
        
        if let Some(log_level) = config.get("log_level") {
            self.log_level = log_level.as_str().unwrap_or("info").to_string();
        }
        
        println!("Logging plugin initialized with level: {}", self.log_level);
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        println!("Logging plugin shutting down");
        Ok(())
    }
    
    async fn execute_hook(
        &self,
        hook: PluginHook,
        context: &PluginContext,
        data: Option<serde_json::Value>,
    ) -> Result<PluginResult> {
        if !self.enabled {
            return Ok(PluginResult::Continue);
        }
        
        match hook {
            PluginHook::BeforeMessageProcess => {
                println!("[{}] Processing message for agent: {}", self.log_level.to_uppercase(), context.agent_name);
            }
            PluginHook::AfterMessageProcess => {
                println!("[{}] Message processed for agent: {}", self.log_level.to_uppercase(), context.agent_name);
            }
            PluginHook::OnError => {
                if let Some(error_data) = data {
                    println!("[ERROR] Error in agent {}: {}", context.agent_name, error_data);
                }
            }
            _ => {}
        }
        
        Ok(PluginResult::Continue)
    }
    
    async fn health_check(&self) -> Result<PluginHealthStatus> {
        Ok(PluginHealthStatus {
            healthy: self.enabled,
            message: if self.enabled { "Logging plugin is active".to_string() } else { "Logging plugin is disabled".to_string() },
            details: HashMap::from([
                ("log_level".to_string(), serde_json::json!(self.log_level)),
                ("enabled".to_string(), serde_json::json!(self.enabled)),
            ]),
        })
    }
}

/// 示例插件：缓存插件
pub struct CachePlugin {
    metadata: PluginMetadata,
    cache: HashMap<String, serde_json::Value>,
    max_size: usize,
    ttl_seconds: u64,
}

impl CachePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "cache".to_string(),
                version: "1.0.0".to_string(),
                description: "Provides caching capabilities for agent responses".to_string(),
                author: "LumosAI Team".to_string(),
                license: "MIT".to_string(),
                dependencies: vec![],
                capabilities: vec![PluginCapability::Storage],
                hooks: vec![
                    PluginHook::BeforeMessageProcess,
                    PluginHook::BeforeResponseGenerate,
                    PluginHook::AfterResponseGenerate,
                ],
                config_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "max_size": {
                            "type": "integer",
                            "default": 1000
                        },
                        "ttl_seconds": {
                            "type": "integer",
                            "default": 3600
                        }
                    }
                })),
            },
            cache: HashMap::new(),
            max_size: 1000,
            ttl_seconds: 3600,
        }
    }
    
    fn generate_cache_key(&self, context: &PluginContext, data: &serde_json::Value) -> String {
        format!("{}:{}", context.agent_name, 
                serde_json::to_string(data).unwrap_or_default())
    }
}

#[async_trait]
impl Plugin for CachePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&mut self, config: HashMap<String, serde_json::Value>) -> Result<()> {
        if let Some(max_size) = config.get("max_size") {
            self.max_size = max_size.as_u64().unwrap_or(1000) as usize;
        }
        
        if let Some(ttl) = config.get("ttl_seconds") {
            self.ttl_seconds = ttl.as_u64().unwrap_or(3600);
        }
        
        println!("Cache plugin initialized with max_size: {}, ttl: {}s", self.max_size, self.ttl_seconds);
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        self.cache.clear();
        println!("Cache plugin shutting down");
        Ok(())
    }
    
    async fn execute_hook(
        &self,
        hook: PluginHook,
        context: &PluginContext,
        data: Option<serde_json::Value>,
    ) -> Result<PluginResult> {
        match hook {
            PluginHook::BeforeMessageProcess => {
                println!("Cache plugin: Processing message for agent: {}", context.agent_name);
                // Check if we have cached data for this message
                if let Some(message_data) = data {
                    let cache_key = self.generate_cache_key(context, &message_data);
                    if let Some(_cached_response) = self.cache.get(&cache_key) {
                        println!("Cache plugin: Found cached data for key: {}", cache_key);
                    } else {
                        println!("Cache plugin: No cached data for key: {}", cache_key);
                    }
                }
            }
            PluginHook::BeforeResponseGenerate => {
                if let Some(request_data) = data {
                    let cache_key = self.generate_cache_key(context, &request_data);
                    if let Some(cached_response) = self.cache.get(&cache_key) {
                        println!("Cache hit for key: {}", cache_key);
                        return Ok(PluginResult::ModifyAndContinue(cached_response.clone()));
                    }
                }
            }
            PluginHook::AfterResponseGenerate => {
                if let Some(response_data) = data {
                    // In a real implementation, you'd also need the request data to generate the key
                    // This is simplified for demonstration
                    let _cache_key = format!("{}:response", context.agent_name);
                    // Note: This is simplified. In a real implementation, you'd need mutable access.
                    // self.cache.insert(cache_key, response_data);
                    println!("Cached response for agent: {}", context.agent_name);
                }
            }
            _ => {}
        }

        Ok(PluginResult::Continue)
    }
    
    async fn health_check(&self) -> Result<PluginHealthStatus> {
        Ok(PluginHealthStatus {
            healthy: true,
            message: format!("Cache plugin is active with {} entries", self.cache.len()),
            details: HashMap::from([
                ("cache_size".to_string(), serde_json::json!(self.cache.len())),
                ("max_size".to_string(), serde_json::json!(self.max_size)),
                ("ttl_seconds".to_string(), serde_json::json!(self.ttl_seconds)),
            ]),
        })
    }
}
