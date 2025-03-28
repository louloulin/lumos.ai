use lomusai_core::workflow::{
    Workflow, BasicStep, StepCondition, StepStatus, StepResult
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Lomusai Workflow Example");
    println!("=======================");
    
    // 创建步骤
    let extract_step = BasicStep::create_simple(
        "extract".to_string(),
        "Extract data from input".to_string(),
        |input| {
            println!("🔍 Extracting data...");
            // 从输入中提取信息
            Ok(json!({
                "source": "example data source",
                "text": "This is example text for processing.",
                "language": "en",
                "extracted": true
            }))
        },
    );
    
    let transform_step = BasicStep::create_simple(
        "transform".to_string(),
        "Transform extracted data".to_string(),
        |input| {
            println!("⚙️ Transforming data...");
            // 转换提取的数据
            Ok(json!({
                "processed_text": "PROCESSED: This is example text for processing.",
                "word_count": 7,
                "transformed": true
            }))
        },
    );
    
    let analyze_en_step = BasicStep::create_simple(
        "analyze_en".to_string(),
        "Analyze English content".to_string(),
        |input| {
            println!("📊 Analyzing English content...");
            // 分析英文内容
            Ok(json!({
                "sentiment": "neutral",
                "topics": ["example", "processing"],
                "language_confirmed": "en"
            }))
        },
    );
    
    let analyze_other_step = BasicStep::create_simple(
        "analyze_other".to_string(),
        "Analyze non-English content".to_string(),
        |input| {
            println!("📊 Analyzing non-English content...");
            // 分析非英文内容
            Ok(json!({
                "detected_language": "unknown",
                "needs_translation": true
            }))
        },
    );
    
    let load_step = BasicStep::create_simple(
        "load".to_string(),
        "Load processed data".to_string(),
        |input| {
            println!("💾 Loading processed data...");
            // 加载处理后的数据
            Ok(json!({
                "loaded": true,
                "destination": "example database",
                "timestamp": "2023-08-15T14:30:00Z"
            }))
        },
    );
    
    // 创建条件
    let english_condition = StepCondition::Reference {
        step_id: "extract".to_string(),
        path: "language".to_string(),
        query: json!({ "$eq": "en" }),
    };
    
    let non_english_condition = StepCondition::Not(Box::new(StepCondition::Reference {
        step_id: "extract".to_string(),
        path: "language".to_string(),
        query: json!({ "$eq": "en" }),
    }));
    
    // 创建工作流
    let workflow = Workflow::new("etl_workflow".to_string(), "ETL Process".to_string())
        // 添加步骤
        .add_step(extract_step, None, None)
        .add_step(transform_step, None, None)
        .add_step(analyze_en_step, None, Some(english_condition))
        .add_step(analyze_other_step, None, Some(non_english_condition))
        .add_step(load_step, None, None)
        // 添加依赖关系
        .add_dependency("extract", "transform")
        .add_dependency("transform", "analyze_en")
        .add_dependency("transform", "analyze_other")
        .add_dependency("analyze_en", "load")
        .add_dependency("analyze_other", "load")
        .build();
    
    println!("\n🚀 Starting workflow: {}", workflow.name());
    
    // 执行工作流
    let trigger_data = json!({
        "job_id": "example-001",
        "source": "user_upload",
        "timestamp": "2023-08-15T14:00:00Z"
    });
    
    let instance = workflow.create_run(trigger_data);
    let result = instance.run().await?;
    
    // 显示结果
    println!("\n✅ Workflow completed!");
    println!("Run ID: {}", result.run_id);
    println!("\nResults:");
    
    for (step_id, step_result) in &result.results {
        match step_result {
            StepResult::Success { output } => {
                println!("  - {} ✓ (Success)", step_id);
            },
            StepResult::Failed { error } => {
                println!("  - {} ❌ (Failed: {})", step_id, error);
            },
            StepResult::Skipped => {
                println!("  - {} ⏭️ (Skipped)", step_id);
            },
            _ => {
                println!("  - {} (Other status)", step_id);
            }
        }
    }
    
    println!("\nWorkflow execution complete!");
    
    Ok(())
} 