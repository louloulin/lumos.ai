//! Function calling utilities for Lumosai
//! 
//! This module provides advanced utilities for working with OpenAI's function calling feature,
//! improving tool schema generation, validation, and schema inference.

use serde_json::{Value, Map, json};
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::tool::Tool;
use crate::llm::function_calling::{FunctionDefinition, FunctionCall};

/// Convert a Rust struct to JSON Schema
/// This can be used with the #[derive(FunctionSchema)] macro
pub fn struct_to_json_schema<T>() -> Value where T: schemars::JsonSchema {
    let schema = schemars::schema_for!(T);
    serde_json::to_value(&schema.schema).unwrap_or_else(|_| json!({}))
}

/// Convert a collection of tools to function definitions
pub fn tools_to_function_definitions(tools: &HashMap<String, Box<dyn Tool>>) -> Vec<FunctionDefinition> {
    tools.values()
        .map(|tool| FunctionDefinition::from_tool(tool.as_ref()))
        .collect()
}

/// Convert tool definitions to OpenAI tool format with enhanced features
pub fn function_definitions_to_openai_tools(functions: &[FunctionDefinition]) -> Value {
    let tools: Vec<Value> = functions.iter().map(|func| {
        let mut function_def = Map::new();
        function_def.insert("name".to_string(), json!(func.name));
        
        if let Some(desc) = &func.description {
            function_def.insert("description".to_string(), json!(desc));
        }
        
        // Ensure parameters has type:object for compatibility
        let mut parameters = if let Some(obj) = func.parameters.as_object() {
            obj.clone()
        } else {
            let mut map = Map::new();
            map.insert("type".to_string(), json!("object"));
            map.insert("properties".to_string(), func.parameters.clone());
            map
        };

        // Add default properties if not present
        if !parameters.contains_key("type") {
            parameters.insert("type".to_string(), json!("object"));
        }
        if !parameters.contains_key("required") {
            parameters.insert("required".to_string(), json!([]));
        }

        function_def.insert("parameters".to_string(), Value::Object(parameters));

        json!({
            "type": "function",
            "function": function_def
        })
    }).collect();
    
    Value::Array(tools)
}

/// Parse OpenAI function calls from response
pub fn parse_openai_function_calls(response: &Value) -> Result<Vec<FunctionCall>> {
    let mut function_calls = Vec::new();
    
    // Handle different OpenAI response formats
    if let Some(choices) = response.get("choices").and_then(|c| c.as_array()) {
        if let Some(choice) = choices.first() {
            // Handle message.tool_calls format (newer API)
            if let Some(message) = choice.get("message") {
                if let Some(tool_calls) = message.get("tool_calls").and_then(|tc| tc.as_array()) {
                    for tool_call in tool_calls {
                        if let Some(function) = tool_call.get("function") {
                            let name = function.get("name")
                                .and_then(|n| n.as_str())
                                .ok_or_else(|| Error::InvalidInput("Missing function name".to_string()))?;
                            
                            let arguments = function.get("arguments")
                                .and_then(|a| a.as_str())
                                .unwrap_or("{}");
                            
                            let id = tool_call.get("id")
                                .and_then(|i| i.as_str())
                                .map(|s| s.to_string());
                            
                            function_calls.push(FunctionCall {
                                id,
                                name: name.to_string(),
                                arguments: arguments.to_string(),
                            });
                        }
                    }
                }
                
                // Handle message.function_call format (legacy API)
                if function_calls.is_empty() {
                    if let Some(function_call) = message.get("function_call") {
                        let name = function_call.get("name")
                            .and_then(|n| n.as_str())
                            .ok_or_else(|| Error::InvalidInput("Missing function name".to_string()))?;
                        
                        let arguments = function_call.get("arguments")
                            .and_then(|a| a.as_str())
                            .unwrap_or("{}");
                        
                        function_calls.push(FunctionCall {
                            id: None,
                            name: name.to_string(),
                            arguments: arguments.to_string(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(function_calls)
}

/// Validate a JSON value against a JSON schema
pub fn validate_against_schema(value: &Value, schema: &Value) -> Result<()> {
    use jsonschema::JSONSchema;

    // Check if schema is null (means no schema validation needed)
    if schema.is_null() {
        return Ok(());
    }

    // Compile schema using the simple compile method
    let compiled_schema = JSONSchema::compile(schema)
        .map_err(|e| Error::InvalidInput(format!("Invalid schema: {}", e)))?;

    // Validate with detailed error reporting
    if let Err(errors) = compiled_schema.validate(value) {
        let error_details = errors.map(|e| {
            format!("{} at path: {}", e.to_string(), e.instance_path)
        }).collect::<Vec<_>>().join(", ");
        return Err(Error::InvalidInput(format!("Schema validation failed: {}", error_details)));
    }

    Ok(())
}

/// Determine if tool calling should be enabled based on the available tools and LLM capabilities
pub fn should_use_function_calling(
    function_calling_enabled: bool,
    llm_supports_function_calling: bool,
    tools_available: bool
) -> bool {
    function_calling_enabled && llm_supports_function_calling && tools_available
}

/// Generate an appropriate system prompt based on whether function calling is enabled
pub fn generate_system_prompt(
    base_instructions: &str,
    use_function_calling: bool,
    tools_description: Option<&str>
) -> String {
    if use_function_calling {
        // For function calling mode, we don't need to explain tool format in system prompt
        format!("{}\n\nYou have access to specialized tools. Use them when needed.", base_instructions)
    } else if let Some(tools_desc) = tools_description {
        // For non-function-calling mode, explain the expected tool format
        format!(
            "{}\n\nYou have access to the following tools:\n{}\n\n\
             To use a tool, use exactly the following format:\n\
             Using the tool 'tool_name' with parameters: {{\"param1\": \"value1\", \"param2\": value2}}",
            base_instructions, tools_desc
        )
    } else {
        base_instructions.to_string()
    }
}

/// Format options for tool descriptions
#[derive(Debug, Clone)]
pub struct ToolDescriptionFormat {
    pub include_parameters: bool,
    pub include_examples: bool,
    pub markdown_formatting: bool,
    pub group_by_category: bool,
}

impl Default for ToolDescriptionFormat {
    fn default() -> Self {
        Self {
            include_parameters: true,
            include_examples: true,
            markdown_formatting: false,
            group_by_category: false,
        }
    }
}

/// Create a human-readable description of available tools with formatting options
pub fn create_tools_description(
    tools: &HashMap<String, Box<dyn Tool>>,
    format: Option<&ToolDescriptionFormat>
) -> String {
    let default_format = ToolDescriptionFormat::default();
    let format = format.unwrap_or(&default_format);
    let mut descriptions = Vec::new();
    
    // Group tools by category if needed
    if format.group_by_category {
        let mut categories: HashMap<String, Vec<&Box<dyn Tool>>> = HashMap::new();
        for tool in tools.values() {
            let category = tool.category().unwrap_or_else(|| "Other".to_string());
            categories.entry(category).or_default().push(tool);
        }
        
        for (category, cat_tools) in categories.iter() {
            if format.markdown_formatting {
                descriptions.push(format!("### {}", category));
            } else {
                descriptions.push(format!("Category: {}", category));
            }
            
            for tool in cat_tools {
                descriptions.push(format_tool_description(tool, format));
            }
            descriptions.push(String::new()); // Add spacing between categories
        }
    } else {
        for tool in tools.values() {
            descriptions.push(format_tool_description(tool, format));
        }
    }
    
    descriptions.join("\n")
}

fn format_tool_description(tool: &Box<dyn Tool>, format: &ToolDescriptionFormat) -> String {
    let mut desc = if format.markdown_formatting {
        format!("#### {}\n{}", tool.id(), tool.description())
    } else {
        format!("- {}: {}", tool.id(), tool.description())
    };
    
    // Add parameters info if requested
    if format.include_parameters {
        let schema = tool.schema();
        if !schema.parameters.is_empty() {
            desc.push_str("\nParameters:");
            for param in &schema.parameters {
                let param_desc = if format.markdown_formatting {
                    format!(
                        "\n- `{}` ({}{}): {}",
                        param.name,
                        param.r#type,
                        if param.required { ", required" } else { "" },
                        param.description
                    )
                } else {
                    format!(
                        "\n  - {} ({}{}): {}",
                        param.name,
                        param.r#type,
                        if param.required { ", required" } else { "" },
                        param.description
                    )
                };
                desc.push_str(&param_desc);
            }
        }
    }
    
    // Add examples if available and requested
    if format.include_examples {
        if let Some(examples) = tool.examples() {
            desc.push_str("\nExamples:");
            for example in examples {
                if format.markdown_formatting {
                    desc.push_str(&format!("\n```json\n{}\n```", example));
                } else {
                    desc.push_str(&format!("\n  {}", example));
                }
            }
        }
    }
    
    desc
}
#[cfg(test)]
mod tests {
    use super::*;
    // use crate::llm::function_calling::FunctionDefinition; // 暂时未使用
    use serde_json::json;
    
    #[test]
    fn test_validate_schema_basic_types() {
        // String validation
        let schema = json!({"type": "string"});
        assert!(validate_against_schema(&json!("test"), &schema).is_ok());
        assert!(validate_against_schema(&json!(42), &schema).is_err());
        
        // Integer validation
        let schema = json!({"type": "integer"});
        assert!(validate_against_schema(&json!(42), &schema).is_ok());
        assert!(validate_against_schema(&json!("test"), &schema).is_err());
        
        // Object with required fields
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            },
            "required": ["name"]
        });
        
        assert!(validate_against_schema(&json!({"name": "John", "age": 30}), &schema).is_ok());
        assert!(validate_against_schema(&json!({"name": "John"}), &schema).is_ok());
        assert!(validate_against_schema(&json!({"age": 30}), &schema).is_err());
    }
    
    #[test]
    fn test_should_use_function_calling() {
        assert!(should_use_function_calling(true, true, true));
        assert!(!should_use_function_calling(false, true, true));
        assert!(!should_use_function_calling(true, false, true));
        assert!(!should_use_function_calling(true, true, false));
    }
    
    #[test]
    fn test_generate_system_prompt() {
        let base = "You are a helpful assistant";
        let tools_desc = "Tool1: Does something\nTool2: Does something else";
        
        // Function calling enabled
        let prompt_fc = generate_system_prompt(base, true, Some(tools_desc));
        assert!(prompt_fc.contains("You have access to specialized tools"));
        assert!(!prompt_fc.contains("To use a tool, use exactly the following format"));
        
        // Function calling disabled
        let prompt_no_fc = generate_system_prompt(base, false, Some(tools_desc));
        assert!(prompt_no_fc.contains("To use a tool, use exactly the following format"));
        assert!(prompt_no_fc.contains(tools_desc));
    }
}
