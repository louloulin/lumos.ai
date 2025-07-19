//! 快速API示例 - 展示LumosAI的各种快速创建方式
//! 
//! 这个示例展示了LumosAI提供的多种快速API，帮助您选择最适合的方式。
//! 
//! 运行方式:
//! ```bash
//! cargo run --example quick_api
//! ```

use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::agent::trait_def::Agent;
use lumosai_core::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI 快速API示例");
    println!("======================");
    
    // 创建共享的LLM提供者
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我是通用助手，可以帮助您处理各种任务。".to_string(),
        "我是网络助手，专门处理网络相关的任务。".to_string(),
        "我是文件助手，专门处理文件操作。".to_string(),
        "我是数据助手，专门处理数据分析任务。".to_string(),
    ]));
    
    // 1. 最基础的快速API
    println!("\n1️⃣ 基础快速API");
    println!("----------------");
    
    let basic_agent = quick_agent("basic", "你是一个通用AI助手")
        .model(llm.clone())
        .build()?;
    
    let response = basic_agent.generate_simple("介绍一下自己").await?;
    println!("🤖 基础助手: {}", response);
    
    // 2. 专用Agent快速创建
    println!("\n2️⃣ 专用Agent快速创建");
    println!("---------------------");
    
    // Web Agent - 预配置了网络工具
    let web_agent = web_agent_quick("web_helper", "你是一个网络助手")
        .model(llm.clone())
        .build()?;
    
    println!("🌐 Web Agent创建成功，工具数量: {}", web_agent.get_tools().len());
    let web_response = web_agent.generate_simple("你能做什么？").await?;
    println!("🤖 网络助手: {}", web_response);
    
    // File Agent - 预配置了文件工具
    let file_agent = file_agent_quick("file_helper", "你是一个文件管理助手")
        .model(llm.clone())
        .build()?;
    
    println!("📁 File Agent创建成功，工具数量: {}", file_agent.get_tools().len());
    let file_response = file_agent.generate_simple("你能处理哪些文件操作？").await?;
    println!("🤖 文件助手: {}", file_response);
    
    // Data Agent - 预配置了数据处理工具
    let data_agent = data_agent_quick("data_helper", "你是一个数据分析助手")
        .model(llm.clone())
        .build()?;
    
    println!("📊 Data Agent创建成功，工具数量: {}", data_agent.get_tools().len());
    let data_response = data_agent.generate_simple("你能进行哪些数据分析？").await?;
    println!("🤖 数据助手: {}", data_response);
    
    // 3. 使用Agent::quick静态方法
    println!("\n3️⃣ Agent::quick 静态方法");
    println!("-------------------------");
    
    let static_agent = quick_agent("static", "你是一个静态创建的助手")
        .model(llm.clone())
        .build()?;
    
    let static_response = static_agent.generate_simple("你是如何创建的？").await?;
    println!("🤖 静态助手: {}", static_response);
    
    // 4. 链式配置示例
    println!("\n4️⃣ 链式配置示例");
    println!("------------------");
    
    let configured_agent = quick_agent("configured", "你是一个配置完善的助手")
        .model(llm.clone())
        .tools(vec![
            calculator(),
            time_tool(),
            uuid_generator(),
        ])
        .build()?;
    
    println!("⚙️ 配置助手创建成功，工具数量: {}", configured_agent.get_tools().len());
    
    // 列出所有工具
    println!("🔧 可用工具:");
    for (name, tool) in configured_agent.get_tools() {
        println!("   - {}: {}", name, tool.name().unwrap_or("未知工具"));
    }
    
    // 5. 错误处理示例
    println!("\n5️⃣ 错误处理示例");
    println!("------------------");
    
    // 演示错误处理
    let result = quick_agent("", "")  // 空名称和指令
        .model(llm.clone())
        .build();
    
    match result {
        Ok(_) => println!("✅ Agent创建成功"),
        Err(e) => println!("❌ Agent创建失败: {}", e),
    }
    
    // 6. 性能测试
    println!("\n6️⃣ 性能测试");
    println!("-------------");
    
    let start = std::time::Instant::now();
    
    // 创建多个Agent测试性能
    let mut agents = Vec::new();
    for i in 0..10 {
        let agent = quick_agent(&format!("agent_{}", i), "测试助手")
            .model(llm.clone())
            .build()?;
        agents.push(agent);
    }
    
    let duration = start.elapsed();
    println!("⏱️ 创建10个Agent耗时: {:?}", duration);
    println!("📊 平均每个Agent创建时间: {:?}", duration / 10);
    
    println!("\n🎉 快速API示例完成!");
    println!("\n📚 下一步学习:");
    println!("   - examples/01_getting_started/basic_tools.rs - 学习工具使用");
    println!("   - examples/02_intermediate/builder_pattern.rs - 学习构建器模式");
    println!("   - docs/api-choice-guide.md - API选择指南");
    
    Ok(())
}

/// 演示不同场景下的API选择
async fn demonstrate_api_choices() -> Result<()> {
    let llm = Arc::new(MockLlmProvider::new(vec!["示例响应".to_string()]));
    
    println!("🎯 API选择场景演示");
    println!("==================");
    
    // 场景1: 快速原型
    println!("\n📝 场景1: 快速原型开发");
    let prototype = quick_agent("prototype", "原型助手")
        .model(llm.clone())
        .build()?;
    println!("✅ 原型Agent: 代码量最少，适合快速验证想法");
    
    // 场景2: 生产应用
    println!("\n🏭 场景2: 生产应用");
    let production = quick_agent("production", "生产级助手")
        .model(llm.clone())
        .max_tool_calls(10)
        .tool_timeout(30)
        .build()?;
    println!("✅ 生产Agent: 完整配置，适合生产环境");
    
    // 场景3: 特定领域
    println!("\n🎯 场景3: 特定领域应用");
    let domain_agent = web_agent_quick("domain", "领域专家")
        .model(llm.clone())
        .build()?;
    println!("✅ 领域Agent: 预配置工具，适合特定场景");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_quick_api_examples() {
        let result = main().await;
        assert!(result.is_ok(), "快速API示例应该成功运行");
    }
    
    #[tokio::test]
    async fn test_specialized_agents() {
        let llm = Arc::new(MockLlmProvider::new(vec!["test".to_string()]));
        
        // 测试Web Agent
        let web_agent = web_agent_quick("test_web", "test")
            .model(llm.clone())
            .build();
        assert!(web_agent.is_ok());
        assert!(web_agent.unwrap().get_tools().len() > 0);
        
        // 测试File Agent
        let file_agent = file_agent_quick("test_file", "test")
            .model(llm.clone())
            .build();
        assert!(file_agent.is_ok());
        assert!(file_agent.unwrap().get_tools().len() > 0);
        
        // 测试Data Agent
        let data_agent = data_agent_quick("test_data", "test")
            .model(llm.clone())
            .build();
        assert!(data_agent.is_ok());
        assert!(data_agent.unwrap().get_tools().len() > 0);
    }
    
    #[tokio::test]
    async fn test_performance() {
        let llm = Arc::new(MockLlmProvider::new(vec!["test".to_string()]));
        
        let start = std::time::Instant::now();
        
        for i in 0..100 {
            let _agent = quick_agent(&format!("test_{}", i), "test")
                .model(llm.clone())
                .build()
                .expect("Agent创建失败");
        }
        
        let duration = start.elapsed();
        println!("创建100个Agent耗时: {:?}", duration);
        
        // 确保性能在合理范围内 (每个Agent < 1ms)
        assert!(duration < std::time::Duration::from_millis(100));
    }
}
