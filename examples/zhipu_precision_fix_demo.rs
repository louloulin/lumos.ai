use lumosai_core::agent::{AgentBuilder, Agent, types::AgentGenerateOptions};
use lumosai_core::llm::{ZhipuProvider, LlmOptions, types::{Temperature, user_message}};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 智谱 AI 精确类型修复演示");
    println!("================================");
    
    // 获取 API Key
    let api_key = env::var("ZHIPU_API_KEY")
        .expect("请设置 ZHIPU_API_KEY 环境变量");
    
    println!("🔑 使用 API Key: {}...", &api_key[..20]);
    
    // 创建智谱 AI 提供商
    let zhipu_provider = ZhipuProvider::new(api_key, Some("glm-4".to_string()));
    
    println!("\n=== 演示1: 使用精确的 Temperature 类型 ===");
    
    // 测试不同的温度设置
    let temperatures = vec![
        ("DETERMINISTIC", Temperature::DETERMINISTIC),
        ("LOW", Temperature::LOW), 
        ("BALANCED", Temperature::BALANCED),
        ("CREATIVE", Temperature::CREATIVE),
        ("Custom 0.8", Temperature::new(0.8)),
        ("Problematic 0.7", Temperature::new(0.7f32)), // 这个之前会有精度问题
    ];
    
    for (name, temp) in temperatures {
        println!("\n🌡️  测试温度设置: {} (值: {})", name, temp);
        
        // 创建带有特定温度的选项
        let options = LlmOptions::new()
            .with_temperature_precise(temp);
            
        println!("   📊 Temperature 对象: {:?}", temp);
        println!("   📊 Temperature 值: {}", temp.value());
        println!("   📊 Temperature 显示: {}", temp);
        
        // 创建 Agent (重新创建 provider 因为没有 Clone)
        let temp_provider = ZhipuProvider::new(
            env::var("ZHIPU_API_KEY").expect("ZHIPU_API_KEY required"),
            Some("glm-4".to_string())
        );
        let agent = AgentBuilder::new()
            .name(&format!("zhipu_temp_{}", name.replace(" ", "_")))
            .instructions("你是一个AI助手，请简洁回答。")
            .model(Arc::new(temp_provider))
            .build()?;
        
        // 创建带有特定温度的 AgentGenerateOptions
        let mut agent_options = AgentGenerateOptions::default();
        agent_options.llm_options.temperature = Some(temp);

        // 测试生成
        println!("   🔄 发送请求...");
        let messages = vec![user_message("请说'你好'")];
        match agent.generate(&messages, &agent_options).await {
            Ok(response) => {
                println!("   ✅ 成功! 回复: {}", response.response.trim());
                break; // 一旦成功就停止测试
            }
            Err(e) => {
                println!("   ❌ 失败: {}", e);
                continue;
            }
        }
    }
    
    println!("\n=== 演示2: 对比旧的 f32 方式 ===");
    
    // 展示浮点精度问题
    let problematic_temp = 0.7f32;
    println!("🔍 原始 f32 值: {}", problematic_temp);
    println!("🔍 精度问题演示: {:.15}", problematic_temp);
    
    let precise_temp = Temperature::new(problematic_temp);
    println!("🔧 Temperature 修复后: {}", precise_temp);
    println!("🔧 Temperature 内部值: {:.15}", precise_temp.value());
    
    println!("\n=== 演示3: 序列化测试 ===");
    
    // 测试序列化行为
    let test_temps = vec![0.1, 0.3, 0.5, 0.7, 0.9, 1.0, 1.2];
    
    for temp_val in test_temps {
        let raw_f32 = temp_val as f32;
        let precise_temp = Temperature::new(temp_val as f32);
        
        let raw_json = serde_json::json!(raw_f32);
        let precise_json = serde_json::to_value(&precise_temp)?;
        
        println!("🔢 原始值: {} -> JSON: {}", temp_val, raw_json);
        println!("🎯 精确值: {} -> JSON: {}", precise_temp, precise_json);
        
        if raw_json != precise_json {
            println!("   ⚠️  发现差异!");
        } else {
            println!("   ✅ 一致");
        }
        println!();
    }
    
    println!("\n🎉 精确类型修复演示完成!");
    println!("💡 Temperature 类型确保了:");
    println!("   - 自动精度控制 (舍入到一位小数)");
    println!("   - 类型安全的 API");
    println!("   - 一致的序列化行为");
    println!("   - 预定义的常用温度值");
    
    Ok(())
}
