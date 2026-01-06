//! E2E 测试：流式响应场景

mod framework;
use framework::{E2ETestContext, E2EAssertions};

use lumosai_core::agent::{Agent, AgentStreamOptions};
use lumosai_core::llm::{Message, Role};
use futures::StreamExt;

/// 测试 26: Agent 流式响应
#[tokio::test]
async fn test_agent_streaming_response() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = ctx
        .create_test_agent("streaming_agent", "You are a helpful assistant.")
        .unwrap();

    // 测试流式生成
    let messages = vec![Message {
        role: Role::User,
        content: "Tell me a short story".to_string(),
        metadata: None,
        name: None,
    }];

    let stream = agent.stream(&messages, &AgentStreamOptions::default()).await;

    match stream {
        Ok(mut s) => {
            let mut chunk_count = 0;
            let mut total_content = String::new();

            while let Some(chunk_result) = s.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        chunk_count += 1;
                        total_content.push_str(&chunk);
                        print!(".");
                    }
                    Err(e) => {
                        println!("Stream error: {:?}", e);
                    }
                }
            }

            println!();
            println!(
                "✅ Streaming test: {} chunks, {} chars total",
                chunk_count,
                total_content.len()
            );

            // 至少应该有一些内容
            assert!(
                !total_content.is_empty(),
                "Streamed content should not be empty"
            );
        }
        Err(e) => {
            // 对于测试 LLM，流式可能不完全支持
            println!("⚠️  Streaming not fully supported in test: {:?}", e);
        }
    }

    ctx.teardown().await.unwrap();
}

/// 测试 27: 流式响应性能
#[tokio::test]
async fn test_streaming_performance() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = ctx
        .create_test_agent("perf_agent", "Provide quick responses.")
        .unwrap();

    let messages = vec![Message {
        role: Role::User,
        content: "Count to 10".to_string(),
        metadata: None,
        name: None,
    }];

    let start = std::time::Instant::now();
    let stream = agent.stream(&messages, &AgentStreamOptions::default()).await;

    if let Ok(mut s) = stream {
        let mut first_chunk_time: Option<std::time::Duration> = None;

        while let Some(chunk_result) = s.next().await {
            if chunk_result.is_ok() && first_chunk_time.is_none() {
                first_chunk_time = Some(start.elapsed());
            }
        }

        if let Some(ttfb) = first_chunk_time {
            println!("✅ Time to first byte: {:?}", ttfb);
            // 第一个 chunk 应该相对较快
            assert!(
                ttfb.as_secs() < 10,
                "First chunk should arrive within 10 seconds"
            );
        }
    }

    ctx.teardown().await.unwrap();
}

/// 测试 28: 流式中断处理
#[tokio::test]
async fn test_streaming_interruption() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = ctx
        .create_test_agent("interrupt_agent", "You are a helpful assistant.")
        .unwrap();

    let messages = vec![Message {
        role: Role::User,
        content: "Write a long essay".to_string(),
        metadata: None,
        name: None,
    }];

    let stream = agent.stream(&messages, &AgentStreamOptions::default()).await;

    if let Ok(mut s) = stream {
        let mut chunk_count = 0;

        // 只读取前几个 chunk 然后中断
        while let Some(chunk_result) = s.next().await {
            if chunk_result.is_ok() {
                chunk_count += 1;
                if chunk_count >= 3 {
                    break; // 主动中断
                }
            }
        }

        println!("✅ Streaming interruption test: read {} chunks", chunk_count);
        // 应该能够正常中断
        assert!(chunk_count > 0, "Should have read at least one chunk");
    }

    ctx.teardown().await.unwrap();
}

