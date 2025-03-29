use lomusai_core::Result;

// Simple demonstration of the lumos_macro
#[tokio::main]
async fn main() -> Result<()> {
    println!("Lomusai Macro Usage Example");
    
    // Check if lumos_macro is available from lomusai_core
    let has_macros = cfg!(feature = "macros");
    println!("Macros feature is {}", if has_macros { "enabled" } else { "disabled" });
    
    if has_macros {
        println!("Macros are enabled!");
        
        // In a real application, you would implement and use the macros here
        println!("Examples of what you can do with lumos_macro:");
        println!("1. Define tools with #[lumos_tool]");
        println!("2. Create agents with #[lumos_agent]");
        println!("3. Implement LLM adapters with #[lumos_llm_adapter]");
        
        // Example of tool definition syntax
        println!("\nExample tool definition:");
        println!(r##"
#[tool(
    name = "calculator",
    description = "Performs basic math operations"
)]
fn calculator(
    #[parameter(
        name = "operation",
        description = "The operation to perform: add, subtract, multiply, divide",
        r#type = "string", 
        required = true
    )]
    operation: String,
    
    #[parameter(
        name = "a",
        description = "First number",
        r#type = "number",
        required = true
    )]
    a: f64,
    
    #[parameter(
        name = "b",
        description = "Second number",
        r#type = "number",
        required = true
    )]
    b: f64,
) -> Result<serde_json::Value, lomusai_core::Error> {{
    let result = match operation.as_str() {{
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {{
            if b == 0.0 {{
                return Err(lomusai_core::Error::InvalidInput("Cannot divide by zero".to_string()));
            }}
            a / b
        }},
        _ => return Err(lomusai_core::Error::InvalidInput(format!("Unknown operation: {{}}", operation))),
    }};
    
    Ok(serde_json::json!({{ "result": result }}))
}}
        "##);
        
        // Example of agent definition syntax
        println!("\nExample agent definition:");
        println!(r##"
#[agent(
    name = "helper_agent",
    instructions = "You are a helpful assistant that can perform calculations and check the weather.",
    model = "gpt-4"
)]
struct HelperAgent {{
    #[tool]
    calculator: calculator,
    
    #[tool]
    weather: weather,
}}
        "##);
        
        // Example of LLM adapter implementation
        println!("\nExample LLM adapter implementation:");
        println!(r##"
#[derive(LlmAdapter)]
struct MockLlmAdapter {{
    responses: Vec<String>,
    current_response: std::sync::Mutex<usize>,
}}

impl MockLlmAdapter {{
    fn new(responses: Vec<String>) -> Self {{
        Self {{
            responses,
            current_response: std::sync::Mutex::new(0),
        }}
    }}
    
    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> lomusai_core::Result<String> {{
        // Implementation details would go here
        Ok("Response from MockLlmAdapter".to_string())
    }}
}}
        "##);
    } else {
        println!("The 'macros' feature is not enabled.");
        println!("Enable it by adding the 'macros' feature to your Cargo.toml:");
        println!(r##"
[dependencies]
lomusai_core = {{ version = "0.1.0", features = ["macros"] }}
        "##);
    }
    
    Ok(())
} 