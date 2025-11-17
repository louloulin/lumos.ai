//! Agent Card Builder Example
//!
//! 演示如何使用 A2A Agent Card Builder

use lumosai_a2a::{AgentCardBuilder, Skill, AuthScheme, Authentication};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建一个复杂的 Agent Card
    let card = AgentCardBuilder::new(
        "LumosAI Assistant".to_string(),
        url::Url::parse("https://lumos-ai.com/assistant")?,
        "1.0.0".to_string(),
    )
    .description("An intelligent AI assistant powered by LumosAI framework".to_string())
    .enable_streaming(true)
    .enable_push_notifications(true)
    .max_concurrent_tasks(10)
    .input_modes(vec!["text".to_string(), "file".to_string(), "json".to_string()])
    .output_modes(vec!["text".to_string(), "audio".to_string(), "image".to_string()])
    .authentication(Authentication {
        schemes: vec![AuthScheme::Bearer, AuthScheme::ApiKey],
        credentials: None,
    })
    .skill(
        Skill::new("text_analysis".to_string(), "Text Analysis".to_string())
            .with_description("Analyze text content for sentiment, entities, and themes".to_string())
            .with_tags(vec!["nlp".to_string(), "analysis".to_string()])
            .with_examples(vec![
                "Analyze the sentiment of this review".to_string(),
                "Extract key entities from this document".to_string(),
            ])
    )
    .skill(
        Skill::new("code_generation".to_string(), "Code Generation".to_string())
            .with_description("Generate code in multiple programming languages".to_string())
            .with_tags(vec!["programming".to_string(), "development".to_string()])
            .with_examples(vec![
                "Write a Python function to sort a list".to_string(),
                "Create a REST API endpoint in Rust".to_string(),
            ])
    )
    .skill(
        Skill::new("image_processing".to_string(), "Image Processing".to_string())
            .with_description("Process and analyze images using computer vision".to_string())
            .with_tags(vec!["cv".to_string(), "image".to_string()])
            .with_examples(vec![
                "Detect objects in this image".to_string(),
                "Apply filters to this photo".to_string(),
            ])
    )
    .metadata("author".to_string(), serde_json::json!("LumosAI Team"))
    .metadata("category".to_string(), serde_json::json!("assistant"))
    .metadata("language".to_string(), serde_json::json!(["en", "zh"]))
    .build()?;

    // 打印 Agent Card 信息
    println!("✅ Agent Card Created Successfully!");
    println!("Name: {}", card.name);
    println!("Description: {:?}", card.description);
    println!("URL: {}", card.url);
    println!("Version: {}", card.version);
    
    println!("\n🚀 Capabilities:");
    println!("- Streaming: {}", card.capabilities.streaming);
    println!("- Push Notifications: {}", card.capabilities.push_notifications);
    println!("- State Transition History: {}", card.capabilities.state_transition_history);
    println!("- Max Concurrent Tasks: {:?}", card.capabilities.max_concurrent_tasks);
    
    println!("\n🔧 Input Modes: {:?}", card.capabilities.input_modes);
    println!("📤 Output Modes: {:?}", card.capabilities.output_modes);
    
    println!("\n🛠️ Skills ({}):", card.skills.len());
    for (i, skill) in card.skills.iter().enumerate() {
        println!("  {}. {}", i + 1, skill.name);
        if let Some(description) = &skill.description {
            println!("     Description: {}", description);
        }
        if let Some(tags) = &skill.tags {
            println!("     Tags: {}", tags.join(", "));
        }
    }
    
    println!("\n🔐 Authentication:");
    if let Some(auth) = &card.authentication {
        println!("  Schemes: {:?}", auth.schemes);
    }
    
    println!("\n📋 Metadata:");
    for (key, value) in &card.metadata {
        println!("  {}: {}", key, value);
    }

    // 验证 Agent Card
    card.validate()?;
    println!("\n✅ Agent Card validation passed!");

    // 生成 Agent ID
    let agent_id = card.generate_id();
    println!("Agent ID: {}", agent_id);

    // 检查能力支持
    println!("\n🔍 Capability Checks:");
    println!("- Supports streaming: {}", card.supports_capability("streaming"));
    println!("- Supports text_analysis skill: {}", card.supports_skill("text_analysis"));
    println!("- Supports unknown_skill: {}", card.supports_skill("unknown_skill"));

    Ok(())
}