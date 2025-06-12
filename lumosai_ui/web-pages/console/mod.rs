pub mod console_stream;
pub mod conversation;
pub mod empty_stream;
pub mod history_drawer;
pub mod index;
pub mod layout;
pub mod model_popup;
pub mod prompt_drawer;
pub mod prompt_form;
pub mod tools_modal;

// New streaming chat components
pub mod message_timeline;

// Enhanced AI Agent UI components
pub mod enhanced_console;
pub mod file_upload;
pub mod voice_input;

use crate::types::{Chat, ToolCall};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// Chat chunks type for UI
#[derive(Debug, Clone, PartialEq)]
pub struct ChatChunks {
    pub id: i32,
    pub chat_id: i32,
    pub content: String,
    pub page_number: Option<i32>,
    pub file_name: Option<String>,
    pub created_at: time::OffsetDateTime,
}

#[derive(PartialEq, Clone)]
pub struct ChatWithChunks {
    pub chat: Chat,
    pub chunks: Vec<ChatChunks>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PendingChat {
    pub chat: Chat,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PendingChatState {
    PendingToolChats(Vec<Chat>, i32),
    PendingUserChat(Box<PendingChat>),
    Thinking,
    Streaming,
    ToolCalling,
    None,
}

// New streaming chat types
/// 聊天消息角色
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChatRole {
    User,
    Assistant,
    System,
    Tool,
}

/// 聊天消息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: u64,
    pub role: ChatRole,
    pub content: Option<String>,
    pub timestamp: String,
    pub tool_calls: Option<String>,
    pub tool_call_id: Option<String>,
}

/// 聊天会话
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatConversation {
    pub id: u64,
    pub title: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: String,
    pub updated_at: String,
}

/// 全局聊天状态
#[derive(Debug, Clone, PartialEq)]
pub struct ChatState {
    pub current_conversation: Option<ChatConversation>,
    pub conversations: Vec<ChatConversation>,
    pub pending_state: PendingChatState,
    pub is_streaming: bool,
    pub is_locked: bool,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            current_conversation: None,
            conversations: Vec::new(),
            pending_state: PendingChatState::None,
            is_streaming: false,
            is_locked: false,
        }
    }
}

/// 发送消息
pub fn send_message(content: String) {
    // 简化实现 - 仅打印消息
    println!("发送消息: {}", content);
}



/// 生成消息ID
pub fn generate_message_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

/// 生成会话ID
fn generate_conversation_id() -> u64 {
    generate_message_id()
}

/// 截断标题
pub fn truncate_title(content: &str) -> String {
    if content.len() > 50 {
        format!("{}...", &content[..47])
    } else {
        content.to_string()
    }
}

// Enhanced UI types for AI Agent components

/// 对话信息
#[derive(Debug, Clone, PartialEq)]
pub struct Conversation {
    pub id: i64,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub summary: Option<String>,
}

/// 聊天历史
#[derive(Debug, Clone, PartialEq)]
pub struct ChatHistory {
    pub conversations: Vec<Conversation>,
    pub total_count: usize,
}

/// 获取相对时间显示
pub fn get_relative_time() -> String {
    // TODO: 实现真实的相对时间计算
    "刚刚".to_string()
}

/// 复制到剪贴板
pub fn copy_to_clipboard(text: &str) {
    // TODO: 实现真实的剪贴板操作
    println!("复制到剪贴板: {}", text);
}

/// 重新生成回复
pub fn regenerate_response(chat_id: i64) {
    // TODO: 实现重新生成功能
    println!("重新生成回复: {}", chat_id);
}

/// 朗读文本
pub fn read_aloud(text: &str) {
    // TODO: 实现文本转语音功能
    println!("朗读文本: {}", text);
}

/// 分享消息
pub fn share_message(text: &str) {
    // TODO: 实现消息分享功能
    println!("分享消息: {}", text);
}
