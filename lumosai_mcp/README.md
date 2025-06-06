# Lumos.ai MCP协议深度集成

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-passing-green.svg)](tests)

Lumos.ai的MCP（Model Context Protocol）协议深度集成模块，提供了与MCP服务器的无缝集成能力，支持工具发现、适配、执行和管理。

## 🚀 主要特性

### 核心功能
- **🔍 自动服务器发现**: 自动发现和注册MCP服务器
- **🔧 工具适配器**: 将MCP工具无缝适配为Lumos工具
- **⚡ 批量执行**: 支持多个工具的并发批量执行
- **💊 健康监控**: 实时监控服务器健康状态
- **📊 性能指标**: 收集和分析执行性能数据
- **🔄 错误处理**: 智能重试和错误恢复机制

### 高级特性
- **🎯 智能路由**: 基于能力和优先级的智能工具路由
- **📋 状态报告**: 详细的服务器状态和活动报告
- **🔒 安全连接**: 支持多种安全连接方式
- **⚙️ 灵活配置**: 丰富的配置选项和自定义能力
- **🧹 资源管理**: 自动资源清理和连接池管理

## 📦 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
lumosai_mcp = "0.1.0"
lumosai_core = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

## 🎯 快速开始

### 基础使用

```rust
use lumosai_mcp::MCPIntegration;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建MCP集成实例
    let integration = MCPIntegration::new();
    
    // 2. 快速设置
    integration.quick_setup().await?;
    
    // 3. 获取所有可用工具
    let tools = integration.get_all_tools().await?;
    println!("发现 {} 个工具", tools.len());
    
    // 4. 执行工具
    if let Some(tool) = tools.first() {
        let params = json!({"input": "Hello, MCP!"});
        let result = tool.execute(params, context, &options).await?;
        println!("执行结果: {}", result);
    }
    
    Ok(())
}
```

### 服务器注册和发现

```rust
use lumosai_mcp::{MCPServerRegistry, ServerConfig, ServerType, ConnectionConfig};
use std::collections::HashMap;

async fn setup_servers() -> Result<(), Box<dyn std::error::Error>> {
    let integration = MCPIntegration::new();
    let mut registry = MCPServerRegistry::new(integration.manager().clone());
    
    // 注册计算器服务器
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
        tags: vec!["utility".to_string()],
        enabled: true,
        priority: 80,
    };
    
    registry.register_server(calculator_server)?;
    
    // 自动发现其他服务器
    let discovered = registry.auto_discover().await?;
    println!("发现了 {} 个服务器", discovered);
    
    Ok(())
}
```

### 批量工具执行

```rust
use std::collections::HashMap;
use serde_json::json;

async fn batch_execution() -> Result<(), Box<dyn std::error::Error>> {
    let integration = MCPIntegration::new();
    let manager = integration.manager();
    
    // 准备批量请求
    let batch_requests = vec![
        ("calculator".to_string(), {
            let mut params = HashMap::new();
            params.insert("operation".to_string(), json!("add"));
            params.insert("a".to_string(), json!(10));
            params.insert("b".to_string(), json!(20));
            params
        }),
        ("calculator".to_string(), {
            let mut params = HashMap::new();
            params.insert("operation".to_string(), json!("multiply"));
            params.insert("a".to_string(), json!(5));
            params.insert("b".to_string(), json!(6));
            params
        }),
    ];
    
    // 执行批量请求
    let results = manager.batch_execute_tools(batch_requests).await;
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(value) => println!("请求 {}: 成功 - {}", i + 1, value),
            Err(e) => println!("请求 {}: 失败 - {}", i + 1, e),
        }
    }
    
    Ok(())
}
```

### 健康监控和指标

```rust
async fn monitoring() -> Result<(), Box<dyn std::error::Error>> {
    let integration = MCPIntegration::new();
    let manager = integration.manager();
    
    // 获取健康状态
    let health_status = manager.get_health_status().await;
    for (server, status) in health_status {
        println!("服务器 {}: {:?}", server, status);
    }
    
    // 获取性能指标
    let metrics = manager.get_metrics().await;
    println!("总请求数: {}", metrics.total_requests);
    println!("成功率: {:.2}%", 
        metrics.successful_requests as f64 / metrics.total_requests as f64 * 100.0);
    println!("平均响应时间: {:.2}ms", metrics.average_response_time);
    
    // 获取详细状态报告
    let status_report = manager.get_server_status_report().await;
    for (server, status) in status_report {
        println!("服务器: {}", server);
        println!("  健康状态: {:?}", status.health);
        println!("  工具数量: {}", status.tool_count);
        println!("  最后活动: {:?}", status.last_activity);
    }
    
    Ok(())
}
```

## 🔧 高级配置

### 自定义管理器配置

```rust
use lumosai_mcp::{EnhancedMCPManager, ManagerConfig};
use tokio::time::Duration;

async fn advanced_setup() -> Result<(), Box<dyn std::error::Error>> {
    let config = ManagerConfig {
        health_check_interval: Duration::from_secs(30),
        max_consecutive_failures: 3,
        connection_timeout: Duration::from_secs(10),
        tool_cache_ttl: Duration::from_secs(300),
        auto_reconnect: true,
        max_retry_attempts: 3,
    };
    
    let manager = EnhancedMCPManager::new(config);
    manager.start_background_tasks().await;
    
    Ok(())
}
```

### 工具适配器自定义

```rust
use lumosai_mcp::MCPToolAdapter;
use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};

async fn custom_tool_adapter() -> Result<(), Box<dyn std::error::Error>> {
    // 创建自定义工具适配器
    let adapter = MCPToolAdapter::new(
        "custom_tool".to_string(),
        "自定义工具描述".to_string(),
        tool_definition,
        manager.clone(),
        "server_name".to_string(),
    );
    
    // 使用适配器
    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::new();
    let result = adapter.execute(params, context, &options).await?;
    
    Ok(())
}
```

## 📚 API文档

### 主要类型

- **`MCPIntegration`**: 主要集成接口
- **`MCPServerRegistry`**: 服务器注册和发现
- **`EnhancedMCPManager`**: 高级MCP管理器
- **`MCPToolAdapter`**: 工具适配器
- **`ServerConfig`**: 服务器配置
- **`ManagerConfig`**: 管理器配置

### 错误处理

```rust
use lumosai_mcp::MCPError;

match manager.execute_mcp_tool("tool_name", params).await {
    Ok(result) => println!("成功: {}", result),
    Err(MCPError::ConnectionError(msg)) => println!("连接错误: {}", msg),
    Err(MCPError::ToolExecutionError(msg)) => println!("执行错误: {}", msg),
    Err(MCPError::TimeoutError(duration)) => println!("超时: {:?}", duration),
    Err(e) => println!("其他错误: {}", e),
}
```

## 🧪 测试

运行所有测试：

```bash
cargo test -p lumosai_mcp
```

运行特定测试：

```bash
cargo test -p lumosai_mcp test_enhanced_mcp_manager
```

运行示例：

```bash
cargo run --example mcp_integration_demo
```

## 📋 示例

查看 `examples/` 目录中的完整示例：

- `mcp_integration_demo.rs` - 完整的MCP集成演示
- 更多示例即将推出...

## 🤝 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](../CONTRIBUTING.md) 了解详细信息。

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](../LICENSE) 文件了解详细信息。

## 🔗 相关链接

- [Lumos.ai 主项目](https://github.com/lumosai/lumos.ai)
- [MCP协议规范](https://modelcontextprotocol.io/)
- [Rust文档](https://doc.rust-lang.org/)

---

**Lumos.ai MCP集成** - 让AI工具集成变得简单而强大 🚀
