//! 华为 ModelArts MaaS 提供商演示
//!
//! 演示如何使用华为云 ModelArts MaaS 服务进行 LLM 调用。
//!
//! # 环境变量
//!
//! 需要设置以下环境变量：
//! - `MAAS_API_KEY` 或 `HUAWEI_MAAS_API_KEY`: 华为 MaaS API Key
//! - `MAAS_MODEL` (可选): 模型名称，默认为 "deepseek-v3.2-exp"
//!
//! # 运行示例
//!
//! ```bash
//! export MAAS_API_KEY="your-api-key-here"
//! cargo run --example huawei_maas_demo
//! ```

use lumosai_core::llm::{HuaweiMaasProvider, LlmOptions, LlmProvider, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("华为 ModelArts MaaS 提供商演示");
    println!("========================================\n");

    // 方式1: 从环境变量创建
    println!("1. 从环境变量创建 Provider");
    let provider = match HuaweiMaasProvider::from_env() {
        Ok(p) => {
            println!("✅ 成功从环境变量创建 Provider");
            p
        }
        Err(e) => {
            eprintln!("❌ 创建 Provider 失败: {}", e);
            eprintln!("\n请设置环境变量:");
            eprintln!("  export MAAS_API_KEY=\"your-api-key\"");
            eprintln!("或:");
            eprintln!("  export HUAWEI_MAAS_API_KEY=\"your-api-key\"");
            return Err(e.into());
        }
    };

    println!("Provider 名称: {}\n", provider.name());

    // 方式2: 手动创建
    // let provider = HuaweiMaasProvider::new(
    //     "your-api-key".to_string(),
    //     Some("deepseek-v3.2-exp".to_string())
    // );

    // 测试1: 简单文本生成
    println!("========================================");
    println!("测试1: 简单文本生成");
    println!("========================================");

    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(500);

    println!("提示: 你好，请用一句话介绍一下自己");
    match provider.generate("你好，请用一句话介绍一下自己", &options).await {
        Ok(response) => {
            println!("✅ 响应: {}\n", response);
        }
        Err(e) => {
            eprintln!("❌ 生成失败: {}\n", e);
        }
    }

    // 测试2: 多轮对话
    println!("========================================");
    println!("测试2: 多轮对话");
    println!("========================================");

    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个有帮助的AI助手，擅长解释技术概念。".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "什么是 Rust 编程语言？".to_string(),
            metadata: None,
            name: None,
        },
    ];

    println!("系统提示: 你是一个有帮助的AI助手，擅长解释技术概念。");
    println!("用户: 什么是 Rust 编程语言？");

    match provider
        .generate_with_messages(&messages, &options)
        .await
    {
        Ok(response) => {
            println!("✅ AI: {}\n", response);
        }
        Err(e) => {
            eprintln!("❌ 对话失败: {}\n", e);
        }
    }

    // 测试3: 代码生成
    println!("========================================");
    println!("测试3: 代码生成");
    println!("========================================");

    let code_prompt = "请用 Rust 编写一个简单的 HTTP 服务器示例，使用 axum 框架";
    println!("提示: {}", code_prompt);

    let code_options = LlmOptions::default()
        .with_temperature(0.3) // 降低温度以获得更确定的代码
        .with_max_tokens(1000);

    match provider.generate(code_prompt, &code_options).await {
        Ok(response) => {
            println!("✅ 生成的代码:\n{}\n", response);
        }
        Err(e) => {
            eprintln!("❌ 代码生成失败: {}\n", e);
        }
    }

    // 测试4: 中文理解和生成
    println!("========================================");
    println!("测试4: 中文理解和生成");
    println!("========================================");

    let chinese_messages = vec![
        Message {
            role: Role::System,
            content: "你是一个中文诗歌创作助手。".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "请写一首关于春天的七言绝句。".to_string(),
            metadata: None,
            name: None,
        },
    ];

    println!("系统提示: 你是一个中文诗歌创作助手。");
    println!("用户: 请写一首关于春天的七言绝句。");

    match provider
        .generate_with_messages(&chinese_messages, &options)
        .await
    {
        Ok(response) => {
            println!("✅ AI:\n{}\n", response);
        }
        Err(e) => {
            eprintln!("❌ 诗歌创作失败: {}\n", e);
        }
    }

    // 测试5: 不同温度参数的影响
    println!("========================================");
    println!("测试5: 温度参数影响");
    println!("========================================");

    let creative_prompt = "用三个词描述人工智能";
    println!("提示: {}", creative_prompt);

    for temp in [0.1, 0.5, 0.9] {
        println!("\n温度 = {}", temp);
        let temp_options = LlmOptions::default()
            .with_temperature(temp)
            .with_max_tokens(50);

        match provider.generate(creative_prompt, &temp_options).await {
            Ok(response) => {
                println!("  响应: {}", response.trim());
            }
            Err(e) => {
                eprintln!("  ❌ 失败: {}", e);
            }
        }
    }

    println!("\n========================================");
    println!("✅ 所有测试完成！");
    println!("========================================");

    Ok(())
}

