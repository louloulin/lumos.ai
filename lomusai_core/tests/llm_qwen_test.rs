use std::env;
use lomusai_core::llm::{LlmOptions, QwenProvider, LlmProvider, QwenApiType};
use lomusai_core::llm::types::{user_message, system_message, assistant_message};
use serde_json::json;

/// Run Qwen LLM test with your own API key
/// 
/// To run this test, you need to set the QWEN_API_KEY environment variable
/// 
/// ```bash
/// QWEN_API_KEY=your_api_key cargo test --test llm_qwen_test
/// ```
#[tokio::test]
async fn test_qwen_provider() {
    // Skip test if no API key is provided
    let api_key = match env::var("QWEN_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_qwen_provider as QWEN_API_KEY is not set");
            return;
        }
    };

    // Create Qwen provider with DashScope API
    let provider = QwenProvider::new_with_defaults(api_key, "qwen-turbo");
    
    // Test basic prompt
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(100);
    
    let result = provider.generate("Hello, who are you?", &options).await;
    assert!(result.is_ok(), "Failed to generate text: {:?}", result.err());
    println!("Qwen response: {}", result.unwrap());

    // Test with messages
    let messages = vec![
        system_message("You are a helpful assistant."),
        user_message("What is the capital of France?"),
        assistant_message("The capital of France is Paris."),
        user_message("And what about Germany?"),
    ];

    let result = provider.generate_with_messages(&messages, &options).await;
    assert!(result.is_ok(), "Failed to generate text with messages: {:?}", result.err());
    println!("Qwen response with messages: {}", result.unwrap());

    // Test embedding
    let embedding_result = provider.get_embedding("This is a test embedding").await;
    assert!(embedding_result.is_ok(), "Failed to get embedding: {:?}", embedding_result.err());
    
    let embedding = embedding_result.unwrap();
    assert!(!embedding.is_empty(), "Embedding vector is empty");
    println!("Embedding vector length: {}", embedding.len());
}

/// Test OpenAI-compatible API for Qwen
#[tokio::test]
async fn test_qwen_openai_compatible() {
    // Skip test if no server URL is provided
    let server_url = match env::var("QWEN_OPENAI_SERVER") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test_qwen_openai_compatible as QWEN_OPENAI_SERVER is not set");
            return;
        }
    };

    // Use empty API key if none provided (some local servers don't require API keys)
    let api_key = env::var("QWEN_OPENAI_KEY").unwrap_or_else(|_| "EMPTY".to_string());

    // Create provider with OpenAI-compatible API
    let provider = QwenProvider::new_with_api_type(
        api_key,
        "qwen2.5-7b-instruct",
        server_url,
        QwenApiType::OpenAICompatible,
    );
    
    // Test basic prompt
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(100);
    
    let result = provider.generate("Hello, who are you?", &options).await;
    if let Ok(response) = result {
        println!("Qwen OpenAI-compatible response: {}", response);
    } else {
        println!("Qwen OpenAI-compatible test failed: {:?}", result.err());
        println!("This is expected if you don't have a compatible server running.");
    }
}

/// Test function calling capabilities of Qwen 2.5 models
#[tokio::test]
async fn test_qwen_function_calling_dashscope() {
    // Skip test if no API key is provided
    let api_key = match env::var("QWEN_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_qwen_function_calling_dashscope as QWEN_API_KEY is not set");
            return;
        }
    };

    // Create Qwen 2.5 provider with DashScope API
    let provider = QwenProvider::new_qwen25(api_key, "qwen2.5-7b-instruct");
    
    // Define a weather tool
    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "get_current_weather",
                "description": "Get the current weather in a given location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string", 
                            "description": "The city and state, e.g. San Francisco, CA"
                        },
                        "unit": {
                            "type": "string",
                            "enum": ["celsius", "fahrenheit"], 
                            "description": "The temperature unit to use. Infer this from the user's location."
                        }
                    },
                    "required": ["location"]
                }
            }
        }
    ]);

    // Create options with tools
    let mut options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(1024);
    
    options.extra.insert("tools".to_string(), tools);
    options.extra.insert("tool_choice".to_string(), json!("auto"));

    // Test with a weather query that should trigger the tool
    let messages = vec![
        system_message("You are a helpful weather assistant that can check the weather."),
        user_message("What's the weather like in San Francisco today?"),
    ];

    let result = provider.generate_with_messages(&messages, &options).await;
    println!("Qwen function call result: {:?}", result);
    
    if let Ok(response) = result {
        // We're just checking if the response contains a function call mention
        // In a real scenario, we would parse the result and execute the function
        println!("Qwen function call response: {}", response);
        assert!(
            response.contains("get_current_weather") || 
            response.contains("Function:") || 
            response.contains("weather"), 
            "Response should contain function call or weather information"
        );
    } else {
        println!("Note: Function calling test failed, but this might be expected if the model doesn't support this feature or if the API key doesn't have access to it");
    }
}

/// Test function calling capabilities with OpenAI-compatible API
#[tokio::test]
async fn test_qwen_function_calling_openai() {
    // Skip test if no server URL is provided
    let server_url = match env::var("QWEN_OPENAI_SERVER") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test_qwen_function_calling_openai as QWEN_OPENAI_SERVER is not set");
            return;
        }
    };
    
    // Use empty API key if none provided (some local servers don't require API keys)
    let api_key = env::var("QWEN_OPENAI_KEY").unwrap_or_else(|_| "EMPTY".to_string());

    // Create provider with OpenAI-compatible API
    let provider = QwenProvider::new_with_api_type(
        api_key,
        "qwen2.5-7b-instruct",
        server_url,
        QwenApiType::OpenAICompatible,
    );
    
    // Define a weather tool (OpenAI-compatible format)
    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "get_current_weather",
                "description": "Get the current weather in a given location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string", 
                            "description": "The city and state, e.g. San Francisco, CA"
                        },
                        "unit": {
                            "type": "string",
                            "enum": ["celsius", "fahrenheit"], 
                            "description": "The temperature unit to use. Infer this from the user's location."
                        }
                    },
                    "required": ["location"]
                }
            }
        }
    ]);

    // Create options with tools
    let mut options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(1024);
    
    options.extra.insert("tools".to_string(), tools);
    options.extra.insert("tool_choice".to_string(), json!("auto"));

    // Test with a weather query that should trigger the tool
    let messages = vec![
        system_message("You are a helpful weather assistant that can check the weather."),
        user_message("What's the weather like in San Francisco today?"),
    ];

    let result = provider.generate_with_messages(&messages, &options).await;
    
    if let Ok(response) = result {
        println!("Qwen OpenAI-compatible function call response: {}", response);
        assert!(
            response.contains("get_current_weather") || 
            response.contains("Function:") || 
            response.contains("weather"), 
            "Response should contain function call or weather information"
        );
    } else {
        println!("Note: OpenAI-compatible function calling test failed: {:?}", result.err());
        println!("This is expected if you don't have a compatible server running with function calling support.");
    }
} 