//! 高级Agent协作系统演示 - 任务分解和调度
//!
//! 展示智能任务分解、依赖关系管理和高级调度功能

use lumosai_core::agent::collaboration::{
    AdvancedScheduler, AgentRole, AgentTask, CollaborationMode, Crew, DependencyGraph,
    IntelligentTaskDecomposer, SchedulingStrategy, TaskDecomposer,
};
use lumosai_core::agent::simplified_api::Agent;
use lumosai_core::llm::MockLlmProvider;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 高级Agent协作系统演示 - 任务分解和调度");
    println!("===========================================");

    // 创建Mock LLM提供商
    let llm = Arc::new(MockLlmProvider::new(vec![
        "数据收集和分析完成".to_string(),
        "市场调研报告生成完毕".to_string(),
        "技术架构设计完成".to_string(),
        "系统开发工作已完成".to_string(),
        "测试计划已制定".to_string(),
        "项目文档编写完毕".to_string(),
        "部署流程已验证".to_string(),
        "最终验证测试通过".to_string(),
    ]));

    // 创建智能任务分解器
    let decomposer = IntelligentTaskDecomposer::new(llm.clone());

    // 演示任务分解功能
    println!("\n📊 第一部分：智能任务分解演示");
    println!("===============================");

    // 创建不同复杂度的任务
    let complex_tasks = vec![
        (
            "构建完整的电商系统",
            "开发一个全功能的电商平台，包括用户管理、商品管理、订单处理、支付集成等功能",
        ),
        (
            "进行市场竞品分析",
            "深入研究主要竞争对手的产品功能、定价策略、市场份额、用户评价等，生成详细的分析报告",
        ),
        (
            "优化数据库性能",
            "分析当前数据库性能瓶颈，实施优化方案，提升查询效率和数据处理能力",
        ),
        (
            "准备产品发布",
            "准备产品发布的各项工作，包括文档、测试、部署、培训等",
        ),
    ];

    for (i, (title, description)) in complex_tasks.iter().enumerate() {
        println!("\n🎯 复杂任务 {}: {}", i + 1, title);
        println!("描述: {}", description);

        let task = AgentTask::new(description.to_string())
            .with_expected_output(format!("完成{}", title))
            .with_priority(8);

        // 分析任务复杂度
        let complexity = decomposer.analyze_complexity(&task);
        println!("🔍 分析复杂度: {:?}", complexity);

        // 分解任务
        let start_time = std::time::Instant::now();
        let subtasks = decomposer.decompose_task(&task).await?;
        let duration = start_time.elapsed();

        println!("⚡ 分解耗时: {:?}", duration);
        println!("📋 生成子任务数: {}", subtasks.len());

        for (j, subtask) in subtasks.iter().enumerate() {
            println!(
                "  {}. {} (优先级: {})",
                j + 1,
                subtask.description,
                subtask.priority
            );
        }
    }

    // 演示依赖关系管理
    println!("\n\n🔗 第二部分：依赖关系管理演示");
    println!("=============================");

    let mut dependency_graph = DependencyGraph::new();

    // 创建示例任务
    let tasks = vec![
        ("需求分析", "分析项目需求和用户故事"),
        ("系统设计", "设计系统架构和数据库结构"),
        ("环境搭建", "搭建开发和测试环境"),
        ("后端开发", "实现后端API和业务逻辑"),
        ("前端开发", "实现用户界面和交互"),
        ("数据库开发", "创建数据库表结构和初始数据"),
        ("集成测试", "进行系统集成测试"),
        ("部署配置", "配置生产环境"),
        ("用户培训", "准备用户培训材料"),
        ("上线发布", "正式发布产品"),
    ];

    println!("📝 创建任务节点...");
    for (name, _desc) in &tasks {
        let task_id = format!("task_{}", name);
        dependency_graph.add_task(task_id);
        println!("  ✅ 添加任务: {}", name);
    }

    // 添加依赖关系
    println!("\n🔗 设置依赖关系...");
    let dependencies = vec![
        ("需求分析", "系统设计"),
        ("需求分析", "环境搭建"),
        ("系统设计", "后端开发"),
        ("系统设计", "前端开发"),
        ("系统设计", "数据库开发"),
        ("环境搭建", "后端开发"),
        ("环境搭建", "前端开发"),
        ("后端开发", "集成测试"),
        ("前端开发", "集成测试"),
        ("数据库开发", "集成测试"),
        ("集成测试", "部署配置"),
        ("集成测试", "用户培训"),
        ("部署配置", "上线发布"),
        ("用户培训", "上线发布"),
    ];

    for (parent, child) in &dependencies {
        let parent_id = format!("task_{}", parent);
        let child_id = format!("task_{}", child);

        use lumosai_core::agent::collaboration::{SubTaskRelation, TaskRelationType};
        let relation = SubTaskRelation {
            parent_id: parent_id.clone(),
            child_id: child_id.clone(),
            relation_type: TaskRelationType::StrongDependency,
            weight: 5,
        };

        if dependency_graph.add_dependency(relation).is_ok() {
            println!("  🔗 {} -> {}", parent, child);
        } else {
            println!("  ❌ 无法添加依赖: {} -> {}", parent, child);
        }
    }

    // 执行拓扑排序
    println!("\n📊 执行拓扑排序...");
    let start_time = std::time::Instant::now();
    match dependency_graph.topological_sort() {
        Ok(order) => {
            let duration = start_time.elapsed();
            println!("✅ 拓扑排序成功 (耗时: {:?})", duration);
            println!("执行顺序:");
            for (i, task_id) in order.iter().enumerate() {
                let task_name = task_id.replace("task_", "");
                println!("  {}. {}", i + 1, task_name);
            }

            // 显示关键路径
            let critical_path = dependency_graph.get_critical_path();
            println!("\n🎯 关键路径: {}", critical_path.len());
            for task_id in &critical_path {
                let task_name = task_id.replace("task_", "");
                print!("{} ", task_name);
            }
            println!();
        }
        Err(e) => {
            println!("❌ 拓扑排序失败: {}", e);
        }
    }

    // 显示图统计信息
    let stats = dependency_graph.get_stats();
    println!("\n📈 依赖图统计:");
    println!("  - 总节点数: {}", stats.total_nodes);
    println!("  - 总边数: {}", stats.total_edges);
    println!("  - 最大深度: {}", stats.max_depth);
    println!("  - 关键路径长度: {}", stats.critical_path_length);
    println!("  - 是否有循环: {}", stats.has_cycle);

    // 演示高级调度器
    println!("\n\n⚡ 第三部分：高级调度器演示");
    println!("=========================");

    // 创建不同调度策略的调度器
    let scheduling_strategies = vec![
        (SchedulingStrategy::FIFO, "先进先出调度"),
        (SchedulingStrategy::Priority, "优先级调度"),
        (SchedulingStrategy::CriticalPath, "关键路径优先调度"),
        (SchedulingStrategy::LoadBalanced, "负载均衡调度"),
    ];

    for (strategy, name) in scheduling_strategies {
        println!("\n🎛️  策略: {}", name);

        let scheduler = AdvancedScheduler::new(strategy.clone());

        // 添加任务到调度器
        let demo_tasks = vec![
            ("紧急需求分析", 10),
            ("系统架构设计", 9),
            ("核心功能开发", 8),
            ("界面优化", 6),
            ("文档编写", 4),
            ("性能测试", 7),
        ];

        for (task_desc, priority) in &demo_tasks {
            let task = AgentTask::new(task_desc.to_string()).with_priority(*priority);
            if scheduler.add_task(&task).await.is_ok() {
                println!("  ✅ 添加任务: {} (优先级: {})", task_desc, priority);
            }
        }

        // 模拟任务执行
        println!("  🚀 开始执行任务...");
        let mut completed_count = 0;

        while !scheduler.is_all_completed().await {
            if let Some(next_task_id) = scheduler.get_next_task().await {
                scheduler.mark_task_started(&next_task_id).await;
                println!("    🏃 执行任务: {}", next_task_id.replace("task_", ""));

                // 模拟任务执行时间
                tokio::time::sleep(Duration::from_millis(100)).await;

                scheduler.mark_task_completed(&next_task_id).await;
                completed_count += 1;
            } else {
                break;
            }

            if completed_count >= demo_tasks.len() {
                break;
            }
        }

        // 显示调度器统计
        let scheduler_stats = scheduler.get_stats().await;
        println!("  📊 调度统计:");
        println!("    - 总任务数: {}", scheduler_stats.total_tasks);
        println!("    - 已完成: {}", scheduler_stats.completed_tasks);
        println!("    - 等待中: {}", scheduler_stats.pending_tasks);
        println!("    - 队列长度: {}", scheduler_stats.queue_length);
    }

    // 创建完整的Crew演示
    println!("\n\n🎭 第四部分：完整Crew协作演示");
    println!("===========================");

    // 创建支持任务分解的高级Crew
    let advanced_crew = Crew::new("高级开发团队".to_string(), CollaborationMode::Parallel, 3);

    // 添加专业Agent
    let agents_data = vec![
        (
            "system_architect",
            AgentRole::new(
                "系统架构师".to_string(),
                "负责系统架构设计和技术选型".to_string(),
                "经验丰富的系统架构师，擅长复杂系统设计".to_string(),
            )
            .with_skills(vec![
                "架构设计".to_string(),
                "技术选型".to_string(),
                "性能优化".to_string(),
            ]),
        ),
        (
            "fullstack_developer",
            AgentRole::new(
                "全栈开发者".to_string(),
                "负责前后端开发和系统集成".to_string(),
                "技术全面的全栈工程师".to_string(),
            )
            .with_skills(vec![
                "前端开发".to_string(),
                "后端开发".to_string(),
                "数据库".to_string(),
            ]),
        ),
        (
            "data_engineer",
            AgentRole::new(
                "数据工程师".to_string(),
                "负责数据处理和分析".to_string(),
                "专业数据工程师，精通大数据处理".to_string(),
            )
            .with_skills(vec![
                "数据处理".to_string(),
                "数据分析".to_string(),
                "ETL".to_string(),
            ]),
        ),
    ];

    // 添加Agent到Crew
    for (agent_id, role) in agents_data {
        let agent = Agent::builder()
            .name(agent_id)
            .instructions(role.goal.clone())
            .model(llm.clone())
            .build()?;

        advanced_crew
            .add_agent(agent_id.to_string(), Arc::new(agent), role)
            .await?;
        println!("✅ 添加Agent: {}", agent_id);
    }

    // 添加复杂项目任务
    let complex_project_task = AgentTask::new("开发智能数据分析平台".to_string())
        .with_expected_output(
            "完整的数据分析平台，包括数据收集、处理、分析、可视化等功能".to_string(),
        )
        .with_priority(9);

    println!(
        "\n🎯 添加复杂项目任务: {}",
        complex_project_task.description
    );

    // 分解复杂任务
    let subtasks = decomposer.decompose_task(&complex_project_task).await?;
    println!("📋 自动分解出 {} 个子任务", subtasks.len());

    // 将子任务添加到Crew
    for subtask in subtasks {
        advanced_crew.add_task(subtask).await?;
    }

    // 显示最终统计
    let final_stats = advanced_crew.get_stats().await;
    println!("\n📊 最终Crew统计:");
    println!("  - Agent数量: {}", final_stats.total_agents);
    println!("  - 总任务数: {}", final_stats.total_tasks);
    println!("  - 待处理任务: {}", final_stats.pending_tasks);

    println!("\n🎉 高级Agent协作系统演示完成!");
    println!("==================================");
    println!("✅ 已演示功能:");
    println!("  🧠 智能任务分解 (基于复杂度和模式匹配)");
    println!("  🔗 依赖关系管理 (拓扑排序、关键路径分析)");
    println!("  ⚡ 高级调度算法 (FIFO、优先级、关键路径、负载均衡)");
    println!("  🎭 多Agent协作 (专业分工、智能负载均衡)");
    println!("\n🚀 系统已准备好处理复杂的企业级多Agent工作流!");

    Ok(())
}
