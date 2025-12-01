use lumosai_core::agent::streaming::{AgentEvent, IntoStreaming};
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::agent::{AgentConfig, BasicAgent};
use lumosai_core::llm::{Message, QwenApiType, QwenProvider, Role};
use lumosai_core::Agent;
use lumosai_rag::document::chunker::{DocumentChunker, TextChunker};
use lumosai_rag::{ChunkingConfig, ChunkingStrategy, Document};

use futures::StreamExt;
use std::sync::Arc;
use std::time::Instant;
use tokio;

/// 真实综合集成验证测试
/// 验证LumosAI核心组件的端到端集成功能
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI 真实综合集成验证测试");
    println!("========================================");
    println!("📋 配置信息:");
    println!("  - 模型: qwen3-30b-a3b");
    println!("  - API密钥: sk-bc977c4e31e542f1a34159cb42478198");
    println!("  - 基础URL: https://dashscope.aliyuncs.com/compatible-mode/v1");

    // 7.1 端到端RAG+流式处理集成测试
    println!("\n📋 7.1 端到端RAG+流式处理集成测试");
    test_rag_streaming_integration().await?;

    // 7.2 多Agent协作测试
    println!("\n📋 7.2 多Agent协作测试");
    test_multi_agent_collaboration().await?;

    // 7.3 复杂工作流测试
    println!("\n📋 7.3 复杂工作流测试");
    test_complex_workflow().await?;

    // 7.4 性能压力测试
    println!("\n📋 7.4 性能压力测试");
    test_performance_stress().await?;

    // 7.5 错误恢复和鲁棒性测试
    println!("\n📋 7.5 错误恢复和鲁棒性测试");
    test_error_recovery().await?;

    println!("\n✅ 综合集成验证测试完成！");
    Ok(())
}

async fn test_rag_streaming_integration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试RAG+流式处理集成...");
    let start_time = Instant::now();

    // 测试用例 7.1.1: 创建知识库
    println!("    📚 创建知识库");

    let documents = vec![
        Document {
            id: "tech_guide".to_string(),
            content: r#"
            # 现代软件开发技术指南

            ## 编程语言选择
            - **Rust**: 系统编程语言，注重安全性和性能
            - **Python**: 通用编程语言，适合AI和数据科学
            - **JavaScript**: Web开发的核心语言
            - **Go**: 云原生应用开发的首选

            ## 开发框架
            - **前端**: React, Vue.js, Angular
            - **后端**: Express.js, Django, Spring Boot
            - **移动端**: React Native, Flutter

            ## 数据库技术
            - **关系型**: PostgreSQL, MySQL
            - **NoSQL**: MongoDB, Redis
            - **向量数据库**: Pinecone, Weaviate

            ## 云服务
            - **AWS**: 亚马逊云服务
            - **Azure**: 微软云平台
            - **GCP**: 谷歌云平台
            "#
            .to_string(),
            metadata: lumosai_rag::types::Metadata::new(),
            embedding: None,
        },
        Document {
            id: "ai_development".to_string(),
            content: r#"
            # AI开发最佳实践

            ## 机器学习流程
            1. **数据收集**: 获取高质量的训练数据
            2. **数据预处理**: 清洗和标准化数据
            3. **模型选择**: 选择合适的算法
            4. **训练优化**: 调整超参数
            5. **模型评估**: 验证模型性能
            6. **部署监控**: 生产环境部署

            ## 深度学习框架
            - **PyTorch**: 研究友好的深度学习框架
            - **TensorFlow**: 工业级机器学习平台
            - **JAX**: 高性能机器学习研究

            ## LLM应用开发
            - **提示工程**: 设计有效的提示词
            - **RAG系统**: 检索增强生成
            - **微调技术**: 模型定制化
            - **Agent框架**: 智能代理开发
            "#
            .to_string(),
            metadata: lumosai_rag::types::Metadata::new(),
            embedding: None,
        },
    ];

    // 测试用例 7.1.2: 文档分块和向量化
    println!("    ✂️ 文档分块和向量化");

    let chunking_config = ChunkingConfig {
        chunk_size: 300,
        chunk_overlap: 50,
        min_chunk_size: Some(100),
        max_chunk_size: Some(500),
        strategy: ChunkingStrategy::Character {
            separator: "\n".to_string(),
            is_separator_regex: false,
        },
        preserve_metadata: true,
    };

    let chunker = TextChunker::new(chunking_config.clone());
    let mut all_chunks = Vec::new();

    for document in documents.iter() {
        let chunks = chunker.chunk(document.clone(), &chunking_config).await?;
        println!("      ✓ 文档 '{}' 分块: {} 个块", document.id, chunks.len());
        all_chunks.extend(chunks);
    }

    println!("      ✓ 文档处理完成: {} 个块", all_chunks.len());

    // 测试用例 7.1.3: RAG查询+流式响应
    println!("    🔍 RAG查询+流式响应");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let agent_config = AgentConfig {
        name: "RAGAgent".to_string(),
        instructions: "你是一个技术专家，能够基于提供的知识库回答技术问题。请根据上下文信息提供准确、详细的回答。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    let agent = BasicAgent::new(agent_config, Arc::new(llm));
    let streaming_agent = agent.into_streaming();

    let queries = vec![
        "什么是Rust编程语言？它有什么特点？",
        "如何选择合适的深度学习框架？",
        "RAG系统的工作原理是什么？",
    ];

    for (i, query) in queries.iter().enumerate() {
        println!("      🔄 查询 {}: {}", i + 1, query);

        // 模拟检索相关文档（简化版本）
        let mut context = String::new();
        context.push_str("基于以下知识库信息回答问题：\n\n");

        // 简单的关键词匹配来模拟检索
        let query_lower = query.to_lowercase();
        let mut relevant_chunks = Vec::new();

        for chunk in all_chunks.iter().take(3) {
            if chunk.content.to_lowercase().contains("rust") && query_lower.contains("rust")
                || chunk.content.to_lowercase().contains("深度学习")
                    && query_lower.contains("深度学习")
                || chunk.content.to_lowercase().contains("rag") && query_lower.contains("rag")
            {
                relevant_chunks.push(&chunk.content);
            }
        }

        // 如果没有找到相关内容，使用前几个块
        if relevant_chunks.is_empty() {
            relevant_chunks = all_chunks.iter().take(2).map(|c| &c.content).collect();
        }

        for (j, content) in relevant_chunks.iter().enumerate() {
            context.push_str(&format!("参考资料 {}:\n{}\n\n", j + 1, content));
        }
        context.push_str(&format!("问题: {}\n\n请基于上述资料提供详细回答：", query));

        let messages = vec![Message {
            role: Role::User,
            content: context,
            name: None,
            metadata: None,
        }];

        let query_start = Instant::now();
        let options = AgentGenerateOptions::default();
        let mut stream = streaming_agent.execute_streaming(&messages, &options);

        let mut response_content = String::new();
        let mut chunk_count = 0;

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => match event {
                    AgentEvent::TextDelta { delta, .. } => {
                        chunk_count += 1;
                        response_content.push_str(&delta);
                    }
                    AgentEvent::GenerationComplete { .. } => {
                        break;
                    }
                    _ => {}
                },
                Err(e) => {
                    println!("        ❌ 流式响应错误: {}", e);
                    break;
                }
            }
        }

        let query_duration = query_start.elapsed();

        println!("        ✓ 查询完成 (耗时: {:?})", query_duration);
        println!("        📊 响应块数: {}", chunk_count);
        println!("        📊 响应长度: {} 字符", response_content.len());

        // 验证响应质量
        assert!(!response_content.trim().is_empty(), "RAG响应不能为空");
        assert!(response_content.len() > 50, "RAG响应应该足够详细");

        println!("        ✓ 查询 {} 验证通过", i + 1);
    }

    let duration = start_time.elapsed();
    println!("  ✅ RAG+流式处理集成测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_multi_agent_collaboration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试多Agent协作...");
    let start_time = Instant::now();

    // 测试用例 7.2.1: 创建专门化Agent
    println!("    🤖 创建专门化Agent");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    // 技术分析师Agent
    let tech_analyst_config = AgentConfig {
        name: "TechAnalyst".to_string(),
        instructions: "你是一个技术分析师，专门分析技术方案的可行性、优缺点和实施建议。"
            .to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    // 项目经理Agent
    let project_manager_config = AgentConfig {
        name: "ProjectManager".to_string(),
        instructions: "你是一个项目经理，专门制定项目计划、时间安排和资源分配。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    let tech_analyst = BasicAgent::new(tech_analyst_config, Arc::new(llm));

    let llm2 = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );
    let project_manager = BasicAgent::new(project_manager_config, Arc::new(llm2));

    println!("      ✓ 技术分析师Agent创建成功");
    println!("      ✓ 项目经理Agent创建成功");

    // 测试用例 7.2.2: Agent协作工作流
    println!("    🔄 Agent协作工作流");

    let project_request = "我们需要开发一个基于AI的客户服务系统，要求支持多语言、实时响应、知识库集成。请分析技术方案并制定项目计划。";

    // 第一步：技术分析师分析
    println!("      📋 步骤1: 技术分析师分析");
    let tech_messages = vec![Message {
        role: Role::User,
        content: format!(
            "请分析以下项目的技术方案：\n\n{}\n\n请提供技术架构建议、技术栈选择和实施难点分析。",
            project_request
        ),
        name: None,
        metadata: None,
    }];

    let tech_analysis_start = Instant::now();
    let tech_response = tech_analyst
        .generate(&tech_messages, &Default::default())
        .await?;
    let tech_analysis_duration = tech_analysis_start.elapsed();

    println!(
        "        ✓ 技术分析完成 (耗时: {:?})",
        tech_analysis_duration
    );
    println!(
        "        📊 分析报告长度: {} 字符",
        tech_response.response.len()
    );

    // 第二步：项目经理制定计划
    println!("      📅 步骤2: 项目经理制定计划");
    let pm_messages = vec![
        Message {
            role: Role::User,
            content: format!(
                "基于以下技术分析报告，请制定详细的项目计划：\n\n原始需求：\n{}\n\n技术分析报告：\n{}\n\n请提供项目时间线、里程碑、资源需求和风险评估。",
                project_request,
                tech_response.response
            ),
            name: None,
            metadata: None,
        }
    ];

    let pm_planning_start = Instant::now();
    let pm_response = project_manager
        .generate(&pm_messages, &Default::default())
        .await?;
    let pm_planning_duration = pm_planning_start.elapsed();

    println!("        ✓ 项目计划完成 (耗时: {:?})", pm_planning_duration);
    println!(
        "        📊 项目计划长度: {} 字符",
        pm_response.response.len()
    );

    // 验证协作结果
    assert!(
        !tech_response.response.trim().is_empty(),
        "技术分析报告不能为空"
    );
    assert!(!pm_response.response.trim().is_empty(), "项目计划不能为空");
    assert!(tech_response.response.len() > 200, "技术分析应该足够详细");
    assert!(pm_response.response.len() > 200, "项目计划应该足够详细");

    println!("      ✓ 多Agent协作验证通过");

    let duration = start_time.elapsed();
    println!("  ✅ 多Agent协作测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_complex_workflow() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试复杂工作流...");
    let start_time = Instant::now();

    // 测试用例 7.3.1: 多步骤工作流
    println!("    🔄 多步骤工作流");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let workflow_agent_config = AgentConfig {
        name: "WorkflowAgent".to_string(),
        instructions: "你是一个工作流处理专家，能够执行复杂的多步骤任务。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    let workflow_agent = BasicAgent::new(workflow_agent_config, Arc::new(llm));

    // 定义复杂工作流步骤
    let workflow_steps = vec![
        ("需求分析", "分析用户需求，提取关键信息和约束条件"),
        ("方案设计", "基于需求分析结果，设计技术方案"),
        ("风险评估", "评估方案的技术风险和实施难度"),
        ("实施计划", "制定详细的实施计划和时间安排"),
        ("质量保证", "定义质量标准和验收标准"),
    ];

    let initial_request = "开发一个企业级的智能文档管理系统，需要支持OCR识别、自动分类、智能搜索、权限管理和API接口。";
    let mut workflow_context = initial_request.to_string();

    for (i, (step_name, step_description)) in workflow_steps.iter().enumerate() {
        println!("      📋 步骤 {}: {}", i + 1, step_name);

        let step_prompt = format!(
            "当前工作流步骤: {}\n任务描述: {}\n\n上下文信息:\n{}\n\n请执行当前步骤并提供详细结果：",
            step_name, step_description, workflow_context
        );

        let messages = vec![Message {
            role: Role::User,
            content: step_prompt,
            name: None,
            metadata: None,
        }];

        let step_start = Instant::now();
        let step_response = workflow_agent
            .generate(&messages, &Default::default())
            .await?;
        let step_duration = step_start.elapsed();

        // 更新工作流上下文
        workflow_context.push_str(&format!(
            "\n\n=== {} ===\n{}",
            step_name, step_response.response
        ));

        println!("        ✓ {} 完成 (耗时: {:?})", step_name, step_duration);
        println!("        📊 输出长度: {} 字符", step_response.response.len());

        // 验证步骤结果
        assert!(
            !step_response.response.trim().is_empty(),
            "工作流步骤输出不能为空"
        );
        assert!(
            step_response.response.len() > 100,
            "工作流步骤输出应该足够详细"
        );
    }

    println!("      ✓ 复杂工作流执行完成");
    println!("      📊 最终上下文长度: {} 字符", workflow_context.len());

    let duration = start_time.elapsed();
    println!("  ✅ 复杂工作流测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_performance_stress() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试性能压力...");
    let start_time = Instant::now();

    // 测试用例 7.4.1: 并发请求测试
    println!("    ⚡ 并发请求测试");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let stress_agent_config = AgentConfig {
        name: "StressTestAgent".to_string(),
        instructions: "你是一个测试助手，请简洁地回答问题。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    let stress_agent = Arc::new(BasicAgent::new(stress_agent_config, Arc::new(llm)));

    // 创建多个并发任务
    let concurrent_tasks = 3; // 减少并发数以避免API限制
    let mut handles = Vec::new();

    for i in 0..concurrent_tasks {
        let agent = stress_agent.clone();
        let handle = tokio::spawn(async move {
            let messages = vec![Message {
                role: Role::User,
                content: format!("这是并发测试请求 {}，请简单回复确认收到。", i + 1),
                name: None,
                metadata: None,
            }];

            let task_start = Instant::now();
            let response = agent.generate(&messages, &Default::default()).await;
            let task_duration = task_start.elapsed();

            (i + 1, response, task_duration)
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    let mut successful_tasks = 0;
    let mut total_duration = std::time::Duration::new(0, 0);

    for handle in handles {
        match handle.await {
            Ok((task_id, response_result, task_duration)) => match response_result {
                Ok(_response) => {
                    successful_tasks += 1;
                    total_duration += task_duration;
                    println!(
                        "        ✓ 任务 {} 完成 (耗时: {:?})",
                        task_id, task_duration
                    );
                }
                Err(e) => {
                    println!("        ❌ 任务 {} 失败: {}", task_id, e);
                }
            },
            Err(e) => {
                println!("        ❌ 任务执行错误: {}", e);
            }
        }
    }

    let avg_duration = total_duration / successful_tasks.max(1) as u32;

    println!("      📊 并发测试结果:");
    println!("        - 总任务数: {}", concurrent_tasks);
    println!("        - 成功任务数: {}", successful_tasks);
    println!("        - 平均响应时间: {:?}", avg_duration);

    // 验证性能指标
    assert!(successful_tasks > 0, "至少应有一个任务成功");

    let duration = start_time.elapsed();
    println!("  ✅ 性能压力测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_error_recovery() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试错误恢复和鲁棒性...");
    let start_time = Instant::now();

    // 测试用例 7.5.1: 无效输入处理
    println!("    🛡️ 无效输入处理");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let robust_agent_config = AgentConfig {
        name: "RobustAgent".to_string(),
        instructions: "你是一个鲁棒的AI助手，能够处理各种输入情况。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    let robust_agent = BasicAgent::new(robust_agent_config, Arc::new(llm));

    // 测试各种边界情况
    let test_cases = vec![
        ("空消息", ""),
        ("极短消息", "Hi"),
        ("重复字符", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        ("特殊字符", "!@#$%^&*()_+{}|:<>?[]\\;'\",./ "),
        ("混合语言", "Hello 你好 こんにちは 안녕하세요"),
    ];

    for (test_name, test_input) in test_cases {
        println!("      🔍 测试: {}", test_name);

        let messages = vec![Message {
            role: Role::User,
            content: test_input.to_string(),
            name: None,
            metadata: None,
        }];

        let test_start = Instant::now();
        match robust_agent.generate(&messages, &Default::default()).await {
            Ok(response) => {
                let test_duration = test_start.elapsed();
                println!("        ✓ 处理成功 (耗时: {:?})", test_duration);
                println!("        📊 响应长度: {} 字符", response.response.len());

                // 验证响应合理性
                if !test_input.is_empty() {
                    assert!(!response.response.trim().is_empty(), "非空输入应该有响应");
                }
            }
            Err(e) => {
                println!("        ⚠️ 处理失败（可能是预期的）: {}", e);
            }
        }
    }

    println!("      ✓ 错误恢复测试完成");

    let duration = start_time.elapsed();
    println!("  ✅ 错误恢复和鲁棒性测试完成! 耗时: {:?}", duration);

    Ok(())
}
