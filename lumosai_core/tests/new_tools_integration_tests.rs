//! 新工具集成测试
//! 
//! 测试AI工具集、数据库工具集和通信工具集的功能

use lumosai_core::tool::builtin::{ai, database, communication};
use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;
use tokio;

#[tokio::test]
async fn test_ai_tools() {
    // 测试图像分析工具
    let image_analyzer = ai::image_analyzer();
    let params = json!({
        "image_data": "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAYEBQYFBAYGBQYHBwYIChAKCgkJChQODwwQFxQYGBcUFhYaHSUfGhsjHBYWICwgIyYnKSopGR8tMC0oMCUoKSj/2wBDAQcHBwoIChMKChMoGhYaKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCj/wAARCAABAAEDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAv/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFQEBAQAAAAAAAAAAAAAAAAAAAAX/xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oADAMBAAIRAxEAPwCdABmX/9k=",
        "analysis_type": "object_detection",
        "confidence_threshold": 0.8
    });
    
    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::default();
    
    let result = image_analyzer.execute(params, context, &options).await.unwrap();
    
    assert!(result["success"].as_bool().unwrap());
    assert!(result["results"]["objects"].is_array());
    assert_eq!(result["results"]["analysis_type"], "object_detection");
    assert_eq!(result["results"]["confidence_threshold"], 0.8);

    // 测试文本摘要工具
    let text_summarizer = ai::text_summarizer();
    let params = json!({
        "text": "这是一个很长的文本，需要进行摘要处理。文本包含了很多重要的信息，但是太长了，需要压缩成更短的版本。摘要应该保留最重要的信息，同时去掉不必要的细节。",
        "max_length": 50,
        "strategy": "extractive"
    });
    
    let context = ToolExecutionContext::new();
    let result = text_summarizer.execute(params, context, &options).await.unwrap();
    
    assert!(result["success"].as_bool().unwrap());
    assert!(result["summary"].is_string());
    assert!(result["summary"].as_str().unwrap().len() <= 50);
    assert_eq!(result["strategy"], "extractive");

    // 测试情感分析工具
    let sentiment_analyzer = ai::sentiment_analyzer();
    let params = json!({
        "text": "这个产品真的很棒！我非常喜欢它的设计和功能。",
        "analysis_depth": "detailed"
    });
    
    let context = ToolExecutionContext::new();
    let result = sentiment_analyzer.execute(params, context, &options).await.unwrap();
    
    assert!(result["success"].as_bool().unwrap());
    assert!(result["sentiment"]["label"].is_string());
    assert!(result["sentiment"]["score"].is_number());
    assert!(result["detailed_analysis"].is_object());
}

#[tokio::test]
async fn test_database_tools() {
    // 测试SQL执行工具
    let sql_executor = database::sql_executor();
    let params = json!({
        "connection_string": "postgresql://user:password@localhost:5432/testdb",
        "query": "SELECT * FROM users WHERE active = true",
        "database_type": "postgresql",
        "timeout_seconds": 30
    });
    
    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::default();
    
    let result = sql_executor.execute(params, context, &options).await.unwrap();
    
    assert!(result["success"].as_bool().unwrap());
    assert_eq!(result["database_type"], "postgresql");
    assert_eq!(result["query_type"], "SELECT");
    assert!(result["rows"].is_array());
    assert_eq!(result["row_count"], 3);

    // 测试MongoDB客户端工具
    let mongodb_client = database::mongodb_client();
    let params = json!({
        "connection_string": "mongodb://localhost:27017",
        "database": "testdb",
        "collection": "users",
        "operation": "find",
        "query": {"active": true}
    });
    
    let context = ToolExecutionContext::new();
    let result = mongodb_client.execute(params, context, &options).await.unwrap();
    
    assert!(result["success"].as_bool().unwrap());
    assert_eq!(result["database"], "testdb");
    assert_eq!(result["collection"], "users");
    assert_eq!(result["operation"], "find");
    assert!(result["documents"].is_array());
    assert_eq!(result["count"], 3);
}

#[tokio::test]
async fn test_communication_tools() {
    // 测试邮件发送工具
    let email_sender = communication::email_sender();
    let params = json!({
        "smtp_config": {
            "host": "smtp.gmail.com",
            "port": 587,
            "username": "test@example.com",
            "password": "password"
        },
        "to": "recipient@example.com",
        "subject": "测试邮件",
        "body": "这是一封测试邮件的内容。",
        "is_html": false
    });
    
    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::default();
    
    let result = email_sender.execute(params, context, &options).await.unwrap();
    
    assert!(result["success"].as_bool().unwrap());
    assert!(result["message_id"].is_string());
    assert_eq!(result["to"], "recipient@example.com");
    assert_eq!(result["subject"], "测试邮件");
    assert_eq!(result["is_html"], false);
    assert_eq!(result["delivery_status"], "sent");

    // 测试Slack消息工具
    let slack_messenger = communication::slack_messenger();
    let params = json!({
        "webhook_url": "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX",
        "channel": "#general",
        "text": "Hello from Lumos AI!",
        "username": "Lumos Bot"
    });
    
    let context = ToolExecutionContext::new();
    let result = slack_messenger.execute(params, context, &options).await.unwrap();
    
    assert!(result["success"].as_bool().unwrap());
    assert_eq!(result["ok"], true);
    assert_eq!(result["channel"], "#general");
    assert_eq!(result["text"], "Hello from Lumos AI!");
    assert_eq!(result["username"], "Lumos Bot");

    // 测试Webhook调用工具
    let webhook_caller = communication::webhook_caller();
    let params = json!({
        "url": "https://api.example.com/webhook",
        "method": "POST",
        "body": {"message": "Hello World"},
        "timeout_seconds": 30
    });
    
    let context = ToolExecutionContext::new();
    let result = webhook_caller.execute(params, context, &options).await.unwrap();
    
    assert!(result["success"].as_bool().unwrap());
    assert_eq!(result["request"]["url"], "https://api.example.com/webhook");
    assert_eq!(result["request"]["method"], "POST");
    assert_eq!(result["response"]["status_code"], 200);
    assert!(result["response"]["body"]["status"] == "success");
}

#[tokio::test]
async fn test_all_tools_collection() {
    // 测试获取所有AI工具
    let ai_tools = ai::all_ai_tools();
    assert_eq!(ai_tools.len(), 3);
    
    let tool_ids: Vec<&str> = ai_tools.iter().map(|t| t.id()).collect();
    assert!(tool_ids.contains(&"image_analyzer"));
    assert!(tool_ids.contains(&"text_summarizer"));
    assert!(tool_ids.contains(&"sentiment_analyzer"));

    // 测试获取所有数据库工具
    let database_tools = database::all_database_tools();
    assert_eq!(database_tools.len(), 2);
    
    let tool_ids: Vec<&str> = database_tools.iter().map(|t| t.id()).collect();
    assert!(tool_ids.contains(&"sql_executor"));
    assert!(tool_ids.contains(&"mongodb_client"));

    // 测试获取所有通信工具
    let communication_tools = communication::all_communication_tools();
    assert_eq!(communication_tools.len(), 3);
    
    let tool_ids: Vec<&str> = communication_tools.iter().map(|t| t.id()).collect();
    assert!(tool_ids.contains(&"email_sender"));
    assert!(tool_ids.contains(&"slack_messenger"));
    assert!(tool_ids.contains(&"webhook_caller"));
}

#[tokio::test]
async fn test_tool_error_handling() {
    // 测试缺少必需参数的错误处理
    let image_analyzer = ai::image_analyzer();
    let params = json!({
        "analysis_type": "object_detection"
        // 缺少必需的 image_data 参数
    });
    
    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::default();
    
    let result = image_analyzer.execute(params, context, &options).await;
    assert!(result.is_err());

    // 测试SQL工具缺少参数的错误处理
    let sql_executor = database::sql_executor();
    let params = json!({
        "connection_string": "postgresql://localhost:5432/testdb"
        // 缺少必需的 query 参数
    });
    
    let context = ToolExecutionContext::new();
    let result = sql_executor.execute(params, context, &options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_schemas() {
    // 测试工具模式定义
    let image_analyzer = ai::image_analyzer();
    let schema = image_analyzer.schema();
    
    assert_eq!(schema.parameters.len(), 3);
    
    let param_names: Vec<&str> = schema.parameters.iter().map(|p| p.name.as_str()).collect();
    assert!(param_names.contains(&"image_data"));
    assert!(param_names.contains(&"analysis_type"));
    assert!(param_names.contains(&"confidence_threshold"));
    
    // 检查必需参数
    let required_params: Vec<&str> = schema.parameters.iter()
        .filter(|p| p.required)
        .map(|p| p.name.as_str())
        .collect();
    assert!(required_params.contains(&"image_data"));
    assert!(!required_params.contains(&"analysis_type"));
}
