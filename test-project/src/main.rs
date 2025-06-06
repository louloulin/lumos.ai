//! test-project - A Lumos.ai Agent
//! 
//! This is a basic Lumos.ai agent project.

use lumosai::{agent, tools, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a simple agent
    let agent = agent!{
        name: "test-project",
        instructions: "You are a helpful assistant.",
        model: "gpt-4",
        tools: []
    };

    // Example interaction
    let response = agent.generate("Hello! How can you help me?").await?;
    println!("Agent: {}", response);

    Ok(())
}
