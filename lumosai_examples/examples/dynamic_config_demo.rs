//! 动态配置系统演示 - 对标 Mastra 的 DynamicArgument
//!
//! 展示如何使用 LumosAI 的动态配置系统创建上下文感知的 Agent
//! 
//! 运行方式：
//! ```bash
//! cargo run --package lumosai_examples --example dynamic_config_demo
//! ```

use lumosai_core::agent::dynamic_config::{
    EnhancedRuntimeContext, ComplexityLevel, dynamic_arg
};
use lumosai_core::agent::AgentBuilder;
use lumosai_core::agent::trait_def::Agent;
use lumosai_core::error::Result;
use lumosai_core::llm::MockLlmProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI 动态配置系统演示");
    println!("对标 Mastra 的 DynamicArgument 功能\n");

    // 演示1: 基于用户角色的动态指令
    demo_role_based_instructions().await?;
    
    // 演示2: 基于复杂度的动态模型选择
    demo_complexity_based_model_selection().await?;
    
    // 演示3: 基于上下文的动态工具配置
    demo_context_based_tools().await?;
    
    // 演示4: 完整的动态配置 Agent
    demo_full_dynamic_agent().await?;

    println!("\n✅ 动态配置系统演示完成！");
    println!("🎯 成功对标 Mastra 的 DynamicArgument 功能");
    
    Ok(())
}

/// 演示1: 基于用户角色的动态指令
async fn demo_role_based_instructions() -> Result<()> {
    println!("📋 演示1: 基于用户角色的动态指令");
    
    // 创建不同角色的上下文
    let contexts = vec![
        ("开发者", "developer", "ai_development"),
        ("分析师", "analyst", "data_analysis"),
        ("管理员", "admin", "system_management"),
    ];
    
    for (role_name, role, domain) in contexts {
        let context = EnhancedRuntimeContext::new("session_123".to_string())
            .with_user_role(role.to_string())
            .with_domain(domain.to_string())
            .with_complexity(ComplexityLevel::Complex);
        
        // 创建动态指令配置
        let dynamic_instructions = dynamic_arg(|ctx: &EnhancedRuntimeContext| {
            let domain = ctx.domain.clone();
            let user_role = ctx.user_role.clone();
            async move {
                Ok(format!("你是一个专业的{}助手，专门处理{}相关的任务。你的用户角色是{}，请根据这个角色提供相应的帮助。",
                    domain.as_deref().unwrap_or("通用"),
                    domain.as_deref().unwrap_or("通用"),
                    user_role.as_deref().unwrap_or("用户")
                ))
            }
        });
        
        // 使用 MockLlmProvider 进行测试
        let llm = Arc::new(MockLlmProvider::new(vec![
            format!("我是为{}角色定制的AI助手，专门处理{}任务。", role_name, domain)
        ]));
        
        let agent = AgentBuilder::new()
            .name(&format!("{}_assistant", role))
            .dynamic_instructions(dynamic_instructions)
            .model(llm)
            .with_runtime_context(context)
            .build_async()
            .await?;
        
        println!("  ✅ {}助手创建成功", role_name);
        println!("     指令: {}", agent.get_instructions());
    }
    
    println!();
    Ok(())
}

/// 演示2: 基于复杂度的动态模型选择
async fn demo_complexity_based_model_selection() -> Result<()> {
    println!("🧠 演示2: 基于复杂度的动态模型选择");
    
    let complexities = vec![
        ("简单任务", ComplexityLevel::Simple, "gpt-3.5-turbo"),
        ("复杂任务", ComplexityLevel::Complex, "gpt-4"),
        ("专家级任务", ComplexityLevel::Expert, "claude-3-opus"),
    ];
    
    for (task_name, complexity, expected_model) in complexities {
        let context = EnhancedRuntimeContext::new("session_456".to_string())
            .with_complexity(complexity)
            .with_user_role("developer".to_string());
        
        // 动态模型选择
        let dynamic_model = dynamic_arg(|ctx: &EnhancedRuntimeContext| {
            let complexity = ctx.complexity.clone();
            async move {
                Ok(match complexity {
                    ComplexityLevel::Simple => "gpt-3.5-turbo".to_string(),
                    ComplexityLevel::Complex => "gpt-4".to_string(),
                    ComplexityLevel::Expert => "claude-3-opus".to_string(),
                })
            }
        });
        
        // 创建模拟的 LLM 提供者
        let llm = Arc::new(MockLlmProvider::new(vec![
            format!("使用{}模型处理{}", expected_model, task_name)
        ]));
        
        let _agent = AgentBuilder::new()
            .name("adaptive_model_agent")
            .instructions("你是一个自适应模型选择的助手")
            .dynamic_model(dynamic_model)
            .model(llm) // 提供默认模型
            .with_runtime_context(context)
            .build_async()
            .await?;
        
        println!("  ✅ {}: 选择模型 {}", task_name, expected_model);
    }
    
    println!();
    Ok(())
}

/// 演示3: 基于上下文的动态工具配置
async fn demo_context_based_tools() -> Result<()> {
    println!("🔧 演示3: 基于上下文的动态工具配置");
    
    let user_roles = vec![
        ("管理员", "admin"),
        ("开发者", "developer"),
        ("分析师", "analyst"),
        ("普通用户", "user"),
    ];
    
    for (role_name, role) in user_roles {
        let context = EnhancedRuntimeContext::new("session_789".to_string())
            .with_user_role(role.to_string())
            .with_complexity(ComplexityLevel::Complex);
        
        // 动态工具配置
        let dynamic_tools = dynamic_arg(|ctx: &EnhancedRuntimeContext| {
            let tools = ctx.get_user_tools();
            async move {
                Ok(tools)
            }
        });
        
        let llm = Arc::new(MockLlmProvider::new(vec![
            format!("为{}配置了相应的工具集", role_name)
        ]));
        
        let _agent = AgentBuilder::new()
            .name(&format!("{}_tools_agent", role))
            .instructions("你是一个工具配置演示助手")
            .dynamic_tools(dynamic_tools)
            .model(llm)
            .with_runtime_context(context.clone())
            .build_async()
            .await?;
        
        let available_tools = context.get_user_tools();
        println!("  ✅ {}: 可用工具 {:?}", role_name, available_tools);
    }
    
    println!();
    Ok(())
}

/// 演示4: 完整的动态配置 Agent
async fn demo_full_dynamic_agent() -> Result<()> {
    println!("🎯 演示4: 完整的动态配置 Agent");
    
    // 创建复杂的运行时上下文
    let context = EnhancedRuntimeContext::new("session_full".to_string())
        .with_user_role("senior_developer".to_string())
        .with_domain("ai_research".to_string())
        .with_complexity(ComplexityLevel::Expert);
    
    // 完整的动态配置
    let dynamic_instructions = dynamic_arg(|ctx: &EnhancedRuntimeContext| {
        let domain = ctx.domain.clone();
        let user_role = ctx.user_role.clone();
        let complexity = ctx.complexity.clone();
        async move {
            Ok(format!(
                "你是一个高级{}助手，专门为{}角色提供{}领域的专业支持。\
                当前任务复杂度为{:?}级别，请提供相应深度的分析和建议。",
                domain.as_deref().unwrap_or("通用"),
                user_role.as_deref().unwrap_or("用户"),
                domain.as_deref().unwrap_or("通用"),
                complexity
            ))
        }
    });

    let dynamic_model = dynamic_arg(|ctx: &EnhancedRuntimeContext| {
        let complexity = ctx.complexity.clone();
        async move {
            Ok(match complexity {
                ComplexityLevel::Simple => "gpt-3.5-turbo".to_string(),
                ComplexityLevel::Complex => "gpt-4".to_string(),
                ComplexityLevel::Expert => "claude-3-opus".to_string(),
            })
        }
    });

    let dynamic_tools = dynamic_arg(|ctx: &EnhancedRuntimeContext| {
        let tools = ctx.get_user_tools();
        async move {
            Ok(tools)
        }
    });
    
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我是一个完全动态配置的AI助手，能够根据上下文自适应调整行为。".to_string()
    ]));
    
    let agent = AgentBuilder::new()
        .name("full_dynamic_agent")
        .dynamic_instructions(dynamic_instructions)
        .dynamic_model(dynamic_model)
        .dynamic_tools(dynamic_tools)
        .model(llm)
        .with_runtime_context(context.clone())
        .build_async()
        .await?;
    
    println!("  ✅ 完整动态配置 Agent 创建成功");
    println!("     名称: {}", agent.get_name());
    println!("     指令: {}", agent.get_instructions());
    println!("     用户角色: {:?}", context.user_role);
    println!("     应用领域: {:?}", context.domain);
    println!("     复杂度级别: {:?}", context.complexity);
    println!("     可用工具: {:?}", context.get_user_tools());
    
    Ok(())
}
