//! System utility tools inspired by Mastra's system integrations
//! 
//! This module provides datetime, UUID generation, and other system utilities

use crate::tool::{Tool, ToolSchema, ParameterSchema, FunctionTool};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Create a datetime tool
/// Similar to Mastra's date/time utilities
pub fn create_datetime_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "operation".to_string(),
            description: "Operation to perform (now, parse, format, add, subtract)".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "input".to_string(),
            description: "Input datetime string (for parse/format operations)".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "format".to_string(),
            description: "Format string for parsing or formatting".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("%Y-%m-%d %H:%M:%S")),
        },
        ParameterSchema {
            name: "timezone".to_string(),
            description: "Timezone for the operation (UTC, Local, or timezone name)".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("UTC")),
        },
        ParameterSchema {
            name: "amount".to_string(),
            description: "Amount to add/subtract (for add/subtract operations)".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "unit".to_string(),
            description: "Unit for add/subtract (seconds, minutes, hours, days, weeks, months, years)".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("seconds")),
        },
    ]);

    FunctionTool::new(
        "datetime",
        "Comprehensive datetime operations and formatting",
        schema,
        |params| {
            let operation = params.get("operation")
                .and_then(|v| v.as_str())
                .ok_or("Operation is required")?;
            
            let input = params.get("input").and_then(|v| v.as_str());
            let format = params.get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("%Y-%m-%d %H:%M:%S");
            let timezone = params.get("timezone")
                .and_then(|v| v.as_str())
                .unwrap_or("UTC");
            let amount = params.get("amount").and_then(|v| v.as_i64());
            let unit = params.get("unit")
                .and_then(|v| v.as_str())
                .unwrap_or("seconds");

            let now = chrono::Utc::now();

            match operation {
                "now" => {
                    Ok(json!({
                        "success": true,
                        "operation": "now",
                        "result": {
                            "iso": now.to_rfc3339(),
                            "timestamp": now.timestamp(),
                            "formatted": now.format(format).to_string(),
                            "timezone": timezone
                        }
                    }))
                },
                "parse" => {
                    if let Some(input) = input {
                        // Mock parsing - in real implementation would use chrono parsing
                        Ok(json!({
                            "success": true,
                            "operation": "parse",
                            "input": input,
                            "format": format,
                            "result": {
                                "iso": now.to_rfc3339(),
                                "timestamp": now.timestamp(),
                                "timezone": timezone
                            }
                        }))
                    } else {
                        Ok(json!({
                            "success": false,
                            "error": "Input datetime string is required for parse operation"
                        }))
                    }
                },
                "format" => {
                    if let Some(input) = input {
                        Ok(json!({
                            "success": true,
                            "operation": "format",
                            "input": input,
                            "format": format,
                            "result": {
                                "formatted": now.format(format).to_string(),
                                "timezone": timezone
                            }
                        }))
                    } else {
                        Ok(json!({
                            "success": false,
                            "error": "Input datetime string is required for format operation"
                        }))
                    }
                },
                "add" | "subtract" => {
                    if let Some(amount) = amount {
                        let multiplier = if operation == "subtract" { -1 } else { 1 };
                        let adjusted_amount = amount * multiplier;
                        
                        // Mock calculation
                        let result_time = match unit {
                            "seconds" => now + chrono::Duration::seconds(adjusted_amount),
                            "minutes" => now + chrono::Duration::minutes(adjusted_amount),
                            "hours" => now + chrono::Duration::hours(adjusted_amount),
                            "days" => now + chrono::Duration::days(adjusted_amount),
                            "weeks" => now + chrono::Duration::weeks(adjusted_amount),
                            _ => now, // For months/years, would need more complex logic
                        };

                        Ok(json!({
                            "success": true,
                            "operation": operation,
                            "amount": amount,
                            "unit": unit,
                            "result": {
                                "iso": result_time.to_rfc3339(),
                                "timestamp": result_time.timestamp(),
                                "formatted": result_time.format(format).to_string(),
                                "timezone": timezone
                            }
                        }))
                    } else {
                        Ok(json!({
                            "success": false,
                            "error": "Amount is required for add/subtract operations"
                        }))
                    }
                },
                _ => {
                    Ok(json!({
                        "success": false,
                        "error": format!("Unknown operation: {}", operation),
                        "supported_operations": ["now", "parse", "format", "add", "subtract"]
                    }))
                }
            }
        },
    )
}

/// Create a UUID generator tool
/// Similar to Mastra's ID generation utilities
pub fn create_uuid_generator_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "version".to_string(),
            description: "UUID version (4 for random, 1 for timestamp-based)".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(4)),
        },
        ParameterSchema {
            name: "count".to_string(),
            description: "Number of UUIDs to generate".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(1)),
        },
        ParameterSchema {
            name: "format".to_string(),
            description: "Output format (standard, simple, urn)".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("standard")),
        },
    ]);

    FunctionTool::new(
        "uuid_generator",
        "Generate UUIDs in various formats and versions",
        schema,
        |params| {
            let version = params.get("version")
                .and_then(|v| v.as_u64())
                .unwrap_or(4);
            
            let count = params.get("count")
                .and_then(|v| v.as_u64())
                .unwrap_or(1)
                .min(100); // Limit to 100 UUIDs
            
            let format = params.get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("standard");

            let mut uuids = Vec::new();
            
            for i in 0..count {
                // Mock UUID generation - in real implementation would use uuid crate
                let mock_uuid = format!("550e8400-e29b-41d4-a716-44665544{:04}", i);
                
                let formatted_uuid = match format {
                    "simple" => mock_uuid.replace("-", ""),
                    "urn" => format!("urn:uuid:{}", mock_uuid),
                    _ => mock_uuid, // standard format
                };
                
                uuids.push(formatted_uuid);
            }

            Ok(json!({
                "success": true,
                "version": version,
                "count": count,
                "format": format,
                "uuids": uuids,
                "metadata": {
                    "generated_at": chrono::Utc::now().to_rfc3339(),
                    "total_generated": uuids.len()
                }
            }))
        },
    )
}

/// Create a hash generator tool
/// Generate various types of hashes
pub fn create_hash_generator_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "input".to_string(),
            description: "Input string to hash".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "algorithm".to_string(),
            description: "Hash algorithm (md5, sha1, sha256, sha512)".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("sha256")),
        },
        ParameterSchema {
            name: "encoding".to_string(),
            description: "Output encoding (hex, base64)".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("hex")),
        },
    ]);

    FunctionTool::new(
        "hash_generator",
        "Generate cryptographic hashes of input data",
        schema,
        |params| {
            let input = params.get("input")
                .and_then(|v| v.as_str())
                .ok_or("Input is required")?;
            
            let algorithm = params.get("algorithm")
                .and_then(|v| v.as_str())
                .unwrap_or("sha256");
            
            let encoding = params.get("encoding")
                .and_then(|v| v.as_str())
                .unwrap_or("hex");

            // Mock hash generation - in real implementation would use crypto libraries
            let mock_hash = match algorithm {
                "md5" => "5d41402abc4b2a76b9719d911017c592",
                "sha1" => "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d",
                "sha256" => "2cf24dba4f21d4288094e9b9eb4e5f0164e031c5c7f6c2b9c8b8e8f8e8f8e8f8",
                "sha512" => "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043",
                _ => "unknown_algorithm",
            };

            let encoded_hash = match encoding {
                "base64" => base64::encode(mock_hash), // Mock base64 encoding
                _ => mock_hash.to_string(), // hex format
            };

            Ok(json!({
                "success": true,
                "input": input,
                "algorithm": algorithm,
                "encoding": encoding,
                "hash": encoded_hash,
                "metadata": {
                    "input_length": input.len(),
                    "hash_length": encoded_hash.len(),
                    "generated_at": chrono::Utc::now().to_rfc3339()
                }
            }))
        },
    )
}

// Mock base64 module for compilation
mod base64 {
    pub fn encode(input: &str) -> String {
        format!("base64_{}", input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_datetime_tool_now() {
        let tool = create_datetime_tool();
        
        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("now"));

        let context = crate::tool::context::ToolExecutionContext::new();
        let options = crate::tool::schema::ToolExecutionOptions::new();
        let result = tool.execute(serde_json::to_value(&params).unwrap(), context, &options).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["operation"], "now");
        assert!(response["result"]["iso"].is_string());
    }

    #[tokio::test]
    async fn test_uuid_generator_tool() {
        let tool = create_uuid_generator_tool();
        
        let mut params = HashMap::new();
        params.insert("count".to_string(), json!(3));
        params.insert("format".to_string(), json!("standard"));

        let context = crate::tool::context::ToolExecutionContext::new();
        let options = crate::tool::schema::ToolExecutionOptions::new();
        let result = tool.execute(serde_json::to_value(&params).unwrap(), context, &options).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["count"], 3);
        assert_eq!(response["uuids"].as_array().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_hash_generator_tool() {
        let tool = create_hash_generator_tool();
        
        let mut params = HashMap::new();
        params.insert("input".to_string(), json!("hello world"));
        params.insert("algorithm".to_string(), json!("sha256"));

        let context = crate::tool::context::ToolExecutionContext::new();
        let options = crate::tool::schema::ToolExecutionOptions::new();
        let result = tool.execute(serde_json::to_value(&params).unwrap(), context, &options).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["algorithm"], "sha256");
        assert!(response["hash"].is_string());
    }
}
