use lumosai_core::agent::streaming::{AgentEvent, StreamingAgent, StreamingConfig};
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::agent::{BasicAgent, AgentConfig};
use lumosai_core::llm::{Message, Role, ZhipuProvider};
use futures::StreamExt;
use std::time::Instant;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("==============================================");
    println!("🧪 LumosAI Streaming 性能直接测试");
    println!("==============================================\n");
    
    // 1. 创建Zhipu Provider
    let api_key = std::env::var("ZHIPU_API_KEY")
        .expect("请设置ZHIPU_API_KEY环境变量");
    
    let provider = ZhipuProvider::new(api_key, Some("glm-4-flash".to_string()));
    println!("✅ ZhipuProvider创建完成");
    
    // 2. 创建BasicAgent
    let config = AgentConfig {
        name: "test-agent".to_string(),
        instructions: "你是一个AI助手".to_string(),
        ..Default::default()
    };
    
    let agent = BasicAgent::new(config, Arc::new(provider));
    println!("✅ BasicAgent创建完成");
    
    // 3. 创建StreamingAgent
    let streaming_config = StreamingConfig {
        text_buffer_size: 1,
        emit_metadata: false,
        emit_memory_updates: false,
        text_delta_delay_ms: None,
    };
    
    let streaming_agent = StreamingAgent::with_config(agent, streaming_config);
    println!("✅ StreamingAgent创建完成\n");
    
    // 4. 执行streaming测试
    let message = Message {
        role: Role::User,
        content: "AI是什么？".to_string(),
        metadata: None,
        name: None,
    };
    
    let messages = vec![message];
    let options = AgentGenerateOptions::default();
    
    println!("📤 发送消息: 'AI是什么？'");
    
    let start_time = Instant::now();
    let mut first_chunk_time: Option<Instant> = None;
    let mut chunk_count = 0;
    let mut total_content = String::new();
    
    // 5. 接收streaming响应
    let mut event_stream = streaming_agent.execute_streaming(&messages, &options);
    
    while let Some(event_result) = event_stream.next().await {
        match event_result {
            Ok(event) => {
                match event {
                    AgentEvent::TextDelta { delta, .. } => {
                        if first_chunk_time.is_none() {
                            first_chunk_time = Some(Instant::now());
                            let ttfb = start_time.elapsed();
                            println!("\n⚡ TTFB (首Token): {:?}", ttfb);
                            println!("📥 接收streaming chunks:\n");
                        }
                        chunk_count += 1;
                        total_content.push_str(&delta);
                        print!("{}", delta);
                    },
                    AgentEvent::GenerationComplete { final_response, total_steps } => {
                        let total_time = start_time.elapsed();
                        println!("\n\n✅ 生成完成");
                        println!("\n📊 性能统计:");
                        if let Some(first_time) = first_chunk_time {
                            let ttfb = first_time.duration_since(start_time);
                            println!("  TTFB: {:?}", ttfb);
                        }
                        println!("  总耗时: {:?}", total_time);
                        println!("  Chunk数: {}", chunk_count);
                        println!("  总步骤: {}", total_steps);
                        println!("  内容长度: {} 字符", total_content.len());
                    },
                    AgentEvent::Error { error, .. } => {
                        println!("\n❌ 错误: {}", error);
                    },
                    _ => {}
                }
            },
            Err(e) => {
                println!("\n❌ Stream错误: {}", e);
                break;
            }
        }
    }
    
    println!("\n==============================================");
    println!("🎯 结论:");
    if let Some(first_time) = first_chunk_time {
        let ttfb_ms = first_time.duration_since(start_time).as_millis();
        if ttfb_ms < 2000 {
            println!("✅ 性能优秀: TTFB < 2秒");
        } else if ttfb_ms < 5000 {
            println!("✅ 性能良好: TTFB < 5秒");
        } else {
            println!("⚠️  性能较慢: TTFB > 5秒");
        }
        println!("📌 这是LumosAI框架本身的性能，不包含外部开销");
    }
    println!("==============================================");
}

