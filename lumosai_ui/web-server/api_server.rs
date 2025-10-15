/*!
# API Server Module

独立的API服务器，提供AI聊天和管理功能。

## 功能特性

- **RESTful API**: 标准的HTTP API接口
- **流式响应**: Server-Sent Events支持
- **CORS支持**: 跨域资源共享
- **错误处理**: 统一的错误响应格式
*/

use axum::{
    extract::{Path, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{delete, get, post},
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::ai_client::{AIClient, AIClientConfig, AIProvider};
use crate::database::Database;
use crate::file_handler::{FileConfig, FileHandler};
use crate::streaming::{self, AppState};
use crate::tools::ToolRegistry;

/// 启动API服务器
pub async fn start_api_server() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting LumosAI API Server...");

    // 创建AI客户端和数据库连接
    let ai_client = create_ai_client();
    let database = create_database().await?;
    let tool_registry = ToolRegistry::new();
    let file_handler = create_file_handler(database.clone()).await?;
    let app_state = AppState {
        ai_client,
        database,
        tool_registry,
        file_handler: file_handler.clone(),
    };

    // 配置CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // 构建路由
    let app = Router::new()
        // 健康检查
        .route("/health", get(streaming::health_check))
        // 聊天API
        .route("/api/chat/stream", post(streaming::stream_chat))
        .route("/api/chat/simple", post(streaming::simple_chat))
        // 对话管理
        .route("/api/conversations", get(streaming::list_conversations))
        .route("/api/conversations/:id", get(streaming::get_conversation))
        .route(
            "/api/conversations/:id",
            delete(streaming::delete_conversation),
        )
        // AI模型管理
        .route("/api/models", get(list_models))
        .route("/api/models/:id", get(get_model))
        // 配置管理
        .route("/api/config", get(get_config))
        .route("/api/config", post(update_config))
        // 工具管理
        .route("/api/tools", get(streaming::list_tools))
        .route("/api/tools/execute", post(streaming::execute_tool))
        // 文件管理
        .route("/api/files/upload", post(upload_files_handler))
        .route("/api/files", get(list_files_handler))
        .route("/api/files/:id", delete(delete_file_handler))
        // 静态文件和文档
        .route("/", get(api_info))
        .route("/docs", get(api_docs))
        .layer(ServiceBuilder::new().layer(cors))
        .with_state(app_state);

    // 启动服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("📡 API Server listening on http://{}", addr);
    println!("📚 API Documentation: http://{}/docs", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 创建AI客户端
fn create_ai_client() -> AIClient {
    // 从环境变量读取配置
    if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
        println!("🔑 Using OpenAI API");
        return AIClient::openai(openai_key);
    }

    if let Ok(deepseek_key) = std::env::var("DEEPSEEK_API_KEY") {
        println!("🔑 Using DeepSeek API");
        return AIClient::deepseek(deepseek_key);
    }

    // 默认使用本地Ollama
    println!("🏠 Using local Ollama");
    AIClient::ollama(
        "http://localhost:11434/v1".to_string(),
        "llama2".to_string(),
    )
}

/// 创建数据库连接
async fn create_database() -> Result<Database, Box<dyn std::error::Error>> {
    // 从环境变量读取数据库URL，默认使用本地SQLite文件
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./lumosai.db".to_string());

    println!("🗄️ Connecting to database: {}", database_url);

    let database = Database::new(&database_url).await?;
    println!("✅ Database connected successfully");

    Ok(database)
}

/// 创建文件处理器
async fn create_file_handler(
    database: Database,
) -> Result<FileHandler, Box<dyn std::error::Error>> {
    let config = FileConfig::default();
    let file_handler = FileHandler::new(config, database);

    // 初始化上传目录
    file_handler.init().await?;
    println!("📁 File handler initialized");

    Ok(file_handler)
}

/// API信息
async fn api_info() -> impl IntoResponse {
    Json(json!({
        "name": "LumosAI API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "AI聊天和管理API服务",
        "endpoints": {
            "health": "/health",
            "chat_stream": "/api/chat/stream",
            "chat_simple": "/api/chat/simple",
            "conversations": "/api/conversations",
            "models": "/api/models",
            "config": "/api/config",
            "tools": "/api/tools",
            "files": "/api/files",
            "docs": "/docs"
        }
    }))
}

/// API文档
async fn api_docs() -> impl IntoResponse {
    let docs = r#"
# LumosAI API Documentation

## 聊天API

### 流式聊天
```
POST /api/chat/stream
Content-Type: application/json

{
    "message": "你好，请介绍一下自己",
    "conversation_id": "optional_conversation_id",
    "model": "optional_model_name"
}
```

### 简单聊天
```
POST /api/chat/simple
Content-Type: application/json

{
    "message": "你好，请介绍一下自己",
    "conversation_id": "optional_conversation_id",
    "model": "optional_model_name"
}
```

## 对话管理

### 获取对话列表
```
GET /api/conversations
```

### 获取特定对话
```
GET /api/conversations/{id}
```

### 删除对话
```
DELETE /api/conversations/{id}
```

## 模型管理

### 获取可用模型
```
GET /api/models
```

### 获取模型详情
```
GET /api/models/{id}
```

## 配置管理

### 获取配置
```
GET /api/config
```

### 更新配置
```
POST /api/config
Content-Type: application/json

{
    "provider": "openai",
    "api_key": "your_api_key",
    "model": "gpt-3.5-turbo",
    "temperature": 0.7,
    "max_tokens": 2048
}
```
"#;

    (StatusCode::OK, [("content-type", "text/markdown")], docs)
}

/// 获取可用模型列表
async fn list_models() -> impl IntoResponse {
    Json(json!({
        "models": [
            {
                "id": "gpt-3.5-turbo",
                "name": "GPT-3.5 Turbo",
                "provider": "openai",
                "description": "快速、经济的通用模型"
            },
            {
                "id": "gpt-4",
                "name": "GPT-4",
                "provider": "openai",
                "description": "最强大的多模态模型"
            },
            {
                "id": "deepseek-chat",
                "name": "DeepSeek Chat",
                "provider": "deepseek",
                "description": "高性能的中文对话模型"
            },
            {
                "id": "llama2",
                "name": "Llama 2",
                "provider": "ollama",
                "description": "开源的本地运行模型"
            }
        ]
    }))
}

/// 获取特定模型信息
async fn get_model(Path(model_id): Path<String>) -> impl IntoResponse {
    // TODO: 从配置或数据库获取模型信息
    Json(json!({
        "id": model_id,
        "name": format!("Model {}", model_id),
        "provider": "unknown",
        "description": "Model description",
        "capabilities": ["chat", "completion"],
        "context_length": 4096,
        "max_tokens": 2048
    }))
}

/// 获取当前配置
async fn get_config() -> impl IntoResponse {
    Json(json!({
        "provider": "openai",
        "model": "gpt-3.5-turbo",
        "temperature": 0.7,
        "max_tokens": 2048,
        "stream": true,
        "available_providers": ["openai", "deepseek", "anthropic", "ollama"]
    }))
}

/// 更新配置
async fn update_config(Json(config): Json<serde_json::Value>) -> impl IntoResponse {
    // TODO: 验证和保存配置
    Json(json!({
        "success": true,
        "message": "配置已更新",
        "config": config
    }))
}

/// 文件上传处理器包装
async fn upload_files_handler(
    State(state): State<AppState>,
    multipart: axum::extract::Multipart,
) -> impl IntoResponse {
    crate::file_handler::upload_files(State(state.file_handler), multipart).await
}

/// 文件列表处理器包装
async fn list_files_handler(State(state): State<AppState>) -> impl IntoResponse {
    crate::file_handler::list_files(State(state.file_handler)).await
}

/// 文件删除处理器包装
async fn delete_file_handler(
    Path(file_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    crate::file_handler::delete_file(Path(file_id), State(state.file_handler)).await
}
