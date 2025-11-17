//! A2A Protocol Core Types
//!
//! 定义 A2A 协议的核心数据结构，与 Google 官方规范完全兼容

use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;
use uuid::Uuid;

/// A2A 任务状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskState {
    /// 任务已提交，等待处理
    Submitted,
    /// 任务正在执行中
    Working,
    /// 等待用户输入
    InputRequired,
    /// 任务已成功完成
    Completed,
    /// 任务已取消
    Canceled,
    /// 任务执行失败
    Failed,
    /// 未知状态
    Unknown,
}

impl Default for TaskState {
    fn default() -> Self {
        Self::Submitted
    }
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Submitted => write!(f, "submitted"),
            Self::Working => write!(f, "working"),
            Self::InputRequired => write!(f, "input-required"),
            Self::Completed => write!(f, "completed"),
            Self::Canceled => write!(f, "canceled"),
            Self::Failed => write!(f, "failed"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Agent 认证信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authentication {
    /// 支持的认证方案列表
    pub schemes: Vec<AuthScheme>,
    /// 认证凭据（token等）
    pub credentials: Option<String>,
}

impl Default for Authentication {
    fn default() -> Self {
        Self {
            schemes: vec![AuthScheme::Bearer],
            credentials: None,
        }
    }
}

/// 认证方案
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthScheme {
    /// Bearer Token 认证
    Bearer,
    /// Basic 认证
    Basic,
    /// API Key 认证
    ApiKey,
    /// OAuth 2.0
    OAuth2,
    /// 自定义认证方案
    Custom(String),
}

/// Agent 能力声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    /// 是否支持流式响应
    pub streaming: bool,
    /// 是否支持推送通知
    pub push_notifications: bool,
    /// 是否支持状态转换历史
    pub state_transition_history: bool,
    /// 支持的最大并发任务数
    pub max_concurrent_tasks: Option<u32>,
    /// 支持的输入模式
    pub input_modes: Vec<String>,
    /// 支持的输出模式
    pub output_modes: Vec<String>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            streaming: false,
            push_notifications: false,
            state_transition_history: true,
            max_concurrent_tasks: Some(1),
            input_modes: vec!["text".to_string()],
            output_modes: vec!["text".to_string()],
        }
    }
}

/// Agent 提供者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    /// 组织名称
    pub organization: String,
    /// 组织URL
    pub url: Option<Url>,
    /// 联系邮箱
    pub email: Option<String>,
}

/// Agent 技能定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// 技能唯一标识符
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能描述
    pub description: Option<String>,
    /// 技能标签
    pub tags: Option<Vec<String>>,
    /// 示例输入
    pub examples: Option<Vec<String>>,
    /// 支持的输入模式
    pub input_modes: Option<Vec<String>>,
    /// 支持的输出模式
    pub output_modes: Option<Vec<String>>,
    /// 技能参数定义
    pub parameters: Option<Vec<Parameter>>,
}

impl Skill {
    /// 创建新技能
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            tags: None,
            examples: None,
            input_modes: None,
            output_modes: None,
            parameters: None,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 添加标签
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// 添加示例
    pub fn with_examples(mut self, examples: Vec<String>) -> Self {
        self.examples = Some(examples);
        self
    }

    /// 验证技能
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("Skill ID cannot be empty".to_string());
        }

        if self.name.trim().is_empty() {
            return Err("Skill name cannot be empty".to_string());
        }

        Ok(())
    }
}

/// 技能参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub param_type: String,
    /// 参数描述
    pub description: Option<String>,
    /// 是否必需
    pub required: bool,
    /// 默认值
    pub default_value: Option<serde_json::Value>,
    /// 枚举值（如果适用）
    pub enum_values: Option<Vec<String>>,
}

/// A2A Agent 卡片 - 核心数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    /// Agent 名称
    pub name: String,
    /// Agent 描述
    pub description: Option<String>,
    /// Agent 服务端点URL
    pub url: Url,
    /// 提供者信息
    pub provider: Option<Provider>,
    /// Agent 版本
    pub version: String,
    /// 文档URL
    pub documentation_url: Option<Url>,
    /// Agent 能力
    pub capabilities: Capabilities,
    /// 认证信息
    pub authentication: Option<Authentication>,
    /// 技能列表
    pub skills: Vec<Skill>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AgentCard {
    /// 验证 Agent 卡片的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }

        if !self.url.scheme().starts_with("http") {
            return Err("Agent URL must be HTTP or HTTPS".to_string());
        }

        if self.version.trim().is_empty() {
            return Err("Agent version cannot be empty".to_string());
        }

        // 验证技能
        for skill in &self.skills {
            skill.validate().map_err(|e| format!("Invalid skill '{}': {}", skill.id, e))?;
        }

        Ok(())
    }

    /// 生成 Agent ID（基于名称和URL）
    pub fn generate_id(&self) -> String {
        format!("{}:{}", 
            self.name.replace(' ', "_").to_lowercase(),
            self.url.host_str().unwrap_or("unknown")
        )
    }

    /// 检查是否支持特定技能
    pub fn supports_skill(&self, skill_id: &str) -> bool {
        self.skills.iter().any(|s| s.id == skill_id)
    }

    /// 检查是否支持特定能力
    pub fn supports_capability(&self, capability: &str) -> bool {
        match capability {
            "streaming" => self.capabilities.streaming,
            "push_notifications" => self.capabilities.push_notifications,
            "state_transition_history" => self.capabilities.state_transition_history,
            _ => false,
        }
    }
}

/// 消息部分 - 文本内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPart {
    /// 类型标识符
    #[serde(rename = "type")]
    pub part_type: TextPartType,
    /// 文本内容
    pub text: String,
    /// 元数据
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 文本部分类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextPartType {
    Text,
}

impl TextPart {
    pub fn new(text: String) -> Self {
        Self {
            part_type: TextPartType::Text,
            text,
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// 消息部分 - 文件内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePart {
    /// 类型标识符
    #[serde(rename = "type")]
    pub part_type: FilePartType,
    /// 文件内容
    pub file: FileContent,
    /// 元数据
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 文件部分类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilePartType {
    File,
}

impl FilePart {
    pub fn new(file: FileContent) -> Self {
        Self {
            part_type: FilePartType::File,
            file,
            metadata: None,
        }
    }
}

/// 文件内容定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    /// 文件名
    pub name: Option<String>,
    /// MIME类型
    pub mime_type: Option<String>,
    /// Base64编码的文件内容
    pub bytes: Option<String>,
    /// 文件URI
    pub uri: Option<String>,
}

impl FileContent {
    pub fn from_bytes(name: String, bytes: Vec<u8>, mime_type: String) -> Self {
        Self {
            name: Some(name),
            mime_type: Some(mime_type),
            bytes: Some(base64::prelude::BASE64_STANDARD.encode(&bytes)),
            uri: None,
        }
    }

    pub fn from_uri(uri: String) -> Self {
        Self {
            name: None,
            mime_type: None,
            bytes: None,
            uri: Some(uri),
        }
    }

    /// 验证文件内容
    pub fn validate(&self) -> Result<(), String> {
        if self.bytes.is_none() && self.uri.is_none() {
            return Err("Either bytes or uri must be provided".to_string());
        }

        if self.bytes.is_some() && self.uri.is_some() {
            return Err("Cannot provide both bytes and uri".to_string());
        }

        Ok(())
    }
}

/// 消息部分 - 数据内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPart {
    /// 类型标识符
    #[serde(rename = "type")]
    pub part_type: DataPartType,
    /// 数据内容
    pub data: serde_json::Value,
    /// 元数据
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 数据部分类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataPartType {
    Data,
}

impl DataPart {
    pub fn new(data: serde_json::Value) -> Self {
        Self {
            part_type: DataPartType::Data,
            data,
            metadata: None,
        }
    }
}

/// 消息部分枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    Text(TextPart),
    File(FilePart),
    Data(DataPart),
}

impl Part {
    /// 创建文本部分
    pub fn text(text: String) -> Self {
        Self::Text(TextPart::new(text))
    }

    /// 创建文件部分
    pub fn file(file: FileContent) -> Self {
        Self::File(FilePart::new(file))
    }

    /// 创建数据部分
    pub fn data(data: serde_json::Value) -> Self {
        Self::Data(DataPart::new(data))
    }
}

/// A2A 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息角色
    pub role: MessageRole,
    /// 消息内容部分
    pub parts: Vec<Part>,
    /// 消息元数据
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 消息角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageRole {
    User,
    Agent,
}

impl Message {
    pub fn new(role: MessageRole, parts: Vec<Part>) -> Self {
        Self {
            role,
            parts,
            metadata: None,
        }
    }

    pub fn user_message(text: String) -> Self {
        Self::new(MessageRole::User, vec![Part::text(text)])
    }

    pub fn agent_message(text: String) -> Self {
        Self::new(MessageRole::Agent, vec![Part::text(text)])
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    /// 当前状态
    pub state: TaskState,
    /// 状态消息
    pub message: Option<Message>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 元数据
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl TaskStatus {
    pub fn new(state: TaskState) -> Self {
        Self {
            state,
            message: None,
            timestamp: Utc::now(),
            metadata: None,
        }
    }

    pub fn with_message(mut self, message: Message) -> Self {
        self.message = Some(message);
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务唯一标识符
    pub id: String,
    /// 任务描述
    pub description: String,
    /// 任务输入消息
    pub input_message: Message,
    /// 当前状态
    pub status: TaskStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 截止时间（可选）
    pub deadline: Option<DateTime<Utc>>,
    /// 分配的Agent ID（可选）
    pub assigned_agent: Option<String>,
    /// 任务参数
    pub parameters: HashMap<String, serde_json::Value>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Task {
    /// 创建新任务
    pub fn new(description: String, input_message: Message) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            description,
            input_message,
            status: TaskStatus::new(TaskState::Submitted),
            created_at: now,
            updated_at: now,
            deadline: None,
            assigned_agent: None,
            parameters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// 更新任务状态
    pub fn update_status(&mut self, new_status: TaskState, message: Option<Message>) {
        self.status = TaskStatus::new(new_status)
            .with_message(message.unwrap_or(Message::agent_message("".to_string())));
        self.updated_at = Utc::now();
    }

    /// 分配给Agent
    pub fn assign_to_agent(&mut self, agent_id: String) {
        self.assigned_agent = Some(agent_id);
        self.update_status(TaskState::Working, None);
    }

    /// 设置截止时间
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// 添加参数
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.parameters.insert(key, value);
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// 工件 - 任务输出结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// 工件唯一标识符
    pub id: String,
    /// 工件名称
    pub name: Option<String>,
    /// 工件描述
    pub description: Option<String>,
    /// 工件内容部分
    pub parts: Vec<Part>,
    /// 工件索引
    pub index: i32,
    /// 是否追加内容（用于流式）
    pub append: Option<bool>,
    /// 工件元数据
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// 是否为最后一块（用于流式）
    pub last_chunk: Option<bool>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Artifact {
    pub fn new(parts: Vec<Part>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: None,
            description: None,
            parts,
            index: 0,
            append: None,
            metadata: None,
            last_chunk: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_index(mut self, index: i32) -> Self {
        self.index = index;
        self
    }

    /// 从文本创建工件
    pub fn from_text(text: String) -> Self {
        Self::new(vec![Part::text(text)])
    }

    /// 从文件创建工件
    pub fn from_file(name: String, content: FileContent) -> Self {
        Self::new(vec![Part::file(content)]).with_name(name)
    }

    /// 验证工件
    pub fn validate(&self) -> Result<(), String> {
        if self.parts.is_empty() {
            return Err("Artifact must contain at least one part".to_string());
        }

        // 验证每个部分
        for part in &self.parts {
            match part {
                Part::File(file_part) => {
                    file_part.file.validate()
                        .map_err(|e| format!("Invalid file part: {}", e))?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

// 使用外部 base64 crate
pub use base64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_state_display() {
        assert_eq!(TaskState::Submitted.to_string(), "submitted");
        assert_eq!(TaskState::Working.to_string(), "working");
        assert_eq!(TaskState::Completed.to_string(), "completed");
    }

    #[test]
    fn test_skill_creation() {
        let skill = Skill::new("test_skill".to_string(), "Test Skill".to_string())
            .with_description("A test skill".to_string())
            .with_tags(vec!["test".to_string()]);

        assert_eq!(skill.id, "test_skill");
        assert_eq!(skill.name, "Test Skill");
        assert_eq!(skill.description, Some("A test skill".to_string()));
        assert_eq!(skill.tags, Some(vec!["test".to_string()]));
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "Test task".to_string(),
            Message::user_message("Hello, world!".to_string()),
        );

        assert_eq!(task.description, "Test task");
        assert_eq!(task.status.state, TaskState::Submitted);
        assert!(task.id.len() > 0);
    }

    #[test]
    fn test_artifact_validation() {
        let valid_artifact = Artifact::from_text("Test content".to_string());
        assert!(valid_artifact.validate().is_ok());

        let invalid_artifact = Artifact::new(vec![]);
        assert!(invalid_artifact.validate().is_err());
    }
}