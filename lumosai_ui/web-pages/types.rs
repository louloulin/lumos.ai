/*!
# Mock Types for UI Components

This module provides simplified mock types to replace database and integration dependencies
for UI-only components.
*/

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// Visibility enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Private,
    Team,
    Company,
}

// Mock database types for UI components
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prompt {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub system_prompt: String,
    pub visibility: Visibility,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptIntegration {
    pub id: i32,
    pub prompt_id: i32,
    pub integration_id: i32,
    pub name: String,
    pub enabled: bool,
    pub integration_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptDataset {
    pub id: i32,
    pub prompt_id: i32,
    pub dataset_id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chat {
    pub id: i32,
    pub user_id: i32,
    pub prompt_id: i32,
    pub message: String,
    pub response: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dataset {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub team_id: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: i32,
    pub name: String,
    pub model_type: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i32,
    pub name: String,
    pub key_value: String,
    pub team_id: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Integration {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub integration_type: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: Option<String>,
}

// Mock integration types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BionicOpenAPI {
    pub name: String,
    pub description: Option<String>,
    pub base_url: String,
    pub spec: serde_json::Value,
}

impl BionicOpenAPI {
    pub fn get_title(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn get_logo_url(&self) -> &str {
        "/icons/integration.svg" // Default logo
    }

    pub fn get_oauth2_config(&self) -> Option<serde_json::Value> {
        None // Simplified for UI
    }

    pub fn has_api_key_security(&self) -> bool {
        true // Simplified for UI
    }
}

// Additional types for compatibility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SinglePrompt {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub system_prompt: String,
    pub visibility: Visibility,
    pub created_at: OffsetDateTime,
    // Additional fields for compatibility
    pub temperature: Option<f64>,
    pub max_history_items: i32,
    pub max_tokens: i32,
    pub max_chunks: i32,
    pub trim_ratio: i32,
    pub model_name: String,
    pub example1: Option<String>,
    pub example2: Option<String>,
    pub example3: Option<String>,
    pub example4: Option<String>,
    pub disclaimer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InviteSummary {
    pub id: i32,
    pub email: String,
    pub team_name: String,
    pub invited_by: String,
    pub created_at: OffsetDateTime,
    pub team_id: i32,
}

// Tool call types for console
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
}

// RBAC (Role-Based Access Control) types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rbac {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub team_id: i32,
    pub role: String,
}

impl Rbac {
    pub fn can_view_datasets(&self) -> bool { true }
    pub fn can_view_prompts(&self) -> bool { true }
    pub fn can_view_integrations(&self) -> bool { true }
    pub fn can_use_api_keys(&self) -> bool { true }
    pub fn can_view_teams(&self) -> bool { true }
    pub fn can_view_audit_trail(&self) -> bool { true }
    pub fn can_setup_models(&self) -> bool { true }
}

// Utility functions
pub fn visibility_to_string(visibility: Visibility) -> String {
    match visibility {
        Visibility::Private => "Private".to_string(),
        Visibility::Team => "Team".to_string(),
        Visibility::Company => "Everyone".to_string(),
    }
}

pub fn string_to_visibility(visibility: &str) -> Visibility {
    match visibility {
        "Team" => Visibility::Team,
        "Everyone" => Visibility::Company,
        _ => Visibility::Private,
    }
}
