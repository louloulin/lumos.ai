//! Agent会话持久化系统
//! 
//! 提供Agent会话的持久化存储、恢复和管理功能，支持多种存储后端。

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::llm::Message;
use crate::error::{Result, Error};

/// 会话状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    /// 活跃状态
    Active,
    /// 暂停状态
    Paused,
    /// 已完成
    Completed,
    /// 已过期
    Expired,
    /// 错误状态
    Error(String),
}

/// 会话元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// 会话ID
    pub session_id: String,
    /// 用户ID
    pub user_id: Option<String>,
    /// Agent名称
    pub agent_name: String,
    /// 会话标题
    pub title: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 会话状态
    pub state: SessionState,
    /// 消息数量
    pub message_count: usize,
    /// 自定义标签
    pub tags: Vec<String>,
    /// 额外属性
    pub properties: HashMap<String, serde_json::Value>,
}

/// 会话数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// 会话元数据
    pub metadata: SessionMetadata,
    /// 消息历史
    pub messages: Vec<Message>,
    /// 会话上下文变量
    pub context: HashMap<String, serde_json::Value>,
    /// 工具调用历史
    pub tool_calls: Vec<ToolCallHistory>,
}

/// 工具调用历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallHistory {
    /// 调用ID
    pub call_id: String,
    /// 工具名称
    pub tool_name: String,
    /// 调用参数
    pub parameters: serde_json::Value,
    /// 调用结果
    pub result: Option<serde_json::Value>,
    /// 调用时间
    pub timestamp: DateTime<Utc>,
    /// 执行状态
    pub status: ToolCallStatus,
    /// 错误信息
    pub error: Option<String>,
}

/// 工具调用状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCallStatus {
    /// 执行中
    Pending,
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 超时
    Timeout,
}

/// 会话存储后端trait
#[async_trait]
pub trait SessionStorage: Send + Sync {
    /// 保存会话数据
    async fn save_session(&self, session: &SessionData) -> Result<()>;
    
    /// 加载会话数据
    async fn load_session(&self, session_id: &str) -> Result<Option<SessionData>>;
    
    /// 删除会话
    async fn delete_session(&self, session_id: &str) -> Result<()>;
    
    /// 列出用户的会话
    async fn list_user_sessions(&self, user_id: &str, limit: Option<usize>) -> Result<Vec<SessionMetadata>>;
    
    /// 搜索会话
    async fn search_sessions(&self, query: &SessionQuery) -> Result<Vec<SessionMetadata>>;
    
    /// 更新会话状态
    async fn update_session_state(&self, session_id: &str, state: SessionState) -> Result<()>;
    
    /// 清理过期会话
    async fn cleanup_expired_sessions(&self, before: DateTime<Utc>) -> Result<usize>;
}

/// 会话查询条件
#[derive(Debug, Clone)]
pub struct SessionQuery {
    /// 用户ID过滤
    pub user_id: Option<String>,
    /// Agent名称过滤
    pub agent_name: Option<String>,
    /// 状态过滤
    pub state: Option<SessionState>,
    /// 标签过滤
    pub tags: Vec<String>,
    /// 时间范围过滤
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    /// 结果限制
    pub limit: Option<usize>,
    /// 偏移量
    pub offset: Option<usize>,
}

/// 内存会话存储实现（用于测试和开发）
pub struct MemorySessionStorage {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
}

impl MemorySessionStorage {
    /// 创建新的内存存储
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemorySessionStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStorage for MemorySessionStorage {
    async fn save_session(&self, session: &SessionData) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.metadata.session_id.clone(), session.clone());
        Ok(())
    }
    
    async fn load_session(&self, session_id: &str) -> Result<Option<SessionData>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }
    
    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }
    
    async fn list_user_sessions(&self, user_id: &str, limit: Option<usize>) -> Result<Vec<SessionMetadata>> {
        let sessions = self.sessions.read().await;
        let mut user_sessions: Vec<SessionMetadata> = sessions
            .values()
            .filter(|session| {
                session.metadata.user_id.as_ref() == Some(&user_id.to_string())
            })
            .map(|session| session.metadata.clone())
            .collect();
        
        // 按更新时间倒序排列
        user_sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        if let Some(limit) = limit {
            user_sessions.truncate(limit);
        }
        
        Ok(user_sessions)
    }
    
    async fn search_sessions(&self, query: &SessionQuery) -> Result<Vec<SessionMetadata>> {
        let sessions = self.sessions.read().await;
        let mut results: Vec<SessionMetadata> = sessions
            .values()
            .filter(|session| {
                let metadata = &session.metadata;
                
                // 用户ID过滤
                if let Some(ref user_id) = query.user_id {
                    if metadata.user_id.as_ref() != Some(user_id) {
                        return false;
                    }
                }
                
                // Agent名称过滤
                if let Some(ref agent_name) = query.agent_name {
                    if &metadata.agent_name != agent_name {
                        return false;
                    }
                }
                
                // 状态过滤
                if let Some(ref state) = query.state {
                    if &metadata.state != state {
                        return false;
                    }
                }
                
                // 标签过滤
                if !query.tags.is_empty() {
                    let has_all_tags = query.tags.iter().all(|tag| metadata.tags.contains(tag));
                    if !has_all_tags {
                        return false;
                    }
                }
                
                // 时间范围过滤
                if let Some(after) = query.created_after {
                    if metadata.created_at < after {
                        return false;
                    }
                }
                
                if let Some(before) = query.created_before {
                    if metadata.created_at > before {
                        return false;
                    }
                }
                
                true
            })
            .map(|session| session.metadata.clone())
            .collect();
        
        // 按更新时间倒序排列
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        // 应用偏移量和限制
        if let Some(offset) = query.offset {
            if offset < results.len() {
                results = results.into_iter().skip(offset).collect();
            } else {
                results.clear();
            }
        }
        
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }
        
        Ok(results)
    }
    
    async fn update_session_state(&self, session_id: &str, state: SessionState) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.metadata.state = state;
            session.metadata.updated_at = Utc::now();
            Ok(())
        } else {
            Err(Error::NotFound(format!("Session not found: {}", session_id)))
        }
    }
    
    async fn cleanup_expired_sessions(&self, before: DateTime<Utc>) -> Result<usize> {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        
        sessions.retain(|_, session| {
            session.metadata.updated_at >= before || 
            session.metadata.state != SessionState::Expired
        });
        
        Ok(initial_count - sessions.len())
    }
}

/// 会话管理器
pub struct SessionManager {
    storage: Arc<dyn SessionStorage>,
    default_expiry: chrono::Duration,
}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new(storage: Arc<dyn SessionStorage>) -> Self {
        Self {
            storage,
            default_expiry: chrono::Duration::hours(24), // 默认24小时过期
        }
    }
    
    /// 设置默认过期时间
    pub fn with_expiry(mut self, expiry: chrono::Duration) -> Self {
        self.default_expiry = expiry;
        self
    }
    
    /// 创建新会话
    pub async fn create_session(
        &self,
        session_id: String,
        agent_name: String,
        user_id: Option<String>,
    ) -> Result<SessionData> {
        let now = Utc::now();
        let metadata = SessionMetadata {
            session_id: session_id.clone(),
            user_id,
            agent_name,
            title: None,
            created_at: now,
            updated_at: now,
            state: SessionState::Active,
            message_count: 0,
            tags: Vec::new(),
            properties: HashMap::new(),
        };
        
        let session = SessionData {
            metadata,
            messages: Vec::new(),
            context: HashMap::new(),
            tool_calls: Vec::new(),
        };
        
        self.storage.save_session(&session).await?;
        Ok(session)
    }
    
    /// 获取会话
    pub async fn get_session(&self, session_id: &str) -> Result<Option<SessionData>> {
        self.storage.load_session(session_id).await
    }
    
    /// 更新会话
    pub async fn update_session(&self, session: &SessionData) -> Result<()> {
        self.storage.save_session(session).await
    }
    
    /// 添加消息到会话
    pub async fn add_message(&self, session_id: &str, message: Message) -> Result<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            session.messages.push(message);
            session.metadata.message_count = session.messages.len();
            session.metadata.updated_at = Utc::now();
            self.update_session(&session).await?;
        }
        Ok(())
    }
    
    /// 添加工具调用记录
    pub async fn add_tool_call(&self, session_id: &str, tool_call: ToolCallHistory) -> Result<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            session.tool_calls.push(tool_call);
            session.metadata.updated_at = Utc::now();
            self.update_session(&session).await?;
        }
        Ok(())
    }
    
    /// 设置会话标题
    pub async fn set_session_title(&self, session_id: &str, title: String) -> Result<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            session.metadata.title = Some(title);
            session.metadata.updated_at = Utc::now();
            self.update_session(&session).await?;
        }
        Ok(())
    }
    
    /// 清理过期会话
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let cutoff = Utc::now() - self.default_expiry;
        self.storage.cleanup_expired_sessions(cutoff).await
    }
}
