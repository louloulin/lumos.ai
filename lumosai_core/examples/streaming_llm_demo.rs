//! 流式LLM响应演示
//!
//! 这个示例展示了如何使用智谱AI和百度ERNIE的流式响应功能
//!
//! 运行前需要设置环境变量:
//! - ZHIPU_API_KEY: 智谱AI的API密钥
//! - BAIDU_API_KEY: 百度的API Key
//! - BAIDU_SECRET_KEY: 百度的Secret Key

use futures::StreamExt;
use lumosai_core::llm::{provider::LlmProvider, providers, types::LlmOptions};
use std::io::{self, Write};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::init();

    println!("🌊 流式LLM响应演示");
    println!("=".repeat(50));

    // 测试智谱AI流式响应
    if let Ok(zhipu) = providers::zhipu_from_env() {
        println!("\n📱 智谱AI流式响应测试");
        println!("-".repeat(30));

        test_streaming_response(&zhipu, "智谱AI").await?;
        test_streaming_conversation(&zhipu, "智谱AI").await?;
        test_streaming_creative_writing(&zhipu, "智谱AI").await?;
    } else {
        println!("⚠️  跳过智谱AI测试 - 未设置ZHIPU_API_KEY环境变量");
    }

    // 测试百度ERNIE流式响应
    if let Ok(baidu) = providers::baidu_from_env() {
        println!("\n🔵 百度ERNIE流式响应测试");
        println!("-".repeat(30));

        test_streaming_response(&baidu, "百度ERNIE").await?;
        test_streaming_conversation(&baidu, "百度ERNIE").await?;
        test_streaming_creative_writing(&baidu, "百度ERNIE").await?;
    } else {
        println!("⚠️  跳过百度ERNIE测试 - 未设置BAIDU_API_KEY或BAIDU_SECRET_KEY环境变量");
    }

    // 对比测试
    if let (Ok(zhipu), Ok(baidu)) = (providers::zhipu_from_env(), providers::baidu_from_env()) {
        println!("\n🆚 对比测试");
        println!("-".repeat(30));

        test_side_by_side_streaming(&zhipu, &baidu).await?;
    }

    println!("\n✨ 流式响应演示完成!");
    Ok(())
}

/// 测试基本流式响应
async fn test_streaming_response(
    provider: &dyn LlmProvider,
    name: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🔤 基本流式响应 ({})", name);

    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(200);

    let prompt = "请用一段话介绍人工智能的发展历程，包括关键里程碑";

    print!("💭 问题: {}\n🤖 {}响应: ", prompt, name);
    io::stdout().flush()?;

    match provider.generate_stream(prompt, &options).await {
        Ok(mut stream) => {
            let mut full_response = String::new();

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => {
                        print!("{}", text);
                        io::stdout().flush()?;
                        full_response.push_str(&text);

                        // 添加小延迟以模拟打字机效果
                        sleep(Duration::from_millis(50)).await;
                    }
                    Err(e) => {
                        println!("\n❌ 流式错误: {}", e);
                        break;
                    }
                }
            }

            println!("\n✅ 流式响应完成 (总长度: {} 字符)", full_response.len());
        }
        Err(e) => {
            println!("❌ 流式生成错误: {}", e);
        }
    }

    println!();
    Ok(())
}

/// 测试流式对话
async fn test_streaming_conversation(
    provider: &dyn LlmProvider,
    name: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("💬 流式对话测试 ({})", name);

    let options = LlmOptions::default()
        .with_temperature(0.8)
        .with_max_tokens(150);

    let questions = vec![
        "什么是深度学习？",
        "深度学习和机器学习有什么区别？",
        "能举个深度学习的实际应用例子吗？",
    ];

    for (i, question) in questions.iter().enumerate() {
        println!("🗣️  问题 {}: {}", i + 1, question);
        print!("🤖 {}回答: ", name);
        io::stdout().flush()?;

        match provider.generate_stream(question, &options).await {
            Ok(mut stream) => {
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(text) => {
                            print!("{}", text);
                            io::stdout().flush()?;
                            sleep(Duration::from_millis(30)).await;
                        }
                        Err(e) => {
                            println!("\n❌ 错误: {}", e);
                            break;
                        }
                    }
                }
                println!("\n");
            }
            Err(e) => {
                println!("❌ 错误: {}", e);
            }
        }

        // 问题间隔
        sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}

/// 测试创意写作流式响应
async fn test_streaming_creative_writing(
    provider: &dyn LlmProvider,
    name: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("✍️  创意写作流式测试 ({})", name);

    let options = LlmOptions::default()
        .with_temperature(0.9)
        .with_max_tokens(300);

    let prompt = "请写一个关于AI助手帮助人类解决问题的短故事，要有情节和对话";

    println!("📝 创作主题: {}", prompt);
    print!("📖 {}创作: ", name);
    io::stdout().flush()?;

    match provider.generate_stream(prompt, &options).await {
        Ok(mut stream) => {
            let mut word_count = 0;

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => {
                        print!("{}", text);
                        io::stdout().flush()?;
                        word_count += text.chars().count();

                        // 创意写作需要更慢的速度
                        sleep(Duration::from_millis(80)).await;
                    }
                    Err(e) => {
                        println!("\n❌ 错误: {}", e);
                        break;
                    }
                }
            }

            println!("\n✅ 创作完成 (总字数: {})", word_count);
        }
        Err(e) => {
            println!("❌ 创作错误: {}", e);
        }
    }

    println!();
    Ok(())
}

/// 并行流式响应对比测试
async fn test_side_by_side_streaming(
    zhipu: &dyn LlmProvider,
    baidu: &dyn LlmProvider,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🔄 并行流式响应对比");

    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(150);

    let prompt = "请解释什么是大语言模型";

    println!("💭 问题: {}", prompt);
    println!("📱 智谱AI vs 🔵 百度ERNIE");
    println!("-".repeat(50));

    // 启动两个并行的流式响应
    let zhipu_future = async {
        let mut result = String::new();
        if let Ok(mut stream) = zhipu.generate_stream(prompt, &options).await {
            while let Some(chunk) = stream.next().await {
                if let Ok(text) = chunk {
                    result.push_str(&text);
                }
            }
        }
        result
    };

    let baidu_future = async {
        let mut result = String::new();
        if let Ok(mut stream) = baidu.generate_stream(prompt, &options).await {
            while let Some(chunk) = stream.next().await {
                if let Ok(text) = chunk {
                    result.push_str(&text);
                }
            }
        }
        result
    };

    // 等待两个响应完成
    let (zhipu_result, baidu_result) = tokio::join!(zhipu_future, baidu_future);

    println!("📱 智谱AI结果:");
    println!("{}", zhipu_result);
    println!("\n🔵 百度ERNIE结果:");
    println!("{}", baidu_result);

    println!("\n📊 对比统计:");
    println!("智谱AI字数: {}", zhipu_result.chars().count());
    println!("百度ERNIE字数: {}", baidu_result.chars().count());

    Ok(())
}
