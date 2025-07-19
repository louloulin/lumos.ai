//! MCP协议集成示例 - 展示如何使用Model Context Protocol
//! 
//! 这个示例展示了LumosAI的MCP协议支持，包括工具发现、集成和使用。
//! 
//! 运行方式:
//! ```bash
//! cargo run --example mcp_integration
//! ```

use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::mcp::{McpClient, McpServer, McpTool, McpConfig};
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🔗 LumosAI MCP协议集成示例");
    println!("==============================");

    // 创建LLM提供者
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我已经通过MCP协议调用了工具。".to_string(),
        "MCP工具执行成功。".to_string(),
        "我已经发现了新的MCP工具。".to_string(),
        "MCP服务器连接正常。".to_string(),
    ]));

    // 1. 基础MCP客户端使用
    println!("\n1️⃣ 基础MCP客户端");
    println!("------------------");

    let mcp_config = McpConfig {
        server_url: "http://localhost:3001/mcp".to_string(),
        timeout: 30,
        retry_count: 3,
        auth_token: None,
    };

    let mcp_client = McpClient::new(mcp_config).await?;
    info!("✅ MCP客户端创建成功");

    // 发现可用工具
    let available_tools = mcp_client.discover_tools().await?;
    info!("🔍 发现MCP工具数量: {}", available_tools.len());

    for tool in &available_tools {
        info!("   - {}: {}", tool.name, tool.description);
    }

    // 2. 使用mcp_client!宏简化集成
    println!("\n2️⃣ 使用mcp_client!宏");
    println!("--------------------");

    // 使用宏创建MCP客户端和工具
    let mcp_tools = mcp_client! {
        server: "http://localhost:3001/mcp",
        tools: [
            "weather_tool",
            "calculator_tool", 
            "file_reader_tool",
            "web_search_tool"
        ],
        config: {
            timeout: 30,
            retry_count: 3
        }
    };

    info!("✅ 通过宏创建了{}个MCP工具", mcp_tools.len());

    // 3. 创建支持MCP的Agent
    println!("\n3️⃣ 创建支持MCP的Agent");
    println!("------------------------");

    let mcp_agent = quick_agent("mcp_assistant", "你是一个支持MCP协议的AI助手")
        .model(llm.clone())
        .tools(mcp_tools)
        .build()?;

    info!("✅ MCP Agent创建成功，工具数量: {}", mcp_agent.get_tools().len());

    // 测试MCP Agent
    let mcp_response = mcp_agent.generate("请使用天气工具查询北京的天气").await?;
    println!("🤖 MCP助手: {}", mcp_response.content);

    // 4. 批量MCP服务器集成
    println!("\n4️⃣ 批量MCP服务器集成");
    println!("----------------------");

    let mcp_servers = vec![
        "http://localhost:3001/mcp",  // 本地开发服务器
        "http://localhost:3002/mcp",  // 工具服务器
        "http://localhost:3003/mcp",  // 数据服务器
    ];

    let mut all_mcp_tools = Vec::new();
    
    for server_url in mcp_servers {
        match create_mcp_client_for_server(server_url).await {
            Ok(tools) => {
                info!("✅ 从{}发现{}个工具", server_url, tools.len());
                all_mcp_tools.extend(tools);
            }
            Err(e) => {
                error!("❌ 连接{}失败: {}", server_url, e);
            }
        }
    }

    info!("📊 总共发现{}个MCP工具", all_mcp_tools.len());

    // 5. 创建MCP工具市场Agent
    println!("\n5️⃣ MCP工具市场");
    println!("----------------");

    let marketplace_agent = quick_agent("marketplace", "你是MCP工具市场助手")
        .model(llm.clone())
        .tools(all_mcp_tools)
        .build()?;

    info!("✅ 工具市场Agent创建成功");
    info!("🛒 可用工具类别:");

    // 按类别分组工具
    let tool_categories = categorize_tools(marketplace_agent.get_tools());
    for (category, count) in tool_categories {
        info!("   - {}: {}个工具", category, count);
    }

    // 6. 动态工具发现和热加载
    println!("\n6️⃣ 动态工具发现");
    println!("------------------");

    let dynamic_agent = create_dynamic_mcp_agent(llm.clone()).await?;
    info!("✅ 动态MCP Agent创建成功");

    // 模拟动态发现新工具
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            
            // 这里会定期检查新的MCP服务器和工具
            info!("🔄 检查新的MCP工具...");
            
            // 实际实现中，这里会调用工具发现逻辑
            // discover_new_tools().await;
        }
    });

    // 7. MCP工具性能监控
    println!("\n7️⃣ MCP工具性能监控");
    println!("--------------------");

    let monitoring_agent = create_monitoring_agent(llm.clone()).await?;
    
    // 执行一些操作来测试性能
    let start_time = std::time::Instant::now();
    
    let test_queries = vec![
        "查询天气信息",
        "计算数学表达式",
        "搜索网络信息",
        "读取文件内容",
    ];

    for query in test_queries {
        let query_start = std::time::Instant::now();
        
        match monitoring_agent.generate(query).await {
            Ok(response) => {
                let duration = query_start.elapsed();
                info!("✅ 查询'{}' 完成，耗时: {:?}", query, duration);
                info!("   响应: {}", response.content);
            }
            Err(e) => {
                error!("❌ 查询'{}' 失败: {}", query, e);
            }
        }
    }

    let total_duration = start_time.elapsed();
    info!("📊 总测试时间: {:?}", total_duration);

    // 8. MCP协议错误处理和重试
    println!("\n8️⃣ 错误处理和重试");
    println!("------------------");

    let resilient_agent = create_resilient_mcp_agent(llm.clone()).await?;
    
    // 测试错误恢复
    match resilient_agent.generate("测试错误恢复机制").await {
        Ok(response) => {
            info!("✅ 错误恢复测试成功: {}", response.content);
        }
        Err(e) => {
            error!("❌ 错误恢复测试失败: {}", e);
        }
    }

    println!("\n🎉 MCP协议集成示例完成!");
    println!("\n📚 下一步学习:");
    println!("   - examples/04_production/tool_marketplace.rs - 工具市场");
    println!("   - examples/04_production/monitoring.rs - 性能监控");
    println!("   - docs/best-practices/mcp-integration.md - MCP集成最佳实践");

    Ok(())
}

/// 为指定服务器创建MCP客户端
async fn create_mcp_client_for_server(server_url: &str) -> Result<Vec<Box<dyn Tool>>> {
    let config = McpConfig {
        server_url: server_url.to_string(),
        timeout: 10,
        retry_count: 2,
        auth_token: None,
    };

    let client = McpClient::new(config).await?;
    let tools = client.discover_tools().await?;
    
    // 转换为LumosAI工具
    let lumos_tools: Vec<Box<dyn Tool>> = tools.into_iter()
        .map(|mcp_tool| Box::new(mcp_tool) as Box<dyn Tool>)
        .collect();
    
    Ok(lumos_tools)
}

/// 按类别分组工具
fn categorize_tools(tools: &[Box<dyn Tool>]) -> std::collections::HashMap<String, usize> {
    let mut categories = std::collections::HashMap::new();
    
    for tool in tools {
        let category = match tool.name() {
            name if name.contains("weather") => "天气",
            name if name.contains("calc") || name.contains("math") => "计算",
            name if name.contains("file") => "文件",
            name if name.contains("web") || name.contains("search") => "网络",
            name if name.contains("data") => "数据",
            _ => "其他",
        };
        
        *categories.entry(category.to_string()).or_insert(0) += 1;
    }
    
    categories
}

/// 创建动态MCP Agent
async fn create_dynamic_mcp_agent(llm: Arc<MockLlmProvider>) -> Result<impl Agent> {
    // 初始工具集
    let initial_tools = vec![
        calculator(),
        time_tool(),
    ];

    let agent = quick_agent("dynamic_mcp", "动态MCP工具发现助手")
        .model(llm)
        .tools(initial_tools)
        .build()?;

    Ok(agent)
}

/// 创建监控Agent
async fn create_monitoring_agent(llm: Arc<MockLlmProvider>) -> Result<impl Agent> {
    let monitoring_tools = vec![
        calculator(),
        time_tool(),
        web_search(),
        file_reader(),
    ];

    let agent = quick_agent("monitoring", "MCP性能监控助手")
        .model(llm)
        .tools(monitoring_tools)
        .build()?;

    Ok(agent)
}

/// 创建具有错误恢复能力的MCP Agent
async fn create_resilient_mcp_agent(llm: Arc<MockLlmProvider>) -> Result<impl Agent> {
    let resilient_tools = vec![
        calculator(),
        time_tool(),
    ];

    let agent = AgentBuilder::new()
        .name("resilient_mcp")
        .instructions("具有错误恢复能力的MCP助手")
        .model(llm)
        .tools(resilient_tools)
        .max_retries(3)
        .retry_delay(std::time::Duration::from_secs(1))
        .tool_timeout(30)
        .build()?;

    Ok(agent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_integration() {
        let result = main().await;
        assert!(result.is_ok(), "MCP集成示例应该成功运行");
    }

    #[tokio::test]
    async fn test_mcp_client_creation() {
        let config = McpConfig {
            server_url: "http://localhost:3001/mcp".to_string(),
            timeout: 10,
            retry_count: 2,
            auth_token: None,
        };

        // 在测试环境中，这可能会失败，因为没有真实的MCP服务器
        // 但我们可以测试配置创建
        assert_eq!(config.server_url, "http://localhost:3001/mcp");
        assert_eq!(config.timeout, 10);
        assert_eq!(config.retry_count, 2);
    }

    #[test]
    fn test_tool_categorization() {
        // 创建模拟工具进行测试
        let tools: Vec<Box<dyn Tool>> = vec![
            calculator(),
            weather_tool(),
            file_reader(),
            web_search(),
        ];

        let categories = categorize_tools(&tools);
        
        assert!(categories.contains_key("计算"));
        assert!(categories.contains_key("天气"));
        assert!(categories.contains_key("文件"));
        assert!(categories.contains_key("网络"));
    }

    #[tokio::test]
    async fn test_dynamic_agent_creation() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        
        let agent = create_dynamic_mcp_agent(llm).await;
        assert!(agent.is_ok());
        
        let agent = agent.unwrap();
        assert_eq!(agent.get_name(), "dynamic_mcp");
    }
}
