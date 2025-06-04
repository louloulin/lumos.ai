//! Data processing tools inspired by Mastra's data handling
//! 
//! This module provides JSON, CSV, and general data transformation tools

use crate::tool::{Tool, ToolSchema, ParameterSchema, FunctionTool};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Create a JSON parser tool
/// Similar to Mastra's JSON processing capabilities
pub fn create_json_parser_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "json_string".to_string(),
            description: "JSON string to parse".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "path".to_string(),
            description: "JSONPath expression to extract specific data".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "validate_schema".to_string(),
            description: "Whether to validate JSON schema".to_string(),
            r#type: "boolean".to_string(),
            required: false,
            properties: None,
            default: Some(json!(false)),
        },
    ]);

    FunctionTool::new(
        "json_parser",
        "Parse and manipulate JSON data with path extraction",
        schema,
        |params| {
            let json_string = params.get("json_string")
                .and_then(|v| v.as_str())
                .ok_or("JSON string is required")?;
            
            let path = params.get("path").and_then(|v| v.as_str());
            let validate_schema = params.get("validate_schema")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Try to parse JSON
            match serde_json::from_str::<Value>(json_string) {
                Ok(parsed) => {
                    let extracted_data = if let Some(path) = path {
                        // Simple path extraction (in real implementation would use jsonpath)
                        if path.starts_with("$.") {
                            let key = &path[2..];
                            parsed.get(key).cloned().unwrap_or(Value::Null)
                        } else {
                            parsed.clone()
                        }
                    } else {
                        parsed.clone()
                    };

                    Ok(json!({
                        "success": true,
                        "parsed": parsed,
                        "extracted": extracted_data,
                        "path": path,
                        "validate_schema": validate_schema,
                        "metadata": {
                            "type": match parsed {
                                Value::Object(_) => "object",
                                Value::Array(_) => "array",
                                Value::String(_) => "string",
                                Value::Number(_) => "number",
                                Value::Bool(_) => "boolean",
                                Value::Null => "null"
                            },
                            "size": json_string.len()
                        }
                    }))
                },
                Err(e) => {
                    Ok(json!({
                        "success": false,
                        "error": format!("JSON parsing error: {}", e),
                        "input": json_string
                    }))
                }
            }
        },
    )
}

/// Create a CSV parser tool
/// Similar to Mastra's CSV processing capabilities
pub fn create_csv_parser_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "csv_data".to_string(),
            description: "CSV data string to parse".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "delimiter".to_string(),
            description: "CSV delimiter character".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!(",")),
        },
        ParameterSchema {
            name: "has_headers".to_string(),
            description: "Whether the first row contains headers".to_string(),
            r#type: "boolean".to_string(),
            required: false,
            properties: None,
            default: Some(json!(true)),
        },
        ParameterSchema {
            name: "max_rows".to_string(),
            description: "Maximum number of rows to parse".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(1000)),
        },
    ]);

    FunctionTool::new(
        "csv_parser",
        "Parse CSV data into structured format",
        schema,
        |params| {
            let csv_data = params.get("csv_data")
                .and_then(|v| v.as_str())
                .ok_or("CSV data is required")?;
            
            let delimiter = params.get("delimiter")
                .and_then(|v| v.as_str())
                .unwrap_or(",");
            
            let has_headers = params.get("has_headers")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            
            let max_rows = params.get("max_rows")
                .and_then(|v| v.as_u64())
                .unwrap_or(1000);

            // Simple CSV parsing (in real implementation would use csv crate)
            let lines: Vec<&str> = csv_data.lines().collect();
            let mut rows = Vec::new();
            let mut headers = Vec::new();

            if !lines.is_empty() {
                if has_headers {
                    headers = lines[0].split(delimiter).map(|s| s.trim().to_string()).collect();
                    for (i, line) in lines.iter().skip(1).enumerate() {
                        if i >= max_rows as usize { break; }
                        let values: Vec<&str> = line.split(delimiter).collect();
                        let mut row = HashMap::new();
                        for (j, value) in values.iter().enumerate() {
                            let header = headers.get(j).cloned().unwrap_or_else(|| format!("col_{}", j));
                            row.insert(header, value.trim().to_string());
                        }
                        rows.push(row);
                    }
                } else {
                    for (i, line) in lines.iter().enumerate() {
                        if i >= max_rows as usize { break; }
                        let values: Vec<&str> = line.split(delimiter).collect();
                        let mut row = HashMap::new();
                        for (j, value) in values.iter().enumerate() {
                            row.insert(format!("col_{}", j), value.trim().to_string());
                        }
                        rows.push(row);
                    }
                }
            }

            Ok(json!({
                "success": true,
                "headers": headers,
                "rows": rows,
                "metadata": {
                    "total_rows": rows.len(),
                    "total_columns": headers.len(),
                    "delimiter": delimiter,
                    "has_headers": has_headers,
                    "max_rows": max_rows
                }
            }))
        },
    )
}

/// Create a data transformer tool
/// General purpose data transformation
pub fn create_data_transformer_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "data".to_string(),
            description: "Data to transform (JSON)".to_string(),
            r#type: "object".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "operation".to_string(),
            description: "Transformation operation (filter, map, sort, group)".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "parameters".to_string(),
            description: "Parameters for the transformation operation".to_string(),
            r#type: "object".to_string(),
            required: false,
            properties: None,
            default: None,
        },
    ]);

    FunctionTool::new(
        "data_transformer",
        "Transform data using various operations",
        schema,
        |params| {
            let data = params.get("data")
                .ok_or("Data is required")?;
            
            let operation = params.get("operation")
                .and_then(|v| v.as_str())
                .ok_or("Operation is required")?;
            
            let parameters = params.get("parameters");

            // Mock data transformation
            let result = match operation {
                "filter" => {
                    json!({
                        "operation": "filter",
                        "original_data": data,
                        "filtered_data": data, // Mock: return same data
                        "parameters": parameters
                    })
                },
                "map" => {
                    json!({
                        "operation": "map",
                        "original_data": data,
                        "mapped_data": data, // Mock: return same data
                        "parameters": parameters
                    })
                },
                "sort" => {
                    json!({
                        "operation": "sort",
                        "original_data": data,
                        "sorted_data": data, // Mock: return same data
                        "parameters": parameters
                    })
                },
                "group" => {
                    json!({
                        "operation": "group",
                        "original_data": data,
                        "grouped_data": {
                            "groups": [data] // Mock: single group
                        },
                        "parameters": parameters
                    })
                },
                _ => {
                    return Ok(json!({
                        "success": false,
                        "error": format!("Unknown operation: {}", operation),
                        "supported_operations": ["filter", "map", "sort", "group"]
                    }));
                }
            };

            Ok(json!({
                "success": true,
                "result": result,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_json_parser_tool() {
        let tool = create_json_parser_tool();
        
        let mut params = HashMap::new();
        params.insert("json_string".to_string(), json!(r#"{"name": "test", "value": 42}"#));
        params.insert("path".to_string(), json!("$.name"));

        let result = tool.execute(&params).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["extracted"], "test");
    }

    #[tokio::test]
    async fn test_csv_parser_tool() {
        let tool = create_csv_parser_tool();
        
        let csv_data = "name,age,city\nJohn,30,NYC\nJane,25,LA";
        let mut params = HashMap::new();
        params.insert("csv_data".to_string(), json!(csv_data));
        params.insert("has_headers".to_string(), json!(true));

        let result = tool.execute(&params).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["metadata"]["total_rows"], 2);
        assert_eq!(response["headers"], json!(["name", "age", "city"]));
    }

    #[tokio::test]
    async fn test_data_transformer_tool() {
        let tool = create_data_transformer_tool();
        
        let mut params = HashMap::new();
        params.insert("data".to_string(), json!({"items": [1, 2, 3]}));
        params.insert("operation".to_string(), json!("filter"));

        let result = tool.execute(&params).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["result"]["operation"], "filter");
    }
}
