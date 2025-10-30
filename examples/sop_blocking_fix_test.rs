//! 测试 AgentCommunicationManager 阻塞问题修复
//!
//! 验证在 async 上下文中创建 AgentCommunicationManager 和 SopEnvironment 不会 panic

use lumosai_core::agent::{
    AgentCommunicationManager, CollaborationMode, Crew, SopEnvironment, SopExecutionMode,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试 AgentCommunicationManager 阻塞问题修复\n");

    // 测试 1: 在 async 上下文中创建 AgentCommunicationManager
    println!("✅ 测试 1: 创建 AgentCommunicationManager");
    let comm_manager = AgentCommunicationManager::new(Default::default());
    println!("   成功创建 AgentCommunicationManager（无 panic）\n");

    // 测试 2: 在 async 上下文中创建 Crew
    println!("✅ 测试 2: 创建 Crew");
    let crew = Crew::new("test_crew".to_string(), CollaborationMode::Sequential, 10);
    println!("   成功创建 Crew（无 panic）\n");

    // 测试 3: 在 async 上下文中创建 SopEnvironment
    println!("✅ 测试 3: 创建 SopEnvironment");
    let sop_env = SopEnvironment::new("test_sop", SopExecutionMode::React);
    println!("   成功创建 SopEnvironment（无 panic）\n");

    // 测试 4: 从 Crew 创建 SopEnvironment
    println!("✅ 测试 4: 从 Crew 创建 SopEnvironment");
    let crew_arc = Arc::new(crew);
    let sop_env_from_crew = SopEnvironment::from_crew(crew_arc, SopExecutionMode::ByOrder);
    println!("   成功从 Crew 创建 SopEnvironment（无 panic）\n");

    // 测试 5: 获取统计信息
    println!("✅ 测试 5: 获取 SOP 统计信息");
    let stats = sop_env.get_stats().await;
    println!("   统计信息: {:?}\n", stats);

    println!("🎉 所有测试通过！阻塞问题已修复。");
    println!("\n📝 修复说明:");
    println!("   - 移除了 MessageQueueManager::with_config() 中的 blocking_write()");
    println!("   - 改为在构造函数中直接初始化 HashMap");
    println!("   - 现在可以在 async 上下文中安全创建 AgentCommunicationManager");

    Ok(())
}
