use lumosai_core::prelude::*;
use lumosai_core::error::Result;
use lumosai_core::base::Base;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 测试 LumosAI 渐进式 API");
    
    // Level 1 API - 5分钟上手（智能默认值）
    println!("\n📝 Level 1 API - 5分钟上手");
    match Agent::new("assistant", "你是一个AI助手").await {
        Ok(agent) => {
            println!("✅ Level 1 Agent 创建成功: {}", agent.name());
            
            // 测试基本生成
            match agent.generate("你好").await {
                Ok(response) => println!("🤖 响应: {}", response),
                Err(e) => println!("❌ 生成失败: {}", e),
            }
        }
        Err(e) => println!("❌ Level 1 Agent 创建失败: {}", e),
    }
    
    // Level 2 API - 链式配置
    println!("\n📝 Level 2 API - 链式配置");
    match Agent::new("assistant", "你是一个AI助手").await {
        Ok(agent) => {
            println!("✅ 基础 Agent 创建成功");
            
            // 测试链式配置
            match agent.with_model("deepseek-chat") {
                Ok(enhanced_agent) => {
                    println!("✅ 模型配置成功: {}", enhanced_agent.name());
                    
                    // 测试工具配置
                    match enhanced_agent.with_tools(vec![calculator(), datetime()]) {
                        Ok(final_agent) => {
                            println!("✅ 工具配置成功，工具数量: 2");
                            
                            match final_agent.generate("计算 2+3").await {
                                Ok(response) => println!("🤖 响应: {}", response),
                                Err(e) => println!("❌ 生成失败: {}", e),
                            }
                        }
                        Err(e) => println!("❌ 工具配置失败: {}", e),
                    }
                }
                Err(e) => println!("❌ 模型配置失败: {}", e),
            }
        }
        Err(e) => println!("❌ Level 2 Agent 创建失败: {}", e),
    }
    
    // Level 3 API - 完整构建器
    println!("\n📝 Level 3 API - 完整构建器");
    
    // 使用智谱 AI（已验证可工作）
    match deepseek("deepseek-chat") {
        Ok(llm) => {
            match Agent::builder()
                .name("research_agent")
                .instructions("你是专业的研究助手")
                .model(llm)
                .tool(calculator())
                .tool(datetime())
                .max_tool_calls(5)
                .enable_smart_defaults()
                .build()
            {
                Ok(agent) => {
                    println!("✅ Level 3 Agent 创建成功: {:?}", agent.name());
                    println!("🔧 Level 3 API 构建器模式验证成功");
                }
                Err(e) => println!("❌ Level 3 Agent 构建失败: {}", e),
            }
        }
        Err(e) => println!("❌ LLM 提供商创建失败: {}", e),
    }
    
    println!("\n✅ 渐进式 API 测试完成！");
    Ok(())
}
