//! 智谱 AI 修复演示
//!
//! 展示如何正确集成智谱 AI API

use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 智谱 AI 修复演示");
    println!("========================================");

    // 获取 API Key
    let api_key = env::var("ZHIPU_API_KEY")
        .unwrap_or_else(|_| "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k".to_string());

    println!("🔑 使用 API Key: {}...", &api_key[..20]);

    // 测试原始 HTTP 请求
    test_raw_api_call(&api_key).await?;

    println!("\n✅ 智谱 AI 修复演示完成！");
    Ok(())
}

async fn test_raw_api_call(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 测试原始 API 调用...");

    let client = reqwest::Client::new();

    // 正确的智谱 AI API 请求格式
    let body = json!({
        "model": "glm-4-plus",
        "messages": [
            {
                "role": "user",
                "content": "你好"
            }
        ],
        "temperature": 0.7,
        "max_tokens": 100,
        "stream": false,
        "top_p": 0.7,
        "do_sample": true
    });

    println!("📤 请求体: {}", serde_json::to_string_pretty(&body)?);

    let response = client
        .post("https://open.bigmodel.cn/api/paas/v4/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;

    println!("📥 响应状态: {}", status);
    println!("📥 响应内容: {}", text);

    if status.is_success() {
        println!("✅ API 调用成功！");

        // 解析响应
        let response_json: serde_json::Value = serde_json::from_str(&text)?;
        if let Some(content) = response_json["choices"][0]["message"]["content"].as_str() {
            println!("🤖 AI 回复: {}", content);
        }
    } else {
        println!("❌ API 调用失败: {}", text);

        // 分析错误
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(error_code) = error_json["error"]["code"].as_str() {
                match error_code {
                    "1210" => println!("🔍 错误分析: API 调用参数有误，检查请求格式"),
                    "1301" => println!("🔍 错误分析: API Key 无效或过期"),
                    "1302" => println!("🔍 错误分析: 余额不足"),
                    _ => println!("🔍 错误分析: 未知错误代码 {}", error_code),
                }
            }
        }
    }

    Ok(())
}

// 修复建议的 ZhipuProvider 实现
#[allow(dead_code)]
mod fixed_implementation {
    use serde_json::json;

    pub struct FixedZhipuProvider {
        api_key: String,
        client: reqwest::Client,
        model: String,
        base_url: String,
    }

    impl FixedZhipuProvider {
        pub fn new(api_key: String, model: Option<String>) -> Self {
            Self {
                api_key,
                client: reqwest::Client::new(),
                model: model.unwrap_or_else(|| "glm-4-plus".to_string()),
                base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            }
        }

        pub async fn generate(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            let body = json!({
                "model": self.model,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": 0.7,
                "max_tokens": 1000,
                "stream": false,
                "top_p": 0.7,
                "do_sample": true
            });

            let response = self
                .client
                .post(&format!("{}/chat/completions", self.base_url))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&body)
                .send()
                .await?;

            let status = response.status();
            let text = response.text().await?;

            if !status.is_success() {
                return Err(format!("智谱AI API 错误: {} - {}", status, text).into());
            }

            let response_json: serde_json::Value = serde_json::from_str(&text)?;
            let content = response_json["choices"][0]["message"]["content"]
                .as_str()
                .ok_or("无效的响应格式")?;

            Ok(content.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_format() {
        let body = json!({
            "model": "glm-4-plus",
            "messages": [{"role": "user", "content": "test"}],
            "temperature": 0.7,
            "max_tokens": 100,
            "stream": false,
            "top_p": 0.7,
            "do_sample": true
        });

        // 验证 JSON 格式正确
        assert!(body.is_object());
        assert_eq!(body["model"], "glm-4-plus");
        assert!(body["messages"].is_array());
    }

    #[test]
    fn test_provider_creation() {
        let provider = fixed_implementation::FixedZhipuProvider::new(
            "test_key".to_string(),
            Some("glm-4-plus".to_string()),
        );

        // 验证提供商创建成功
        assert_eq!(provider.model, "glm-4-plus");
        assert_eq!(provider.base_url, "https://open.bigmodel.cn/api/paas/v4");
    }
}
