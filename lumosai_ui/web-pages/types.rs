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
    pub category_id: i32,
    pub image_icon_object_id: Option<String>,
    pub updated_at: OffsetDateTime,
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
    pub content: Option<String>,
    pub role: ChatRole,
    pub tool_call_id: Option<String>,
    pub tool_calls: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dataset {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub team_id: i32,
    pub visibility: Visibility,
    pub count: i64,
    pub combine_under_n_chars: i32,
    pub new_after_n_chars: i32,
    pub embeddings_model_name: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: i32,
    pub name: String,
    pub model_type: ModelType,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub tpm_limit: Option<i32>,
    pub rpm_limit: Option<i32>,
    pub context_size: Option<i32>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i32,
    pub name: String,
    pub key_value: String,
    pub api_key: String,
    pub team_id: i32,
    pub prompt_type: PromptType,
    pub prompt_name: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Integration {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub integration_type: IntegrationType,
    pub enabled: bool,
    pub integration_status: IntegrationStatus,
    pub created_at: OffsetDateTime,
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
    pub first_name: Option<String>,
    pub last_name: Option<String>,
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

    pub fn has_oauth2_security(&self) -> bool {
        // Simple implementation - check if spec contains oauth2 security
        self.spec.get("components")
            .and_then(|c| c.get("securitySchemes"))
            .map(|s| s.to_string().contains("oauth2"))
            .unwrap_or(false)
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

// Additional enums and types for compatibility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChatRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenUsageType {
    Prompt,
    Completion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntegrationStatus {
    Configured,
    NotConfigured,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PromptType {
    Chat,
    Completion,
    Assistant,
    Model,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Member,
    Viewer,
    SystemAdministrator,
    TeamManager,
    Collaborator,
}

// Database query result types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyTokenUsage {
    pub date: OffsetDateTime,
    pub total_tokens: i64,
    pub usage_type: TokenUsageType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyApiRequests {
    pub date: OffsetDateTime,
    pub request_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct History {
    pub id: i32,
    pub user_id: i32,
    pub prompt_id: Option<i32>,
    pub message: String,
    pub response: Option<String>,
    pub summary: String,
    pub prompt_type: PromptType,
    pub created_at: OffsetDateTime,
    pub created_at_iso: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimit {
    pub id: i32,
    pub model_id: i32,
    pub requests_per_minute: i32,
    pub tokens_per_minute: i32,
    pub api_key_id: Option<i32>,
    pub tpm_limit: Option<i32>,
    pub rpm_limit: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub id: i32,
    pub dataset_id: i32,
    pub name: String,
    pub content: String,
    pub file_name: String,
    pub content_size: i64,
    pub batches: i32,
    pub waiting: i32,
    pub fail_count: i32,
    pub failure_reason: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelWithPrompt {
    pub id: i32,
    pub name: String,
    pub model_type: String,
    pub prompt_count: i64,
    pub base_url: Option<String>,
    pub tpm_limit: Option<i32>,
    pub rpm_limit: Option<i32>,
    pub context_size: Option<i32>,
    pub prompt_id: Option<i32>,
    pub display_name: String,
    pub api_key: Option<String>,
    pub description: Option<String>,
    pub disclaimer: Option<String>,
    pub example1: Option<String>,
    pub example2: Option<String>,
    pub example3: Option<String>,
    pub example4: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Member {
    pub id: i32,
    pub user_id: i32,
    pub team_id: i32,
    pub role: Role,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub roles: Vec<Role>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Invitation {
    pub id: i32,
    pub email: String,
    pub team_id: i32,
    pub invited_by: i32,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub roles: Vec<Role>,
    pub created_at: OffsetDateTime,
}

// Capability and model types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub capability: ModelCapability,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModelCapability {
    TextGeneration,
    ImageGeneration,
    CodeGeneration,
    Embedding,
    #[serde(rename = "tool_use")]
    ToolUse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

// Additional types for integrations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiKeyConnection {
    pub id: i32,
    pub integration_id: i32,
    pub api_key: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Oauth2Connection {
    pub id: i32,
    pub integration_id: i32,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntegrationType {
    OpenAPI,
    OAuth2,
    ApiKey,
    Custom,
    #[serde(rename = "MCP_Server")]
    MCP_Server,
    BuiltIn,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModelType {
    OpenAI,
    Anthropic,
    Local,
    Custom,
    #[serde(rename = "LLM")]
    LLM,
    Embeddings,
    TextToSpeech,
    Image,
}

// Tool definition for OpenAI API compatibility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BionicToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

// Additional types for UI components
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamOwner {
    pub team_id: i32,
    pub user_id: i32,
    pub team_name: String,
    pub user_email: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditTrail {
    pub id: i32,
    pub user_id: i32,
    pub action: String,
    pub details: Option<String>,
    pub created_at: OffsetDateTime,
}

// Additional dataset and model types





// Tool call types for console
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallFunction {
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
    pub fn can_view_system_prompt(&self) -> bool { true }
    pub fn can_delete_chat(&self) -> bool { true }
    pub fn can_edit_dataset(&self, _dataset: &Dataset) -> bool { true }
    pub fn can_manage_integrations(&self) -> bool { true }
    pub fn can_make_assistant_public(&self) -> bool { true }
    pub fn can_make_invitations(&self) -> bool { true }
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
