//! Tests for OpenAI Function Calling implementation

#[cfg(test)]
mod tests {
    use super::super::function_calling::*;
    use serde_json::{json, Value};
    use std::collections::HashMap;

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
    fn test_function_call_parse_invalid_arguments() {
        let func_call = FunctionCall {
            id: Some("call_123".to_string()),
            name: "test_function".to_string(),
            arguments: r#"{"invalid": json"#.to_string(),
        };

        assert!(func_call.parse_arguments().is_err());
        assert!(func_call.parse_arguments_as_map().is_err());
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
    fn test_function_call_result_error() {
        let result = FunctionCallResult::error(
            Some("call_123".to_string()),
            "test_function".to_string(),
            "Something went wrong".to_string(),
        );

        assert_eq!(result.call_id, Some("call_123".to_string()));
        assert_eq!(result.name, "test_function");
        assert!(!result.success);
        assert_eq!(result.error, Some("Something went wrong".to_string()));
        assert_eq!(result.result, Value::Null);
    }

    #[test]
    fn test_tool_choice_default() {
        let choice = ToolChoice::default();
        match choice {
            ToolChoice::Auto => assert!(true),
            _ => panic!("Default tool choice should be Auto"),
        }
    }

    #[test]
    fn test_tool_choice_serialization() {
        // Test ToolChoice serialization
        let auto = ToolChoice::Auto;
        let serialized = serde_json::to_string(&auto).unwrap();
        assert_eq!(serialized, r#""auto""#);

        let none = ToolChoice::None;
        let serialized = serde_json::to_string(&none).unwrap();
        assert_eq!(serialized, r#""none""#);

        let required = ToolChoice::Required;
        let serialized = serde_json::to_string(&required).unwrap();
        assert_eq!(serialized, r#""required""#);

        let function = ToolChoice::Function { name: "test_func".to_string() };
        let serialized = serde_json::to_string(&function).unwrap();
        assert!(serialized.contains("test_func"));
    }

    #[test]
    fn test_utils_function_definitions_to_openai_tools() {
        let func_def = FunctionDefinition::new(
            "test_function".to_string(),
            Some("A test function".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "param1": {
                        "type": "string",
                        "description": "First parameter"
                    }
                },
                "required": ["param1"]
            }),
        );

        let tools = utils::function_definitions_to_openai_tools(&[func_def]);
        assert!(tools.is_array());
        
        let tool_array = tools.as_array().unwrap();
        assert_eq!(tool_array.len(), 1);
        
        let tool = &tool_array[0];
        assert_eq!(tool["type"], "function");
        assert_eq!(tool["function"]["name"], "test_function");
        assert_eq!(tool["function"]["description"], "A test function");
    }
}
