//! 增强应用演示
//! 
//! 展示新的增强应用功能，包括工具注册表、增强内存管理和统一应用管理

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use tokio;

use lumosai_core::{
    app::{EnhancedApp, EnhancedAppConfig, ToolsConfig},
    agent::{AgentConfig, BasicAgent},
    tool::{ToolMetadata, ToolCategory, GenericTool, ToolSchema, ParameterSchema},
    memory::{EnhancedMemory, MemoryConfig, MemoryEntry, MemoryEntryType, MemoryQueryOptions},
    llm::{MockLlmProvider, Message, Role},
    vector::MemoryVectorStorage,
    logger::{Component, LogLevel, ConsoleLogger},
    error::Result,
};

/// 创建示例工具
fn create_sample_tools() -> Vec<(Arc<dyn lumosai_core::tool::Tool>, ToolMetadata)> {
    let mut tools = Vec::new();

    // 计算器工具
    let calculator_tool: Arc<dyn lumosai_core::tool::Tool> = Arc::new(GenericTool::new(
        "calculator".to_string(),
        "执行基本数学计算".to_string(),
        ToolSchema::new(vec![
            ParameterSchema {
                name: "operation".to_string(),
                description: "数学运算符 (+, -, *, /)".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            },
            ParameterSchema {
                name: "a".to_string(),
                description: "第一个数字".to_string(),
                r#type: "number".to_string(),
                required: true,
                properties: None,
                default: None,
            },
            ParameterSchema {
                name: "b".to_string(),
                description: "第二个数字".to_string(),
                r#type: "number".to_string(),
                required: true,
                properties: None,
                default: None,
            },
        ]),
        |params, _context| {
            let operation = params.get("operation").and_then(|v| v.as_str()).unwrap_or("+");
            let a = params.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = params.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);

            let result = match operation {
                "+" => a + b,
                "-" => a - b,
                "*" => a * b,
                "/" => if b != 0.0 { a / b } else { f64::NAN },
                _ => f64::NAN,
            };

            Ok(json!({
                "result": result,
                "operation": format!("{} {} {} = {}", a, operation, b, result)
            }))
        },
    ));

    let calculator_metadata = ToolMetadata {
        name: "calculator".to_string(),
        description: "基本数学计算工具".to_string(),
        version: "1.0.0".to_string(),
        author: Some("Lumosai Team".to_string()),
        category: ToolCategory::Math,
        tags: vec!["math".to_string(), "calculation".to_string()],
        requires_auth: false,
        permissions: vec![],
        dependencies: vec![],
    };

    tools.push((calculator_tool, calculator_metadata));

    tools
}

/// 创建示例代理
async fn create_sample_agent(llm: Arc<dyn lumosai_core::llm::LlmProvider>) -> Result<Arc<BasicAgent>> {
    let config = AgentConfig {
        name: "助手代理".to_string(),
        instructions: "你是一个有用的AI助手，可以帮助用户进行计算和文本处理。".to_string(),
        memory_config: Some(MemoryConfig::default()),
        model_id: Some("mock-model".to_string()),
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(true),
        context: Some(HashMap::new()),
        metadata: Some(HashMap::new()),
        max_tool_calls: Some(5),
        tool_timeout: Some(30),
    };

    let agent = BasicAgent::new(config, llm);
    Ok(Arc::new(agent))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 增强应用演示");
    println!("{}", "=".repeat(50));

    // 初始化日志器
    let _logger = Arc::new(ConsoleLogger::new(
        "EnhancedAppDemo", 
        Component::Agent, 
        LogLevel::Info
    ));

    // 创建应用配置
    let app_config = EnhancedAppConfig {
        name: "演示应用".to_string(),
        description: Some("展示增强功能的演示应用".to_string()),
        version: Some("1.0.0".to_string()),
        default_llm: Some("mock".to_string()),
        memory: Some(MemoryConfig::default()),
        tools: Some(ToolsConfig {
            enable_registry: true,
            auto_discover: false,
            tool_directories: vec![],
            preload: vec!["calculator".to_string(), "text_processor".to_string()],
        }),
        agents: None,
        workflows: None,
        rag: None,
        env: None,
    };

    // 创建增强应用
    let mut app = EnhancedApp::new(app_config);

    // 添加LLM提供者 - 创建具有正确维度的嵌入向量
    let embeddings = vec![
        vec![0.1; 384], // 384维的嵌入向量
        vec![0.2; 384], // 另一个384维的嵌入向量
        vec![0.3; 384], // 第三个384维的嵌入向量
        vec![0.4; 384], // 第四个384维的嵌入向量
    ];
    let llm = Arc::new(MockLlmProvider::new_with_embeddings(embeddings));
    llm.add_response("计算结果: 100".to_string());
    app.add_llm_provider("mock".to_string(), llm.clone())?;

    // 创建并设置向量存储
    let vector_storage = Arc::new(MemoryVectorStorage::new(384, None));
    app.set_vector_storage(vector_storage.clone());

    // 创建并设置增强内存
    let memory_config = MemoryConfig::default();
    let enhanced_memory = Arc::new(EnhancedMemory::new(
        vector_storage,
        llm.clone(),
        memory_config,
    ));
    app.set_memory(enhanced_memory.clone());

    // 添加工具
    println!("\n📦 注册工具...");
    let tools = create_sample_tools();
    for (tool, metadata) in tools {
        let tool_name = metadata.name.clone();
        app.add_tool(tool, metadata)?;
        println!("✅ 工具 '{}' 注册成功", tool_name);
    }

    // 添加代理
    println!("\n🤖 创建代理...");
    let agent = create_sample_agent(llm.clone()).await?;
    app.add_agent("assistant".to_string(), agent)?;
    println!("✅ 代理 'assistant' 创建成功");

    // 启动应用
    println!("\n🚀 启动应用...");
    app.start().await?;

    // 演示1: 工具搜索和发现
    println!("\n📊 演示1: 工具搜索和发现");
    println!("{}", "=".repeat(30));
    
    let all_tools = app.search_tools("")?;
    println!("📋 所有工具: {:?}", all_tools);
    
    let math_tools = app.find_tools_by_category(&ToolCategory::Math)?;
    println!("🔢 数学工具: {:?}", math_tools);
    
    let text_tools = app.find_tools_by_category(&ToolCategory::Text)?;
    println!("📝 文本工具: {:?}", text_tools);

    // 演示2: 工具执行
    println!("\n🔧 演示2: 工具执行");
    println!("{}", "=".repeat(30));
    
    if let Some(calculator) = app.get_tool("calculator")? {
        let params = json!({
            "operation": "+",
            "a": 15,
            "b": 25
        });
        
        let context = lumosai_core::tool::ToolExecutionContext::default();
        let options = lumosai_core::tool::ToolExecutionOptions::default();
        
        match calculator.execute(params, context, &options).await {
            Ok(result) => println!("🧮 计算结果: {}", serde_json::to_string_pretty(&result)?),
            Err(e) => println!("❌ 计算失败: {}", e),
        }
    }

    // 演示3: 内存管理
    println!("\n🧠 演示3: 增强内存管理");
    println!("{}", "=".repeat(30));
    
    // 存储一些内存条目
    let entries = vec![
        MemoryEntry {
            id: "entry1".to_string(),
            entry_type: MemoryEntryType::Fact,
            content: "用户喜欢数学计算".to_string(),
            metadata: HashMap::new(),
            created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            updated_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            importance: 0.8,
            access_count: 0,
            last_accessed: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            thread_id: Some("demo_thread".to_string()),
            resource_id: None,
        },
        MemoryEntry {
            id: "entry2".to_string(),
            entry_type: MemoryEntryType::Context,
            content: "用户正在学习AI应用开发".to_string(),
            metadata: HashMap::new(),
            created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            updated_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            importance: 0.9,
            access_count: 0,
            last_accessed: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            thread_id: Some("demo_thread".to_string()),
            resource_id: None,
        },
    ];

    for entry in entries {
        enhanced_memory.store_entry(entry).await?;
    }
    println!("💾 内存条目存储完成");

    // 查询内存
    let query_options = MemoryQueryOptions {
        query: "数学".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.1),
        filters: None,
        thread_id: Some("demo_thread".to_string()),
        resource_id: None,
        entry_types: None,
        time_range: None,
        importance_threshold: Some(0.5),
    };

    let results = enhanced_memory.query(&query_options).await?;
    println!("🔍 内存查询结果: {} 条", results.len());
    for result in results {
        println!("  - {:?}: {}", result.entry_type, result.content);
    }

    // 演示4: 代理交互
    println!("\n🤖 演示4: 代理交互");
    println!("{}", "=".repeat(30));
    
    let messages = vec![
        Message {
            role: Role::User,
            content: "请帮我计算 42 + 58".to_string(),
            name: None,
            metadata: None,
        }
    ];

    match app.run_agent("assistant", &messages).await {
        Ok(response) => println!("🗣️ 代理回复: {}", response),
        Err(e) => println!("❌ 代理执行失败: {}", e),
    }

    // 演示5: 应用统计
    println!("\n📊 演示5: 应用统计");
    println!("{}", "=".repeat(30));
    
    let stats = app.get_stats()?;
    println!("📈 应用统计:");
    println!("  - 代理数量: {}", stats.agents_count);
    println!("  - 工具数量: {}", stats.tools_count);
    println!("  - 工具类别数量: {}", stats.tool_categories_count);
    println!("  - LLM提供者数量: {}", stats.llm_providers_count);
    println!("  - 工作流数量: {}", stats.workflows_count);
    println!("  - RAG管道数量: {}", stats.rag_pipelines_count);

    // 停止应用
    println!("\n🛑 停止应用...");
    app.stop().await?;

    println!("\n✅ 演示完成！");
    println!("这个演示展示了 Lumosai 的增强功能:");
    println!("  - 🔧 动态工具注册和发现");
    println!("  - 🧠 增强的内存管理系统");
    println!("  - 🚀 统一的应用管理接口");
    println!("  - 📊 实时统计和监控");
    println!("  - 🤖 智能代理集成");

    Ok(())
}
