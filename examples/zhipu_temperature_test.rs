//! 智谱 AI 温度参数测试
//!
//! 测试不同的温度值，找出智谱 AI 接受的范围

use lumosai_core::llm::{LlmProvider, LlmOptions, ZhipuProvider};
use lumosai_core::llm::types::Temperature;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k";
    let zhipu = ZhipuProvider::new(api_key.to_string(), Some("glm-4.6".to_string()));

    println!("🌡️  智谱 AI 温度参数测试");
    println!("{}", "=".repeat(60));

    // 测试不同的温度值
    let test_temps = vec![
        ("0.0 (DETERMINISTIC)", 0.0),
        ("0.1", 0.1),
        ("0.3 (LOW)", 0.3),
        ("0.5", 0.5),
        ("0.7 (BALANCED)", 0.7),
        ("0.9", 0.9),
        ("1.0 (CREATIVE)", 1.0),
        ("1.2", 1.2),
        ("1.5 (HIGH)", 1.5),
        ("2.0", 2.0),
    ];

    for (label, temp_value) in test_temps {
        println!("\n📊 Testing temperature = {} ({})", temp_value, label);
        println!("{}", "-".repeat(60));

        let options = LlmOptions {
            temperature: Some(Temperature::new(temp_value)),
            max_tokens: Some(50),
            ..Default::default()
        };

        match zhipu.generate("你好", &options).await {
            Ok(response) => {
                println!("✅ SUCCESS: {}", response.chars().take(50).collect::<String>());
            }
            Err(e) => {
                println!("❌ FAILED: {:?}", e);
            }
        }
    }

    // 测试不设置温度
    println!("\n📊 Testing without temperature (None)");
    println!("{}", "-".repeat(60));

    let options_no_temp = LlmOptions {
        temperature: None,
        max_tokens: Some(50),
        ..Default::default()
    };

    match zhipu.generate("你好", &options_no_temp).await {
        Ok(response) => {
            println!("✅ SUCCESS: {}", response);
        }
        Err(e) => {
            println!("❌ FAILED: {:?}", e);
        }
    }

    // 测试不设置 max_tokens
    println!("\n📊 Testing without max_tokens (None)");
    println!("{}", "-".repeat(60));

    let options_no_max = LlmOptions {
        temperature: Some(Temperature::CREATIVE),
        max_tokens: None,
        ..Default::default()
    };

    match zhipu.generate("你好", &options_no_max).await {
        Ok(response) => {
            println!("✅ SUCCESS: {}", response.chars().take(50).collect::<String>());
        }
        Err(e) => {
            println!("❌ FAILED: {:?}", e);
        }
    }

    // 测试什么都不设置（使用默认值）
    println!("\n📊 Testing with default LlmOptions");
    println!("{}", "-".repeat(60));

    let options_default = LlmOptions::default();

    match zhipu.generate("你好", &options_default).await {
        Ok(response) => {
            println!("✅ SUCCESS: {}", response);
        }
        Err(e) => {
            println!("❌ FAILED: {:?}", e);
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("✅ 测试完成！");

    Ok(())
}

