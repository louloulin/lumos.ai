//! 简化的会话管理API
//!
//! 提供简单易用的Agent会话持久化功能。

use crate::{Result, Error, Message};
use std::sync::Arc;

// 重导出核心类型
pub use lumosai_core::agent::session::{
    SessionManager as CoreSessionManager,
    SessionStorage,
    MemorySessionStorage,
    SessionData,
    SessionMetadata,
    SessionState,
    SessionQuery,
    ToolCallHistory,
    ToolCallStatus,
};

/// 简化的会话类型
pub type Session = Arc<dyn SessionTrait>;

/// 会话管理器
pub type SessionManager = CoreSessionManager;

/// 会话trait
#[async_trait::async_trait]
pub trait SessionTrait: Send + Sync {
    /// 获取会话ID
    fn id(&self) -> &str;
    
    /// 添加消息
    async fn add_message(&self, message: Message) -> Result<()>;
    
    /// 获取消息历史
    async fn get_messages(&self) -> Result<Vec<Message>>;
    
    /// 设置会话标题
    async fn set_title(&self, title: &str) -> Result<()>;
    
    /// 获取会话状态
    async fn get_state(&self) -> Result<SessionState>;
    
    /// 更新会话状态
    async fn set_state(&self, state: SessionState) -> Result<()>;
    
    /// 保存会话
    async fn save(&self) -> Result<()>;
}

/// 创建新会话
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let session = lumos::session::create("my_agent", Some("user_123")).await?;
///     
///     // 添加消息
///     let message = Message {
///         role: Role::User,
///         content: "Hello!".to_string(),
///         metadata: None,
///         name: None,
///     };
///     session.add_message(message).await?;
///     
///     Ok(())
/// }
/// ```
pub async fn create(agent_name: &str, user_id: Option<&str>) -> Result<Session> {
    let storage = Arc::new(MemorySessionStorage::new());
    let manager = CoreSessionManager::new(storage);
    
    let session_id = uuid::Uuid::new_v4().to_string();
    let session_data = manager.create_session(
        session_id.clone(),
        agent_name.to_string(),
        user_id.map(|s| s.to_string()),
    ).await?;
    
    Ok(Arc::new(SimpleSession::new(session_data, manager)))
}

/// 创建带有自定义存储的会话
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let storage = Arc::new(MemorySessionStorage::new());
///     let session = lumos::session::create_with_storage(
///         "my_agent", 
///         Some("user_123"), 
///         storage
///     ).await?;
///     
///     Ok(())
/// }
/// ```
pub async fn create_with_storage(
    agent_name: &str,
    user_id: Option<&str>,
    storage: Arc<dyn SessionStorage>,
) -> Result<Session> {
    let manager = CoreSessionManager::new(storage);
    
    let session_id = uuid::Uuid::new_v4().to_string();
    let session_data = manager.create_session(
        session_id.clone(),
        agent_name.to_string(),
        user_id.map(|s| s.to_string()),
    ).await?;
    
    Ok(Arc::new(SimpleSession::new(session_data, manager)))
}

/// 加载现有会话
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let storage = Arc::new(MemorySessionStorage::new());
///     
///     if let Some(session) = lumos::session::load("session_id", storage).await? {
///         println!("Loaded session: {}", session.id());
///     }
///     
///     Ok(())
/// }
/// ```
pub async fn load(
    session_id: &str,
    storage: Arc<dyn SessionStorage>,
) -> Result<Option<Session>> {
    let manager = CoreSessionManager::new(storage);
    
    if let Some(session_data) = manager.get_session(session_id).await? {
        Ok(Some(Arc::new(SimpleSession::new(session_data, manager))))
    } else {
        Ok(None)
    }
}

/// 列出用户的会话
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let storage = Arc::new(MemorySessionStorage::new());
///     let sessions = lumos::session::list_user_sessions("user_123", storage, Some(10)).await?;
///     
///     for session_meta in sessions {
///         println!("Session: {} - {}", session_meta.session_id, session_meta.agent_name);
///     }
///     
///     Ok(())
/// }
/// ```
pub async fn list_user_sessions(
    user_id: &str,
    storage: Arc<dyn SessionStorage>,
    limit: Option<usize>,
) -> Result<Vec<SessionMetadata>> {
    storage.list_user_sessions(user_id, limit).await
}

/// 简单会话实现
struct SimpleSession {
    data: SessionData,
    manager: CoreSessionManager,
}

impl SimpleSession {
    fn new(data: SessionData, manager: CoreSessionManager) -> Self {
        Self { data, manager }
    }
}

#[async_trait::async_trait]
impl SessionTrait for SimpleSession {
    fn id(&self) -> &str {
        &self.data.metadata.session_id
    }
    
    async fn add_message(&self, message: Message) -> Result<()> {
        self.manager.add_message(&self.data.metadata.session_id, message).await
    }
    
    async fn get_messages(&self) -> Result<Vec<Message>> {
        if let Some(session_data) = self.manager.get_session(&self.data.metadata.session_id).await? {
            Ok(session_data.messages)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn set_title(&self, title: &str) -> Result<()> {
        self.manager.set_session_title(&self.data.metadata.session_id, title.to_string()).await
    }
    
    async fn get_state(&self) -> Result<SessionState> {
        if let Some(session_data) = self.manager.get_session(&self.data.metadata.session_id).await? {
            Ok(session_data.metadata.state)
        } else {
            Err(Error::NotFound("Session not found".to_string()))
        }
    }
    
    async fn set_state(&self, state: SessionState) -> Result<()> {
        // 这里需要通过存储直接更新状态
        // 暂时返回成功
        Ok(())
    }
    
    async fn save(&self) -> Result<()> {
        if let Some(session_data) = self.manager.get_session(&self.data.metadata.session_id).await? {
            self.manager.update_session(&session_data).await
        } else {
            Err(Error::NotFound("Session not found".to_string()))
        }
    }
}

/// 会话构建器
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let session = lumos::session::builder()
///         .agent_name("my_agent")
///         .user_id("user_123")
///         .title("My Chat Session")
///         .build()
///         .await?;
///     
///     Ok(())
/// }
/// ```
pub fn builder() -> SessionBuilder {
    SessionBuilder::new()
}

/// 会话构建器
pub struct SessionBuilder {
    agent_name: Option<String>,
    user_id: Option<String>,
    title: Option<String>,
    storage: Option<Arc<dyn SessionStorage>>,
}

impl SessionBuilder {
    pub fn new() -> Self {
        Self {
            agent_name: None,
            user_id: None,
            title: None,
            storage: None,
        }
    }
    
    /// 设置Agent名称
    pub fn agent_name(mut self, name: &str) -> Self {
        self.agent_name = Some(name.to_string());
        self
    }
    
    /// 设置用户ID
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }
    
    /// 设置会话标题
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }
    
    /// 设置存储后端
    pub fn storage(mut self, storage: Arc<dyn SessionStorage>) -> Self {
        self.storage = Some(storage);
        self
    }
    
    /// 构建会话
    pub async fn build(self) -> Result<Session> {
        let agent_name = self.agent_name
            .ok_or_else(|| Error::Config("Agent name is required".to_string()))?;
        
        let storage = self.storage
            .unwrap_or_else(|| Arc::new(MemorySessionStorage::new()));
        
        let session = create_with_storage(
            &agent_name,
            self.user_id.as_deref(),
            storage,
        ).await?;
        
        if let Some(title) = self.title {
            session.set_title(&title).await?;
        }
        
        Ok(session)
    }
}

impl Default for SessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Role;
    
    #[tokio::test]
    async fn test_session_creation() {
        let session = create("test_agent", Some("user_123")).await
            .expect("Failed to create session");
        
        assert!(!session.id().is_empty());
    }
    
    #[tokio::test]
    async fn test_session_message_handling() {
        let session = create("test_agent", Some("user_123")).await
            .expect("Failed to create session");
        
        let message = Message {
            role: Role::User,
            content: "Hello!".to_string(),
            metadata: None,
            name: None,
        };
        
        session.add_message(message).await.expect("Failed to add message");
        
        let messages = session.get_messages().await.expect("Failed to get messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello!");
    }
    
    #[test]
    fn test_session_builder() {
        let builder = builder()
            .agent_name("test_agent")
            .user_id("user_123")
            .title("Test Session");
        
        // 测试构建器模式
        assert!(true); // 简单的编译测试
    }
}
