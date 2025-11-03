//! DAG Workflow 演示
//!
//! 展示如何使用 DAG Workflow 构建复杂的数据处理管道

use async_trait::async_trait;
use lumosai_core::agent::types::RuntimeContext;
use lumosai_core::workflow::{
    DagWorkflow, DagWorkflowBuilder, StepExecutor, Workflow, WorkflowStep,
};
use lumosai_core::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// 自定义步骤执行器
// ============================================================================

/// 数据提取执行器
struct DataExtractorExecutor {
    source: String,
}

#[async_trait]
impl StepExecutor for DataExtractorExecutor {
    async fn execute(&self, input: Value, _context: &RuntimeContext) -> Result<Value> {
        println!("📥 从 {} 提取数据...", self.source);
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(json!({
            "source": self.source,
            "data": vec!["record1", "record2", "record3"],
            "count": 3,
            "extracted_at": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// 数据清洗执行器
struct DataCleanerExecutor {
    rules: Vec<String>,
}

#[async_trait]
impl StepExecutor for DataCleanerExecutor {
    async fn execute(&self, input: Value, _context: &RuntimeContext) -> Result<Value> {
        println!("🧹 清洗数据（规则: {:?}）...", self.rules);
        tokio::time::sleep(Duration::from_millis(150)).await;

        let source = input.get("source").and_then(|v| v.as_str()).unwrap_or("unknown");
        let count = input.get("count").and_then(|v| v.as_u64()).unwrap_or(0);

        Ok(json!({
            "source": source,
            "cleaned_data": vec!["clean_record1", "clean_record2"],
            "count": count.saturating_sub(1), // 安全地减 1，避免溢出
            "rules_applied": self.rules,
            "cleaned_at": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// 数据转换执行器
struct DataTransformerExecutor {
    format: String,
}

#[async_trait]
impl StepExecutor for DataTransformerExecutor {
    async fn execute(&self, input: Value, _context: &RuntimeContext) -> Result<Value> {
        println!("🔄 转换数据到 {} 格式...", self.format);
        tokio::time::sleep(Duration::from_millis(120)).await;

        Ok(json!({
            "format": self.format,
            "transformed_data": "transformed_content",
            "input_summary": format!("Processed {} records", input.get("count").and_then(|v| v.as_u64()).unwrap_or(0)),
            "transformed_at": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// 数据验证执行器
struct DataValidatorExecutor {
    schema: String,
}

#[async_trait]
impl StepExecutor for DataValidatorExecutor {
    async fn execute(&self, input: Value, _context: &RuntimeContext) -> Result<Value> {
        println!("✅ 验证数据（schema: {}）...", self.schema);
        tokio::time::sleep(Duration::from_millis(80)).await;

        Ok(json!({
            "valid": true,
            "schema": self.schema,
            "validation_errors": [],
            "validated_at": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// 数据聚合执行器
struct DataAggregatorExecutor;

#[async_trait]
impl StepExecutor for DataAggregatorExecutor {
    async fn execute(&self, input: Value, _context: &RuntimeContext) -> Result<Value> {
        println!("📊 聚合所有数据源...");
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(json!({
            "aggregated": true,
            "total_sources": 3,
            "aggregated_at": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// 报告生成执行器
struct ReportGeneratorExecutor {
    format: String,
}

#[async_trait]
impl StepExecutor for ReportGeneratorExecutor {
    async fn execute(&self, input: Value, _context: &RuntimeContext) -> Result<Value> {
        println!("📄 生成 {} 格式报告...", self.format);
        tokio::time::sleep(Duration::from_millis(200)).await;

        Ok(json!({
            "report_format": self.format,
            "report_url": format!("https://example.com/reports/{}.{}", chrono::Utc::now().timestamp(), self.format),
            "generated_at": chrono::Utc::now().to_rfc3339()
        }))
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

fn create_step(id: &str, name: &str, executor: Arc<dyn StepExecutor>) -> WorkflowStep {
    WorkflowStep {
        id: id.to_string(),
        description: Some(name.to_string()),
        step_type: lumosai_core::workflow::StepType::Simple,
        execute: executor,
        input_schema: None,
        output_schema: None,
    }
}

// ============================================================================
// 主函数
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 DAG Workflow 演示");
    println!("{}", "=".repeat(80));
    println!();

    // ========================================
    // 场景 1: 简单的数据处理管道
    // ========================================
    println!("📋 场景 1: 简单的数据处理管道");
    println!("{}", "-".repeat(80));

    let simple_workflow = DagWorkflowBuilder::new("simple_pipeline".to_string())
        .description("简单的数据处理管道".to_string())
        .max_concurrency(4)
        .build();

    // 构建管道: 提取 -> 清洗 -> 转换 -> 验证
    simple_workflow
        .add_root_node(
            "extract".to_string(),
            "数据提取".to_string(),
            create_step(
                "extract",
                "数据提取",
                Arc::new(DataExtractorExecutor {
                    source: "database".to_string(),
                }),
            ),
        )
        .await?;

    simple_workflow
        .add_node(
            "clean".to_string(),
            "数据清洗".to_string(),
            vec!["extract".to_string()],
            create_step(
                "clean",
                "数据清洗",
                Arc::new(DataCleanerExecutor {
                    rules: vec!["remove_nulls".to_string(), "deduplicate".to_string()],
                }),
            ),
        )
        .await?;

    simple_workflow
        .add_node(
            "transform".to_string(),
            "数据转换".to_string(),
            vec!["clean".to_string()],
            create_step(
                "transform",
                "数据转换",
                Arc::new(DataTransformerExecutor {
                    format: "json".to_string(),
                }),
            ),
        )
        .await?;

    simple_workflow
        .add_node(
            "validate".to_string(),
            "数据验证".to_string(),
            vec!["transform".to_string()],
            create_step(
                "validate",
                "数据验证",
                Arc::new(DataValidatorExecutor {
                    schema: "v1.0".to_string(),
                }),
            ),
        )
        .await?;

    // 验证 DAG
    simple_workflow.validate().await?;
    println!("✅ DAG 验证通过（无环）");

    // 获取执行层级
    let levels = simple_workflow.get_execution_levels().await?;
    println!("📊 执行层级: {:?}", levels);
    println!();

    // 执行工作流
    let context = RuntimeContext::new();
    let input = json!({"job_id": "job_001"});

    println!("🚀 开始执行工作流...");
    let start_time = std::time::Instant::now();
    let result = simple_workflow.execute(input, &context).await?;
    let duration = start_time.elapsed();

    println!();
    println!("✅ 工作流执行完成！");
    println!("⏱️  执行时间: {:?}", duration);
    println!("📊 结果: {}", serde_json::to_string_pretty(&result)?);
    println!();

    // ========================================
    // 场景 2: 并行数据处理管道
    // ========================================
    println!("📋 场景 2: 并行数据处理管道（多数据源）");
    println!("{}", "-".repeat(80));

    let parallel_workflow = DagWorkflowBuilder::new("parallel_pipeline".to_string())
        .description("并行数据处理管道".to_string())
        .max_concurrency(8)
        .build();

    // 构建并行管道:
    //           root
    //         /  |  \
    //    extract1 extract2 extract3 (并行提取)
    //         \  |  /
    //        aggregate (聚合)
    //            |
    //         report (生成报告)

    // 根节点（触发器）
    parallel_workflow
        .add_root_node(
            "root".to_string(),
            "触发器".to_string(),
            create_step(
                "root",
                "触发器",
                Arc::new(DataExtractorExecutor {
                    source: "trigger".to_string(),
                }),
            ),
        )
        .await?;

    // 并行提取三个数据源
    parallel_workflow
        .add_node(
            "extract_db".to_string(),
            "提取数据库数据".to_string(),
            vec!["root".to_string()],
            create_step(
                "extract_db",
                "提取数据库数据",
                Arc::new(DataExtractorExecutor {
                    source: "database".to_string(),
                }),
            ),
        )
        .await?;

    parallel_workflow
        .add_node(
            "extract_api".to_string(),
            "提取API数据".to_string(),
            vec!["root".to_string()],
            create_step(
                "extract_api",
                "提取API数据",
                Arc::new(DataExtractorExecutor {
                    source: "api".to_string(),
                }),
            ),
        )
        .await?;

    parallel_workflow
        .add_node(
            "extract_file".to_string(),
            "提取文件数据".to_string(),
            vec!["root".to_string()],
            create_step(
                "extract_file",
                "提取文件数据",
                Arc::new(DataExtractorExecutor {
                    source: "file".to_string(),
                }),
            ),
        )
        .await?;

    // 聚合所有数据源
    parallel_workflow
        .add_node(
            "aggregate".to_string(),
            "数据聚合".to_string(),
            vec![
                "extract_db".to_string(),
                "extract_api".to_string(),
                "extract_file".to_string(),
            ],
            create_step("aggregate", "数据聚合", Arc::new(DataAggregatorExecutor)),
        )
        .await?;

    // 生成报告
    parallel_workflow
        .add_node(
            "report".to_string(),
            "生成报告".to_string(),
            vec!["aggregate".to_string()],
            create_step(
                "report",
                "生成报告",
                Arc::new(ReportGeneratorExecutor {
                    format: "pdf".to_string(),
                }),
            ),
        )
        .await?;

    // 验证 DAG
    parallel_workflow.validate().await?;
    println!("✅ DAG 验证通过（无环）");

    // 获取执行层级
    let levels = parallel_workflow.get_execution_levels().await?;
    println!("📊 执行层级: {:?}", levels);
    println!("📊 节点数量: {}", parallel_workflow.node_count().await);
    println!();

    // 执行工作流
    let context = RuntimeContext::new();
    let input = json!({"job_id": "job_002", "sources": ["db", "api", "file"]});

    println!("🚀 开始执行并行工作流...");
    let start_time = std::time::Instant::now();
    let result = parallel_workflow.execute(input, &context).await?;
    let duration = start_time.elapsed();

    println!();
    println!("✅ 并行工作流执行完成！");
    println!("⏱️  执行时间: {:?}", duration);
    println!("📊 结果节点数: {}", result.as_object().unwrap().len());
    println!();

    // ========================================
    // 总结
    // ========================================
    println!("{}", "=".repeat(80));
    println!("🎉 DAG Workflow 演示完成！");
    println!();
    println!("💡 关键特性:");
    println!("   ✅ 自动依赖分析和拓扑排序");
    println!("   ✅ 智能并行调度（同层级节点并行执行）");
    println!("   ✅ 环检测（防止无限循环）");
    println!("   ✅ 依赖输入自动合并");
    println!("   ✅ 并发度控制");
    println!("   ✅ 执行状态追踪");
    println!();
    println!("🚀 使用场景:");
    println!("   • 数据处理管道");
    println!("   • ETL 工作流");
    println!("   • 机器学习训练管道");
    println!("   • 微服务编排");
    println!("   • CI/CD 流水线");
    println!();

    Ok(())
}

