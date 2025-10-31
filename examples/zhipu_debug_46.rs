//! Debug glm-4.6 empty response issue

use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k";
    let base_url = "https://open.bigmodel.cn/api/paas/v4";

    println!("🔍 Debugging glm-4.6 Empty Response Issue");
    println!("{}", "=".repeat(70));

    // Test 1: glm-4 (working)
    println!("\n📝 Test 1: glm-4 (baseline - should work)");
    println!("{}", "-".repeat(70));
    test_model(&api_key, &base_url, "glm-4", 50).await?;

    // Test 2: glm-4.6 with max_tokens=50
    println!("\n📝 Test 2: glm-4.6 with max_tokens=50 (returns empty)");
    println!("{}", "-".repeat(70));
    test_model(&api_key, &base_url, "glm-4.6", 50).await?;

    // Test 3: glm-4.6 without max_tokens
    println!("\n📝 Test 3: glm-4.6 without max_tokens");
    println!("{}", "-".repeat(70));
    test_model_no_max_tokens(&api_key, &base_url, "glm-4.6").await?;

    // Test 4: glm-4.6 with larger max_tokens
    println!("\n📝 Test 4: glm-4.6 with max_tokens=200");
    println!("{}", "-".repeat(70));
    test_model(&api_key, &base_url, "glm-4.6", 200).await?;

    // Test 5: glm-4.6 with max_tokens=1000
    println!("\n📝 Test 5: glm-4.6 with max_tokens=1000");
    println!("{}", "-".repeat(70));
    test_model(&api_key, &base_url, "glm-4.6", 1000).await?;

    println!("\n{}", "=".repeat(70));
    println!("✅ Debug complete!");

    Ok(())
}

async fn test_model(
    api_key: &str,
    base_url: &str,
    model: &str,
    max_tokens: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", base_url);

    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": "你好，请介绍一下你自己"}],
        "temperature": 0.5,
        "max_tokens": max_tokens
    });

    println!("📤 Request: model={}, max_tokens={}", model, max_tokens);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;

    if let Ok(json) = serde_json::from_str::<Value>(&text) {
        if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
            println!("✅ Status: {}", status);
            println!("📝 Content length: {} chars", content.len());
            if content.is_empty() {
                println!("⚠️  EMPTY RESPONSE!");
                println!("📋 Full response:");
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                println!("📝 Content: {}", content.chars().take(100).collect::<String>());
            }
        } else {
            println!("❌ No content field");
            println!("📋 Response: {}", serde_json::to_string_pretty(&json)?);
        }
    } else {
        println!("❌ Failed to parse JSON: {}", text);
    }

    Ok(())
}

async fn test_model_no_max_tokens(
    api_key: &str,
    base_url: &str,
    model: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", base_url);

    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": "你好，请介绍一下你自己"}],
        "temperature": 0.5
    });

    println!("📤 Request: model={}, max_tokens=None", model);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;

    if let Ok(json) = serde_json::from_str::<Value>(&text) {
        if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
            println!("✅ Status: {}", status);
            println!("📝 Content length: {} chars", content.len());
            if content.is_empty() {
                println!("⚠️  EMPTY RESPONSE!");
                println!("📋 Full response:");
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                println!("📝 Content: {}", content.chars().take(100).collect::<String>());
            }
        } else {
            println!("❌ No content field");
            println!("📋 Response: {}", serde_json::to_string_pretty(&json)?);
        }
    } else {
        println!("❌ Failed to parse JSON: {}", text);
    }

    Ok(())
}

