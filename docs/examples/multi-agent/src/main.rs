use clap::{Parser, Subcommand};
use lumosai::prelude::SimpleAgent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use futures::future::join_all;

#[derive(Parser)]
#[command(name = "multi-agent")]
#[command(about = "多 Agent 协作系统示例")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 创建新的协作任务
    Create {
        /// 任务描述
        description: String,
        /// 参与的 Agent 数量
        #[arg(short, long, default_value = "3")]
        agents: usize,
    },
    /// 执行预定义的协作场景
    Scenario {
        /// 场景名称
        name: String,
    },
    /// 启动交互式协作模式
    Interactive,
    /// 查看任务状态
    Status {
        /// 任务 ID
        task_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: String,
    description: String,
    status: TaskStatus,
    created_at: DateTime<Utc>,
    agents: Vec<AgentInfo>,
    messages: Vec<Message>,
    result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum TaskStatus {
    Created,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentInfo {
    id: String,
    name: String,
    role: String,
    specialization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    from: String,
    to: Option<String>, // None 表示广播消息
    content: String,
    timestamp: DateTime<Utc>,
    message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    Task,
    Question,
    Answer,
    Proposal,
    Agreement,
    Disagreement,
    FinalResult,
}

struct MultiAgentSystem {
    agents: HashMap<String, SimpleAgent>,
    tasks: Arc<Mutex<HashMap<String, Task>>>,
}

impl MultiAgentSystem {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            agents: HashMap::new(),
            tasks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn create_agents(&mut self, count: usize) -> anyhow::Result<Vec<AgentInfo>> {
        let roles = vec![
            ("研究员", "负责收集和分析信息，提供数据支持"),
            ("策略师", "负责制定计划和策略，协调整体方向"),
            ("执行者", "负责具体实施和操作，确保任务完成"),
            ("评估师", "负责质量控制和结果评估"),
            ("协调员", "负责沟通协调，确保团队协作顺畅"),
        ];

        let mut agent_infos = Vec::new();

        for i in 0..count {
            let (role, specialization) = &roles[i % roles.len()];
            let agent_id = Uuid::new_v4().to_string();
            let agent_name = format!("Agent-{}", i + 1);

            let system_prompt = format!(
                "你是一个多 Agent 协作系统中的 {}。你的专长是：{}。\n\
                 你需要与其他 Agent 协作完成任务。请始终保持专业、协作的态度，\n\
                 并根据你的角色提供相应的专业建议。",
                role, specialization
            );

            let agent = lumosai::agent::simple("gpt-3.5-turbo", &system_prompt)
                .await
                .map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;

            let agent_info = AgentInfo {
                id: agent_id.clone(),
                name: agent_name,
                role: role.to_string(),
                specialization: specialization.to_string(),
            };

            self.agents.insert(agent_id, agent);
            agent_infos.push(agent_info);
        }

        Ok(agent_infos)
    }

    pub async fn create_task(&self, description: String, agents: Vec<AgentInfo>) -> anyhow::Result<String> {
        let task_id = Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            description,
            status: TaskStatus::Created,
            created_at: Utc::now(),
            agents,
            messages: Vec::new(),
            result: None,
        };

        let mut tasks = self.tasks.lock().await;
        tasks.insert(task_id.clone(), task);

        Ok(task_id)
    }

    pub async fn execute_task(&self, task_id: &str) -> anyhow::Result<String> {
        let mut tasks = self.tasks.lock().await;
        let task = tasks.get_mut(task_id).ok_or_else(|| anyhow::anyhow!("任务不存在"))?;
        
        task.status = TaskStatus::InProgress;
        let task_description = task.description.clone();
        let agent_infos = task.agents.clone();
        drop(tasks);

        println!("🚀 开始执行多 Agent 协作任务: {}", task_description);
        println!("👥 参与 Agent: {}", agent_infos.len());

        // 第一阶段：任务分析和规划
        println!("\n📋 第一阶段：任务分析和规划");
        let planning_results = self.planning_phase(&task_description, &agent_infos).await?;

        // 第二阶段：并行执行
        println!("\n⚡ 第二阶段：并行执行");
        let execution_results = self.execution_phase(&task_description, &agent_infos, &planning_results).await?;

        // 第三阶段：结果整合
        println!("\n🔄 第三阶段：结果整合");
        let final_result = self.integration_phase(&task_description, &execution_results).await?;

        // 更新任务状态
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Completed;
            task.result = Some(final_result.clone());
        }

        Ok(final_result)
    }

    async fn planning_phase(&self, task_description: &str, agents: &[AgentInfo]) -> anyhow::Result<Vec<String>> {
        let mut results = Vec::new();

        for agent_info in agents {
            if let Some(agent) = self.agents.get(&agent_info.id) {
                let prompt = format!(
                    "任务：{}\n\n作为 {}，请分析这个任务并提出你的执行计划和建议。\n\
                     请考虑你的专长：{}",
                    task_description, agent_info.role, agent_info.specialization
                );

                println!("  {} ({}) 正在分析任务...", agent_info.name, agent_info.role);
                let response = agent.chat(&prompt).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
                results.push(format!("{}: {}", agent_info.role, response));
            }
        }

        Ok(results)
    }

    async fn execution_phase(&self, task_description: &str, agents: &[AgentInfo], planning_results: &[String]) -> anyhow::Result<Vec<String>> {
        let planning_context = planning_results.join("\n\n");
        
        let futures: Vec<_> = agents.iter().map(|agent_info| {
            let agent = self.agents.get(&agent_info.id).unwrap();
            let prompt = format!(
                "任务：{}\n\n规划阶段的讨论结果：\n{}\n\n\
                 现在请作为 {} 执行你负责的部分。请提供具体的执行结果。",
                task_description, planning_context, agent_info.role
            );
            
            async move {
                println!("  {} ({}) 正在执行任务...", agent_info.name, agent_info.role);
                let response = agent.chat(&prompt).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
                Ok::<String, anyhow::Error>(format!("{}: {}", agent_info.role, response))
            }
        }).collect();

        let results = join_all(futures).await;
        let mut execution_results = Vec::new();
        
        for result in results {
            execution_results.push(result?);
        }

        Ok(execution_results)
    }

    async fn integration_phase(&self, task_description: &str, execution_results: &[String]) -> anyhow::Result<String> {
        // 选择第一个 Agent 作为整合者
        if let Some((_, agent)) = self.agents.iter().next() {
            let execution_context = execution_results.join("\n\n");
            let prompt = format!(
                "任务：{}\n\n各个 Agent 的执行结果：\n{}\n\n\
                 请整合所有结果，提供一个完整、连贯的最终答案。",
                task_description, execution_context
            );

            println!("  正在整合所有结果...");
            let final_result = agent.chat(&prompt).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
            Ok(final_result)
        } else {
            Err(anyhow::anyhow!("没有可用的 Agent 进行结果整合"))
        }
    }

    pub async fn get_task_status(&self, task_id: &str) -> anyhow::Result<Task> {
        let tasks = self.tasks.lock().await;
        tasks.get(task_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("任务不存在"))
    }
}

async fn create_collaboration_task(description: String, agent_count: usize) -> anyhow::Result<()> {
    let mut system = MultiAgentSystem::new().await?;
    
    println!("🔧 创建 {} 个 Agent...", agent_count);
    let agents = system.create_agents(agent_count).await?;
    
    for agent in &agents {
        println!("  ✅ {} - {} ({})", agent.name, agent.role, agent.specialization);
    }

    let task_id = system.create_task(description, agents).await?;
    println!("\n📝 任务已创建，ID: {}", task_id);

    let result = system.execute_task(&task_id).await?;
    
    println!("\n🎯 最终结果：");
    println!("{}", result);

    Ok(())
}

async fn run_scenario(scenario_name: String) -> anyhow::Result<()> {
    match scenario_name.as_str() {
        "product-launch" => {
            println!("🚀 运行产品发布协作场景");
            create_collaboration_task(
                "我们需要为一个新的 AI 编程助手产品制定完整的发布计划，包括市场分析、技术准备、营销策略和风险评估。".to_string(),
                4
            ).await
        },
        "research-project" => {
            println!("🔬 运行研究项目协作场景");
            create_collaboration_task(
                "研究大语言模型在代码生成领域的最新进展，分析技术趋势、应用场景和未来发展方向。".to_string(),
                3
            ).await
        },
        "crisis-response" => {
            println!("🚨 运行危机响应协作场景");
            create_collaboration_task(
                "系统出现严重性能问题，需要快速诊断问题原因、制定解决方案并实施修复措施。".to_string(),
                5
            ).await
        },
        _ => {
            println!("❌ 未知场景: {}", scenario_name);
            println!("可用场景: product-launch, research-project, crisis-response");
            Ok(())
        }
    }
}

async fn interactive_mode() -> anyhow::Result<()> {
    println!("🎮 进入交互式多 Agent 协作模式");
    println!("输入任务描述，系统将自动创建 Agent 团队来协作完成任务");
    println!("输入 'quit' 或 'exit' 退出\n");

    let mut system = MultiAgentSystem::new().await?;

    loop {
        print!("请输入任务描述: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "quit" || input == "exit" {
            println!("👋 再见！");
            break;
        }

        if input.is_empty() {
            continue;
        }

        println!("\n🔧 为任务创建专门的 Agent 团队...");
        let agents = system.create_agents(3).await?;
        
        for agent in &agents {
            println!("  ✅ {} - {}", agent.name, agent.role);
        }

        let task_id = system.create_task(input.to_string(), agents).await?;
        let result = system.execute_task(&task_id).await?;
        
        println!("\n🎯 协作结果：");
        println!("{}", result);
        println!("\n{}\n", "=".repeat(60));
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { description, agents } => {
            create_collaboration_task(description, agents).await
        },
        Commands::Scenario { name } => {
            run_scenario(name).await
        },
        Commands::Interactive => {
            interactive_mode().await
        },
        Commands::Status { task_id } => {
            let system = MultiAgentSystem::new().await?;
            let task = system.get_task_status(&task_id).await?;
            println!("任务状态: {:?}", task);
            Ok(())
        },
    }
}
