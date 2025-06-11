use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub fn routes() -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/version", get(version))
        .route("/api/assistants", get(list_assistants))
        .route("/api/chat", post(chat_completion))
        .route("/api/stats", get(get_stats))
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    version: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn version() -> Json<Value> {
    Json(json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "authors": env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<_>>(),
    }))
}

#[derive(Serialize)]
struct Assistant {
    id: u32,
    name: String,
    description: String,
    status: String,
    conversations: u32,
    created_at: String,
}

async fn list_assistants() -> Json<Vec<Assistant>> {
    let assistants = vec![
        Assistant {
            id: 1,
            name: "Customer Support Bot".to_string(),
            description: "AI assistant for customer support and inquiries".to_string(),
            status: "Active".to_string(),
            conversations: 156,
            created_at: "2024-01-15T10:30:00Z".to_string(),
        },
        Assistant {
            id: 2,
            name: "Sales Assistant".to_string(),
            description: "AI assistant for sales and lead qualification".to_string(),
            status: "Active".to_string(),
            conversations: 89,
            created_at: "2024-01-10T14:20:00Z".to_string(),
        },
        Assistant {
            id: 3,
            name: "Technical Helper".to_string(),
            description: "AI assistant for technical documentation and support".to_string(),
            status: "Inactive".to_string(),
            conversations: 23,
            created_at: "2024-01-05T09:15:00Z".to_string(),
        },
    ];
    
    Json(assistants)
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
    assistant_id: Option<u32>,
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
    assistant_id: u32,
    timestamp: String,
}

async fn chat_completion(Json(payload): Json<ChatRequest>) -> Result<Json<ChatResponse>, StatusCode> {
    // Simulate AI response
    let response = format!(
        "Thank you for your message: '{}'. This is a demo response from the LumosAI assistant. In a real implementation, this would connect to an AI service.",
        payload.message
    );
    
    Ok(Json(ChatResponse {
        response,
        assistant_id: payload.assistant_id.unwrap_or(1),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

#[derive(Serialize)]
struct Stats {
    active_assistants: u32,
    total_conversations: u32,
    api_calls_today: u32,
    success_rate: String,
    uptime: String,
}

async fn get_stats() -> Json<Stats> {
    Json(Stats {
        active_assistants: 12,
        total_conversations: 1234,
        api_calls_today: 5678,
        success_rate: "98.5%".to_string(),
        uptime: "99.9%".to_string(),
    })
}
