//! # 配置示例
//! 
//! 这个示例展示了如何配置 Agent 的各种参数。

use lumosai::prelude::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 LumosAI Agent 配置示例");
    println!("=" .repeat(40));
    
    // 示例 1: 基础配置
    println!("\n📝 示例 1: 基础配置");
    let basic_agent = Agent::builder()
        .name("基础助手")
        .instructions("你是一个基础助手")
        .model("gpt-3.5-turbo")
        .build()?;
    
    println!("Agent 名称: {}", basic_agent.name());
    println!("Agent 模型: {}", basic_agent.model());
    
    let response = basic_agent.generate("简单介绍一下你自己").await?;
    println!("回复: {}\n", response);
    
    // 示例 2: 详细配置
    println!("📝 示例 2: 详细配置");
    let detailed_agent = Agent::builder()
        .name("专业助手")
        .instructions(
            "你是一个专业的助手，具有以下特点：\
            1. 回答准确、详细\
            2. 语言正式但友好\
            3. 会主动提供相关建议\
            4. 遇到不确定的问题会诚实说明"
        )
        .model("gpt-3.5-turbo")
        .temperature(0.3)  // 降低创造性，提高准确性
        .max_tokens(200)   // 限制回复长度
        .build()?;
    
    println!("Agent 名称: {}", detailed_agent.name());
    println!("Agent 模型: {}", detailed_agent.model());
    
    let response = detailed_agent.generate("什么是人工智能？").await?;
    println!("回复: {}\n", response);
    
    // 示例 3: 创造性配置
    println!("📝 示例 3: 创造性配置");
    let creative_agent = Agent::builder()
        .name("创意助手")
        .instructions(
            "你是一个富有创意的助手，善于：\
            1. 提供创新的想法和解决方案\
            2. 用生动有趣的方式表达\
            3. 鼓励用户思考和探索\
            4. 保持积极乐观的态度"
        )
        .model("gpt-3.5-turbo")
        .temperature(0.9)  // 提高创造性
        .max_tokens(300)
        .build()?;
    
    println!("Agent 名称: {}", creative_agent.name());
    println!("Agent 模型: {}", creative_agent.model());
    
    let response = creative_agent.generate("给我一个有趣的编程项目想法").await?;
    println!("回复: {}\n", response);
    
    // 示例 4: 对比不同温度设置
    println!("📝 示例 4: 温度对比");
    
    let conservative_agent = Agent::builder()
        .name("保守助手")
        .instructions("你是一个保守的助手，回答要准确可靠。")
        .model("gpt-3.5-turbo")
        .temperature(0.1)  // 非常保守
        .build()?;
    
    let balanced_agent = Agent::builder()
        .name("平衡助手")
        .instructions("你是一个平衡的助手，既要准确也要有趣。")
        .model("gpt-3.5-turbo")
        .temperature(0.7)  // 平衡
        .build()?;
    
    let creative_agent_2 = Agent::builder()
        .name("创意助手2")
        .instructions("你是一个创意助手，可以大胆想象。")
        .model("gpt-3.5-turbo")
        .temperature(1.2)  // 非常创意
        .build()?;
    
    let question = "描述一下未来的城市";
    
    println!("问题: {}", question);
    
    println!("\n🔒 保守回答 (温度=0.1):");
    let response = conservative_agent.generate(question).await?;
    println!("{}", response);
    
    println!("\n⚖️ 平衡回答 (温度=0.7):");
    let response = balanced_agent.generate(question).await?;
    println!("{}", response);
    
    println!("\n🎨 创意回答 (温度=1.2):");
    let response = creative_agent_2.generate(question).await?;
    println!("{}", response);
    
    println!("\n🎉 配置示例完成！");
    println!("💡 关键要点:");
    println!("   - temperature 控制创造性 (0.0-2.0)");
    println!("   - max_tokens 限制回复长度");
    println!("   - instructions 定义 Agent 行为");
    println!("   - 不同配置适用于不同场景");
    
    Ok(())
}
