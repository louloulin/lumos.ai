//! MCP协议深度集成演示
//! 
//! 这个示例展示了如何使用Lumos.ai的MCP协议深度集成功能，
//! 包括服务器发现、工具适配、批量执行等高级特性。

use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use serde_json::json;

use lumosai_mcp::{
    MCPIntegration, MCPServerRegistry, ServerConfig, ServerType, ConnectionConfig,
    EnhancedMCPManager, ManagerConfig,
};
use lumosai_core::tool::{ToolExecutionContext, ToolExecutionOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Lumos.ai MCP协议深度集成演示");
    println!("=====================================");

    // 1. 创建MCP集成实例
    println!("\n📦 1. 初始化MCP集成...");
    let integration = MCPIntegration::new();
    
    // 快速设置
    integration.quick_setup().await?;
    
    // 2. 设置服务器注册表
    println!("\n🔍 2. 配置MCP服务器注册表...");
    let mut registry = MCPServerRegistry::new(integration.manager().clone());
    
    // 注册示例服务器配置
    let calculator_server = ServerConfig {
        name: "calculator".to_string(),
        description: "基础计算器服务".to_string(),
        server_type: ServerType::Stdio,
        connection: ConnectionConfig::Stdio {
            command: "npx".to_string(),
            args: vec!["@modelcontextprotocol/calculator".to_string()],
            env: HashMap::new(),
            working_dir: None,
        },
        capabilities: vec!["math".to_string(), "calculator".to_string()],
        tags: vec!["utility".to_string(), "math".to_string()],
        enabled: true,
        priority: 80,
    };
    
    registry.register_server(calculator_server)?;
    
    let weather_server = ServerConfig {
        name: "weather".to_string(),
        description: "天气信息服务".to_string(),
        server_type: ServerType::Stdio,
        connection: ConnectionConfig::Stdio {
            command: "npx".to_string(),
            args: vec!["@modelcontextprotocol/weather".to_string()],
            env: HashMap::new(),
            working_dir: None,
        },
        capabilities: vec!["weather".to_string(), "forecast".to_string()],
        tags: vec!["utility".to_string(), "weather".to_string()],
        enabled: true,
        priority: 75,
    };
    
    registry.register_server(weather_server)?;
    
    // 3. 自动发现服务器
    println!("\n🔍 3. 自动发现MCP服务器...");
    let discovered_count = registry.auto_discover().await?;
    println!("   发现了 {} 个MCP服务器", discovered_count);
    
    // 显示所有注册的服务器
    let servers = registry.get_servers();
    println!("   注册的服务器:");
    for (name, config) in servers {
        println!("   - {}: {} (优先级: {})", name, config.description, config.priority);
        println!("     能力: {:?}", config.capabilities);
        println!("     标签: {:?}", config.tags);
    }
    
    // 4. 按能力查找服务器
    println!("\n🔧 4. 按能力查找服务器...");
    let math_servers = registry.get_servers_by_capability("math");
    println!("   数学能力服务器: {} 个", math_servers.len());
    for server in math_servers {
        println!("   - {}: {}", server.name, server.description);
    }
    
    let weather_servers = registry.get_servers_by_tag("weather");
    println!("   天气标签服务器: {} 个", weather_servers.len());
    for server in weather_servers {
        println!("   - {}: {}", server.name, server.description);
    }
    
    // 5. 创建工具适配器
    println!("\n🔧 5. 创建Lumos工具适配器...");
    let tools = integration.get_all_tools().await?;
    println!("   创建了 {} 个Lumos工具", tools.len());
    
    // 显示工具信息
    for tool in &tools {
        println!("   - 工具: {}", tool.id());
        println!("     描述: {}", tool.description());
        let schema = tool.schema();
        println!("     参数数量: {}", schema.parameters.len());
    }
    
    // 6. 演示工具执行
    println!("\n⚡ 6. 演示工具执行...");
    if !tools.is_empty() {
        let tool = &tools[0];
        let context = ToolExecutionContext::new();
        let options = ToolExecutionOptions::new();
        
        // 创建示例参数
        let params = json!({
            "operation": "add",
            "a": 10,
            "b": 20
        });
        
        println!("   执行工具: {}", tool.id());
        println!("   参数: {}", params);
        
        match tool.execute(params, context, &options).await {
            Ok(result) => {
                println!("   ✅ 执行成功: {}", result);
            }
            Err(e) => {
                println!("   ❌ 执行失败: {}", e);
            }
        }
    }
    
    // 7. 演示批量工具执行
    println!("\n🚀 7. 演示批量工具执行...");
    let manager = integration.manager();
    
    let batch_requests = vec![
        ("calculator".to_string(), {
            let mut params = HashMap::new();
            params.insert("operation".to_string(), json!("add"));
            params.insert("a".to_string(), json!(5));
            params.insert("b".to_string(), json!(3));
            params
        }),
        ("calculator".to_string(), {
            let mut params = HashMap::new();
            params.insert("operation".to_string(), json!("multiply"));
            params.insert("a".to_string(), json!(4));
            params.insert("b".to_string(), json!(7));
            params
        }),
    ];
    
    println!("   执行 {} 个批量请求...", batch_requests.len());
    let batch_results = manager.batch_execute_tools(batch_requests).await;
    
    for (i, result) in batch_results.iter().enumerate() {
        match result {
            Ok(value) => println!("   请求 {}: ✅ {}", i + 1, value),
            Err(e) => println!("   请求 {}: ❌ {}", i + 1, e),
        }
    }
    
    // 8. 健康状态监控
    println!("\n💊 8. 健康状态监控...");
    let health_status = manager.get_health_status().await;
    println!("   服务器健康状态:");
    for (server_name, status) in health_status {
        println!("   - {}: {:?}", server_name, status);
    }
    
    // 9. 性能指标
    println!("\n📊 9. 性能指标...");
    let metrics = manager.get_metrics().await;
    println!("   总请求数: {}", metrics.total_requests);
    println!("   成功请求数: {}", metrics.successful_requests);
    println!("   失败请求数: {}", metrics.failed_requests);
    println!("   平均响应时间: {:.2}ms", metrics.average_response_time.as_millis());
    
    // 10. 服务器状态报告
    println!("\n📋 10. 服务器状态报告...");
    let status_report = manager.get_server_status_report().await;
    for (server_name, status) in status_report {
        println!("   服务器: {}", server_name);
        println!("     健康状态: {:?}", status.health);
        println!("     工具数量: {}", status.tool_count);
        println!("     订阅数量: {}", status.subscription_count);
        println!("     最后活动: {:?}", status.last_activity);
    }
    
    // 11. 演示错误处理和重试
    println!("\n🔄 11. 演示错误处理和重试...");
    
    // 尝试执行一个不存在的工具
    let invalid_params = HashMap::new();
    match manager.execute_mcp_tool("nonexistent_tool", invalid_params).await {
        Ok(_) => println!("   意外成功"),
        Err(e) => println!("   ✅ 正确处理错误: {}", e),
    }
    
    // 12. 清理和关闭
    println!("\n🧹 12. 清理资源...");
    
    // 等待一段时间让后台任务完成
    sleep(Duration::from_millis(100)).await;
    
    println!("\n✅ MCP协议深度集成演示完成!");
    println!("=====================================");
    println!("主要功能演示:");
    println!("✓ MCP服务器自动发现和注册");
    println!("✓ 工具适配器创建和管理");
    println!("✓ 单个和批量工具执行");
    println!("✓ 健康状态监控");
    println!("✓ 性能指标收集");
    println!("✓ 错误处理和重试机制");
    println!("✓ 服务器状态报告");
    
    Ok(())
}

/// 演示高级MCP功能
async fn demonstrate_advanced_features() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 高级功能演示");
    println!("=================");
    
    // 创建自定义配置的MCP管理器
    let config = ManagerConfig {
        health_check_interval: Duration::from_secs(30),
        max_consecutive_failures: 3,
        connection_timeout: Duration::from_secs(10),
        tool_cache_ttl: Duration::from_secs(300),
        auto_reconnect: true,
        max_retry_attempts: 3,
    };
    
    let manager = EnhancedMCPManager::new(config);
    
    // 启动后台任务
    manager.start_background_tasks().await;
    
    println!("✅ 高级MCP管理器已启动");
    println!("   - 健康检查间隔: 30秒");
    println!("   - 最大连续失败次数: 3");
    println!("   - 连接超时: 10秒");
    println!("   - 工具缓存TTL: 300秒");
    println!("   - 自动重连: 启用");
    println!("   - 最大重试次数: 3");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mcp_integration_demo() {
        // 测试MCP集成演示的基本功能
        let integration = MCPIntegration::new();
        let result = integration.quick_setup().await;
        assert!(result.is_ok(), "MCP集成设置应该成功");
        
        let tools = integration.get_all_tools().await;
        assert!(tools.is_ok(), "获取工具列表应该成功");
    }
    
    #[tokio::test]
    async fn test_server_registry() {
        let integration = MCPIntegration::new();
        let mut registry = MCPServerRegistry::new(integration.manager().clone());
        
        let server_config = ServerConfig {
            name: "test_server".to_string(),
            description: "测试服务器".to_string(),
            server_type: ServerType::Stdio,
            connection: ConnectionConfig::Stdio {
                command: "echo".to_string(),
                args: vec!["test".to_string()],
                env: HashMap::new(),
                working_dir: None,
            },
            capabilities: vec!["test".to_string()],
            tags: vec!["test".to_string()],
            enabled: true,
            priority: 50,
        };
        
        let result = registry.register_server(server_config);
        assert!(result.is_ok(), "服务器注册应该成功");
        
        let servers = registry.get_servers();
        assert!(servers.contains_key("test_server"), "应该包含注册的服务器");
    }
}
