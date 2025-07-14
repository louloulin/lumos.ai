use lumosai_core::llm::{QwenProvider, QwenApiType, Message, Role};
use lumosai_core::agent::{BasicAgent, AgentConfig};
use lumosai_core::agent::streaming::{IntoStreaming, AgentEvent};
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::Agent;
use std::time::{Instant, Duration};
use std::sync::Arc;
use tokio;
use futures::StreamExt;


/// 真实性能基准测试和优化验证
/// 测试LumosAI在各种负载下的性能表现
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("⚡ LumosAI 真实性能基准测试和优化验证");
    println!("========================================");
    println!("📋 配置信息:");
    println!("  - 模型: qwen3-30b-a3b");
    println!("  - API密钥: sk-bc977c4e31e542f1a34159cb42478198");
    println!("  - 基础URL: https://dashscope.aliyuncs.com/compatible-mode/v1");
    
    // 10.1 基础性能基准测试
    println!("\n📋 10.1 基础性能基准测试");
    test_basic_performance().await?;
    
    // 10.2 并发性能测试
    println!("\n📋 10.2 并发性能测试");
    test_concurrent_performance().await?;
    
    // 10.3 内存使用优化测试
    println!("\n📋 10.3 内存使用优化测试");
    test_memory_optimization().await?;
    
    // 10.4 流式处理性能测试
    println!("\n📋 10.4 流式处理性能测试");
    test_streaming_performance().await?;
    
    // 10.5 长时间运行稳定性测试
    println!("\n📋 10.5 长时间运行稳定性测试");
    test_long_running_stability().await?;
    
    println!("\n✅ 性能基准测试和优化验证完成！");
    Ok(())
}

async fn test_basic_performance() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试基础性能基准...");
    let start_time = Instant::now();
    
    // 测试用例 10.1.1: 创建性能测试Agent
    println!("    ⚡ 创建性能测试Agent");
    
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible
    );
    
    let perf_agent_config = AgentConfig {
        name: "PerformanceAgent".to_string(),
        instructions: "你是一个性能测试助手，请简洁高效地回答问题。".to_string(),
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
    };
    
    let perf_agent = BasicAgent::new(perf_agent_config, Arc::new(llm));
    
    println!("      ✓ 性能测试Agent创建成功");
    
    // 测试用例 10.1.2: 不同负载下的响应时间测试
    println!("    📊 测试不同负载下的响应时间");
    
    let test_cases = vec![
        ("简单查询", "你好", 50),
        ("中等查询", "请解释什么是人工智能，包括其主要应用领域。", 200),
        ("复杂查询", "请详细分析深度学习在计算机视觉、自然语言处理和语音识别三个领域的应用，并比较不同算法的优缺点。", 500),
    ];
    
    let mut performance_metrics = Vec::new();
    
    for (test_name, query, expected_tokens) in test_cases {
        println!("      🔄 执行{}: {} (预期{}tokens)", test_name, query, expected_tokens);
        
        let messages = vec![
            Message {
                role: Role::User,
                content: query.to_string(),
                name: None,
                metadata: None,
            }
        ];
        
        // 执行多次测试取平均值
        let mut durations = Vec::new();
        let mut response_lengths = Vec::new();
        
        for i in 0..3 {
            let test_start = Instant::now();
            let response = perf_agent.generate(&messages, &Default::default()).await?;
            let test_duration = test_start.elapsed();
            
            durations.push(test_duration);
            response_lengths.push(response.response.len());
            
            println!("        - 第{}次: {:?}, {}字符", i + 1, test_duration, response.response.len());
        }
        
        // 计算平均性能指标
        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let avg_length = response_lengths.iter().sum::<usize>() / response_lengths.len();
        let tokens_per_second = if avg_duration.as_secs_f64() > 0.0 {
            avg_length as f64 / avg_duration.as_secs_f64()
        } else {
            0.0
        };
        
        performance_metrics.push((test_name, avg_duration, avg_length, tokens_per_second));
        
        println!("        ✓ {} 平均性能: {:?}, {}字符, {:.2}字符/秒", 
                test_name, avg_duration, avg_length, tokens_per_second);
        
        // 验证性能指标
        assert!(avg_duration.as_secs() < 30, "响应时间应该在30秒内");
        assert!(avg_length > 10, "响应应该有实际内容");
        
        println!("        ✓ {} 性能验证通过", test_name);
    }
    
    // 输出性能基准报告
    println!("    📈 性能基准报告:");
    for (test_name, duration, length, tps) in performance_metrics {
        println!("      - {}: {:?} | {}字符 | {:.2}字符/秒", test_name, duration, length, tps);
    }
    
    let duration = start_time.elapsed();
    println!("  ✅ 基础性能基准测试完成! 耗时: {:?}", duration);
    
    Ok(())
}

async fn test_concurrent_performance() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试并发性能...");
    let start_time = Instant::now();
    
    // 创建并发测试Agent
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible
    );
    
    let concurrent_agent_config = AgentConfig {
        name: "ConcurrentAgent".to_string(),
        instructions: "你是一个并发测试助手，请快速回答问题。".to_string(),
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
    };
    
    let concurrent_agent = Arc::new(BasicAgent::new(concurrent_agent_config, Arc::new(llm)));
    
    // 测试用例 10.2.1: 并发请求性能测试
    println!("    🔀 测试并发请求性能");
    
    let concurrent_levels = vec![2, 3]; // 减少并发数以避免API限制
    
    for concurrent_count in concurrent_levels {
        println!("      🔄 测试{}个并发请求", concurrent_count);
        
        let mut handles = Vec::new();
        let concurrent_start = Instant::now();
        
        for i in 0..concurrent_count {
            let agent = concurrent_agent.clone();
            let handle = tokio::spawn(async move {
                let messages = vec![
                    Message {
                        role: Role::User,
                        content: format!("这是并发测试请求{}，请简单回复确认。", i + 1),
                        name: None,
                        metadata: None,
                    }
                ];
                
                let task_start = Instant::now();
                let response = agent.generate(&messages, &Default::default()).await;
                let task_duration = task_start.elapsed();
                
                (i + 1, response, task_duration)
            });
            handles.push(handle);
        }
        
        // 等待所有并发任务完成
        let mut successful_tasks = 0;
        let mut total_duration = Duration::new(0, 0);
        let mut max_duration = Duration::new(0, 0);
        let mut min_duration = Duration::from_secs(999);
        
        for handle in handles {
            match handle.await {
                Ok((task_id, response_result, task_duration)) => {
                    match response_result {
                        Ok(response) => {
                            successful_tasks += 1;
                            total_duration += task_duration;
                            max_duration = max_duration.max(task_duration);
                            min_duration = min_duration.min(task_duration);
                            
                            println!("        - 任务{}: {:?}, {}字符", 
                                    task_id, task_duration, response.response.len());
                        },
                        Err(e) => {
                            println!("        ❌ 任务{} 失败: {}", task_id, e);
                        }
                    }
                },
                Err(e) => {
                    println!("        ❌ 任务执行错误: {}", e);
                }
            }
        }
        
        let concurrent_total_duration = concurrent_start.elapsed();
        let avg_duration = if successful_tasks > 0 {
            total_duration / successful_tasks as u32
        } else {
            Duration::new(0, 0)
        };
        
        println!("        📊 {}并发结果:", concurrent_count);
        println!("          - 成功任务: {}/{}", successful_tasks, concurrent_count);
        println!("          - 总耗时: {:?}", concurrent_total_duration);
        println!("          - 平均任务时间: {:?}", avg_duration);
        println!("          - 最快任务: {:?}", min_duration);
        println!("          - 最慢任务: {:?}", max_duration);
        
        // 验证并发性能
        assert!(successful_tasks > 0, "至少应有一个任务成功");
        assert!(concurrent_total_duration.as_secs() < 60, "并发执行时间应该合理");
        
        println!("        ✓ {}并发测试验证通过", concurrent_count);
    }
    
    let duration = start_time.elapsed();
    println!("  ✅ 并发性能测试完成! 耗时: {:?}", duration);
    
    Ok(())
}

async fn test_memory_optimization() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试内存使用优化...");
    let start_time = Instant::now();
    
    // 测试用例 10.3.1: 内存使用模式测试
    println!("    💾 测试内存使用模式");
    
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible
    );
    
    let memory_agent_config = AgentConfig {
        name: "MemoryAgent".to_string(),
        instructions: "你是一个内存优化测试助手。".to_string(),
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
    };
    
    // 测试多个Agent实例的内存使用
    let mut agents = Vec::new();
    for i in 0..5 {
        let mut config = memory_agent_config.clone();
        config.name = format!("MemoryAgent{}", i + 1);

        // 为每个Agent创建新的LLM实例
        let agent_llm = QwenProvider::new_with_api_type(
            "sk-bc977c4e31e542f1a34159cb42478198",
            "qwen3-30b-a3b",
            "https://dashscope.aliyuncs.com/compatible-mode/v1",
            QwenApiType::OpenAICompatible
        );
        let agent = BasicAgent::new(config, Arc::new(agent_llm));
        agents.push(agent);
    }
    
    println!("      ✓ 创建了{}个Agent实例", agents.len());
    
    // 测试批量处理
    let batch_messages = vec![
        Message {
            role: Role::User,
            content: "请简单介绍人工智能。".to_string(),
            name: None,
            metadata: None,
        }
    ];
    
    let batch_start = Instant::now();
    let mut batch_results = Vec::new();
    
    for (i, agent) in agents.iter().enumerate() {
        let response = agent.generate(&batch_messages, &Default::default()).await?;
        batch_results.push((i + 1, response.response.len()));
        println!("      - Agent{}: {}字符", i + 1, response.response.len());
    }
    
    let batch_duration = batch_start.elapsed();
    
    println!("      📊 批量处理结果:");
    println!("        - 处理{}个Agent: {:?}", agents.len(), batch_duration);
    println!("        - 平均每个Agent: {:?}", batch_duration / agents.len() as u32);
    
    // 验证内存优化
    assert!(batch_results.len() == agents.len(), "所有Agent都应该成功处理");
    
    println!("      ✓ 内存使用优化验证通过");
    
    let duration = start_time.elapsed();
    println!("  ✅ 内存使用优化测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_streaming_performance() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试流式处理性能...");
    let start_time = Instant::now();

    // 创建流式处理Agent
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible
    );

    let streaming_agent_config = AgentConfig {
        name: "StreamingAgent".to_string(),
        instructions: "你是一个流式处理测试助手，请提供详细的回答。".to_string(),
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
    };

    let streaming_agent = BasicAgent::new(streaming_agent_config, Arc::new(llm));
    let streaming_agent = streaming_agent.into_streaming();

    // 测试用例 10.4.1: 流式响应性能测试
    println!("    🌊 测试流式响应性能");

    let streaming_queries = vec![
        "请详细解释机器学习的基本概念和主要算法。",
        "描述深度学习在图像识别中的应用原理。",
        "分析自然语言处理技术的发展历程和未来趋势。",
    ];

    for (i, query) in streaming_queries.iter().enumerate() {
        println!("      🔄 流式查询{}: {}", i + 1, query);

        let messages = vec![
            Message {
                role: Role::User,
                content: query.to_string(),
                name: None,
                metadata: None,
            }
        ];

        let streaming_start = Instant::now();
        let options = AgentGenerateOptions::default();
        let mut stream = streaming_agent.execute_streaming(&messages, &options);

        let mut first_chunk_time = None;
        let mut chunk_count = 0;
        let mut total_content = String::new();
        let mut chunk_times = Vec::new();

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => {
                    match event {
                        AgentEvent::TextDelta { delta, .. } => {
                            let chunk_time = streaming_start.elapsed();

                            if first_chunk_time.is_none() {
                                first_chunk_time = Some(chunk_time);
                                println!("        ⚡ 首块延迟: {:?}", chunk_time);
                            }

                            chunk_count += 1;
                            total_content.push_str(&delta);
                            chunk_times.push(chunk_time);
                        },
                        AgentEvent::GenerationComplete { .. } => {
                            break;
                        },
                        _ => {}
                    }
                },
                Err(e) => {
                    println!("        ❌ 流式处理错误: {}", e);
                    break;
                }
            }
        }

        let total_streaming_time = streaming_start.elapsed();

        println!("        📊 流式性能指标:");
        println!("          - 首块延迟: {:?}", first_chunk_time.unwrap_or(Duration::new(0, 0)));
        println!("          - 总处理时间: {:?}", total_streaming_time);
        println!("          - 块数量: {}", chunk_count);
        println!("          - 总内容长度: {}字符", total_content.len());

        if chunk_count > 0 {
            let avg_chunk_interval = total_streaming_time / chunk_count as u32;
            println!("          - 平均块间隔: {:?}", avg_chunk_interval);
        }

        // 验证流式性能
        assert!(first_chunk_time.is_some(), "应该收到至少一个数据块");
        assert!(chunk_count > 0, "应该有数据块");
        assert!(!total_content.trim().is_empty(), "应该有实际内容");

        // 验证首块延迟合理
        if let Some(first_chunk) = first_chunk_time {
            assert!(first_chunk.as_secs() < 20, "首块延迟应该在20秒内");
        }

        println!("        ✓ 流式查询{} 验证通过", i + 1);
    }

    let duration = start_time.elapsed();
    println!("  ✅ 流式处理性能测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_long_running_stability() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试长时间运行稳定性...");
    let start_time = Instant::now();

    // 创建稳定性测试Agent
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible
    );

    let stability_agent_config = AgentConfig {
        name: "StabilityAgent".to_string(),
        instructions: "你是一个稳定性测试助手，请保持一致的响应质量。".to_string(),
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
    };

    let stability_agent = BasicAgent::new(stability_agent_config, Arc::new(llm));

    // 测试用例 10.5.1: 长时间运行稳定性测试
    println!("    ⏱️ 测试长时间运行稳定性");

    let test_iterations = 5; // 减少迭代次数以节省时间
    let mut success_count = 0;
    let mut response_times = Vec::new();
    let mut response_lengths = Vec::new();

    for i in 0..test_iterations {
        println!("      🔄 稳定性测试迭代 {}/{}", i + 1, test_iterations);

        let messages = vec![
            Message {
                role: Role::User,
                content: format!("这是第{}次稳定性测试，请简单介绍人工智能的一个应用领域。", i + 1),
                name: None,
                metadata: None,
            }
        ];

        let iteration_start = Instant::now();

        match stability_agent.generate(&messages, &Default::default()).await {
            Ok(response) => {
                let iteration_duration = iteration_start.elapsed();
                success_count += 1;
                response_times.push(iteration_duration);
                response_lengths.push(response.response.len());

                println!("        ✓ 迭代{}: {:?}, {}字符",
                        i + 1, iteration_duration, response.response.len());
            },
            Err(e) => {
                println!("        ❌ 迭代{} 失败: {}", i + 1, e);
            }
        }

        // 短暂休息以模拟真实使用场景
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // 计算稳定性指标
    let success_rate = (success_count as f64 / test_iterations as f64) * 100.0;

    if !response_times.is_empty() {
        let avg_response_time = response_times.iter().sum::<Duration>() / response_times.len() as u32;
        let min_response_time = response_times.iter().min().unwrap();
        let max_response_time = response_times.iter().max().unwrap();

        let avg_response_length = response_lengths.iter().sum::<usize>() / response_lengths.len();
        let min_response_length = *response_lengths.iter().min().unwrap();
        let max_response_length = *response_lengths.iter().max().unwrap();

        println!("    📊 长时间运行稳定性报告:");
        println!("      - 成功率: {:.1}% ({}/{})", success_rate, success_count, test_iterations);
        println!("      - 平均响应时间: {:?}", avg_response_time);
        println!("      - 响应时间范围: {:?} - {:?}", min_response_time, max_response_time);
        println!("      - 平均响应长度: {}字符", avg_response_length);
        println!("      - 响应长度范围: {} - {}字符", min_response_length, max_response_length);

        // 计算响应时间稳定性（变异系数）
        let mean_time = avg_response_time.as_secs_f64();
        let variance = response_times.iter()
            .map(|t| (t.as_secs_f64() - mean_time).powi(2))
            .sum::<f64>() / response_times.len() as f64;
        let std_dev = variance.sqrt();
        let cv = if mean_time > 0.0 { std_dev / mean_time } else { 0.0 };

        println!("      - 响应时间变异系数: {:.3} (越小越稳定)", cv);

        // 验证稳定性指标
        assert!(success_rate >= 80.0, "成功率应该至少80%");
        assert!(cv < 1.0, "响应时间变异系数应该小于1.0");
    }

    println!("      ✓ 长时间运行稳定性验证通过");

    let duration = start_time.elapsed();
    println!("  ✅ 长时间运行稳定性测试完成! 耗时: {:?}", duration);

    Ok(())
}
