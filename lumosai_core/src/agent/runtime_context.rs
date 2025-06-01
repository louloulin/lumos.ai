//! Agent运行时上下文系统
//! 
//! 提供Agent执行过程中的上下文管理，包括会话状态、工具调用历史、
//! 内存访问和执行环境信息。

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;

use crate::llm::Message;
use crate::tool::Tool;
use crate::memory::WorkingMemory;
use crate::error::Result;

/// Agent运行时上下文
/// 
/// 包含Agent执行过程中需要的所有上下文信息，支持动态更新和状态管理
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// 会话ID，用于跟踪对话
    pub session_id: String,
    /// 线程ID，用于多轮对话管理
    pub thread_id: Option<String>,
    /// 运行ID，唯一标识本次执行
    pub run_id: String,
    /// 用户ID，用于多用户环境
    pub user_id: Option<String>,
    /// 当前消息历史
    pub messages: Vec<Message>,
    /// 动态上下文变量
    pub variables: HashMap<String, Value>,
    /// 可用工具列表
    pub available_tools: Vec<Arc<dyn Tool>>,
    /// 工具调用历史
    pub tool_calls: Vec<ToolCallRecord>,
    /// 执行步骤计数
    pub step_count: u32,
    /// 最大执行步骤
    pub max_steps: u32,
    /// 执行开始时间
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 上下文元数据
    pub metadata: HashMap<String, String>,
}

/// 工具调用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    /// 工具ID
    pub tool_id: String,
    /// 工具名称
    pub tool_name: String,
    /// 调用参数
    pub parameters: Value,
    /// 调用结果
    pub result: Option<Value>,
    /// 调用时间
    pub called_at: chrono::DateTime<chrono::Utc>,
    /// 执行耗时（毫秒）
    pub duration_ms: Option<u64>,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

/// 上下文管理器
/// 
/// 负责管理多个Agent的运行时上下文，支持并发访问和状态同步
pub struct ContextManager {
    /// 活跃的上下文映射
    contexts: Arc<RwLock<HashMap<String, RuntimeContext>>>,
    /// 上下文配置
    config: ContextManagerConfig,
}

/// 上下文管理器配置
#[derive(Debug, Clone)]
pub struct ContextManagerConfig {
    /// 最大上下文数量
    pub max_contexts: usize,
    /// 上下文过期时间（秒）
    pub context_ttl: u64,
    /// 是否启用自动清理
    pub auto_cleanup: bool,
    /// 清理间隔（秒）
    pub cleanup_interval: u64,
}

impl Default for ContextManagerConfig {
    fn default() -> Self {
        Self {
            max_contexts: 1000,
            context_ttl: 3600, // 1小时
            auto_cleanup: true,
            cleanup_interval: 300, // 5分钟
        }
    }
}

impl RuntimeContext {
    /// 创建新的运行时上下文
    pub fn new(session_id: String, run_id: String) -> Self {
        Self {
            session_id,
            thread_id: None,
            run_id,
            user_id: None,
            messages: Vec::new(),
            variables: HashMap::new(),
            available_tools: Vec::new(),
            tool_calls: Vec::new(),
            step_count: 0,
            max_steps: 10,
            started_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// 设置线程ID
    pub fn with_thread_id(mut self, thread_id: String) -> Self {
        self.thread_id = Some(thread_id);
        self
    }

    /// 设置用户ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// 设置最大步骤数
    pub fn with_max_steps(mut self, max_steps: u32) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// 添加消息到历史
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// 设置上下文变量
    pub fn set_variable(&mut self, key: String, value: Value) {
        self.variables.insert(key, value);
    }

    /// 获取上下文变量
    pub fn get_variable(&self, key: &str) -> Option<&Value> {
        self.variables.get(key)
    }

    /// 添加可用工具
    pub fn add_tool(&mut self, tool: Arc<dyn Tool>) {
        self.available_tools.push(tool);
    }

    /// 记录工具调用
    pub fn record_tool_call(&mut self, record: ToolCallRecord) {
        self.tool_calls.push(record);
    }

    /// 增加步骤计数
    pub fn increment_step(&mut self) -> bool {
        self.step_count += 1;
        self.step_count <= self.max_steps
    }

    /// 检查是否达到最大步骤
    pub fn is_max_steps_reached(&self) -> bool {
        self.step_count >= self.max_steps
    }

    /// 获取执行时长
    pub fn get_duration(&self) -> chrono::Duration {
        chrono::Utc::now() - self.started_at
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// 克隆上下文（用于并发场景）
    pub fn clone_for_parallel(&self) -> Self {
        let mut cloned = self.clone();
        cloned.run_id = format!("{}-parallel-{}", self.run_id, uuid::Uuid::new_v4());
        cloned
    }
}

impl ContextManager {
    /// 创建新的上下文管理器
    pub fn new(config: ContextManagerConfig) -> Self {
        let manager = Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        // 启动自动清理任务
        if manager.config.auto_cleanup {
            manager.start_cleanup_task();
        }

        manager
    }

    /// 创建新的运行时上下文
    pub async fn create_context(&self, session_id: String, run_id: String) -> Result<RuntimeContext> {
        let context = RuntimeContext::new(session_id.clone(), run_id);
        
        let mut contexts = self.contexts.write().await;
        
        // 检查容量限制
        if contexts.len() >= self.config.max_contexts {
            // 移除最旧的上下文
            if let Some(oldest_key) = contexts.keys().next().cloned() {
                contexts.remove(&oldest_key);
            }
        }
        
        contexts.insert(session_id, context.clone());
        Ok(context)
    }

    /// 获取运行时上下文
    pub async fn get_context(&self, session_id: &str) -> Option<RuntimeContext> {
        let contexts = self.contexts.read().await;
        contexts.get(session_id).cloned()
    }

    /// 更新运行时上下文
    pub async fn update_context(&self, session_id: &str, context: RuntimeContext) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        contexts.insert(session_id.to_string(), context);
        Ok(())
    }

    /// 删除运行时上下文
    pub async fn remove_context(&self, session_id: &str) -> Option<RuntimeContext> {
        let mut contexts = self.contexts.write().await;
        contexts.remove(session_id)
    }

    /// 获取活跃上下文数量
    pub async fn active_context_count(&self) -> usize {
        let contexts = self.contexts.read().await;
        contexts.len()
    }

    /// 启动清理任务
    fn start_cleanup_task(&self) {
        let contexts = Arc::clone(&self.contexts);
        let ttl = self.config.context_ttl;
        let interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval));
            
            loop {
                interval.tick().await;
                
                let now = chrono::Utc::now();
                let mut contexts = contexts.write().await;
                
                // 移除过期的上下文
                contexts.retain(|_, context| {
                    let age = now - context.started_at;
                    age.num_seconds() < ttl as i64
                });
            }
        });
    }
}

/// 创建默认的上下文管理器
pub fn create_context_manager() -> ContextManager {
    ContextManager::new(ContextManagerConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_context_creation() {
        let context = RuntimeContext::new("session1".to_string(), "run1".to_string());
        assert_eq!(context.session_id, "session1");
        assert_eq!(context.run_id, "run1");
        assert_eq!(context.step_count, 0);
        assert_eq!(context.max_steps, 10);
    }

    #[tokio::test]
    async fn test_context_manager() {
        let manager = create_context_manager();
        
        // 创建上下文
        let context = manager.create_context("session1".to_string(), "run1".to_string()).await.unwrap();
        assert_eq!(context.session_id, "session1");
        
        // 获取上下文
        let retrieved = manager.get_context("session1").await.unwrap();
        assert_eq!(retrieved.session_id, "session1");
        
        // 删除上下文
        let removed = manager.remove_context("session1").await.unwrap();
        assert_eq!(removed.session_id, "session1");
        
        // 确认已删除
        assert!(manager.get_context("session1").await.is_none());
    }

    #[test]
    fn test_context_variables() {
        let mut context = RuntimeContext::new("session1".to_string(), "run1".to_string());
        
        // 设置变量
        context.set_variable("user_name".to_string(), Value::String("Alice".to_string()));
        context.set_variable("age".to_string(), Value::Number(serde_json::Number::from(25)));
        
        // 获取变量
        assert_eq!(context.get_variable("user_name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(context.get_variable("age"), Some(&Value::Number(serde_json::Number::from(25))));
        assert_eq!(context.get_variable("unknown"), None);
    }

    #[test]
    fn test_step_counting() {
        let mut context = RuntimeContext::new("session1".to_string(), "run1".to_string())
            .with_max_steps(3);
        
        // 测试步骤计数
        assert!(!context.is_max_steps_reached());
        assert!(context.increment_step()); // 步骤 1
        assert!(context.increment_step()); // 步骤 2
        assert!(context.increment_step()); // 步骤 3
        assert!(!context.increment_step()); // 步骤 4，超过限制
        assert!(context.is_max_steps_reached());
    }
}
