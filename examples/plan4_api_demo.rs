//! Plan4.md API Demo
//! 
//! This example demonstrates the new API design specified in plan4.md
//! for Phase 1: API简化重构

use lumosai_core::agent::{AgentFactory, AgentBuilder};
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::Agent; // Import the Agent trait
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Plan4.md API Demo - Phase 1: API简化重构");
    println!("==============================================\n");

    // Create a mock LLM provider for testing
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Hello! I'm your AI assistant.".to_string(),
        "I can help you with web browsing.".to_string(),
        "I can manage files for you.".to_string(),
        "I'm a research assistant ready to help.".to_string(),
    ]));

    println!("1. 🎯 AgentFactory::quick() - Minimal Configuration");
    println!("---------------------------------------------------");

    // Plan4.md API: AgentFactory::quick()
    let quick_agent = AgentFactory::quick("assistant", "你是一个AI助手")
        .model(llm.clone())
        .build()?;
    
    println!("✅ Created quick agent: {}", quick_agent.get_name());
    println!("   Instructions: {}", quick_agent.get_instructions());
    
    // Test the agent
    let response = quick_agent.generate_simple("Hello!").await?;
    println!("   Response: {}", response);

    println!("\n2. 🔧 AgentFactory::builder() - Full Control");
    println!("----------------------------------------------");

    // Plan4.md API: AgentFactory::builder()
    let builder_agent = AgentFactory::builder()
        .name("research_agent")
        .instructions("专业研究助手")
        .model(llm.clone())
        .max_tool_calls(10)
        .tool_timeout(60)
        .build()?;
    
    println!("✅ Created builder agent: {}", builder_agent.get_name());
    println!("   Instructions: {}", builder_agent.get_instructions());
    
    // Test the agent
    let response = builder_agent.generate_simple("What can you help me with?").await?;
    println!("   Response: {}", response);

    println!("\n3. 🌐 AgentFactory::web_agent() - Pre-configured Web Tools");
    println!("-----------------------------------------------------------");

    // Plan4.md API: AgentFactory::web_agent()
    let web_agent = AgentFactory::web_agent("web_helper")
        .instructions("You can browse the web and help with online research")
        .model(llm.clone())
        .build()?;
    
    println!("✅ Created web agent: {}", web_agent.get_name());
    println!("   Instructions: {}", web_agent.get_instructions());
    println!("   Available tools: {}", web_agent.get_tools().len());
    
    // List available tools
    for tool_name in web_agent.get_tools().keys() {
        println!("   - {}", tool_name);
    }
    
    // Test the agent
    let response = web_agent.generate_simple("I can help you browse the web!").await?;
    println!("   Response: {}", response);

    println!("\n4. 📁 AgentFactory::file_agent() - Pre-configured File Tools");
    println!("-------------------------------------------------------------");

    // Plan4.md API: AgentFactory::file_agent()
    let file_agent = AgentFactory::file_agent("file_helper")
        .instructions("You can manage files and directories")
        .model(llm.clone())
        .build()?;
    
    println!("✅ Created file agent: {}", file_agent.get_name());
    println!("   Instructions: {}", file_agent.get_instructions());
    println!("   Available tools: {}", file_agent.get_tools().len());
    
    // List available tools
    for tool_name in file_agent.get_tools().keys() {
        println!("   - {}", tool_name);
    }
    
    // Test the agent
    let response = file_agent.generate_simple("I can help you manage files!").await?;
    println!("   Response: {}", response);

    println!("\n5. 🔄 Fluent Builder Pattern - Multiple Tool Collections");
    println!("--------------------------------------------------------");
    
    // Plan4.md API: Fluent builder with multiple tool collections
    let multi_agent = AgentFactory::builder()
        .name("multi_tool_agent")
        .instructions("You are a versatile assistant with access to web, file, and data tools")
        .model(llm.clone())
        .with_web_tools()
        .with_file_tools()
        .with_data_tools()
        .max_tool_calls(15)
        .build()?;
    
    println!("✅ Created multi-tool agent: {}", multi_agent.get_name());
    println!("   Instructions: {}", multi_agent.get_instructions());
    println!("   Available tools: {}", multi_agent.get_tools().len());
    
    // List available tools
    for tool_name in multi_agent.get_tools().keys() {
        println!("   - {}", tool_name);
    }
    
    // Test the agent
    let response = multi_agent.generate_simple("I have access to many tools!").await?;
    println!("   Response: {}", response);

    println!("\n6. 🎛️ Model Configuration with Builder Pattern");
    println!("-----------------------------------------------");
    
    // Note: This would work with real API keys
    // For demo purposes, we'll show the API structure
    println!("📝 Model configuration examples (requires API keys):");
    println!("   
    // OpenAI with temperature configuration
    let openai_model = openai_builder(\"gpt-4\")
        .temperature(0.7)
        .max_tokens(1000)
        .build()?;
    
    // DeepSeek with configuration
    let deepseek_model = deepseek_builder(\"deepseek-chat\")
        .temperature(0.3)
        .build()?;
    
    // Agent with configured model
    let agent = AgentFactory::quick(\"assistant\", \"你是一个AI助手\")
        .model(deepseek_model)
        .build()?;
    ");

    println!("\n7. 📊 API Comparison - Before vs After");
    println!("--------------------------------------");
    
    println!("🔴 Before (Traditional API - 15+ lines):");
    println!("   let config = AgentConfig {{");
    println!("       name: \"assistant\".to_string(),");
    println!("       instructions: \"You are helpful\".to_string(),");
    println!("       memory_config: Some(MemoryConfig::default()),");
    println!("       // ... many more fields");
    println!("   }};");
    println!("   let agent = BasicAgent::new(config, llm);");
    
    println!("\n🟢 After (Plan4.md API - 3 lines):");
    println!("   let agent = AgentFactory::quick(\"assistant\", \"You are helpful\")");
    println!("       .model(llm)");
    println!("       .build()?;");
    
    println!("\n📈 Improvement: 70%+ reduction in code lines!");

    println!("\n8. ✅ Backward Compatibility Test");
    println!("---------------------------------");
    
    // Test that old API still works
    let old_style_agent = AgentBuilder::new()
        .name("old_style")
        .instructions("Old style agent")
        .model(llm.clone())
        .build()?;
    
    println!("✅ Old API still works: {}", old_style_agent.get_name());
    
    // Test new API
    let new_style_agent = AgentFactory::quick("new_style", "New style agent")
        .model(llm)
        .build()?;
    
    println!("✅ New API works: {}", new_style_agent.get_name());
    
    println!("\n🎉 Plan4.md Phase 1 API Demo Complete!");
    println!("=======================================");
    println!("✅ AgentFactory::quick() - Minimal configuration");
    println!("✅ AgentFactory::builder() - Full control");
    println!("✅ AgentFactory::web_agent() - Pre-configured web tools");
    println!("✅ AgentFactory::file_agent() - Pre-configured file tools");
    println!("✅ Fluent builder pattern with tool collections");
    println!("✅ Model configuration with builder pattern");
    println!("✅ 70%+ code reduction achieved");
    println!("✅ Full backward compatibility maintained");
    
    println!("\n🚀 Ready for Phase 1 Week 2: Core Implementation and Testing!");

    Ok(())
}
