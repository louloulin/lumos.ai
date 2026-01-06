#!/usr/bin/env cargo
//! # 工作流自动化示例
//!
//! 本示例展示如何使用 LumosAI 创建智能工作流，
//! 自动化处理复杂的业务流程。
//!
//! ## 功能特性
//! - 智能任务调度和执行
//! - 条件分支和决策逻辑
//! - 错误处理和重试机制
//! - 实时状态监控和报告

use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::agent::BasicAgent;
use lumosai_core::llm::types::user_message;
use lumosai_core::prelude::*;
use serde_json::json;
use std::time::{Duration, Instant};

/// 工作流任务状态
#[derive(Debug, Clone, PartialEq)]
enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// 工作流任务
#[derive(Debug, Clone)]
struct WorkflowTask {
    id: String,
    name: String,
    description: String,
    status: TaskStatus,
    input: serde_json::Value,
    output: Option<serde_json::Value>,
    duration: Option<Duration>,
    error: Option<String>,
}

/// 工作流执行结果
#[derive(Debug)]
struct WorkflowResult {
    workflow_id: String,
    total_tasks: usize,
    completed_tasks: usize,
    failed_tasks: usize,
    total_duration: Duration,
    success_rate: f32,
    final_output: serde_json::Value,
}

/// 智能工作流引擎
struct IntelligentWorkflow {
    coordinator: BasicAgent,
    processor: BasicAgent,
    validator: BasicAgent,
    tasks: Vec<WorkflowTask>,
    execution_log: Vec<String>,
}

impl IntelligentWorkflow {
    /// 创建工作流引擎
    async fn new() -> Result<Self> {
        println!("🚀 正在初始化智能工作流引擎...");

        // 创建协调器 Agent
        let coordinator = quick_agent(
            "工作流协调器",
            "你是一个智能工作流协调器，负责：
            1. 分析任务依赖关系和执行顺序
            2. 做出智能调度决策
            3. 处理异常情况和错误恢复
            4. 优化工作流执行效率",
        )
        .build()?;

        // 创建处理器 Agent
        let processor = data_agent(
            "数据处理器",
            "你是一个专业的数据处理器，负责：
            1. 执行各种数据处理任务
            2. 转换和清洗数据
            3. 生成处理报告
            4. 确保数据质量和一致性",
        )
        .build()?;

        // 创建验证器 Agent
        let validator = quick_agent(
            "质量验证器",
            "你是一个质量验证专家，负责：
            1. 验证任务执行结果
            2. 检查数据质量和完整性
            3. 识别潜在问题和风险
            4. 提供改进建议",
        )
        .build()?;

        println!("✅ 工作流引擎初始化完成！");
        println!("   🎯 协调器 - 智能调度和决策");
        println!("   ⚙️  处理器 - 数据处理和转换");
        println!("   🔍 验证器 - 质量控制和验证");

        Ok(Self {
            coordinator,
            processor,
            validator,
            tasks: Vec::new(),
            execution_log: Vec::new(),
        })
    }

    /// 添加工作流任务
    fn add_task(&mut self, id: &str, name: &str, description: &str, input: serde_json::Value) {
        let task = WorkflowTask {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            status: TaskStatus::Pending,
            input,
            output: None,
            duration: None,
            error: None,
        };
        self.tasks.push(task);
        println!("📝 添加任务: {} - {}", id, name);
    }

    /// 执行工作流
    async fn execute_workflow(&mut self, workflow_id: &str) -> Result<WorkflowResult> {
        println!("\n🚀 开始执行工作流: {}", workflow_id);
        let start_time = Instant::now();

        // 阶段 1: 智能调度规划
        self.plan_execution().await?;

        // 阶段 2: 执行任务
        self.execute_tasks().await?;

        // 阶段 3: 验证结果
        self.validate_results().await?;

        // 计算执行统计
        let total_duration = start_time.elapsed();
        let completed_tasks = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let failed_tasks = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Failed)
            .count();
        let success_rate = completed_tasks as f32 / self.tasks.len() as f32;

        let result = WorkflowResult {
            workflow_id: workflow_id.to_string(),
            total_tasks: self.tasks.len(),
            completed_tasks,
            failed_tasks,
            total_duration,
            success_rate,
            final_output: json!({
                "status": "completed",
                "tasks_processed": completed_tasks,
                "execution_time": total_duration.as_secs_f32(),
                "success_rate": success_rate
            }),
        };

        println!("🎉 工作流执行完成！");
        Ok(result)
    }

    /// 阶段 1: 智能调度规划
    async fn plan_execution(&mut self) -> Result<()> {
        println!("\n📋 阶段 1: 智能调度规划...");

        let planning_prompt = format!(
            "请分析以下工作流任务并制定执行计划：

            任务列表:
            {}

            请考虑：
            1. 任务之间的依赖关系
            2. 最优执行顺序
            3. 并行执行的可能性
            4. 风险评估和预防措施

            请提供详细的执行计划。",
            self.tasks
                .iter()
                .map(|t| format!("- {}: {}", t.id, t.description))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let messages = vec![user_message(&planning_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.coordinator.generate(&messages, &options).await?;

        self.execution_log
            .push(format!("📋 执行计划: {}", response.response));
        println!("✅ 调度规划完成！");

        Ok(())
    }

    /// 阶段 2: 执行任务
    async fn execute_tasks(&mut self) -> Result<()> {
        println!("\n⚙️  阶段 2: 执行工作流任务...");

        for i in 0..self.tasks.len() {
            let task_start = Instant::now();
            self.tasks[i].status = TaskStatus::Running;

            println!(
                "   🔄 执行任务: {} - {}",
                self.tasks[i].id, self.tasks[i].name
            );

            // 模拟任务执行
            match self.execute_single_task(i).await {
                Ok(output) => {
                    self.tasks[i].status = TaskStatus::Completed;
                    self.tasks[i].output = Some(output);
                    self.tasks[i].duration = Some(task_start.elapsed());
                    println!("   ✅ 任务完成: {}", self.tasks[i].id);
                }
                Err(e) => {
                    self.tasks[i].status = TaskStatus::Failed;
                    self.tasks[i].error = Some(e.to_string());
                    self.tasks[i].duration = Some(task_start.elapsed());
                    println!("   ❌ 任务失败: {} - {}", self.tasks[i].id, e);

                    // 智能错误处理
                    self.handle_task_failure(i).await?;
                }
            }
        }

        println!("✅ 所有任务执行完成！");
        Ok(())
    }

    /// 执行单个任务
    async fn execute_single_task(&self, task_index: usize) -> Result<serde_json::Value> {
        let task = &self.tasks[task_index];

        let execution_prompt = format!(
            "请执行以下数据处理任务：

            任务ID: {}
            任务名称: {}
            任务描述: {}
            输入数据: {}

            请处理数据并返回结果。确保输出格式正确且数据完整。",
            task.id, task.name, task.description, task.input
        );

        let messages = vec![user_message(&execution_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.processor.generate(&messages, &options).await?;

        // 模拟处理结果
        let output = match task.id.as_str() {
            "data_collection" => json!({
                "collected_records": 1500,
                "data_sources": ["database", "api", "files"],
                "collection_time": "2024-01-16T10:30:00Z",
                "quality_score": 0.95
            }),
            "data_cleaning" => json!({
                "original_records": 1500,
                "cleaned_records": 1450,
                "removed_duplicates": 30,
                "fixed_errors": 20,
                "cleaning_rate": 0.967
            }),
            "data_analysis" => json!({
                "analysis_type": "statistical_summary",
                "key_metrics": {
                    "mean": 75.6,
                    "median": 78.2,
                    "std_dev": 12.4
                },
                "insights": ["数据分布正常", "无明显异常值", "质量良好"]
            }),
            "report_generation" => json!({
                "report_type": "executive_summary",
                "pages": 15,
                "sections": 6,
                "charts": 8,
                "completion_status": "ready_for_review"
            }),
            _ => json!({
                "status": "completed",
                "message": format!("任务 {} 执行成功", task.id),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        };

        Ok(output)
    }

    /// 处理任务失败
    async fn handle_task_failure(&mut self, task_index: usize) -> Result<()> {
        let task = &self.tasks[task_index];

        let error_prompt = format!(
            "任务执行失败，请分析原因并提供解决方案：

            失败任务: {} - {}
            错误信息: {}
            输入数据: {}

            请提供：
            1. 失败原因分析
            2. 可能的解决方案
            3. 是否建议重试
            4. 预防措施建议",
            task.id,
            task.name,
            task.error.as_ref().unwrap_or(&"未知错误".to_string()),
            task.input
        );

        let messages = vec![user_message(&error_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.coordinator.generate(&messages, &options).await?;

        self.execution_log.push(format!(
            "❌ 错误处理: 任务 {} - {}",
            task.id, response.response
        ));

        // 简单的重试逻辑（在实际应用中可以更复杂）
        if task.id.contains("critical") {
            println!("   🔄 关键任务失败，尝试恢复...");
            // 这里可以实现重试逻辑
        }

        Ok(())
    }

    /// 阶段 3: 验证结果
    async fn validate_results(&mut self) -> Result<()> {
        println!("\n🔍 阶段 3: 验证执行结果...");

        let completed_tasks: Vec<_> = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .collect();

        let validation_prompt = format!(
            "请验证以下工作流执行结果：

            完成任务数: {}
            总任务数: {}
            
            任务结果:
            {}

            请评估：
            1. 结果的准确性和完整性
            2. 数据质量和一致性
            3. 是否达到预期目标
            4. 潜在问题和改进建议",
            completed_tasks.len(),
            self.tasks.len(),
            completed_tasks
                .iter()
                .map(|t| format!("- {}: {:?}", t.id, t.output))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let messages = vec![user_message(&validation_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.validator.generate(&messages, &options).await?;

        self.execution_log
            .push(format!("🔍 质量验证: {}", response.response));
        println!("✅ 结果验证完成！");

        Ok(())
    }

    /// 显示执行统计
    fn show_execution_stats(&self, result: &WorkflowResult) {
        println!("\n📊 工作流执行统计:");
        println!("   🆔 工作流ID: {}", result.workflow_id);
        println!("   📝 总任务数: {}", result.total_tasks);
        println!("   ✅ 完成任务: {}", result.completed_tasks);
        println!("   ❌ 失败任务: {}", result.failed_tasks);
        println!(
            "   ⏱️  执行时间: {:.2} 秒",
            result.total_duration.as_secs_f32()
        );
        println!("   📈 成功率: {:.1}%", result.success_rate * 100.0);

        println!("\n📋 任务详情:");
        for task in &self.tasks {
            let status_icon = match task.status {
                TaskStatus::Completed => "✅",
                TaskStatus::Failed => "❌",
                TaskStatus::Running => "🔄",
                TaskStatus::Pending => "⏳",
                TaskStatus::Skipped => "⏭️",
            };
            let duration = task
                .duration
                .map(|d| format!("{:.2}s", d.as_secs_f32()))
                .unwrap_or_else(|| "N/A".to_string());
            println!(
                "   {} {} - {} ({})",
                status_icon, task.id, task.name, duration
            );
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 LumosAI 工作流自动化示例");
    println!("================================");

    // 创建工作流引擎
    let mut workflow = IntelligentWorkflow::new().await?;

    // 添加示例任务
    workflow.add_task(
        "data_collection",
        "数据收集",
        "从多个数据源收集原始数据",
        json!({"sources": ["database", "api", "files"], "filters": {"date_range": "2024-01"}}),
    );

    workflow.add_task(
        "data_cleaning",
        "数据清洗",
        "清洗和预处理收集的数据",
        json!({"operations": ["remove_duplicates", "fix_formats", "validate_fields"]}),
    );

    workflow.add_task(
        "data_analysis",
        "数据分析",
        "对清洗后的数据进行统计分析",
        json!({"analysis_type": "descriptive", "metrics": ["mean", "median", "std_dev"]}),
    );

    workflow.add_task(
        "report_generation",
        "报告生成",
        "基于分析结果生成执行报告",
        json!({"format": "pdf", "sections": ["summary", "details", "recommendations"]}),
    );

    // 执行工作流
    let result = workflow
        .execute_workflow("data_processing_workflow_001")
        .await?;

    // 显示执行统计
    workflow.show_execution_stats(&result);

    // 显示最终输出
    println!("\n📄 最终输出:");
    println!("{}", serde_json::to_string_pretty(&result.final_output)?);

    println!("\n🎉 工作流自动化示例运行完成！");
    println!("💡 这个示例展示了如何使用 LumosAI 创建智能工作流引擎，");
    println!("   实现任务的自动调度、执行、监控和质量控制。");

    Ok(())
}
