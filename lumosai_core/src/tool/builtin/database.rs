//! 数据库工具集
//! 
//! 提供SQL执行、MongoDB、Redis、Elasticsearch等数据库客户端

use crate::tool::{ToolSchema, ParameterSchema, FunctionTool};
use crate::error::Result;
use serde_json::{Value, json};

/// SQL执行工具
pub fn sql_executor() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "connection_string".to_string(),
            description: "数据库连接字符串".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "query".to_string(),
            description: "SQL查询语句".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "database_type".to_string(),
            description: "数据库类型：postgresql, mysql, sqlite, sqlserver".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("postgresql")),
        },
        ParameterSchema {
            name: "timeout_seconds".to_string(),
            description: "查询超时时间（秒）".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(30)),
        },
        ParameterSchema {
            name: "max_rows".to_string(),
            description: "最大返回行数".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(1000)),
        },
    ]);

    FunctionTool::new(
        "sql_executor",
        "执行SQL查询，支持多种数据库类型",
        schema,
        |params| {
            let _connection_string = params.get("connection_string")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing connection_string parameter".to_string()))?;
            
            let query = params.get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing query parameter".to_string()))?;
            
            let database_type = params.get("database_type")
                .and_then(|v| v.as_str())
                .unwrap_or("postgresql");
            
            let timeout_seconds = params.get("timeout_seconds")
                .and_then(|v| v.as_u64())
                .unwrap_or(30);
            
            let _max_rows = params.get("max_rows")
                .and_then(|v| v.as_u64())
                .unwrap_or(1000);

            // 模拟SQL查询执行
            let query_type = if query.trim().to_uppercase().starts_with("SELECT") {
                "SELECT"
            } else if query.trim().to_uppercase().starts_with("INSERT") {
                "INSERT"
            } else if query.trim().to_uppercase().starts_with("UPDATE") {
                "UPDATE"
            } else if query.trim().to_uppercase().starts_with("DELETE") {
                "DELETE"
            } else {
                "OTHER"
            };

            let mut result = json!({
                "success": true,
                "database_type": database_type,
                "query_type": query_type,
                "execution_time_ms": 125,
                "timeout_seconds": timeout_seconds
            });

            match query_type {
                "SELECT" => {
                    result["rows"] = json!([
                        {"id": 1, "name": "Alice", "email": "alice@example.com"},
                        {"id": 2, "name": "Bob", "email": "bob@example.com"},
                        {"id": 3, "name": "Charlie", "email": "charlie@example.com"}
                    ]);
                    result["row_count"] = json!(3);
                    result["columns"] = json!(["id", "name", "email"]);
                },
                "INSERT" => {
                    result["affected_rows"] = json!(1);
                    result["last_insert_id"] = json!(4);
                },
                "UPDATE" => {
                    result["affected_rows"] = json!(2);
                },
                "DELETE" => {
                    result["affected_rows"] = json!(1);
                },
                _ => {
                    result["message"] = json!("Query executed successfully");
                }
            }

            Ok(result)
        },
    )
}

/// MongoDB客户端工具
pub fn mongodb_client() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "connection_string".to_string(),
            description: "MongoDB连接字符串".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "database".to_string(),
            description: "数据库名称".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "collection".to_string(),
            description: "集合名称".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "operation".to_string(),
            description: "操作类型：find, insert, update, delete, aggregate".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "query".to_string(),
            description: "查询条件或操作数据（JSON格式）".to_string(),
            r#type: "object".to_string(),
            required: false,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "options".to_string(),
            description: "操作选项（JSON格式）".to_string(),
            r#type: "object".to_string(),
            required: false,
            properties: None,
            default: None,
        },
    ]);

    FunctionTool::new(
        "mongodb_client",
        "MongoDB数据库操作客户端",
        schema,
        |params| {
            let _connection_string = params.get("connection_string")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing connection_string parameter".to_string()))?;
            
            let database = params.get("database")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing database parameter".to_string()))?;
            
            let collection = params.get("collection")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing collection parameter".to_string()))?;
            
            let operation = params.get("operation")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing operation parameter".to_string()))?;
            
            let query = params.get("query").cloned();
            let options = params.get("options").cloned();

            // 模拟MongoDB操作
            let mut result = json!({
                "success": true,
                "database": database,
                "collection": collection,
                "operation": operation,
                "execution_time_ms": 85
            });

            match operation {
                "find" => {
                    result["documents"] = json!([
                        {"_id": "507f1f77bcf86cd799439011", "name": "Alice", "age": 25},
                        {"_id": "507f1f77bcf86cd799439012", "name": "Bob", "age": 30},
                        {"_id": "507f1f77bcf86cd799439013", "name": "Charlie", "age": 35}
                    ]);
                    result["count"] = json!(3);
                },
                "insert" => {
                    result["inserted_id"] = json!("507f1f77bcf86cd799439014");
                    result["acknowledged"] = json!(true);
                },
                "update" => {
                    result["matched_count"] = json!(2);
                    result["modified_count"] = json!(2);
                    result["acknowledged"] = json!(true);
                },
                "delete" => {
                    result["deleted_count"] = json!(1);
                    result["acknowledged"] = json!(true);
                },
                "aggregate" => {
                    result["results"] = json!([
                        {"_id": "group1", "count": 5, "avg_age": 28.5},
                        {"_id": "group2", "count": 3, "avg_age": 32.0}
                    ]);
                },
                _ => {
                    result["message"] = json!("Operation completed successfully");
                }
            }

            if let Some(q) = query {
                result["query"] = q;
            }
            if let Some(opts) = options {
                result["options"] = opts;
            }

            Ok(result)
        },
    )
}

/// 获取所有数据库工具
pub fn all_database_tools() -> Vec<FunctionTool> {
    vec![
        sql_executor(),
        mongodb_client(),
    ]
}
