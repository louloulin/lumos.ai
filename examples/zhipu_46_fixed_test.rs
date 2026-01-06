//! Test glm-4.6 with the fixed ZhipuProvider

use lumosai_core::llm::{LlmOptions, LlmProvider, ZhipuProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k";

    println!("🧪 Testing glm-4.6 with Fixed ZhipuProvider");
    println!("{}", "=".repeat(70));

    // Test 1: glm-4 baseline
    println!("\n📝 Test 1: glm-4 with max_tokens=50");
    println!("{}", "-".repeat(70));
    let provider = ZhipuProvider::new(api_key.to_string(), Some("glm-4".to_string()));
    let options = LlmOptions {
        max_tokens: Some(50),
        ..Default::default()
    };
    match provider.generate("你好，请介绍一下你自己", &options).await {
        Ok(response) => {
            println!("✅ glm-4 SUCCESS");
            println!("📝 Length: {} chars", response.len());
            println!(
                "📝 Content: {}",
                response.chars().take(100).collect::<String>()
            );
        }
        Err(e) => println!("❌ glm-4 FAILED: {:?}", e),
    }

    // Test 2: glm-4.6 with max_tokens=50 (should now work with reasoning_content)
    println!("\n📝 Test 2: glm-4.6 with max_tokens=50 (using reasoning_content)");
    println!("{}", "-".repeat(70));
    let provider = ZhipuProvider::new(api_key.to_string(), Some("glm-4.6".to_string()));
    let options = LlmOptions {
        max_tokens: Some(50),
        ..Default::default()
    };
    match provider.generate("你好，请介绍一下你自己", &options).await {
        Ok(response) => {
            println!("✅ glm-4.6 SUCCESS!");
            println!("📝 Length: {} chars", response.len());
            println!(
                "📝 Content: {}",
                response.chars().take(100).collect::<String>()
            );
        }
        Err(e) => println!("❌ glm-4.6 FAILED: {:?}", e),
    }

    // Test 3: glm-4.6 with max_tokens=200
    println!("\n📝 Test 3: glm-4.6 with max_tokens=200");
    println!("{}", "-".repeat(70));
    let provider = ZhipuProvider::new(api_key.to_string(), Some("glm-4.6".to_string()));
    let options = LlmOptions {
        max_tokens: Some(200),
        ..Default::default()
    };
    match provider.generate("你好，请介绍一下你自己", &options).await {
        Ok(response) => {
            println!("✅ glm-4.6 SUCCESS!");
            println!("📝 Length: {} chars", response.len());
            println!(
                "📝 Content: {}",
                response.chars().take(150).collect::<String>()
            );
        }
        Err(e) => println!("❌ glm-4.6 FAILED: {:?}", e),
    }

    // Test 4: glm-4.6 with max_tokens=1000
    println!("\n📝 Test 4: glm-4.6 with max_tokens=1000");
    println!("{}", "-".repeat(70));
    let provider = ZhipuProvider::new(api_key.to_string(), Some("glm-4.6".to_string()));
    let options = LlmOptions {
        max_tokens: Some(1000),
        ..Default::default()
    };
    match provider
        .generate("写一个简短的Python函数来计算斐波那契数列", &options)
        .await
    {
        Ok(response) => {
            println!("✅ glm-4.6 SUCCESS!");
            println!("📝 Length: {} chars", response.len());
            println!("📝 Content preview:");
            println!("{}", response.chars().take(300).collect::<String>());
        }
        Err(e) => println!("❌ glm-4.6 FAILED: {:?}", e),
    }

    // Test 5: Compare glm-4 vs glm-4.6 vs glm-4-plus
    println!("\n📝 Test 5: Model Comparison (all with max_tokens=100)");
    println!("{}", "=".repeat(70));

    let models = vec!["glm-4", "glm-4.6", "glm-4-plus"];
    let prompt = "用一句话介绍人工智能";

    for model in models {
        println!("\n🔹 Model: {}", model);
        let provider = ZhipuProvider::new(api_key.to_string(), Some(model.to_string()));
        let options = LlmOptions {
            max_tokens: Some(100),
            ..Default::default()
        };

        match provider.generate(prompt, &options).await {
            Ok(response) => {
                println!("   ✅ SUCCESS - {} chars", response.len());
                println!("   📝 {}", response.chars().take(80).collect::<String>());
            }
            Err(e) => println!("   ❌ FAILED: {:?}", e),
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("✅ All tests complete!");

    Ok(())
}
