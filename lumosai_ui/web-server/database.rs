/*!
# Database Module

数据库集成模块，实现对话历史持久化和用户管理。

## 功能特性

- **对话管理**: 创建、查询、删除对话
- **消息存储**: 聊天消息的持久化存储
- **用户管理**: 基础的用户认证和授权
- **内存存储**: 简化的内存数据库实现
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// 数据库错误类型
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("用户未找到")]
    UserNotFound,
    #[error("对话未找到")]
    ConversationNotFound,
    #[error("权限不足")]
    PermissionDenied,
    #[error("内部错误: {0}")]
    Internal(String),
}

/// 用户模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 对话模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 消息角色
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "tool")]
    Tool,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
            MessageRole::Tool => write!(f, "tool"),
        }
    }
}

/// 消息模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub conversation_id: i64,
    pub role: MessageRole,
    pub content: Option<String>,
    pub tool_calls: Option<String>, // JSON格式
    pub tool_call_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 内存数据存储
#[derive(Debug)]
struct MemoryStore {
    users: HashMap<i64, User>,
    conversations: HashMap<i64, Conversation>,
    messages: HashMap<i64, Vec<Message>>,
    next_user_id: i64,
    next_conversation_id: i64,
    next_message_id: i64,
}

impl MemoryStore {
    fn new() -> Self {
        let mut store = Self {
            users: HashMap::new(),
            conversations: HashMap::new(),
            messages: HashMap::new(),
            next_user_id: 1,
            next_conversation_id: 1,
            next_message_id: 1,
        };

        // 创建默认系统用户
        let system_user = User {
            id: 1,
            email: "system@lumosai.local".to_string(),
            name: "System User".to_string(),
            created_at: chrono::Utc::now(),
        };
        store.users.insert(1, system_user);
        store.next_user_id = 2;

        store
    }
}

/// 数据库连接
#[derive(Clone)]
pub struct Database {
    store: Arc<Mutex<MemoryStore>>,
}

impl Database {
    /// 创建新的数据库连接
    pub async fn new(_database_url: &str) -> Result<Self, DatabaseError> {
        Ok(Self {
            store: Arc::new(Mutex::new(MemoryStore::new())),
        })
    }

    /// 创建内存数据库（用于测试）
    pub async fn new_in_memory() -> Result<Self, DatabaseError> {
        Ok(Self {
            store: Arc::new(Mutex::new(MemoryStore::new())),
        })
    }

    /// 创建文件数据库
    pub async fn new_file<P: AsRef<std::path::Path>>(_path: P) -> Result<Self, DatabaseError> {
        Ok(Self {
            store: Arc::new(Mutex::new(MemoryStore::new())),
        })
    }

    /// 创建用户
    pub async fn create_user(&self, email: &str, name: &str) -> Result<User, DatabaseError> {
        let mut store = self.store.lock().map_err(|e| DatabaseError::Internal(e.to_string()))?;

        let user_id = store.next_user_id;
        let user = User {
            id: user_id,
            email: email.to_string(),
            name: name.to_string(),
            created_at: chrono::Utc::now(),
        };

        store.users.insert(user_id, user.clone());
        store.next_user_id += 1;

        Ok(user)
    }

    /// 根据邮箱获取用户
    pub async fn get_user_by_email(&self, email: &str) -> Result<User, DatabaseError> {
        let store = self.store.lock().map_err(|e| DatabaseError::Internal(e.to_string()))?;

        for user in store.users.values() {
            if user.email == email {
                return Ok(user.clone());
            }
        }

        Err(DatabaseError::UserNotFound)
    }

    /// 创建对话
    pub async fn create_conversation(
        &self,
        user_id: i64,
        title: &str,
    ) -> Result<Conversation, DatabaseError> {
        let mut store = self.store.lock().map_err(|e| DatabaseError::Internal(e.to_string()))?;

        let conversation_id = store.next_conversation_id;
        let conversation = Conversation {
            id: conversation_id,
            user_id,
            title: title.to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        store.conversations.insert(conversation_id, conversation.clone());
        store.messages.insert(conversation_id, Vec::new());
        store.next_conversation_id += 1;

        Ok(conversation)
    }

    /// 获取用户的对话列表
    pub async fn get_conversations(&self, user_id: i64) -> Result<Vec<Conversation>, DatabaseError> {
        let store = self.store.lock().map_err(|e| DatabaseError::Internal(e.to_string()))?;

        let mut conversations: Vec<Conversation> = store
            .conversations
            .values()
            .filter(|conv| conv.user_id == user_id)
            .cloned()
            .collect();

        // 按更新时间倒序排序
        conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(conversations)
    }

    /// 获取特定对话
    pub async fn get_conversation(
        &self,
        conversation_id: i64,
        user_id: i64,
    ) -> Result<Conversation, DatabaseError> {
        let store = self.store.lock().map_err(|e| DatabaseError::Internal(e.to_string()))?;

        match store.conversations.get(&conversation_id) {
            Some(conversation) if conversation.user_id == user_id => Ok(conversation.clone()),
            Some(_) => Err(DatabaseError::PermissionDenied),
            None => Err(DatabaseError::ConversationNotFound),
        }
    }

    /// 删除对话
    pub async fn delete_conversation(
        &self,
        conversation_id: i64,
        user_id: i64,
    ) -> Result<(), DatabaseError> {
        let mut store = self.store.lock().map_err(|e| DatabaseError::Internal(e.to_string()))?;

        match store.conversations.get(&conversation_id) {
            Some(conversation) if conversation.user_id == user_id => {
                store.conversations.remove(&conversation_id);
                store.messages.remove(&conversation_id);
                Ok(())
            }
            Some(_) => Err(DatabaseError::PermissionDenied),
            None => Err(DatabaseError::ConversationNotFound),
        }
    }

    /// 添加消息到对话
    pub async fn add_message(
        &self,
        conversation_id: i64,
        role: MessageRole,
        content: Option<String>,
        tool_calls: Option<String>,
        tool_call_id: Option<String>,
    ) -> Result<Message, DatabaseError> {
        let mut store = self.store.lock().map_err(|e| DatabaseError::Internal(e.to_string()))?;

        // 检查对话是否存在
        if !store.conversations.contains_key(&conversation_id) {
            return Err(DatabaseError::ConversationNotFound);
        }

        let message = Message {
            id: store.next_message_id,
            conversation_id,
            role,
            content,
            tool_calls,
            tool_call_id,
            created_at: chrono::Utc::now(),
        };

        store.next_message_id += 1;

        // 添加消息到对话
        store.messages.entry(conversation_id).or_insert_with(Vec::new).push(message.clone());

        // 更新对话的更新时间
        if let Some(conversation) = store.conversations.get_mut(&conversation_id) {
            conversation.updated_at = chrono::Utc::now();
        }

        Ok(message)
    }

    /// 获取对话的消息列表
    pub async fn get_messages(&self, conversation_id: i64) -> Result<Vec<Message>, DatabaseError> {
        let store = self.store.lock().map_err(|e| DatabaseError::Internal(e.to_string()))?;

        match store.messages.get(&conversation_id) {
            Some(messages) => Ok(messages.clone()),
            None => Ok(Vec::new()),
        }
    }

    /// 更新对话的更新时间
    pub async fn touch_conversation(&self, conversation_id: i64) -> Result<(), DatabaseError> {
        let mut store = self.store.lock().map_err(|e| DatabaseError::Internal(e.to_string()))?;

        if let Some(conversation) = store.conversations.get_mut(&conversation_id) {
            conversation.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(DatabaseError::ConversationNotFound)
        }
    }
}
