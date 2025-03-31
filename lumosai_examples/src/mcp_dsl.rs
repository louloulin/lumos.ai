use lumosai_core::{Result, Error};
use lumosai_core::agent::{Agent, SimpleAgent};
use lumosai_core::llm::{LlmProvider, OpenAiAdapter};
use lumosai_mcp::{MCPConfiguration, ServerDefinition};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Create an LLM adapter for the agent
    let llm = Arc::new(OpenAiAdapter::new(
        "your-api-key",  // Replace with your actual API key
        "gpt-4",         // Or another suitable model
    ));
    
    // Define server configurations
    let mut servers = HashMap::new();
    
    // Stock price MCP server (using stdio)
    servers.insert(
        "stockPrice".to_string(),
        ServerDefinition::Stdio {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "tsx".to_string(), "./tools/stock-price.ts".to_string()],
            env: Some(HashMap::from([
                ("FAKE_CREDS".to_string(), "let me in!".to_string()),
            ])),
        },
    );
    
    // Weather MCP server (using SSE)
    servers.insert(
        "weather".to_string(),
        ServerDefinition::SSE {
            url: "http://localhost:8080/sse".to_string(),
            request_init: None,
        },
    );
    
    println!("Creating MCP configuration...");
    
    // Create the MCP configuration
    let mcp = MCPConfiguration::new(servers, Some("example_config".to_string()));
    
    println!("Creating agent...");
    
    // Create an agent that can use the tools
    let mut agent = SimpleAgent::new(
        "stock_weather_agent",
        "You are a helpful assistant that provides current stock prices and weather information. When asked about a stock, use the stockPrice_getStockPrice tool. When asked about weather, use the weather_getWeather tool.",
        llm,
    );
    
    println!("Getting MCP tools...");
    
    // Add MCP tools to the agent
    match mcp.get_tools().await {
        Ok(tools) => {
            for (name, tool) in tools {
                println!("Adding tool: {}", name);
                agent.add_tool(tool);
            }
        }
        Err(e) => {
            println!("Error getting tools: {:?}", e);
            return Err(Error::Other(format!("Failed to get tools: {:?}", e)));
        }
    }
    
    println!("Running agent...");
    
    // Run the agent with a query
    let response = agent.run("What is the current stock price of Apple (AAPL) and what is the weather in Seattle?").await?;
    
    println!("Agent response: {}", response);
    
    // Disconnect from the MCP servers
    mcp.disconnect().await.map_err(|e| Error::Other(format!("Failed to disconnect: {:?}", e)))?;
    
    Ok(())
} 