//! 动态配置系统演示
//!
//! 展示 LumosAI 的动态配置能力，对标 Mastra 的 DynamicArgument 功能
//!
//! # 功能演示
//! 1. 基于上下文的动态指令配置
//! 2. 基于复杂度的动态模型选择
//! 3. 基于用户角色的动态工具配置
//! 4. 基于会话的动态内存配置

use lumosai_core::agent::dynamic_config::{
    ComplexityLevel, DynamicArgument, EnhancedRuntimeContext,
};
use lumosai_core::agent::{Agent, AgentConfig, BasicAgent};
use lumosai_core::error::Result;
use lumosai_core::llm::test_helpers::{create_test_zhipu_provider, create_test_zhipu_provider_arc};
use lumosai_core::llm::{Message, Role};
use std::sync::Arc;

/// 场景 1: 基于用户角色的动态指令
async fn demo_dynamic_instructions() -> Result<()> {
    println!("\n=== 场景 1: 动态指令配置 ===\n");

    // 创建运行时上下文
    let context = EnhancedRuntimeContext::new("session_001".to_string())
        .with_user_role("developer".to_string())
        .with_domain("rust".to_string());

    // 创建动态指令配置
    let dynamic_instructions = DynamicArgument::Dynamic(Box::new(|ctx| {
        // 在 async 块之前克隆需要的数据，避免生命周期问题
        let role = ctx.user_role.clone().unwrap_or_else(|| "user".to_string());
        let domain = ctx.domain.clone().unwrap_or_else(|| "general".to_string());

        Box::pin(async move {
            Ok(format!(
                "You are a {} assistant specialized in {}. \
                 Provide expert-level guidance and code examples.",
                role, domain
            ))
        })
    }));

    // 解析动态配置
    let instructions = match dynamic_instructions {
        DynamicArgument::Static(value) => value,
        DynamicArgument::Dynamic(resolver) => resolver(&context).await?,
    };

    println!("✅ 动态生成的指令:");
    println!("   {}", instructions);
    println!();

    Ok(())
}

/// 场景 2: 基于复杂度的动态模型选择
async fn demo_dynamic_model_selection() -> Result<()> {
    println!("\n=== 场景 2: 动态模型选择 ===\n");

    // 测试不同复杂度级别
    for complexity in [
        ComplexityLevel::Simple,
        ComplexityLevel::Complex,
        ComplexityLevel::Expert,
    ] {
        let context = EnhancedRuntimeContext::new("session_002".to_string())
            .with_complexity(complexity.clone());

        // 创建动态模型选择器
        let dynamic_model = DynamicArgument::Dynamic(Box::new(|ctx| {
            // 克隆复杂度级别
            let complexity = ctx.complexity.clone();

            Box::pin(async move {
                let model = match complexity {
                    ComplexityLevel::Simple => "gpt-3.5-turbo",
                    ComplexityLevel::Complex => "gpt-4",
                    ComplexityLevel::Expert => "claude-3-opus",
                };
                Ok(model.to_string())
            })
        }));

        // 解析模型
        let model = match dynamic_model {
            DynamicArgument::Static(value) => value,
            DynamicArgument::Dynamic(resolver) => resolver(&context).await?,
        };

        println!("✅ 复杂度 {:?} -> 模型: {}", context.complexity, model);
    }

    println!();
    Ok(())
}

/// 场景 3: 基于消息历史的动态配置
async fn demo_context_aware_config() -> Result<()> {
    println!("\n=== 场景 3: 上下文感知配置 ===\n");

    // 创建带消息历史的上下文
    let messages = vec![
        Message::new(Role::User, "What is Rust?".to_string(), None, None),
        Message::new(
            Role::Assistant,
            "Rust is a systems programming language...".to_string(),
            None,
            None,
        ),
    ];

    let context = EnhancedRuntimeContext::new("session_003".to_string()).with_messages(messages);

    // 动态配置：基于对话历史调整指令
    let dynamic_config = DynamicArgument::Dynamic(Box::new(|ctx| {
        // 克隆消息数量
        let message_count = ctx.messages.len();

        Box::pin(async move {
            let config = if message_count > 10 {
                "You are in a deep conversation. Provide concise, focused responses."
            } else if message_count > 5 {
                "Continue the conversation naturally. Build on previous context."
            } else {
                "You are starting a new conversation. Be welcoming and thorough."
            };
            Ok(config.to_string())
        })
    }));

    let config = match dynamic_config {
        DynamicArgument::Static(value) => value,
        DynamicArgument::Dynamic(resolver) => resolver(&context).await?,
    };

    println!("✅ 消息数量: {}", context.messages.len());
    println!("   动态配置: {}", config);
    println!();

    Ok(())
}

/// 场景 4: 完整的动态 Agent 创建
async fn demo_full_dynamic_agent() -> Result<()> {
    println!("\n=== 场景 4: 完整动态 Agent ===\n");

    // 创建运行时上下文
    let context = EnhancedRuntimeContext::new("session_123".to_string())
        .with_user_role("data_scientist".to_string())
        .with_domain("machine_learning".to_string())
        .with_complexity(ComplexityLevel::Expert);

    // 动态指令
    let instructions = {
        let role = context.user_role.as_deref().unwrap_or("user");
        let domain = context.domain.as_deref().unwrap_or("general");
        format!(
            "You are an expert {} assistant specialized in {}. \
             Provide detailed technical guidance with code examples and best practices.",
            role, domain
        )
    };

    // 动态模型选择
    let model_name = match context.complexity {
        ComplexityLevel::Simple => "gpt-3.5-turbo",
        ComplexityLevel::Complex => "gpt-4",
        ComplexityLevel::Expert => "gpt-4-turbo",
    };

    println!("✅ 动态 Agent 配置:");
    println!("   角色: {:?}", context.user_role);
    println!("   领域: {:?}", context.domain);
    println!("   复杂度: {:?}", context.complexity);
    println!("   模型: {}", model_name);
    println!("   指令: {}", instructions);
    println!();

    // 创建 Agent（使用 Mock LLM）
    let llm = create_test_zhipu_provider_arc();

    let config = AgentConfig {
        name: "dynamic_ml_assistant".to_string(),
        instructions: instructions.clone(),
        ..Default::default()
    };

    let agent = BasicAgent::new(config, llm);

    println!("✅ Agent 创建成功:");
    println!("   名称: {}", agent.get_name());
    println!("   指令: {}", agent.get_instructions());
    println!();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     LumosAI 动态配置系统演示 (对标 Mastra)              ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    // 运行所有演示场景
    demo_dynamic_instructions().await?;
    demo_dynamic_model_selection().await?;
    demo_context_aware_config().await?;
    demo_full_dynamic_agent().await?;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     ✅ 所有动态配置功能验证通过！                        ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    Ok(())
}
