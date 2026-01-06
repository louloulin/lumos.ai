//! Basic A2A Client Example
//!
//! 演示如何使用 A2A 客户端进行基本的 Agent 通信

use lumosai_a2a::{A2AClientBuilder, Message, Task};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 A2A Basic Client Example");
    
    // 创建 A2A 客户端
    let client = A2AClientBuilder::new("http://localhost:8080".to_string())
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    
    println!("✅ Client created successfully");
    
    // 创建测试任务
    let message = Message::user_message("Hello, A2A Agent! Please analyze this text.".to_string());
    let task = Task::new("Test Analysis Task".to_string(), message);
    
    println!("\n📤 Submitting task: {}", task.description);
    
    // 提交任务
    match client.submit_task(&task).await {
        Ok(task_id) => {
            println!("✅ Task submitted successfully!");
            println!("Task ID: {}", task_id);
            
            // 获取任务状态
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            
            match client.get_task_status(&task_id).await {
                Ok(status) => {
                    println!("\n📋 Task Status:");
                    println!("- State: {}", status.state);
                    println!("- Timestamp: {}", status.timestamp);
                    
                    if let Some(msg) = &status.message {
                        println!("- Latest Message: {:?}", msg.parts[0]);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to get task status: {}", e);
                }
            }
            
            // 获取任务结果
            match client.get_task_result(&task_id).await {
                Ok(artifacts) => {
                    println!("\n📦 Task Results: {} artifacts", artifacts.len());
                    for (i, artifact) in artifacts.iter().enumerate() {
                        println!("  {}. {:?}", i + 1, artifact.name);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to get task result: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to submit task: {}", e);
        }
    }
    
    // 演示获取Agent Card
    println!("\n🔍 Testing Agent Card retrieval...");
    match client.get_agent_card("test-agent").await {
        Ok(card) => {
            println!("✅ Agent Card retrieved successfully!");
            println!("Name: {}", card.name);
            println!("URL: {}", card.url);
        }
        Err(e) => {
            println!("❌ Failed to get agent card (expected for demo): {}", e);
        }
    }
    
    println!("\n🎉 Basic client example completed!");
    Ok(())
}