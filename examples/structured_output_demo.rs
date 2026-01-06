//! Structured Output Demo
//!
//! 演示如何使用 LumosAI 的结构化输出功能
//!
//! 运行方式:
//! ```bash
//! cargo run --example structured_output_demo
//! ```

use lumosai_core::agent::{AgentBuilder, AgentStructuredOutput};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::llm::{Message, Role};
use lumosai_core::agent::types::AgentGenerateOptions;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// 任务分解结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskBreakdown {
    /// 任务标题
    title: String,
    /// 子任务列表
    subtasks: Vec<String>,
    /// 优先级
    priority: String,
    /// 预计时间（小时）
    estimated_hours: f32,
}

/// 产品特性分析
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProductAnalysis {
    /// 产品名称
    product_name: String,
    /// 核心特性
    key_features: Vec<String>,
    /// 目标用户
    target_audience: String,
    /// 优势
    advantages: Vec<String>,
    /// 劣势
    disadvantages: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 LumosAI Structured Output Demo\n");
    println!("{}", "=".repeat(60));

    // 创建 LLM 提供者
    let llm = create_test_zhipu_provider_arc();

    // ========================================
    // 示例 1: 任务分解 - 使用简单方法
    // ========================================
    println!("\n📋 Example 1: Task Breakdown (Simple Method)\n");

    let agent = AgentBuilder::new()
        .name("planner")
        .instructions("You are a project planning assistant. Break down tasks into subtasks.")
        .model(llm.clone())
        .build()?;

    println!("输入: 'Plan a website development project'");
    println!("处理中...");

    match agent.generate_structured_simple::<TaskBreakdown>(
        "Break down this project into subtasks: Build a modern e-commerce website with React and Node.js"
    ).await {
        Ok(breakdown) => {
            println!("\n✅ 结构化输出成功:");
            println!("标题: {}", breakdown.title);
            println!("优先级: {}", breakdown.priority);
            println!("预计时间: {} 小时", breakdown.estimated_hours);
            println!("子任务:");
            for (i, task) in breakdown.subtasks.iter().enumerate() {
                println!("  {}. {}", i + 1, task);
            }
        }
        Err(e) => {
            println!("⚠️  结构化输出失败: {:?}", e);
            println!("注意: 测试 LLM 可能不完全支持结构化输出");
        }
    }

    // ========================================
    // 示例 2: 产品分析 - 使用自定义 Schema
    // ========================================
    println!("\n\n🔍 Example 2: Product Analysis (Custom Schema)\n");
    println!("{}", "=".repeat(60));

    // 创建分析 Agent（schema 通过 prompt engineering 传递）
    let analyzer = AgentBuilder::new()
        .name("analyzer")
        .instructions("You are a product analyst. Analyze products comprehensively.")
        .model(llm.clone())
        .build()?;

    // 使用自定义 schema
    let schema = json!({
        "type": "object",
        "properties": {
            "product_name": {"type": "string"},
            "key_features": {"type": "array", "items": {"type": "string"}},
            "target_audience": {"type": "string"},
            "advantages": {"type": "array", "items": {"type": "string"}},
            "disadvantages": {"type": "array", "items": {"type": "string"}}
        },
        "required": ["product_name", "key_features", "target_audience"]
    });

    println!("输入: 'Analyze LumosAI framework'");
    println!("处理中...");

    match analyzer.generate_with_schema::<ProductAnalysis>(
        "Analyze the LumosAI framework: an enterprise-grade AI agent framework built with Rust",
        schema
    ).await {
        Ok(analysis) => {
            println!("\n✅ 产品分析完成:");
            println!("产品: {}", analysis.product_name);
            println!("目标用户: {}", analysis.target_audience);
            println!("\n核心特性:");
            for feature in &analysis.key_features {
                println!("  • {}", feature);
            }
            println!("\n优势:");
            for adv in &analysis.advantages {
                println!("  + {}", adv);
            }
            println!("\n劣势:");
            for dis in &analysis.disadvantages {
                println!("  - {}", dis);
            }
        }
        Err(e) => {
            println!("⚠️  分析失败: {:?}", e);
            println!("注意: 测试 LLM 可能不完全支持结构化输出");
        }
    }

    // ========================================
    // 示例 3: 使用消息历史的结构化输出
    // ========================================
    println!("\n\n💬 Example 3: Structured Output with Message History\n");
    println!("{}", "=".repeat(60));

    let messages = vec![
        Message {
            role: Role::User,
            content: "I need to organize a conference".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::Assistant,
            content: "I'll help you break that down into tasks".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "Focus on the venue and speakers".to_string(),
            metadata: None,
            name: None,
        },
    ];

    println!("使用3轮对话历史生成结构化输出...");

    match agent.generate_structured::<TaskBreakdown>(
        &messages,
        &AgentGenerateOptions::default()
    ).await {
        Ok(breakdown) => {
            println!("\n✅ 基于对话历史的结构化输出:");
            println!("标题: {}", breakdown.title);
            println!("子任务数: {}", breakdown.subtasks.len());
        }
        Err(e) => {
            println!("⚠️  生成失败: {:?}", e);
        }
    }

    // ========================================
    // 总结
    // ========================================
    println!("\n\n");
    println!("{}", "=".repeat(60));
    println!("📊 Demo Summary");
    println!("{}", "=".repeat(60));
    println!();
    println!("✅ Structured Output features demonstrated:");
    println!("  1. Simple structured output generation");
    println!("  2. Custom schema definition");
    println!("  3. Message history with structured output");
    println!();
    println!("🎯 Key capabilities:");
    println!("  • Type-safe output generation");
    println!("  • Flexible schema definition");
    println!("  • JSON extraction from responses");
    println!("  • Error handling and validation");
    println!();
    println!("📖 Learn more: docs/agent/structured_output.md");
    println!();

    Ok(())
}

