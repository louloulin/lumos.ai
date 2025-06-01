use lumosai_core::{Result, Error};
use lumosai_core::agent::{Agent, BasicAgent, create_basic_agent};
use lumosai_core::llm::{LlmProvider, MockLlmProvider};
use lumosai_mcp::{MCPConfiguration, ServerDefinition};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a mock LLM provider for demonstration
    let mock_responses = vec![
        "I'll help you get the stock price and weather information.".to_string(),
        "Based on the tools available, I can provide that information.".to_string(),
    ];
    let llm = Arc::new(MockLlmProvider::new(mock_responses));
    
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
    let mut agent = create_basic_agent(
        "stock_weather_agent".to_string(),
        "You are a helpful assistant that provides current stock prices and weather information. When asked about a stock, use the stockPrice_getStockPrice tool. When asked about weather, use the weather_getWeather tool.".to_string(),
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
    use lumosai_core::llm::{Message, Role};
    use lumosai_core::agent::AgentGenerateOptions;

    let message = Message {
        role: Role::User,
        content: "What is the current stock price of Apple (AAPL) and what is the weather in Seattle?".to_string(),
        metadata: None,
        name: None,
    };

    let options = AgentGenerateOptions::default();
    let response = agent.generate(&[message], &options).await?;
    
    println!("Agent response: {}", response.response);
    
    // Disconnect from the MCP servers
    mcp.disconnect().await.map_err(|e| Error::Other(format!("Failed to disconnect: {:?}", e)))?;
    
    Ok(())
} 