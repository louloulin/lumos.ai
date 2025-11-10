//! 全面测试 prelude.rs 的功能
//!
//! 这个测试文件验证 prelude.rs 中导出的所有主要功能

use lumosai_core::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 开始全面测试 prelude.rs 功能");

    // 测试 1: 快速创建 Agent
    println!("\n1. 测试快速创建 Agent");
    test_quick_agent_creation().await?;

    // 测试 2: 测试工具创建函数
    println!("\n2. 测试工具创建函数");
    test_tool_creation_functions()?;

    // 测试 3: 测试向量存储
    println!("\n3. 测试向量存储");
    test_vector_storage()?;

    // 测试 4: 测试专门的 Agent 创建函数
    println!("\n4. 测试专门的 Agent 创建函数");
    test_specialized_agents().await?;

    println!("\n✅ 所有 prelude.rs 功能测试通过！");
    Ok(())
}

async fn test_quick_agent_creation() -> Result<()> {
    // 使用 MockLlmProvider 进行测试
    let llm = lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc();

    let agent = quick_agent("test_agent", "You are a test assistant")
        .model(llm)
        .build()?;

    // Agent 创建成功验证
    println!("   ✅ 快速 Agent 创建成功");
    Ok(())
}

fn test_tool_creation_functions() -> Result<()> {
    // 测试各种工具创建函数 (只测试实际存在的函数)
    let tools = vec![
        ("calculator", calculator()),
        ("file_reader", file_reader()),
        ("file_writer", file_writer()),
        ("json_parser", json_parser()),
        ("csv_parser", csv_parser()),
        ("web_scraper", web_scraper()),
        ("web_search", web_search()),
        ("datetime", datetime()), // 使用实际存在的函数名
        ("uuid_generator", uuid_generator()),
        ("statistics", statistics()),
        ("json_api", json_api()),
        // 注释掉不存在的函数
        // ("url_validator", url_validator()),
        // ("hash_tool", hash_tool()),
        // ("directory_lister", directory_lister()),
        // ("file_info", file_info()),
        // ("data_transformer", data_transformer()),
        // ("excel_reader", excel_reader()),
    ];

    for (name, tool) in tools {
        if let Some(tool_name) = tool.name() {
            assert!(!tool_name.is_empty(), "工具 {} 名称不能为空", name);
        }
        println!("   ✅ {} 工具创建成功", name);
    }

    println!("   ✅ 所有工具创建函数测试通过");
    Ok(())
}

fn test_vector_storage() -> Result<()> {
    // 测试内存向量存储创建
    let _storage = memory_vector_storage(128, Some(1000))?;

    // 验证存储是否正确创建
    // 注意：由于 VectorStorage trait 的方法是异步的，这里只验证创建成功
    println!("   ✅ 内存向量存储创建成功");

    Ok(())
}

async fn test_specialized_agents() -> Result<()> {
    let llm = lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc();

    // TODO: 这些专用 agent 快速创建函数尚未实现
    // 使用通用的 agent_quick 函数代替

    // 测试 Web Agent (使用专用函数)
    let _web_agent = web_agent("web_helper", "You can browse the web")
        .model(llm.clone())
        .build()?;
    println!("   ✅ Web Agent 创建成功");

    // 测试 File Agent (使用专用函数)
    let _file_agent = file_agent("file_helper", "You can manage files")
        .model(llm.clone())
        .build()?;
    println!("   ✅ File Agent 创建成功");

    // 测试 Data Agent (使用专用函数)
    let _data_agent = data_agent("data_helper", "You can process data")
        .model(llm.clone())
        .build()?;
    println!("   ✅ Data Agent 创建成功");

    Ok(())
}
