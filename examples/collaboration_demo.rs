//! Agent 协作系统演示
//!
//! 展示智能负载均衡和任务分配功能

use lumosai_core::agent::collaboration::{Crew, AgentTask, AgentRole, CollaborationMode};
use lumosai_core::agent::simplified_api::Agent;
use lumosai_core::llm::MockLlmProvider;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 简单的日志初始化
    println!("🚀 Agent 协作系统演示启动");

    println!("🚀 Agent 协作系统演示");
    println!("==================");

    // 创建 Mock LLM 提供商
    let llm = Arc::new(MockLlmProvider::new(vec![
        "任务完成：我已经分析了数据并生成了报告。".to_string(),
        "代码编写完成：我已经实现了所需的功能。".to_string(),
        "文档编写完成：我已经创建了完整的技术文档。".to_string(),
        "测试执行完成：所有测试都通过了。".to_string(),
    ]));

    // 创建一个 Crew 团队
    let crew = Crew::new(
        "演示团队".to_string(),
        CollaborationMode::Parallel,
        3, // 最大并发任务数
    );

    // 创建不同角色的 Agent
    let agents_data = vec![
        (
            "data_analyst",
            AgentRole::new(
                "数据分析师".to_string(),
                "负责数据分析和报告生成".to_string(),
                "专业的数据分析师，擅长数据处理和可视化".to_string(),
            )
            .with_skills(vec!["数据分析".to_string(), "报告生成".to_string(), "Python".to_string()]),
        ),
        (
            "software_engineer",
            AgentRole::new(
                "软件工程师".to_string(),
                "负责代码开发和实现".to_string(),
                "经验丰富的软件工程师，精通多种编程语言".to_string(),
            )
            .with_skills(vec!["Rust".to_string(), "TypeScript".to_string(), "系统设计".to_string()]),
        ),
        (
            "technical_writer",
            AgentRole::new(
                "技术写手".to_string(),
                "负责文档编写和维护".to_string(),
                "专业的技术写手，擅长创作清晰易懂的技术文档".to_string(),
            )
            .with_skills(vec!["文档编写".to_string(), "Markdown".to_string(), "API文档".to_string()]),
        ),
        (
            "qa_engineer",
            AgentRole::new(
                "QA工程师".to_string(),
                "负责测试和质量保证".to_string(),
                "细心的QA工程师，确保产品质量".to_string(),
            )
            .with_skills(vec!["测试".to_string(), "自动化测试".to_string(), "质量保证".to_string()]),
        ),
    ];

    // 添加 Agent 到团队
    for (agent_id, role) in agents_data {
        let agent = Agent::builder()
            .name(agent_id)
            .instructions(role.goal.clone())
            .model(llm.clone())
            .build()?;

        crew.add_agent(agent_id.to_string(), Arc::new(agent), role).await?;
        println!("✅ 已添加 Agent: {}", agent_id);
    }

    // 创建不同类型的任务
    let tasks = vec![
        AgentTask::new("分析用户数据并生成月度报告".to_string())
            .with_expected_output("包含图表和分析的详细报告".to_string())
            .with_priority(8) // 高优先级
            .with_dependency("".to_string()), // 无依赖
        AgentTask::new("实现用户认证模块".to_string())
            .with_expected_output("完整的认证功能和API".to_string())
            .with_priority(9) // 紧急优先级
            .with_dependency("".to_string()), // 无依赖
        AgentTask::new("编写API文档".to_string())
            .with_expected_output("详细的API使用文档".to_string())
            .with_priority(6) // 普通优先级
            .with_dependency("".to_string()), // 无依赖
        AgentTask::new("执行集成测试".to_string())
            .with_expected_output("测试报告和结果".to_string())
            .with_priority(7) // 高优先级
            .with_dependency("".to_string()), // 无依赖
    ];

    // 为任务添加技能要求元数据
    let mut tasks_with_skills = Vec::new();
    for (i, mut task) in tasks.into_iter().enumerate() {
        let mut metadata = HashMap::new();
        match i {
            0 => {
                metadata.insert("required_skills".to_string(), serde_json::to_value(vec!["数据分析", "报告生成", "Python"])?);
                metadata.insert("domain".to_string(), serde_json::to_value("数据分析".to_string())?);
            }
            1 => {
                metadata.insert("required_skills".to_string(), serde_json::to_value(vec!["Rust", "系统设计"])?);
                metadata.insert("domain".to_string(), serde_json::to_value("软件开发".to_string())?);
            }
            2 => {
                metadata.insert("required_skills".to_string(), serde_json::to_value(vec!["文档编写", "Markdown", "API文档"])?);
                metadata.insert("domain".to_string(), serde_json::to_value("技术写作".to_string())?);
            }
            3 => {
                metadata.insert("required_skills".to_string(), serde_json::to_value(vec!["测试", "自动化测试"])?);
                metadata.insert("domain".to_string(), serde_json::to_value("质量保证".to_string())?);
            }
            _ => {}
        }
        task.metadata = metadata;
        tasks_with_skills.push(task);
    }

    // 添加任务到团队
    for task in tasks_with_skills {
        crew.add_task(task).await?;
    }

    println!("\n📊 初始团队统计:");
    let stats = crew.get_stats().await;
    println!("  - Agent 数量: {}", stats.total_agents);
    println!("  - 总任务数: {}", stats.total_tasks);
    println!("  - 待处理任务: {}", stats.pending_tasks);

    // 执行团队任务
    println!("\n🚀 开始执行团队任务...");
    let completed_tasks = crew.kickoff().await?;

    println!("\n✅ 任务执行完成!");
    println!("\n📊 最终团队统计:");
    let final_stats = crew.get_stats().await;
    println!("  - Agent 数量: {}", final_stats.total_agents);
    println!("  - 总任务数: {}", final_stats.total_tasks);
    println!("  - 已完成任务: {}", final_stats.completed_tasks);
    println!("  - 失败任务: {}", final_stats.failed_tasks);

    // 显示 Agent 性能指标
    println!("\n📈 Agent 性能指标:");
    let all_metrics = crew.get_all_agent_metrics().await;
    for (agent_id, metrics) in all_metrics {
        println!("  🤖 {}:", agent_id);
        println!("    - 完成任务数: {}", metrics.total_completed_tasks);
        println!("    - 失败任务数: {}", metrics.total_failed_tasks);
        println!("    - 成功率: {:.2}%", metrics.success_rate * 100.0);
        println!("    - 平均执行时间: {:.2}ms", metrics.avg_task_duration);
        println!("    - 负载分数: {:.2}", metrics.load_score());
    }

    // 显示任务执行结果
    println!("\n📋 任务执行结果:");
    for task in completed_tasks {
        println!("  🎯 任务: {}", task.description);
        println!("    - 状态: {:?}", task.status);
        if let Some(agent_id) = &task.agent_id {
            println!("    - 执行 Agent: {}", agent_id);
        }
        if let Some(result) = &task.result {
            println!("    - 结果: {}", result);
        }
        if let Some(started_at) = task.started_at {
            if let Some(completed_at) = task.completed_at {
                let duration = completed_at - started_at;
                println!("    - 执行时间: {}ms", duration);
            }
        }
    }

    println!("\n🎉 Agent 协作系统演示完成!");
    println!("智能负载均衡成功地将任务分配给了最合适的 Agent");

    Ok(())
}