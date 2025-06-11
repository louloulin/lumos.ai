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

use crate::types::{Chat, ToolCall};

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

#[derive(PartialEq, Clone)]
pub struct PendingChat {
    pub chat: Chat,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(PartialEq, Clone)]
pub enum PendingChatState {
    PendingToolChats(Vec<Chat>, i32),
    PendingUserChat(Box<PendingChat>),
    None,
}
