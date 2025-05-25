//! OpenAI Function Calling support for Lumosai
//! 
//! This module provides types and utilities for OpenAI's function calling feature,
//! allowing LLM providers to natively call functions instead of relying on 
//! regex-based parsing.

use serde::{Deserialize, Serialize};
use serde_json::{Value, Map};
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::tool::Tool;

/// Represents a function definition for OpenAI function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function
    pub name: String,
    /// A description of what the function does  
    pub description: Option<String>,
    /// JSON Schema defining the function's parameters
    pub parameters: Value,
}

impl FunctionDefinition {
    /// Create a new function definition
    pub fn new(name: String, description: Option<String>, parameters: Value) -> Self {
        Self {
            name,
            description,
            parameters,
        }
    }

    /// Create a function definition from a tool
    pub fn from_tool(tool: &dyn Tool) -> Self {
        let schema = tool.schema();
        
        // Convert tool schema to OpenAI function parameters format
        let mut properties = Map::new();
        let mut required = Vec::new();
        
        for param in &schema.parameters {
            let mut param_schema = Map::new();
            param_schema.insert("type".to_string(), Value::String(param.r#type.clone()));
            param_schema.insert("description".to_string(), Value::String(param.description.clone()));
            
            if let Some(default) = &param.default {
                param_schema.insert("default".to_string(), default.clone());
            }
            
            if param.required {
                required.push(param.name.clone());
            }
            
            properties.insert(param.name.clone(), Value::Object(param_schema));
        }
        
        let mut parameters = Map::new();
        parameters.insert("type".to_string(), Value::String("object".to_string()));
        parameters.insert("properties".to_string(), Value::Object(properties));
        parameters.insert("required".to_string(), Value::Array(
            required.into_iter().map(Value::String).collect()
        ));
        
        Self {
            name: tool.id().to_string(),
            description: Some(tool.description().to_string()),
            parameters: Value::Object(parameters),
        }
    }
}

/// Represents a function call made by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Unique identifier for this function call
    pub id: Option<String>,
    /// The name of the function to call
    pub name: String,
    /// The arguments to pass to the function (as JSON string)
    pub arguments: String,
}

impl FunctionCall {
    /// Create a new function call
    pub fn new(name: String, arguments: String) -> Self {
        Self {
            id: None,
            name,
            arguments,
        }
    }

    /// Create a new function call with an ID
    pub fn with_id(id: String, name: String, arguments: String) -> Self {
        Self {
            id: Some(id),
            name,
            arguments,
        }
    }

    /// Parse the arguments as JSON
    pub fn parse_arguments(&self) -> Result<Value> {
        serde_json::from_str(&self.arguments)
            .map_err(|e| Error::Json(e))
    }

    /// Parse the arguments into a HashMap
    pub fn parse_arguments_as_map(&self) -> Result<HashMap<String, Value>> {
        let value = self.parse_arguments()?;
        match value {
            Value::Object(map) => {
                Ok(map.into_iter().collect())
            },
            _ => Err(Error::InvalidInput("Function arguments must be a JSON object".to_string()))
        }
    }
}

/// Represents a tool choice for OpenAI function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoice {
    /// Let the model choose whether to call functions
    Auto,
    /// Force the model to not call any functions
    None,
    /// Force the model to call at least one function
    Required,
    /// Force the model to call a specific function
    Function { name: String },
}

impl Default for ToolChoice {
    fn default() -> Self {
        ToolChoice::Auto
    }
}

/// Represents the result of a function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallResult {
    /// The ID of the function call this result corresponds to
    pub call_id: Option<String>,
    /// The name of the function that was called
    pub name: String,
    /// The result of the function call
    pub result: Value,
    /// Whether the function call was successful
    pub success: bool,
    /// Error message if the function call failed
    pub error: Option<String>,
}

impl FunctionCallResult {
    /// Create a successful function call result
    pub fn success(call_id: Option<String>, name: String, result: Value) -> Self {
        Self {
            call_id,
            name,
            result,
            success: true,
            error: None,
        }
    }

    /// Create a failed function call result
    pub fn error(call_id: Option<String>, name: String, error: String) -> Self {
        Self {
            call_id,
            name,
            result: Value::Null,
            success: false,
            error: Some(error),
        }
    }
}

/// Utility functions for function calling
pub mod utils {
    use super::*;
    use crate::tool::Tool;
    use std::collections::HashMap;

    /// Convert a collection of tools to function definitions
    pub fn tools_to_function_definitions(tools: &HashMap<String, Box<dyn Tool>>) -> Vec<FunctionDefinition> {
        tools.values()
            .map(|tool| FunctionDefinition::from_tool(tool.as_ref()))
            .collect()
    }

    /// Create OpenAI tools format from function definitions
    pub fn function_definitions_to_openai_tools(functions: &[FunctionDefinition]) -> Value {
        let tools: Vec<Value> = functions.iter().map(|func| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": func.name,
                    "description": func.description,
                    "parameters": func.parameters
                }
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
                }
            }
        }
        
        Ok(function_calls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn test_function_definition_creation() {
        let func_def = FunctionDefinition::new(
            "test_function".to_string(),
            Some("A test function".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "param1": {
                        "type": "string",
                        "description": "First parameter"
                    },
                    "param2": {
                        "type": "number",
                        "description": "Second parameter"
                    }
                },
                "required": ["param1"]
            }),
        );

        assert_eq!(func_def.name, "test_function");
        assert_eq!(func_def.description, Some("A test function".to_string()));
        assert!(func_def.parameters.is_object());
    }

    #[test]
    fn test_function_call_parse_arguments() {
        let func_call = FunctionCall {
            id: Some("call_123".to_string()),
            name: "test_function".to_string(),
            arguments: r#"{"param1": "value1", "param2": 42}"#.to_string(),
        };

        let parsed = func_call.parse_arguments().unwrap();
        assert_eq!(parsed["param1"], Value::String("value1".to_string()));
        assert_eq!(parsed["param2"], Value::Number(serde_json::Number::from(42)));

        let parsed_map = func_call.parse_arguments_as_map().unwrap();
        assert_eq!(parsed_map.len(), 2);
        assert!(parsed_map.contains_key("param1"));
        assert!(parsed_map.contains_key("param2"));
    }

    #[test]
    fn test_function_call_result_success() {
        let result = FunctionCallResult::success(
            Some("call_123".to_string()),
            "test_function".to_string(),
            json!({"result": "success"}),
        );

        assert_eq!(result.call_id, Some("call_123".to_string()));
        assert_eq!(result.name, "test_function");
        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.result["result"], Value::String("success".to_string()));
    }

    #[test]
    fn test_tool_choice_default() {
        let choice = ToolChoice::default();
        match choice {
            ToolChoice::Auto => assert!(true),
            _ => panic!("Default tool choice should be Auto"),
        }
    }
}
