//! Agent编排系统
//! 
//! 提供多Agent协作、编排和调度功能，支持复杂的Agent交互模式。

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock, Mutex};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::agent::trait_def::Agent;
use crate::llm::Message;
use crate::error::{Result, Error};
use super::events::{AgentEvent, EventBus};

/// Agent编排模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationPattern {
    /// 顺序执行
    Sequential,
    /// 并行执行
    Parallel,
    /// 条件分支
    Conditional {
        condition: String,
        true_branch: Box<OrchestrationPattern>,
        false_branch: Box<OrchestrationPattern>,
    },
    /// 循环执行
    Loop {
        condition: String,
        pattern: Box<OrchestrationPattern>,
        max_iterations: Option<usize>,
    },
    /// 竞争执行（第一个完成的获胜）
    Race,
    /// 投票决策
    Voting {
        threshold: f32, // 0.0-1.0
        strategy: VotingStrategy,
    },
}

/// 投票策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingStrategy {
    /// 多数决
    Majority,
    /// 一致同意
    Unanimous,
    /// 加权投票
    Weighted(HashMap<String, f32>),
}

/// Agent角色定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRole {
    /// 角色ID
    pub id: String,
    /// 角色名称
    pub name: String,
    /// 角色描述
    pub description: String,
    /// 权限级别
    pub permission_level: u8,
    /// 专业领域
    pub expertise: Vec<String>,
    /// 可访问的资源
    pub accessible_resources: Vec<String>,
}

/// Agent协作任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationTask {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务描述
    pub description: String,
    /// 参与的Agent角色
    pub participants: Vec<String>,
    /// 编排模式
    pub pattern: OrchestrationPattern,
    /// 输入数据
    pub input: serde_json::Value,
    /// 预期输出格式
    pub expected_output: Option<serde_json::Value>,
    /// 超时时间（秒）
    pub timeout: Option<u64>,
    /// 重试配置
    pub retry_config: Option<RetryConfig>,
}

/// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_attempts: usize,
    /// 重试间隔（毫秒）
    pub delay_ms: u64,
    /// 指数退避
    pub exponential_backoff: bool,
}

/// Agent执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentExecutionState {
    /// 等待中
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed(serde_json::Value),
    /// 失败
    Failed(String),
    /// 超时
    Timeout,
    /// 被取消
    Cancelled,
}

/// 协作会话
#[derive(Debug)]
pub struct CollaborationSession {
    /// 会话ID
    pub id: String,
    /// 任务定义
    pub task: CollaborationTask,
    /// 参与的Agent实例
    pub agents: HashMap<String, Arc<dyn Agent>>,
    /// Agent状态
    pub agent_states: Arc<RwLock<HashMap<String, AgentExecutionState>>>,
    /// 消息历史
    pub message_history: Arc<RwLock<Vec<Message>>>,
    /// 会话上下文
    pub context: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// 事件总线
    pub event_bus: Arc<EventBus>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: Arc<Mutex<DateTime<Utc>>>,
}

impl CollaborationSession {
    /// 创建新的协作会话
    pub fn new(
        task: CollaborationTask,
        agents: HashMap<String, Arc<dyn Agent>>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // 初始化Agent状态
        let mut agent_states = HashMap::new();
        for agent_id in &task.participants {
            agent_states.insert(agent_id.clone(), AgentExecutionState::Pending);
        }
        
        Self {
            id: session_id,
            task,
            agents,
            agent_states: Arc::new(RwLock::new(agent_states)),
            message_history: Arc::new(RwLock::new(Vec::new())),
            context: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
            created_at: now,
            updated_at: Arc::new(Mutex::new(now)),
        }
    }
    
    /// 添加消息到历史
    pub async fn add_message(&self, message: Message) {
        let mut history = self.message_history.write().await;
        history.push(message);
        
        // 更新时间戳
        let mut updated_at = self.updated_at.lock().await;
        *updated_at = Utc::now();
    }
    
    /// 更新Agent状态
    pub async fn update_agent_state(&self, agent_id: &str, state: AgentExecutionState) {
        let mut states = self.agent_states.write().await;
        states.insert(agent_id.to_string(), state.clone());
        
        // 发送状态变更事件
        let event = AgentEvent::StateChanged {
            agent_id: agent_id.to_string(),
            old_state: "unknown".to_string(), // 简化实现
            new_state: format!("{:?}", state),
            timestamp: Utc::now(),
        };
        
        if let Err(e) = self.event_bus.publish(event).await {
            eprintln!("Failed to publish state change event: {}", e);
        }
        
        // 更新时间戳
        let mut updated_at = self.updated_at.lock().await;
        *updated_at = Utc::now();
    }
    
    /// 获取Agent状态
    pub async fn get_agent_state(&self, agent_id: &str) -> Option<AgentExecutionState> {
        let states = self.agent_states.read().await;
        states.get(agent_id).cloned()
    }
    
    /// 检查会话是否完成
    pub async fn is_completed(&self) -> bool {
        let states = self.agent_states.read().await;
        states.values().all(|state| {
            matches!(state, 
                AgentExecutionState::Completed(_) | 
                AgentExecutionState::Failed(_) | 
                AgentExecutionState::Timeout | 
                AgentExecutionState::Cancelled
            )
        })
    }
    
    /// 获取会话结果
    pub async fn get_results(&self) -> HashMap<String, AgentExecutionState> {
        let states = self.agent_states.read().await;
        states.clone()
    }
}

/// Agent编排器trait
#[async_trait]
pub trait AgentOrchestrator: Send + Sync {
    /// 执行协作任务
    async fn execute_collaboration(&self, session: &mut CollaborationSession) -> Result<serde_json::Value>;
    
    /// 取消执行
    async fn cancel_execution(&self, session_id: &str) -> Result<()>;
    
    /// 获取执行状态
    async fn get_execution_status(&self, session_id: &str) -> Result<HashMap<String, AgentExecutionState>>;
}

/// 基本Agent编排器实现
pub struct BasicOrchestrator {
    /// 活跃会话
    active_sessions: Arc<RwLock<HashMap<String, Arc<Mutex<CollaborationSession>>>>>,
    /// 事件总线
    event_bus: Arc<EventBus>,
}

impl BasicOrchestrator {
    /// 创建新的编排器
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
        }
    }
    
    /// 创建协作会话
    pub async fn create_session(
        &self,
        task: CollaborationTask,
        agents: HashMap<String, Arc<dyn Agent>>,
    ) -> Result<String> {
        let session = CollaborationSession::new(task, agents, self.event_bus.clone());
        let session_id = session.id.clone();
        
        // 添加到活跃会话
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session_id.clone(), Arc::new(Mutex::new(session)));
        
        Ok(session_id)
    }
    
    /// 获取会话
    pub async fn get_session(&self, session_id: &str) -> Option<Arc<Mutex<CollaborationSession>>> {
        let sessions = self.active_sessions.read().await;
        sessions.get(session_id).cloned()
    }
    
    /// 移除会话
    pub async fn remove_session(&self, session_id: &str) {
        let mut sessions = self.active_sessions.write().await;
        sessions.remove(session_id);
    }
    
    /// 执行顺序模式
    async fn execute_sequential(&self, session: &mut CollaborationSession) -> Result<serde_json::Value> {
        let mut results = serde_json::Map::new();
        
        for agent_id in &session.task.participants {
            if let Some(agent) = session.agents.get(agent_id) {
                // 更新状态为运行中
                session.update_agent_state(agent_id, AgentExecutionState::Running).await;
                
                // 执行Agent
                match self.execute_single_agent(agent.clone(), &session.task.input).await {
                    Ok(result) => {
                        results.insert(agent_id.clone(), result.clone());
                        session.update_agent_state(agent_id, AgentExecutionState::Completed(result)).await;
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        session.update_agent_state(agent_id, AgentExecutionState::Failed(error_msg.clone())).await;
                        return Err(Error::Agent(format!("Agent {} failed: {}", agent_id, error_msg)));
                    }
                }
            }
        }
        
        Ok(serde_json::Value::Object(results))
    }
    
    /// 执行并行模式
    async fn execute_parallel(&self, session: &mut CollaborationSession) -> Result<serde_json::Value> {
        let mut handles = Vec::new();

        // 启动所有Agent
        for agent_id in &session.task.participants {
            if let Some(agent) = session.agents.get(agent_id) {
                session.update_agent_state(agent_id, AgentExecutionState::Running).await;

                let agent_clone = agent.clone();
                let input_clone = session.task.input.clone();
                let agent_id_clone = agent_id.clone();

                // 创建一个闭包来捕获self的方法
                let execute_fn = |agent: Arc<dyn Agent>, input: &serde_json::Value| async move {
                    // 简化实现：将JSON输入转换为消息
                    let input_text = input.to_string();
                    let messages = vec![crate::llm::Message {
                        role: crate::llm::Role::User,
                        content: input_text,
                        metadata: None,
                        name: None,
                    }];

                    let options = crate::agent::types::AgentGenerateOptions::default();
                    let result = agent.generate(&messages, &options).await?;

                    Ok(serde_json::Value::String(result.response))
                };

                let handle = tokio::spawn(async move {
                    let result = execute_fn(agent_clone, &input_clone).await;
                    (agent_id_clone, result)
                });

                handles.push(handle);
            }
        }
        
        // 等待所有Agent完成
        let mut results = serde_json::Map::new();
        for handle in handles {
            match handle.await {
                Ok((agent_id, result)) => {
                    match result {
                        Ok(value) => {
                            results.insert(agent_id.clone(), value.clone());
                            session.update_agent_state(&agent_id, AgentExecutionState::Completed(value)).await;
                        }
                        Err(e) => {
                            let error_msg = e.to_string();
                            session.update_agent_state(&agent_id, AgentExecutionState::Failed(error_msg)).await;
                        }
                    }
                }
                Err(e) => {
                    return Err(Error::Agent(format!("Task join error: {}", e)));
                }
            }
        }
        
        Ok(serde_json::Value::Object(results))
    }
    
    /// 执行单个Agent
    async fn execute_single_agent(&self, agent: Arc<dyn Agent>, input: &serde_json::Value) -> Result<serde_json::Value> {
        // 简化实现：将JSON输入转换为消息
        let input_text = input.to_string();
        let messages = vec![crate::llm::Message {
            role: crate::llm::Role::User,
            content: input_text,
            metadata: None,
            name: None,
        }];
        
        let options = crate::agent::types::AgentGenerateOptions::default();
        let result = agent.generate(&messages, &options).await?;
        
        Ok(serde_json::Value::String(result.response))
    }
}

#[async_trait]
impl AgentOrchestrator for BasicOrchestrator {
    async fn execute_collaboration(&self, session: &mut CollaborationSession) -> Result<serde_json::Value> {
        match &session.task.pattern {
            OrchestrationPattern::Sequential => {
                self.execute_sequential(session).await
            }
            OrchestrationPattern::Parallel => {
                self.execute_parallel(session).await
            }
            _ => {
                Err(Error::Agent("Unsupported orchestration pattern".to_string()))
            }
        }
    }
    
    async fn cancel_execution(&self, session_id: &str) -> Result<()> {
        if let Some(session_arc) = self.get_session(session_id).await {
            let mut session = session_arc.lock().await;
            
            // 将所有运行中的Agent标记为取消
            let agent_ids: Vec<String> = session.task.participants.clone();
            for agent_id in agent_ids {
                if let Some(state) = session.get_agent_state(&agent_id).await {
                    if matches!(state, AgentExecutionState::Running | AgentExecutionState::Pending) {
                        session.update_agent_state(&agent_id, AgentExecutionState::Cancelled).await;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn get_execution_status(&self, session_id: &str) -> Result<HashMap<String, AgentExecutionState>> {
        if let Some(session_arc) = self.get_session(session_id).await {
            let session = session_arc.lock().await;
            Ok(session.get_results().await)
        } else {
            Err(Error::NotFound(format!("Session not found: {}", session_id)))
        }
    }
}
