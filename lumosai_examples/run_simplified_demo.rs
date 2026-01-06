// 简化API演示运行器
// 这个文件用于测试和演示简化API的功能

use std::env;

fn main() {
    println!("🚀 Lumos简化API演示");
    println!("=====================================");

    // 检查环境变量
    let api_key = env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| "test-key".to_string());

    println!("✅ 环境检查:");
    println!("  • DeepSeek API Key: {}", if api_key == "test-key" { "未设置 (使用测试密钥)" } else { "已设置" });

    println!("\n🎯 简化API特性:");
    println!("  • 🏗️  构建器模式 - 更直观的Agent创建");
    println!("  • 🔧 简化工具定义 - 减少样板代码");
    println!("  • 🎯 类型安全 - 编译时错误检查");
    println!("  • 🔌 DeepSeek集成 - 真实AI模型支持");

    println!("\n📝 代码示例:");
    println!("```rust");
    println!("// 使用构建器模式创建Agent");
    println!("let agent = AgentBuilder::new()");
    println!("    .name(\"stock_agent\")");
    println!("    .instructions(\"你是一个专业的股票分析师\")");
    println!("    .model(deepseek_provider)");
    println!("    .tool(stock_price_tool)");
    println!("    .build()?;");
    println!("```");

    println!("\n🎉 演示完成！");
    println!("💡 提示: 设置 DEEPSEEK_API_KEY 环境变量以使用真实的AI模型");
}
