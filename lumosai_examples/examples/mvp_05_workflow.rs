//! MVP 示例 5: Workflow 工作流编排
//!
//! 这个示例展示如何使用 DAG Workflow 编排复杂的 Agent 任务。
//!
//! 运行方式:
//! ```bash
//! cargo run --example mvp_05_workflow
//! ```

use lumosai_core::agent::{create_basic_agent, Agent};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::llm::LlmProvider;
use lumosai_core::workflow::dag_workflow::{DagWorkflow, DagWorkflowBuilder};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 5: Workflow 工作流编排\n");
    println!("=".repeat(50));
    
    // 步骤 1: 创建 LLM 提供商
    println!("\n📝 步骤 1: 创建 LLM 提供商");
    let llm: Arc<dyn LlmProvider> = create_test_zhipu_provider_arc();
    println!("✅ LLM 提供商创建成功");
    
    // 步骤 2: 创建 Agent 团队
    println!("\n📝 步骤 2: 创建 Agent 团队");
    
    // 需求分析 Agent
    let requirements_agent = create_basic_agent(
        "requirements",
        "You are a requirements analyst. Analyze and clarify project requirements.",
        llm.clone(),
    );
    println!("✅ 创建 Requirements Agent");
    
    // 设计 Agent
    let design_agent = create_basic_agent(
        "design",
        "You are a system designer. Create technical designs based on requirements.",
        llm.clone(),
    );
    println!("✅ 创建 Design Agent");
    
    // 开发 Agent
    let development_agent = create_basic_agent(
        "development",
        "You are a developer. Implement features based on design specifications.",
        llm.clone(),
    );
    println!("✅ 创建 Development Agent");
    
    // 测试 Agent
    let testing_agent = create_basic_agent(
        "testing",
        "You are a QA engineer. Test the implementation and report issues.",
        llm.clone(),
    );
    println!("✅ 创建 Testing Agent");
    
    // 部署 Agent
    let deployment_agent = create_basic_agent(
        "deployment",
        "You are a DevOps engineer. Plan and execute deployment.",
        llm,
    );
    println!("✅ 创建 Deployment Agent");
    
    // 步骤 3: 创建 DAG Workflow
    println!("\n📝 步骤 3: 创建 DAG Workflow");
    let mut workflow = DagWorkflowBuilder::new("software_development")
        .description("A complete software development workflow")
        .build();
    println!("✅ Workflow '{}' 创建成功", workflow.name());
    
    // 步骤 4: 添加节点
    println!("\n📝 步骤 4: 添加 Agent 节点");
    workflow.add_agent_node("requirements", requirements_agent)?;
    workflow.add_agent_node("design", design_agent)?;
    workflow.add_agent_node("development", development_agent)?;
    workflow.add_agent_node("testing", testing_agent)?;
    workflow.add_agent_node("deployment", deployment_agent)?;
    println!("✅ 添加了 5 个 Agent 节点");
    
    // 步骤 5: 定义依赖关系
    println!("\n📝 步骤 5: 定义依赖关系");
    println!("   设置工作流依赖:");
    
    // design 依赖 requirements
    workflow.add_dependency("design", "requirements")?;
    println!("   - design → requirements");
    
    // development 依赖 design
    workflow.add_dependency("development", "design")?;
    println!("   - development → design");
    
    // testing 依赖 development
    workflow.add_dependency("testing", "development")?;
    println!("   - testing → development");
    
    // deployment 依赖 testing
    workflow.add_dependency("deployment", "testing")?;
    println!("   - deployment → testing");
    
    println!("✅ 依赖关系定义完成");
    
    // 步骤 6: 验证 DAG
    println!("\n📝 步骤 6: 验证 DAG");
    match workflow.validate() {
        Ok(_) => println!("✅ DAG 验证通过（无环）"),
        Err(e) => {
            eprintln!("❌ DAG 验证失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 步骤 7: 获取拓扑排序
    println!("\n📝 步骤 7: 获取拓扑排序");
    let levels = workflow.get_topological_levels()?;
    println!("✅ 拓扑排序完成，共 {} 层:", levels.len());
    for (i, level) in levels.iter().enumerate() {
        println!("   层级 {}: {:?}", i + 1, level);
    }
    
    // 步骤 8: 执行 Workflow
    println!("\n📝 步骤 8: 执行 Workflow");
    println!("━".repeat(50));
    
    let project_description = "Build a simple web API for user management with authentication";
    println!("\n📌 项目: {}", project_description);
    println!("\n🔄 开始执行工作流...\n");
    
    match workflow.execute(project_description).await {
        Ok(results) => {
            println!("━".repeat(50));
            println!("\n✅ Workflow 执行完成！");
            println!("\n📊 执行结果:");
            println!("{}", "─".repeat(50));
            
            for (node_id, result) in results.iter() {
                println!("\n🔹 节点: {}", node_id);
                println!("   状态: {:?}", result.status);
                if let Some(output) = &result.output {
                    println!("   输出: {}", output);
                }
                if let Some(error) = &result.error {
                    println!("   错误: {}", error);
                }
                println!("   耗时: {:?}", result.duration);
            }
            
            println!("\n{}", "─".repeat(50));
        }
        Err(e) => {
            eprintln!("❌ Workflow 执行失败: {}", e);
        }
    }
    
    // 步骤 9: 展示 Workflow 统计
    println!("\n📝 步骤 9: Workflow 统计");
    println!("━".repeat(50));
    println!("\n📊 Workflow 信息:");
    println!("   - 名称: {}", workflow.name());
    println!("   - 描述: {}", workflow.description().unwrap_or("无"));
    println!("   - 节点数: {}", workflow.node_count());
    println!("   - 层级数: {}", levels.len());
    
    println!("\n" + &"=".repeat(50));
    println!("✅ 示例完成！");
    println!("\n💡 提示:");
    println!("   - DAG Workflow 支持基于依赖的智能调度");
    println!("   - 自动检测环形依赖");
    println!("   - 支持并行执行独立节点");
    println!("   - 可以追踪每个节点的执行状态和结果");
    println!("\n🎉 恭喜！你已经完成了所有 MVP 示例！");
    println!("   下一步: 探索更多高级功能");
    println!("   - RAG 系统（文档检索）");
    println!("   - 资源池优化（连接池、对象池）");
    println!("   - 缓存机制（多层缓存）");
    println!("   - 性能监控（资源监控）");
    
    Ok(())
}

