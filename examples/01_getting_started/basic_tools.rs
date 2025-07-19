//! 基础工具使用示例 - 展示如何为Agent添加和使用工具
//! 
//! 这个示例展示了LumosAI内置工具库的使用方法，以及如何为Agent配置工具。
//! 
//! 运行方式:
//! ```bash
//! cargo run --example basic_tools
//! ```

use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::agent::trait_def::Agent;
use lumosai_core::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 LumosAI 基础工具使用示例");
    println!("============================");
    
    // 创建LLM提供者
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我已经为您计算了结果。".to_string(),
        "我已经获取了当前时间信息。".to_string(),
        "我已经生成了UUID。".to_string(),
        "我已经处理了您的文件操作请求。".to_string(),
        "我已经完成了数据处理任务。".to_string(),
    ]));
    
    // 1. 数学工具演示
    println!("\n1️⃣ 数学工具演示");
    println!("----------------");
    
    let math_agent = quick_agent("math_assistant", "你是一个数学计算助手")
        .model(llm.clone())
        .tools(vec![
            calculator(),
            statistics(),
        ])
        .build()?;
    
    println!("🧮 数学助手创建成功，可用工具:");
    for (name, tool) in math_agent.get_tools() {
        println!("   - {}: {}", name, tool.name().unwrap_or("未知工具"));
    }

    let math_response = math_agent.generate_simple("帮我计算 15 * 23 + 45").await?;
    println!("🤖 数学助手: {}", math_response);
    
    // 2. 时间和系统工具演示
    println!("\n2️⃣ 时间和系统工具演示");
    println!("----------------------");
    
    let system_agent = quick_agent("system_assistant", "你是一个系统工具助手")
        .model(llm.clone())
        .tools(vec![
            time_tool(),
            uuid_generator(),
            hash_tool(),
        ])
        .build()?;
    
    println!("⚙️ 系统助手创建成功，可用工具:");
    for (name, tool) in system_agent.get_tools() {
        println!("   - {}: {}", name, tool.name().unwrap_or("未知工具"));
    }
    
    let system_response = system_agent.generate_simple("请告诉我当前时间并生成一个UUID").await?;
    println!("🤖 系统助手: {}", system_response);
    
    // 3. 文件操作工具演示
    println!("\n3️⃣ 文件操作工具演示");
    println!("--------------------");
    
    let file_agent = quick_agent("file_assistant", "你是一个文件操作助手")
        .model(llm.clone())
        .tools(vec![
            file_reader(),
            file_writer(),
            directory_lister(),
            file_info(),
        ])
        .build()?;
    
    println!("📁 文件助手创建成功，可用工具:");
    for (name, tool) in file_agent.get_tools() {
        println!("   - {}: {}", name, tool.name().unwrap_or("未知工具"));
    }
    
    let file_response = file_agent.generate_simple("请列出当前目录的文件").await?;
    println!("🤖 文件助手: {}", file_response);
    
    // 4. 数据处理工具演示
    println!("\n4️⃣ 数据处理工具演示");
    println!("--------------------");
    
    let data_agent = quick_agent("data_assistant", "你是一个数据处理助手")
        .model(llm.clone())
        .tools(vec![
            json_parser(),
            csv_parser(),
            data_transformer(),
            excel_reader(),
        ])
        .build()?;
    
    println!("📊 数据助手创建成功，可用工具:");
    for (name, tool) in data_agent.get_tools() {
        println!("   - {}: {}", name, tool.name().unwrap_or("未知工具"));
    }
    
    let data_response = data_agent.generate_simple("请解析这个JSON数据: {\"name\": \"张三\", \"age\": 25}").await?;
    println!("🤖 数据助手: {}", data_response);
    
    // 5. 网络工具演示
    println!("\n5️⃣ 网络工具演示");
    println!("----------------");
    
    let web_agent = quick_agent("web_assistant", "你是一个网络操作助手")
        .model(llm.clone())
        .tools(vec![
            http_request(),
            web_scraper(),
            json_api(),
            url_validator(),
        ])
        .build()?;
    
    println!("🌐 网络助手创建成功，可用工具:");
    for (name, tool) in web_agent.get_tools() {
        println!("   - {}: {}", name, tool.name().unwrap_or("未知工具"));
    }
    
    let web_response = web_agent.generate_simple("请验证这个URL是否有效: https://www.example.com").await?;
    println!("🤖 网络助手: {}", web_response);
    
    // 6. 组合工具演示
    println!("\n6️⃣ 组合工具演示");
    println!("----------------");
    
    let multi_tool_agent = quick_agent("multi_assistant", "你是一个多功能助手")
        .model(llm.clone())
        .tools(vec![
            calculator(),
            time_tool(),
            file_reader(),
            json_parser(),
            web_scraper(),
        ])
        .build()?;
    
    println!("🎯 多功能助手创建成功，工具数量: {}", multi_tool_agent.get_tools().len());
    println!("🔧 可用工具类别:");
    println!("   - 数学计算: calculator");
    println!("   - 时间处理: time_tool");
    println!("   - 文件操作: file_reader");
    println!("   - 数据处理: json_parser");
    println!("   - 网络操作: web_scraper");
    
    let multi_response = multi_tool_agent.generate_simple("请告诉我你的所有能力").await?;
    println!("🤖 多功能助手: {}", multi_response);
    
    // 7. 专用Agent快速创建演示
    println!("\n7️⃣ 专用Agent快速创建演示");
    println!("---------------------------");
    
    // 使用预配置的专用Agent
    let quick_web_agent = web_agent_quick("quick_web", "快速网络助手")
        .model(llm.clone())
        .build()?;
    
    let quick_file_agent = file_agent_quick("quick_file", "快速文件助手")
        .model(llm.clone())
        .build()?;
    
    let quick_data_agent = data_agent_quick("quick_data", "快速数据助手")
        .model(llm.clone())
        .build()?;
    
    println!("⚡ 快速创建的专用Agent:");
    println!("   - 网络助手工具数: {}", quick_web_agent.get_tools().len());
    println!("   - 文件助手工具数: {}", quick_file_agent.get_tools().len());
    println!("   - 数据助手工具数: {}", quick_data_agent.get_tools().len());
    
    // 8. 工具性能测试
    println!("\n8️⃣ 工具性能测试");
    println!("------------------");
    
    let start = std::time::Instant::now();
    
    // 创建包含多个工具的Agent
    let _performance_agent = quick_agent("performance", "性能测试助手")
        .model(llm.clone())
        .tools(vec![
            calculator(),
            time_tool(),
            uuid_generator(),
            hash_tool(),
            file_reader(),
            json_parser(),
            http_request(),
        ])
        .build()?;
    
    let duration = start.elapsed();
    println!("⏱️ 创建包含7个工具的Agent耗时: {:?}", duration);
    
    println!("\n🎉 基础工具使用示例完成!");
    println!("\n📚 下一步学习:");
    println!("   - examples/02_intermediate/custom_tools.rs - 学习自定义工具");
    println!("   - examples/02_intermediate/builder_pattern.rs - 学习构建器模式");
    println!("   - docs/best-practices/tool-development.md - 工具开发最佳实践");
    
    Ok(())
}

/// 演示工具的详细使用方法
async fn demonstrate_tool_usage() -> Result<()> {
    let llm = Arc::new(MockLlmProvider::new(vec!["工具调用完成".to_string()]));
    
    println!("🔍 工具使用详细演示");
    println!("==================");
    
    // 创建Agent
    let agent = quick_agent("demo", "演示助手")
        .model(llm)
        .tools(vec![calculator(), time_tool()])
        .build()?;
    
    // 获取工具信息
    println!("\n📋 工具详细信息:");
    for (name, tool) in agent.get_tools() {
        println!("工具名称: {}", name);
        println!("工具描述: {}", tool.name().unwrap_or("未知工具"));
        println!("参数模式: {:?}", tool.schema());
        println!("---");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_tools_example() {
        let result = main().await;
        assert!(result.is_ok(), "基础工具示例应该成功运行");
    }
    
    #[tokio::test]
    async fn test_tool_categories() {
        let llm = Arc::new(MockLlmProvider::new(vec!["test".to_string()]));
        
        // 测试数学工具
        let math_agent = quick_agent("math", "test")
            .model(llm.clone())
            .tools(vec![calculator(), statistics()])
            .build();
        assert!(math_agent.is_ok());
        assert_eq!(math_agent.unwrap().get_tools().len(), 2);
        
        // 测试系统工具
        let system_agent = quick_agent("system", "test")
            .model(llm.clone())
            .tools(vec![time_tool(), uuid_generator(), hash_tool()])
            .build();
        assert!(system_agent.is_ok());
        assert_eq!(system_agent.unwrap().get_tools().len(), 3);
        
        // 测试文件工具
        let file_agent = quick_agent("file", "test")
            .model(llm.clone())
            .tools(vec![file_reader(), file_writer(), directory_lister()])
            .build();
        assert!(file_agent.is_ok());
        assert_eq!(file_agent.unwrap().get_tools().len(), 3);
    }
    
    #[tokio::test]
    async fn test_specialized_agents() {
        let llm = Arc::new(MockLlmProvider::new(vec!["test".to_string()]));
        
        // 测试专用Agent的工具数量
        let web_agent = web_agent_quick("web", "test").model(llm.clone()).build().unwrap();
        let file_agent = file_agent_quick("file", "test").model(llm.clone()).build().unwrap();
        let data_agent = data_agent_quick("data", "test").model(llm.clone()).build().unwrap();
        
        assert!(web_agent.get_tools().len() > 0, "Web Agent应该有预配置的工具");
        assert!(file_agent.get_tools().len() > 0, "File Agent应该有预配置的工具");
        assert!(data_agent.get_tools().len() > 0, "Data Agent应该有预配置的工具");
    }
}
