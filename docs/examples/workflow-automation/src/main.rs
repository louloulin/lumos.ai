use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};

use lumosai::prelude::SimpleAgent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 工作流自动化系统 - 展示如何使用 LumosAI 构建复杂的自动化工作流
#[derive(Parser)]
#[command(name = "workflow-automation")]
#[command(about = "LumosAI 工作流自动化示例")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 创建新的工作流
    Create {
        /// 工作流名称
        name: String,
        /// 工作流描述
        description: String,
        /// 步骤数量
        #[arg(short, long, default_value = "3")]
        steps: usize,
    },
    /// 执行工作流
    Execute {
        /// 工作流ID
        workflow_id: String,
        /// 输入数据
        #[arg(short, long)]
        input: Option<String>,
    },
    /// 运行预定义模板
    Template {
        /// 模板名称
        template: String,
        /// 模板参数
        #[arg(short, long)]
        params: Option<String>,
    },
    /// 查看工作流状态
    Status {
        /// 工作流ID
        workflow_id: String,
    },
    /// 列出所有工作流
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub description: String,
    pub agent_role: String,
    pub input_mapping: HashMap<String, String>,
    pub output_mapping: HashMap<String, String>,
    pub dependencies: Vec<String>,
    pub status: StepStatus,
    pub result: Option<String>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub status: WorkflowStatus,
    pub context: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStatus {
    Created,
    Running,
    Completed,
    Failed,
    Paused,
}

pub struct WorkflowEngine {
    workflows: Arc<Mutex<HashMap<String, Workflow>>>,
    agents: HashMap<String, SimpleAgent>,
}

impl WorkflowEngine {
    pub async fn new() -> Result<Self> {
        let mut agents = HashMap::new();
        
        // 创建不同角色的专业 Agent
        let data_analyst = lumosai::agent::simple(
            "gpt-3.5-turbo",
            "你是专业的数据分析师，擅长数据处理、统计分析和洞察提取。"
        ).await?;
        agents.insert("data_analyst".to_string(), data_analyst);
        
        let content_writer = lumosai::agent::simple(
            "gpt-3.5-turbo",
            "你是专业的内容创作者，擅长撰写各种类型的文档、报告和营销内容。"
        ).await?;
        agents.insert("content_writer".to_string(), content_writer);
        
        let project_manager = lumosai::agent::simple(
            "gpt-3.5-turbo",
            "你是经验丰富的项目经理，擅长任务规划、进度跟踪和团队协调。"
        ).await?;
        agents.insert("project_manager".to_string(), project_manager);
        
        let quality_reviewer = lumosai::agent::simple(
            "gpt-3.5-turbo",
            "你是质量保证专家，擅长审查、验证和改进工作成果。"
        ).await?;
        agents.insert("quality_reviewer".to_string(), quality_reviewer);
        
        Ok(Self {
            workflows: Arc::new(Mutex::new(HashMap::new())),
            agents,
        })
    }
    
    pub fn create_workflow(&self, name: String, description: String, steps: usize) -> Result<String> {
        let workflow_id = Uuid::new_v4().to_string();
        let mut workflow_steps = Vec::new();
        
        // 根据步骤数量创建默认步骤
        for i in 0..steps {
            let step_id = Uuid::new_v4().to_string();
            let step = WorkflowStep {
                id: step_id,
                name: format!("步骤 {}", i + 1),
                description: format!("工作流的第 {} 个步骤", i + 1),
                agent_role: match i % 4 {
                    0 => "data_analyst".to_string(),
                    1 => "content_writer".to_string(),
                    2 => "project_manager".to_string(),
                    _ => "quality_reviewer".to_string(),
                },
                input_mapping: HashMap::new(),
                output_mapping: HashMap::new(),
                dependencies: if i > 0 { vec![format!("step_{}", i)] } else { vec![] },
                status: StepStatus::Pending,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            };
            workflow_steps.push(step);
        }
        
        let workflow = Workflow {
            id: workflow_id.clone(),
            name,
            description,
            steps: workflow_steps,
            status: WorkflowStatus::Created,
            context: HashMap::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };
        
        self.workflows.lock().unwrap().insert(workflow_id.clone(), workflow);
        Ok(workflow_id)
    }
    
    pub async fn execute_workflow(&self, workflow_id: &str, input: Option<String>) -> Result<()> {
        let mut workflow = {
            let workflows = self.workflows.lock().unwrap();
            workflows.get(workflow_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("工作流不存在: {}", workflow_id))?
        };
        
        workflow.status = WorkflowStatus::Running;
        workflow.started_at = Some(Utc::now());
        
        if let Some(input_data) = input {
            workflow.context.insert("input".to_string(), input_data);
        }
        
        println!("开始执行工作流: {} ({})", workflow.name, workflow.id);
        println!("{}", "=".repeat(60));
        
        // 按依赖关系执行步骤
        for i in 0..workflow.steps.len() {
            let step_name = workflow.steps[i].name.clone();
            let agent_role = workflow.steps[i].agent_role.clone();
            let step_id = workflow.steps[i].id.clone();

            println!("\n执行步骤: {} ({})", step_name, agent_role);
            workflow.steps[i].status = StepStatus::Running;
            workflow.steps[i].started_at = Some(Utc::now());

            // 准备步骤输入
            let step_input = self.prepare_step_input(&workflow, &workflow.steps[i])?;
            
            // 执行步骤
            match self.execute_step(&workflow.steps[i], &step_input).await {
                Ok(result) => {
                    workflow.steps[i].result = Some(result.clone());
                    workflow.steps[i].status = StepStatus::Completed;
                    workflow.steps[i].completed_at = Some(Utc::now());

                    // 更新工作流上下文
                    workflow.context.insert(
                        format!("step_{}_output", step_id),
                        result
                    );

                    println!("✅ 步骤完成");
                }
                Err(e) => {
                    workflow.steps[i].error = Some(e.to_string());
                    workflow.steps[i].status = StepStatus::Failed;
                    workflow.steps[i].completed_at = Some(Utc::now());

                    println!("❌ 步骤失败: {}", e);
                    workflow.status = WorkflowStatus::Failed;
                    break;
                }
            }
        }
        
        // 检查工作流完成状态
        if workflow.status == WorkflowStatus::Running {
            let all_completed = workflow.steps.iter()
                .all(|step| matches!(step.status, StepStatus::Completed));
            
            if all_completed {
                workflow.status = WorkflowStatus::Completed;
                workflow.completed_at = Some(Utc::now());
                println!("\n🎉 工作流执行完成！");
            }
        }
        
        // 更新工作流状态
        self.workflows.lock().unwrap().insert(workflow_id.to_string(), workflow);
        
        Ok(())
    }
    
    fn prepare_step_input(&self, workflow: &Workflow, step: &WorkflowStep) -> Result<String> {
        let mut inputs = Vec::new();
        
        // 添加工作流上下文
        if let Some(input) = workflow.context.get("input") {
            inputs.push(format!("输入数据: {}", input));
        }
        
        // 添加依赖步骤的输出
        for dep_id in &step.dependencies {
            if let Some(output) = workflow.context.get(&format!("step_{}_output", dep_id)) {
                inputs.push(format!("前置步骤输出: {}", output));
            }
        }
        
        // 添加步骤描述
        inputs.push(format!("任务描述: {}", step.description));
        
        Ok(inputs.join("\n"))
    }
    
    async fn execute_step(&self, step: &WorkflowStep, input: &str) -> Result<String> {
        let agent = self.agents.get(&step.agent_role)
            .ok_or_else(|| anyhow::anyhow!("Agent 角色不存在: {}", step.agent_role))?;
        
        let prompt = format!(
            "请根据以下信息完成任务：\n{}\n\n请提供详细的分析和建议。",
            input
        );
        
        agent.chat(&prompt).await.map_err(|e| anyhow::anyhow!(e))
    }
    
    pub fn get_workflow(&self, workflow_id: &str) -> Option<Workflow> {
        self.workflows.lock().unwrap().get(workflow_id).cloned()
    }
    
    pub fn list_workflows(&self) -> Vec<Workflow> {
        self.workflows.lock().unwrap().values().cloned().collect()
    }
    
    pub async fn create_template_workflow(&self, template: &str, params: Option<String>) -> Result<String> {
        match template {
            "content-pipeline" => self.create_content_pipeline(params).await,
            "data-analysis" => self.create_data_analysis_workflow(params).await,
            "project-review" => self.create_project_review_workflow(params).await,
            _ => Err(anyhow::anyhow!("未知模板: {}", template))
        }
    }
    
    async fn create_content_pipeline(&self, _params: Option<String>) -> Result<String> {
        let workflow_id = Uuid::new_v4().to_string();
        
        let steps = vec![
            WorkflowStep {
                id: "research".to_string(),
                name: "内容研究".to_string(),
                description: "研究主题，收集相关信息和数据".to_string(),
                agent_role: "data_analyst".to_string(),
                input_mapping: HashMap::new(),
                output_mapping: HashMap::new(),
                dependencies: vec![],
                status: StepStatus::Pending,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
            WorkflowStep {
                id: "writing".to_string(),
                name: "内容创作".to_string(),
                description: "基于研究结果创作高质量内容".to_string(),
                agent_role: "content_writer".to_string(),
                input_mapping: HashMap::new(),
                output_mapping: HashMap::new(),
                dependencies: vec!["research".to_string()],
                status: StepStatus::Pending,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
            WorkflowStep {
                id: "review".to_string(),
                name: "质量审查".to_string(),
                description: "审查内容质量，提出改进建议".to_string(),
                agent_role: "quality_reviewer".to_string(),
                input_mapping: HashMap::new(),
                output_mapping: HashMap::new(),
                dependencies: vec!["writing".to_string()],
                status: StepStatus::Pending,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
        ];
        
        let workflow = Workflow {
            id: workflow_id.clone(),
            name: "内容创作流水线".to_string(),
            description: "从研究到发布的完整内容创作流程".to_string(),
            steps,
            status: WorkflowStatus::Created,
            context: HashMap::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };
        
        self.workflows.lock().unwrap().insert(workflow_id.clone(), workflow);
        Ok(workflow_id)
    }
    
    async fn create_data_analysis_workflow(&self, _params: Option<String>) -> Result<String> {
        let workflow_id = Uuid::new_v4().to_string();
        
        let steps = vec![
            WorkflowStep {
                id: "data_collection".to_string(),
                name: "数据收集".to_string(),
                description: "收集和整理分析所需的数据".to_string(),
                agent_role: "data_analyst".to_string(),
                input_mapping: HashMap::new(),
                output_mapping: HashMap::new(),
                dependencies: vec![],
                status: StepStatus::Pending,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
            WorkflowStep {
                id: "analysis".to_string(),
                name: "数据分析".to_string(),
                description: "进行深入的数据分析和模式识别".to_string(),
                agent_role: "data_analyst".to_string(),
                input_mapping: HashMap::new(),
                output_mapping: HashMap::new(),
                dependencies: vec!["data_collection".to_string()],
                status: StepStatus::Pending,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
            WorkflowStep {
                id: "report".to_string(),
                name: "报告生成".to_string(),
                description: "生成专业的分析报告".to_string(),
                agent_role: "content_writer".to_string(),
                input_mapping: HashMap::new(),
                output_mapping: HashMap::new(),
                dependencies: vec!["analysis".to_string()],
                status: StepStatus::Pending,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
        ];
        
        let workflow = Workflow {
            id: workflow_id.clone(),
            name: "数据分析工作流".to_string(),
            description: "完整的数据分析和报告生成流程".to_string(),
            steps,
            status: WorkflowStatus::Created,
            context: HashMap::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };
        
        self.workflows.lock().unwrap().insert(workflow_id.clone(), workflow);
        Ok(workflow_id)
    }
    
    async fn create_project_review_workflow(&self, _params: Option<String>) -> Result<String> {
        let workflow_id = Uuid::new_v4().to_string();
        
        let steps = vec![
            WorkflowStep {
                id: "planning".to_string(),
                name: "项目规划".to_string(),
                description: "制定项目计划和里程碑".to_string(),
                agent_role: "project_manager".to_string(),
                input_mapping: HashMap::new(),
                output_mapping: HashMap::new(),
                dependencies: vec![],
                status: StepStatus::Pending,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
            WorkflowStep {
                id: "execution_review".to_string(),
                name: "执行审查".to_string(),
                description: "审查项目执行情况和进度".to_string(),
                agent_role: "project_manager".to_string(),
                input_mapping: HashMap::new(),
                output_mapping: HashMap::new(),
                dependencies: vec!["planning".to_string()],
                status: StepStatus::Pending,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
            WorkflowStep {
                id: "quality_check".to_string(),
                name: "质量检查".to_string(),
                description: "进行全面的质量检查和验证".to_string(),
                agent_role: "quality_reviewer".to_string(),
                input_mapping: HashMap::new(),
                output_mapping: HashMap::new(),
                dependencies: vec!["execution_review".to_string()],
                status: StepStatus::Pending,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
        ];
        
        let workflow = Workflow {
            id: workflow_id.clone(),
            name: "项目审查工作流".to_string(),
            description: "项目规划、执行和质量检查的完整流程".to_string(),
            steps,
            status: WorkflowStatus::Created,
            context: HashMap::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };
        
        self.workflows.lock().unwrap().insert(workflow_id.clone(), workflow);
        Ok(workflow_id)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    let engine = WorkflowEngine::new().await?;
    
    match cli.command {
        Commands::Create { name, description, steps } => {
            let workflow_id = engine.create_workflow(name.clone(), description, steps)?;
            println!("✅ 工作流创建成功！");
            println!("工作流ID: {}", workflow_id);
            println!("名称: {}", name);
            println!("步骤数量: {}", steps);
        }
        
        Commands::Execute { workflow_id, input } => {
            engine.execute_workflow(&workflow_id, input).await?;
        }
        
        Commands::Template { template, params } => {
            let workflow_id = engine.create_template_workflow(&template, params).await?;
            println!("✅ 模板工作流创建成功！");
            println!("工作流ID: {}", workflow_id);
            println!("模板: {}", template);
            
            // 自动执行模板工作流
            println!("\n开始执行模板工作流...");
            engine.execute_workflow(&workflow_id, Some("示例输入数据".to_string())).await?;
        }
        
        Commands::Status { workflow_id } => {
            if let Some(workflow) = engine.get_workflow(&workflow_id) {
                println!("工作流状态: {} ({})", workflow.name, workflow.id);
                println!("状态: {:?}", workflow.status);
                println!("创建时间: {}", workflow.created_at.format("%Y-%m-%d %H:%M:%S"));
                
                if let Some(started) = workflow.started_at {
                    println!("开始时间: {}", started.format("%Y-%m-%d %H:%M:%S"));
                }
                
                if let Some(completed) = workflow.completed_at {
                    println!("完成时间: {}", completed.format("%Y-%m-%d %H:%M:%S"));
                }
                
                println!("\n步骤详情:");
                for (i, step) in workflow.steps.iter().enumerate() {
                    println!("  {}. {} ({:?})", i + 1, step.name, step.status);
                    if let Some(result) = &step.result {
                        println!("     结果: {}", result.chars().take(100).collect::<String>());
                        if result.len() > 100 {
                            println!("     ...");
                        }
                    }
                    if let Some(error) = &step.error {
                        println!("     错误: {}", error);
                    }
                }
            } else {
                println!("❌ 工作流不存在: {}", workflow_id);
            }
        }
        
        Commands::List => {
            let workflows = engine.list_workflows();
            if workflows.is_empty() {
                println!("没有找到工作流");
            } else {
                println!("工作流列表:");
                for workflow in workflows {
                    println!("  {} - {} ({:?})", 
                        workflow.id, 
                        workflow.name, 
                        workflow.status
                    );
                }
            }
        }
    }
    
    Ok(())
}
