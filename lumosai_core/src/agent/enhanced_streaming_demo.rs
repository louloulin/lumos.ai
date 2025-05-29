//! Enhanced Streaming Processing Demo
//!
//! This example demonstrates the enhanced streaming capabilities
//! including true function calling support and event-driven processing.

use std::sync::Arc;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

use lumosai_core::agent::{
    AgentConfig, BasicAgent, AgentGenerateOptions,
    StreamingConfig, IntoStreaming, AgentEvent
};
use lumosai_core::llm::{MockLlmProvider, LlmOptions};
use lumosai_core::memory::WorkingMemoryConfig;
use lumosai_core::agent::message_utils::user_message;
use lumosai_core::tools::{Tool, ToolConfig, ToolsConfig, ParameterDefinition, ParameterType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Enhanced Streaming Processing Demo Starting...");
    
    // Configure enhanced streaming
    let streaming_config = StreamingConfig {
        text_buffer_size: 10, // Stream in 10-character chunks
        emit_metadata: true,
        emit_memory_updates: true,
        text_delta_delay_ms: Some(50), // 50ms delay for realistic streaming
    };
    
    // Create working memory config
    let wm_config = WorkingMemoryConfig {
        enabled: true,
        template: Some("Current context: {}".to_string()),
        content_type: Some("application/json".to_string()),
        max_capacity: Some(2048),
    };
    
    // Create enhanced agent config with tools
    let mut agent_config = AgentConfig {
        name: "enhanced_streaming_agent".to_string(),
        instructions: r#"You are an advanced AI assistant with enhanced streaming capabilities. 
        You have access to tools for weather information and calculations. 
        When users ask about weather or math, use the appropriate tools and stream your responses in real-time."#.to_string(),
        working_memory: Some(wm_config),
        ..Default::default()
    };
    
    // Add weather tool
    let weather_tool = Tool {
        config: ToolConfig {
            name: "get_weather".to_string(),
            description: "Get current weather information for a location".to_string(),
            parameters: vec![
                ParameterDefinition {
                    name: "location".to_string(),
                    param_type: ParameterType::String,
                    description: "The city and country (e.g., 'New York, USA')".to_string(),
                    required: true,
                    ..Default::default()
                }
            ],
            ..Default::default()
        },
        implementation: Arc::new(|args| {
            Box::pin(async move {
                let location = args.get("location")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                
                Ok(format!("Weather in {}: 22°C, Sunny with light clouds. Humidity: 65%, Wind: 8 km/h SW", location))
            })
        }),
    };
    
    // Add calculator tool
    let calculator_tool = Tool {
        config: ToolConfig {
            name: "calculate".to_string(),
            description: "Perform mathematical calculations".to_string(),
            parameters: vec![
                ParameterDefinition {
                    name: "expression".to_string(),
                    param_type: ParameterType::String,
                    description: "Mathematical expression to evaluate (e.g., '2 + 2', '10 * 5')".to_string(),
                    required: true,
                    ..Default::default()
                }
            ],
            ..Default::default()
        },
        implementation: Arc::new(|args| {
            Box::pin(async move {
                let expression = args.get("expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");
                
                // Simple calculation logic (in real implementation, use a proper parser)
                let result = match expression {
                    "2 + 2" => "4",
                    "10 * 5" => "50",
                    "100 / 10" => "10",
                    "15 - 7" => "8",
                    _ => "42", // Default answer to everything
                };
                
                Ok(format!("The result of {} is: {}", expression, result))
            })
        }),
    };
    
    agent_config.tools_config = Some(ToolsConfig {
        tools: vec![weather_tool, calculator_tool],
        auto_select: true,
    });
    
    // Create mock LLM with function calling support
    let mock_responses = vec![
        "I'll help you with that! Let me check the weather information and perform some calculations. ".to_string(),
        "The weather data shows it's a beautiful day with pleasant temperatures. ".to_string(),
        "I can also help with any mathematical calculations you might need. ".to_string(),
        "This streaming response demonstrates real-time processing with tool integration!".to_string(),
    ];
    
    let llm = Arc::new(MockLlmProvider::new_with_function_calling(mock_responses, true));
    let agent = BasicAgent::new(agent_config, llm);
    
    // Create streaming agent
    let streaming_agent = agent.into_streaming_with_config(streaming_config);
    
    println!("✅ Enhanced Streaming Agent Created");
    println!("🔧 Tools configured: weather and calculator");
    println!("📡 Function calling: enabled");
    
    // Test scenarios
    let test_scenarios = vec![
        ("Weather Query", "What's the weather like in Tokyo, Japan?"),
        ("Math Query", "Can you calculate 2 + 2 for me?"),
        ("Mixed Query", "Show me the weather in Paris and calculate 10 * 5"),
        ("General Query", "Tell me about your streaming capabilities"),
    ];
    
    for (scenario_name, query) in test_scenarios {
        println!("\n" + "=".repeat(60).as_str());
        println!("🎯 Scenario: {}", scenario_name);
        println!("❓ Query: {}", query);
        println!("=".repeat(60));
        
        let messages = vec![user_message(query)];
        let options = AgentGenerateOptions {
            llm_options: LlmOptions::default(),
            max_steps: Some(5),
            tools_config: None,
        };
        
        let mut stream = streaming_agent.execute_streaming(&messages, &options);
        
        println!("\n🔥 Streaming Response:");
        let mut response_text = String::new();
        let mut step_count = 0;
        let mut tool_calls = 0;
        
        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => {
                    match &event {
                        AgentEvent::TextDelta { delta, step_id } => {
                            print!("{}", delta);
                            response_text.push_str(delta);
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                        },
                        AgentEvent::ToolCallStart { tool_call, step_id } => {
                            tool_calls += 1;
                            println!("\n🔧 Tool Call Started: {} ({})", tool_call.name, tool_call.id);
                            println!("   Arguments: {}", tool_call.arguments);
                            println!("   Step ID: {}", step_id);
                        },
                        AgentEvent::ToolCallComplete { tool_result, step_id } => {
                            println!("\n✅ Tool Call Complete: {}", tool_result.call_id);
                            println!("   Output: {}", tool_result.output);
                            println!("   Error: {}", tool_result.is_error);
                            println!("   Step ID: {}", step_id);
                        },
                        AgentEvent::StepComplete { step, step_id } => {
                            step_count += 1;
                            println!("\n📍 Step {} Complete ({})", step_count, step_id);
                        },
                        AgentEvent::GenerationComplete { final_response, total_steps } => {
                            println!("\n\n🎉 Generation Complete!");
                            println!("   Total Steps: {}", total_steps);
                            println!("   Final Response Length: {} characters", final_response.len());
                        },
                        AgentEvent::MemoryUpdate { key, operation } => {
                            println!("\n🧠 Memory Update: {} = {:?}", key, operation);
                        },
                        AgentEvent::Metadata { key, value } => {
                            println!("\n📋 Metadata: {} = {:?}", key, value);
                        },
                        AgentEvent::Error { error, step_id } => {
                            println!("\n❌ Error in step {:?}: {}", step_id, error);
                        }
                    }
                },
                Err(e) => {
                    println!("\n💥 Stream Error: {}", e);
                    break;
                }
            }
        }
        
        println!("\n\n📊 Scenario Summary:");
        println!("   Response Length: {} characters", response_text.len());
        println!("   Steps Completed: {}", step_count);
        println!("   Tool Calls Made: {}", tool_calls);
        
        // Brief pause between scenarios
        sleep(Duration::from_millis(1000)).await;
    }
    
    println!("\n" + "=".repeat(80).as_str());
    println!("🎉 Enhanced Streaming Processing Demo Complete!");
    println!("=".repeat(80));
    
    println!("\n📝 Features Demonstrated:");
    println!("   ✅ True streaming with text deltas");
    println!("   ✅ Function calling integration");
    println!("   ✅ Tool execution with real-time events");
    println!("   ✅ Step-by-step processing");
    println!("   ✅ Memory operations");
    println!("   ✅ Metadata emission");
    println!("   ✅ Error handling");
    println!("   ✅ Event-driven architecture");
    
    println!("\n🔮 Next Steps:");
    println!("   - Integrate with WebSocket for real-time web apps");
    println!("   - Add more sophisticated tool parsing");
    println!("   - Implement multi-turn conversation streaming");
    println!("   - Add streaming visualization tools");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enhanced_streaming_agent() {
        let streaming_config = StreamingConfig {
            text_buffer_size: 1,
            emit_metadata: false,
            emit_memory_updates: false,
            text_delta_delay_ms: None,
        };
        
        let wm_config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };
        
        let agent_config = AgentConfig {
            name: "test_agent".to_string(),
            instructions: "Test agent".to_string(),
            working_memory: Some(wm_config),
            ..Default::default()
        };
        
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        let agent = BasicAgent::new(agent_config, llm);
        
        let streaming_agent = agent.into_streaming_with_config(streaming_config);
        
        let messages = vec![user_message("Test message")];
        let options = AgentGenerateOptions::default();
        
        let mut stream = streaming_agent.execute_streaming(&messages, &options);
        let mut event_count = 0;
        
        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(_) => event_count += 1,
                Err(_) => break,
            }
            
            // Limit test to prevent infinite loop
            if event_count > 100 {
                break;
            }
        }
        
        assert!(event_count > 0, "Should have received at least one event");
    }
}
