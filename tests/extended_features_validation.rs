use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use lumosai_core::agent::executor::BasicAgent;
use lumosai_core::agent::trait_def::{Agent, AgentStatus};
use lumosai_core::agent::config::AgentConfig;
use lumosai_core::llm::mock::MockLlmProvider;
use lumosai_core::documentation::{ApiDocumentationGenerator, DocumentationFormat};
use lumosai_core::plugin::{PluginManager, LoggingPlugin, CachePlugin, Plugin, PluginHook, PluginContext};
use lumosai_core::distributed::{DistributedAgentManager, ClusterConfig, RoundRobinLoadBalancer};
use lumosai_core::error::Result;

/// 测试文档生成功能
#[tokio::test]
async fn test_documentation_generation() -> Result<()> {
    // 创建测试Agent
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Hello! I'm a test agent.".to_string(),
    ]));
    
    let config = AgentConfig {
        name: "test_agent".to_string(),
        instructions: "You are a helpful test assistant.".to_string(),
        model_id: Some("test-model".to_string()),
        memory_config: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(true),
        context: None,
        metadata: None,
        max_tool_calls: Some(10),
        tool_timeout: Some(30),
    };
    
    let agent = BasicAgent::new(config, llm);
    
    // 创建文档生成器
    let doc_generator = ApiDocumentationGenerator::new(
        "test_output".to_string(),
        DocumentationFormat::Markdown,
    );
    
    // 生成文档
    let documentation = doc_generator.generate_agent_documentation(&agent).await?;
    
    // 验证文档结构
    assert!(!documentation.title.is_empty());
    assert!(!documentation.description.is_empty());
    assert!(!documentation.endpoints.is_empty());
    
    // 验证核心端点存在
    let endpoint_paths: Vec<&String> = documentation.endpoints.iter()
        .map(|e| &e.path)
        .collect();
    
    assert!(endpoint_paths.contains(&&"/api/v1/generate".to_string()));
    assert!(endpoint_paths.contains(&&"/api/v1/stream".to_string()));
    assert!(endpoint_paths.contains(&&"/api/v1/health".to_string()));
    assert!(endpoint_paths.contains(&&"/api/v1/metrics".to_string()));
    
    println!("✅ 文档生成功能测试通过");
    Ok(())
}

/// 测试插件系统功能
#[tokio::test]
async fn test_plugin_system() -> Result<()> {
    let mut plugin_manager = PluginManager::new();
    
    // 创建并注册日志插件
    let logging_plugin = Arc::new(LoggingPlugin::new());
    plugin_manager.register_plugin(logging_plugin.clone()).await?;
    
    // 创建并注册缓存插件
    let cache_plugin = Arc::new(CachePlugin::new());
    plugin_manager.register_plugin(cache_plugin.clone()).await?;
    
    // 验证插件注册
    let plugins = plugin_manager.list_plugins();
    assert_eq!(plugins.len(), 2);
    
    let plugin_names: Vec<&String> = plugins.iter().map(|p| &p.name).collect();
    assert!(plugin_names.contains(&&"logging".to_string()));
    assert!(plugin_names.contains(&&"cache".to_string()));
    
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
    
    assert_eq!(results.len(), 2); // 两个插件都应该响应这个钩子
    
    // 测试健康检查
    let health_status = plugin_manager.health_check_all().await;
    assert_eq!(health_status.len(), 2);
    
    for (name, status) in &health_status {
        assert!(status.healthy, "Plugin {} should be healthy", name);
    }
    
    println!("✅ 插件系统功能测试通过");
    Ok(())
}

/// 测试插件配置和初始化
#[tokio::test]
async fn test_plugin_configuration() -> Result<()> {
    let mut plugin_manager = PluginManager::new();
    
    // 创建日志插件
    let logging_plugin = Arc::new(LoggingPlugin::new());
    plugin_manager.register_plugin(logging_plugin.clone()).await?;
    
    // 测试插件配置模式
    let config_schema = logging_plugin.config_schema();
    assert!(config_schema.is_some());
    
    let schema = config_schema.unwrap();
    assert!(schema.get("type").is_some());
    assert!(schema.get("properties").is_some());
    
    // 验证配置属性
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("enabled"));
    assert!(properties.contains_key("log_level"));
    
    println!("✅ 插件配置功能测试通过");
    Ok(())
}

/// 测试分布式系统基础功能
#[tokio::test]
async fn test_distributed_system_basics() -> Result<()> {
    // 创建集群配置
    let cluster_config = ClusterConfig {
        cluster_name: "test_cluster".to_string(),
        node_id: "node_1".to_string(),
        bind_address: "127.0.0.1".to_string(),
        bind_port: 8080,
        seed_nodes: vec!["127.0.0.1:8081".to_string()],
        heartbeat_interval: Duration::from_secs(5),
        election_timeout: Duration::from_secs(10),
        max_retries: 3,
    };
    
    // 创建负载均衡器
    let load_balancer = Arc::new(RoundRobinLoadBalancer::new());
    
    // 验证负载均衡策略
    use lumosai_core::distributed::LoadBalancer;
    let strategy = load_balancer.strategy();
    assert!(matches!(strategy, lumosai_core::distributed::LoadBalancingStrategy::RoundRobin));
    
    println!("✅ 分布式系统基础功能测试通过");
    Ok(())
}

/// 测试负载均衡器节点选择
#[tokio::test]
async fn test_load_balancer_node_selection() -> Result<()> {
    use lumosai_core::distributed::{LoadBalancer, NodeInfo, NodeStatus, SelectionCriteria};
    use std::time::SystemTime;
    
    let load_balancer = RoundRobinLoadBalancer::new();
    
    // 创建测试节点
    let nodes = vec![
        NodeInfo {
            node_id: "node_1".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            status: NodeStatus::Active,
            capabilities: vec!["agent_execution".to_string()],
            load: 0.3,
            last_heartbeat: SystemTime::now(),
            metadata: HashMap::new(),
        },
        NodeInfo {
            node_id: "node_2".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8081,
            status: NodeStatus::Active,
            capabilities: vec!["agent_execution".to_string(), "gpu_compute".to_string()],
            load: 0.7,
            last_heartbeat: SystemTime::now(),
            metadata: HashMap::new(),
        },
        NodeInfo {
            node_id: "node_3".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8082,
            status: NodeStatus::Active,
            capabilities: vec!["agent_execution".to_string()],
            load: 0.9, // 高负载
            last_heartbeat: SystemTime::now(),
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
    let selected_node = low_load_selected.unwrap();
    assert!(selected_node.load <= 0.5);
    
    // 测试排除节点
    let exclude_criteria = SelectionCriteria {
        required_capabilities: vec!["agent_execution".to_string()],
        preferred_region: None,
        max_load: None,
        exclude_nodes: vec!["node_1".to_string(), "node_2".to_string()],
    };
    
    let exclude_selected = load_balancer.select_node(&nodes, &exclude_criteria).await?;
    assert!(exclude_selected.is_some());
    assert_eq!(exclude_selected.unwrap().node_id, "node_3");
    
    println!("✅ 负载均衡器节点选择测试通过");
    Ok(())
}

/// 测试文档格式支持
#[tokio::test]
async fn test_documentation_formats() -> Result<()> {
    // 创建测试Agent
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Test response".to_string(),
    ]));
    
    let config = AgentConfig {
        name: "format_test_agent".to_string(),
        instructions: "Test agent for format testing".to_string(),
        model_id: Some("test-model".to_string()),
        memory_config: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: Some(5),
        tool_timeout: Some(15),
    };
    
    let agent = BasicAgent::new(config, llm);
    
    // 测试不同格式的文档生成器
    let formats = vec![
        DocumentationFormat::Markdown,
        DocumentationFormat::Html,
        DocumentationFormat::Json,
        DocumentationFormat::OpenApi,
    ];
    
    for format in formats {
        let doc_generator = ApiDocumentationGenerator::new(
            "test_output".to_string(),
            format.clone(),
        );
        
        let documentation = doc_generator.generate_agent_documentation(&agent).await?;
        
        // 验证基本结构
        assert!(!documentation.title.is_empty());
        assert!(!documentation.endpoints.is_empty());
        
        // 验证模式定义（如果启用）
        if !documentation.schemas.is_empty() {
            assert!(documentation.schemas.contains_key("Message"));
            assert!(documentation.schemas.contains_key("GenerateRequest"));
        }
    }
    
    println!("✅ 文档格式支持测试通过");
    Ok(())
}

/// 测试插件依赖管理
#[tokio::test]
async fn test_plugin_dependency_management() -> Result<()> {
    use lumosai_core::plugin::{PluginRegistry, PluginMetadata, PluginCapability, PluginHook};
    
    let mut registry = PluginRegistry::new();
    
    // 注册基础插件
    let base_plugin = PluginMetadata {
        name: "base_plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Base plugin".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        dependencies: vec![],
        capabilities: vec![PluginCapability::ProvideTools],
        hooks: vec![PluginHook::BeforeAgentInit],
        config_schema: None,
    };
    
    registry.register(base_plugin)?;
    
    // 注册依赖插件
    let dependent_plugin = PluginMetadata {
        name: "dependent_plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Plugin with dependencies".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        dependencies: vec!["base_plugin".to_string()],
        capabilities: vec![PluginCapability::ProcessMessages],
        hooks: vec![PluginHook::BeforeMessageProcess],
        config_schema: None,
    };
    
    registry.register(dependent_plugin)?;
    
    // 测试依赖检查
    let deps = registry.check_dependencies("dependent_plugin")?;
    assert_eq!(deps, vec!["base_plugin".to_string()]);
    
    // 测试依赖顺序
    let order = registry.get_dependency_order()?;
    assert!(order.len() >= 2);
    
    // base_plugin应该在dependent_plugin之前
    let base_index = order.iter().position(|x| x == "base_plugin").unwrap();
    let dependent_index = order.iter().position(|x| x == "dependent_plugin").unwrap();
    assert!(base_index < dependent_index);
    
    println!("✅ 插件依赖管理测试通过");
    Ok(())
}

/// 测试综合功能集成
#[tokio::test]
async fn test_comprehensive_integration() -> Result<()> {
    // 创建Agent
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Integration test response".to_string(),
    ]));
    
    let config = AgentConfig {
        name: "integration_test_agent".to_string(),
        instructions: "Comprehensive integration test agent".to_string(),
        model_id: Some("test-model".to_string()),
        memory_config: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(true),
        context: None,
        metadata: None,
        max_tool_calls: Some(20),
        tool_timeout: Some(45),
    };
    
    let agent = BasicAgent::new(config, llm);
    
    // 测试Agent状态
    assert_eq!(agent.get_status(), AgentStatus::Ready);
    assert!(agent.has_own_memory());
    
    // 创建插件管理器并注册插件
    let mut plugin_manager = PluginManager::new();
    let logging_plugin = Arc::new(LoggingPlugin::new());
    plugin_manager.register_plugin(logging_plugin).await?;
    
    // 生成文档
    let doc_generator = ApiDocumentationGenerator::new(
        "integration_test_output".to_string(),
        DocumentationFormat::Json,
    );
    
    let documentation = doc_generator.generate_agent_documentation(&agent).await?;
    
    // 验证集成结果
    assert!(!documentation.title.is_empty());
    assert!(documentation.title.contains("integration_test_agent"));
    assert!(!documentation.endpoints.is_empty());
    
    // 验证插件系统
    let plugins = plugin_manager.list_plugins();
    assert!(!plugins.is_empty());
    
    // 执行插件钩子
    let context = PluginContext {
        agent_name: agent.get_name().to_string(),
        request_id: "integration_test_123".to_string(),
        metadata: HashMap::new(),
        config: HashMap::new(),
    };
    
    let results = plugin_manager.execute_hook(
        PluginHook::BeforeMessageProcess,
        &context,
        Some(serde_json::json!({"test": "integration"})),
    ).await?;
    
    assert!(!results.is_empty());
    
    println!("✅ 综合功能集成测试通过");
    Ok(())
}

/// 测试错误处理和恢复
#[tokio::test]
async fn test_error_handling_and_recovery() -> Result<()> {
    use lumosai_core::plugin::{PluginRegistry, PluginMetadata};
    
    let mut registry = PluginRegistry::new();
    
    // 测试缺失依赖的错误处理
    let invalid_plugin = PluginMetadata {
        name: "invalid_plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Plugin with missing dependencies".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        dependencies: vec!["nonexistent_plugin".to_string()],
        capabilities: vec![],
        hooks: vec![],
        config_schema: None,
    };
    
    registry.register(invalid_plugin)?;
    
    // 检查依赖应该失败
    let result = registry.check_dependencies("invalid_plugin");
    assert!(result.is_err());
    
    // 测试循环依赖检测
    let plugin_a = PluginMetadata {
        name: "plugin_a".to_string(),
        version: "1.0.0".to_string(),
        description: "Plugin A".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        dependencies: vec!["plugin_b".to_string()],
        capabilities: vec![],
        hooks: vec![],
        config_schema: None,
    };
    
    let plugin_b = PluginMetadata {
        name: "plugin_b".to_string(),
        version: "1.0.0".to_string(),
        description: "Plugin B".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        dependencies: vec!["plugin_a".to_string()],
        capabilities: vec![],
        hooks: vec![],
        config_schema: None,
    };
    
    registry.register(plugin_a)?;
    registry.register(plugin_b)?;
    
    // 获取依赖顺序应该检测到循环依赖
    let order_result = registry.get_dependency_order();
    assert!(order_result.is_err());
    
    println!("✅ 错误处理和恢复测试通过");
    Ok(())
}
