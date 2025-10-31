//! 测试不同的智谱 AI 模型名称

use lumosai_core::llm::{LlmProvider, LlmOptions, ZhipuProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k";

    println!("🧪 测试不同的智谱 AI 模型名称");
    println!("{}", "=".repeat(60));

    let models = vec![
        "glm-4",
        "glm-4-plus",
        "glm-4-plus",
        "glm-4-air",
        "glm-4-airx",
        "glm-4-flash",
        "glm-4-flashx",
        "glm-3-turbo",
    ];

    for model_name in models {
        println!("\n📝 Testing model: {}", model_name);
        println!("{}", "-".repeat(60));

        let provider = ZhipuProvider::new(api_key.to_string(), Some(model_name.to_string()));
        
        let options = LlmOptions {
            max_tokens: Some(50),
            ..Default::default()
        };

        match provider.generate("你好，请用一句话介绍你自己。", &options).await {
            Ok(response) => {
                if response.is_empty() {
                    println!("⚠️  Model {} returned EMPTY response", model_name);
                } else {
                    println!("✅ Model {} SUCCESS: {}", model_name, response.chars().take(100).collect::<String>());
                }
            }
            Err(e) => {
                println!("❌ Model {} FAILED: {:?}", model_name, e);
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("✅ 测试完成！");

    Ok(())
}

