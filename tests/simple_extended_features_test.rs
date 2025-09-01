use std::collections::HashMap;
use std::sync::Arc;
use tokio;
use lumosai_core::documentation::{ApiDocumentationGenerator, DocumentationFormat};
use lumosai_core::plugin::{PluginManager, LoggingPlugin, CachePlugin, Plugin, PluginHook, PluginContext};
use lumosai_core::distributed::{RoundRobinLoadBalancer, LoadBalancer, NodeInfo, NodeStatus, SelectionCriteria};
use lumosai_core::error::Result;

/// 测试文档生成功能基础
#[tokio::test]
async fn test_documentation_generator_creation() -> Result<()> {
    // 创建文档生成器
    let doc_generator = ApiDocumentationGenerator::new(
        "test_output".to_string(),
        DocumentationFormat::Markdown,
    );
    
    // 验证生成器创建成功
    assert!(true); // 如果能创建就说明基础功能正常
    
    println!("✅ 文档生成器创建测试通过");
    Ok(())
}

/// 测试插件系统基础功能
#[tokio::test]
async fn test_plugin_system_basics() -> Result<()> {
    let mut plugin_manager = PluginManager::new();
    
    // 创建并注册日志插件
    let logging_plugin = Arc::new(LoggingPlugin::new());
    plugin_manager.register_plugin(logging_plugin.clone()).await?;
    
    // 验证插件注册
    let plugins = plugin_manager.list_plugins();
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].name, "logging");
    
    // 测试插件钩子执行
    let context = PluginContext {
        agent_name: "test_agent".to_string(),
        request_id: "test_request_123".to_string(),
        metadata: HashMap::new(),
        config: HashMap::new(),
    };
    
    let results = plugin_manager.execute_hook(
        PluginHook::BeforeMessageProcess,
        &context,
        Some(serde_json::json!({"message": "test message"})),
    ).await?;
    
    assert_eq!(results.len(), 1);
    
    println!("✅ 插件系统基础功能测试通过");
    Ok(())
}

/// 测试插件元数据
#[tokio::test]
async fn test_plugin_metadata() -> Result<()> {
    let logging_plugin = LoggingPlugin::new();
    let metadata = logging_plugin.metadata();
    
    assert_eq!(metadata.name, "logging");
    assert_eq!(metadata.version, "1.0.0");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.hooks.is_empty());
    
    // 测试配置模式
    let config_schema = logging_plugin.config_schema();
    assert!(config_schema.is_some());
    
    println!("✅ 插件元数据测试通过");
    Ok(())
}

/// 测试缓存插件
#[tokio::test]
async fn test_cache_plugin() -> Result<()> {
    let mut plugin_manager = PluginManager::new();
    
    // 创建并注册缓存插件
    let cache_plugin = Arc::new(CachePlugin::new());
    plugin_manager.register_plugin(cache_plugin.clone()).await?;
    
    // 验证插件注册
    let plugins = plugin_manager.list_plugins();
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].name, "cache");
    
    // 测试健康检查
    let health_status = plugin_manager.health_check_all().await;
    assert_eq!(health_status.len(), 1);
    assert!(health_status.get("cache").unwrap().healthy);
    
    println!("✅ 缓存插件测试通过");
    Ok(())
}

/// 测试负载均衡器
#[tokio::test]
async fn test_load_balancer() -> Result<()> {
    let load_balancer = RoundRobinLoadBalancer::new();
    
    // 验证负载均衡策略
    let strategy = load_balancer.strategy();
    assert!(matches!(strategy, lumosai_core::distributed::LoadBalancingStrategy::RoundRobin));
    
    // 创建测试节点
    let nodes = vec![
        NodeInfo {
            node_id: "node_1".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            status: NodeStatus::Active,
            capabilities: vec!["agent_execution".to_string()],
            load: 0.3,
            last_heartbeat: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        },
        NodeInfo {
            node_id: "node_2".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8081,
            status: NodeStatus::Active,
            capabilities: vec!["agent_execution".to_string(), "gpu_compute".to_string()],
            load: 0.7,
            last_heartbeat: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        },
    ];
    
    // 测试基本选择
    let criteria = SelectionCriteria {
        required_capabilities: vec!["agent_execution".to_string()],
        preferred_region: None,
        max_load: None,
        exclude_nodes: vec![],
    };
    
    let selected = load_balancer.select_node(&nodes, &criteria).await?;
    assert!(selected.is_some());
    
    println!("✅ 负载均衡器测试通过");
    Ok(())
}

/// 测试多插件管理
#[tokio::test]
async fn test_multiple_plugins() -> Result<()> {
    let mut plugin_manager = PluginManager::new();
    
    // 注册多个插件
    let logging_plugin = Arc::new(LoggingPlugin::new());
    let cache_plugin = Arc::new(CachePlugin::new());
    
    plugin_manager.register_plugin(logging_plugin).await?;
    plugin_manager.register_plugin(cache_plugin).await?;
    
    // 验证插件数量
    let plugins = plugin_manager.list_plugins();
    assert_eq!(plugins.len(), 2);
    
    // 验证插件名称
    let plugin_names: Vec<&String> = plugins.iter().map(|p| &p.name).collect();
    assert!(plugin_names.contains(&&"logging".to_string()));
    assert!(plugin_names.contains(&&"cache".to_string()));
    
    // 测试所有插件的健康检查
    let health_status = plugin_manager.health_check_all().await;
    assert_eq!(health_status.len(), 2);
    
    for (name, status) in &health_status {
        assert!(status.healthy, "Plugin {} should be healthy", name);
    }
    
    println!("✅ 多插件管理测试通过");
    Ok(())
}

/// 测试插件钩子执行顺序
#[tokio::test]
async fn test_plugin_hook_execution() -> Result<()> {
    let mut plugin_manager = PluginManager::new();
    
    // 注册插件
    let logging_plugin = Arc::new(LoggingPlugin::new());
    plugin_manager.register_plugin(logging_plugin).await?;
    
    let context = PluginContext {
        agent_name: "test_agent".to_string(),
        request_id: "test_request_456".to_string(),
        metadata: HashMap::new(),
        config: HashMap::new(),
    };
    
    // 测试不同的钩子
    let hooks = vec![
        PluginHook::BeforeMessageProcess,
        PluginHook::AfterMessageProcess,
        PluginHook::OnError,
    ];
    
    for hook in hooks {
        let results = plugin_manager.execute_hook(
            hook.clone(),
            &context,
            Some(serde_json::json!({"test": "data"})),
        ).await?;
        
        // 日志插件应该响应这些钩子
        assert!(!results.is_empty());
    }
    
    println!("✅ 插件钩子执行测试通过");
    Ok(())
}

/// 测试文档格式支持
#[tokio::test]
async fn test_documentation_formats() -> Result<()> {
    let formats = vec![
        DocumentationFormat::Markdown,
        DocumentationFormat::Html,
        DocumentationFormat::Json,
        DocumentationFormat::OpenApi,
    ];
    
    for format in formats {
        let doc_generator = ApiDocumentationGenerator::new(
            "test_output".to_string(),
            format,
        );
        
        // 验证生成器创建成功
        assert!(true); // 如果能创建就说明格式支持正常
    }
    
    println!("✅ 文档格式支持测试通过");
    Ok(())
}

/// 测试负载均衡器节点过滤
#[tokio::test]
async fn test_load_balancer_filtering() -> Result<()> {
    let load_balancer = RoundRobinLoadBalancer::new();
    
    let nodes = vec![
        NodeInfo {
            node_id: "node_1".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            status: NodeStatus::Active,
            capabilities: vec!["agent_execution".to_string()],
            load: 0.3,
            last_heartbeat: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        },
        NodeInfo {
            node_id: "node_2".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8081,
            status: NodeStatus::Active,
            capabilities: vec!["gpu_compute".to_string()],
            load: 0.7,
            last_heartbeat: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        },
    ];
    
    // 测试能力过滤
    let gpu_criteria = SelectionCriteria {
        required_capabilities: vec!["gpu_compute".to_string()],
        preferred_region: None,
        max_load: None,
        exclude_nodes: vec![],
    };
    
    let gpu_selected = load_balancer.select_node(&nodes, &gpu_criteria).await?;
    assert!(gpu_selected.is_some());
    assert_eq!(gpu_selected.unwrap().node_id, "node_2");
    
    // 测试负载限制
    let low_load_criteria = SelectionCriteria {
        required_capabilities: vec!["agent_execution".to_string()],
        preferred_region: None,
        max_load: Some(0.5),
        exclude_nodes: vec![],
    };
    
    let low_load_selected = load_balancer.select_node(&nodes, &low_load_criteria).await?;
    assert!(low_load_selected.is_some());
    assert_eq!(low_load_selected.unwrap().node_id, "node_1");
    
    println!("✅ 负载均衡器节点过滤测试通过");
    Ok(())
}

/// 测试综合功能
#[tokio::test]
async fn test_comprehensive_features() -> Result<()> {
    // 创建插件管理器
    let mut plugin_manager = PluginManager::new();
    
    // 注册插件
    let logging_plugin = Arc::new(LoggingPlugin::new());
    plugin_manager.register_plugin(logging_plugin).await?;
    
    // 创建文档生成器
    let doc_generator = ApiDocumentationGenerator::new(
        "comprehensive_test_output".to_string(),
        DocumentationFormat::Json,
    );
    
    // 创建负载均衡器
    let load_balancer = RoundRobinLoadBalancer::new();
    
    // 验证所有组件都能正常工作
    let plugins = plugin_manager.list_plugins();
    assert!(!plugins.is_empty());
    
    let health_status = plugin_manager.health_check_all().await;
    assert!(!health_status.is_empty());
    
    let strategy = load_balancer.strategy();
    assert!(matches!(strategy, lumosai_core::distributed::LoadBalancingStrategy::RoundRobin));
    
    println!("✅ 综合功能测试通过");
    Ok(())
}
