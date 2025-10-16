//! 统一 API 演示 - 展示简化后的 LumosAI API 设计
//!
//! 这个示例展示了重新设计后的统一 API，遵循"简洁优于复杂"的设计原则。
//!
//! 运行方式：
//! ```bash
//! cargo run --package lumosai_examples --example unified_api_demo
//! ```

use lumosai_core::llm::types::user_message;
use lumosai_core::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI 统一 API 演示");
    println!("========================");

    // 创建 Mock LLM 提供商用于演示
    let mock_responses = vec![
        "你好！我是AI助手，很高兴为你服务。".to_string(),
        "我可以帮你进行网络搜索、文件处理和数据分析。".to_string(),
        "计算结果：2 + 3 = 5".to_string(),
        "文件内容已成功读取。".to_string(),
    ];
    let llm = Arc::new(lumosai_core::llm::MockLlmProvider::new(mock_responses));

    // ============================================================================
    // 1. 基本 Agent 创建 - 推荐的主要方式
    // ============================================================================

    println!("\n📝 1. 基本 Agent 创建");
    println!("-------------------");

    let basic_agent = quick_agent("assistant", "你是一个友好的AI助手")
        .model(llm.clone())
        .build()?;

    println!("✅ 创建基本 Agent: {}", basic_agent.get_name());

    // 使用正确的 generate 方法
    let messages = vec![user_message("你好")];
    let options = lumosai_core::agent::types::AgentGenerateOptions::default();
    let response = basic_agent.generate(&messages, &options).await?;
    println!("🤖 Agent 响应: {}", response.response);

    // ============================================================================
    // 2. 专业化 Agent 创建 - 预配置工具
    // ============================================================================

    println!("\n🌐 2. 网络功能 Agent");
    println!("------------------");

    let web_agent = web_agent("web_helper", "你是一个网络搜索助手")
        .model(llm.clone())
        .build()?;

    println!("✅ 创建网络 Agent: {}", web_agent.get_name());
    println!("🔧 预配置工具数量: {}", web_agent.get_tools().len());

    let messages = vec![user_message("帮我搜索信息")];
    let options = lumosai_core::agent::types::AgentGenerateOptions::default();
    let response = web_agent.generate(&messages, &options).await?;
    println!("🤖 Agent 响应: {}", response.response);

    // ============================================================================
    // 3. 数据处理 Agent - 数学和数据工具
    // ============================================================================

    println!("\n📊 3. 数据处理 Agent");
    println!("------------------");

    let data_agent = data_agent("data_helper", "你是一个数据分析助手")
        .model(llm.clone())
        .build()?;

    println!("✅ 创建数据 Agent: {}", data_agent.get_name());
    println!("🔧 预配置工具数量: {}", data_agent.get_tools().len());

    let messages = vec![user_message("计算 2 + 3")];
    let options = lumosai_core::agent::types::AgentGenerateOptions::default();
    let response = data_agent.generate(&messages, &options).await?;
    println!("🤖 Agent 响应: {}", response.response);

    // ============================================================================
    // 4. 文件处理 Agent - 文件操作工具
    // ============================================================================

    println!("\n📁 4. 文件处理 Agent");
    println!("------------------");

    let file_agent = file_agent("file_helper", "你是一个文件管理助手")
        .model(llm.clone())
        .build()?;

    println!("✅ 创建文件 Agent: {}", file_agent.get_name());
    println!("🔧 预配置工具数量: {}", file_agent.get_tools().len());

    let messages = vec![user_message("读取文件内容")];
    let options = lumosai_core::agent::types::AgentGenerateOptions::default();
    let response = file_agent.generate(&messages, &options).await?;
    println!("🤖 Agent 响应: {}", response.response);

    // ============================================================================
    // 5. 高级配置 Agent - 使用 Builder 模式
    // ============================================================================

    println!("\n⚙️ 5. 高级配置 Agent");
    println!("------------------");

    let advanced_agent = AgentBuilder::new()
        .name("advanced_assistant")
        .instructions("你是一个高级AI助手，具备多种能力")
        .model(llm.clone())
        .tools(vec![
            calculator(),
            web_search(),
            file_reader(),
            json_parser(),
        ])
        .max_tool_calls(15)
        .tool_timeout(60)
        .build()?;

    println!("✅ 创建高级 Agent: {}", advanced_agent.get_name());
    println!("🔧 自定义工具数量: {}", advanced_agent.get_tools().len());

    // ============================================================================
    // 6. API 设计总结
    // ============================================================================

    println!("\n📋 API 设计总结");
    println!("==============");
    println!("✨ 统一入口：Agent::new() - 推荐的主要创建方式");
    println!("🎯 专业化函数：web_agent(), data_agent(), file_agent()");
    println!("🔧 高级配置：Agent::builder() - 完全自定义");
    println!("📦 工具便利函数：calculator(), web_search(), file_reader() 等");
    println!("🚀 简洁设计：移除重复函数，统一命名规范");

    println!("\n✅ 统一 API 演示完成！");
    println!("新的 API 设计更加简洁、直观、易用。");

    Ok(())
}
