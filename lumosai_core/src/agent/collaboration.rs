//! 多 Agent 协作模块
//!
//! 实现 Agent 团队协作、任务分配和编排，对标 CrewAI 的多 Agent 能力

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

use super::communication::AgentCommunicationManager;
use super::Agent;
use crate::error::{Error, Result};

/// 任务状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 待处理
    Pending,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// Agent 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// 任务 ID
    pub id: String,
    /// 任务描述
    pub description: String,
    /// 期望输出
    pub expected_output: Option<String>,
    /// 分配的 Agent ID
    pub agent_id: Option<String>,
    /// 任务状态
    pub status: TaskStatus,
    /// 任务结果
    pub result: Option<String>,
    /// 任务优先级（1-10，10 最高）
    pub priority: u8,
    /// 依赖的任务 ID 列表
    pub dependencies: Vec<String>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 创建时间
    pub created_at: i64,
    /// 开始时间
    pub started_at: Option<i64>,
    /// 完成时间
    pub completed_at: Option<i64>,
    /// 错误信息
    pub error: Option<String>,
}

impl AgentTask {
    /// 创建新任务
    pub fn new(description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description,
            expected_output: None,
            agent_id: None,
            status: TaskStatus::Pending,
            result: None,
            priority: 5,
            dependencies: Vec::new(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now().timestamp(),
            started_at: None,
            completed_at: None,
            error: None,
        }
    }

    /// 设置期望输出
    pub fn with_expected_output(mut self, expected_output: String) -> Self {
        self.expected_output = Some(expected_output);
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }

    /// 添加依赖任务
    pub fn with_dependency(mut self, task_id: String) -> Self {
        self.dependencies.push(task_id);
        self
    }

    /// 分配给 Agent
    pub fn assign_to(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// 标记为进行中
    pub fn mark_in_progress(&mut self) {
        self.status = TaskStatus::InProgress;
        self.started_at = Some(chrono::Utc::now().timestamp());
    }

    /// 标记为完成
    pub fn mark_completed(&mut self, result: String) {
        self.status = TaskStatus::Completed;
        self.result = Some(result);
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }

    /// 标记为失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }
}

/// 协作模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollaborationMode {
    /// 顺序执行（一个接一个）
    Sequential,
    /// 并行执行（同时执行）
    Parallel,
    /// 层级执行（有管理者协调）
    Hierarchical,
}

/// Agent 角色定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRole {
    /// 角色名称
    pub name: String,
    /// 角色目标
    pub goal: String,
    /// 角色背景故事
    pub backstory: String,
    /// 允许委托任务
    pub allow_delegation: bool,
    /// 详细程度（verbose）
    pub verbose: bool,
}

impl AgentRole {
    /// 创建新角色
    pub fn new(name: String, goal: String, backstory: String) -> Self {
        Self {
            name,
            goal,
            backstory,
            allow_delegation: false,
            verbose: false,
        }
    }

    /// 允许委托
    pub fn with_delegation(mut self) -> Self {
        self.allow_delegation = true;
        self
    }

    /// 启用详细输出
    pub fn with_verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
}

/// Agent 团队（Crew）
pub struct Crew {
    /// 团队 ID
    id: String,
    /// 团队名称
    name: String,
    /// Agent 列表（Agent ID -> Agent）
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
    /// Agent 角色（Agent ID -> Role）
    roles: Arc<RwLock<HashMap<String, AgentRole>>>,
    /// 任务列表
    tasks: Arc<RwLock<Vec<AgentTask>>>,
    /// 协作模式
    mode: CollaborationMode,
    /// 通信管理器
    communication: Arc<AgentCommunicationManager>,
    /// 任务队列
    task_queue: Arc<RwLock<Vec<String>>>,
    /// 并发限制
    concurrency_limit: Arc<Semaphore>,
    /// 最大并发数
    max_concurrent_tasks: usize,
}

impl Crew {
    /// 创建新团队
    pub fn new(name: String, mode: CollaborationMode, max_concurrent_tasks: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            agents: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(Vec::new())),
            mode,
            communication: Arc::new(AgentCommunicationManager::new(Default::default())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
            concurrency_limit: Arc::new(Semaphore::new(max_concurrent_tasks)),
            max_concurrent_tasks,
        }
    }

    /// 添加 Agent 到团队
    pub async fn add_agent(
        &self,
        agent_id: String,
        agent: Arc<dyn Agent>,
        role: AgentRole,
    ) -> Result<()> {
        // 注册 Agent 到通信管理器
        self.communication
            .register_agent(agent_id.clone(), agent.clone())
            .await?;

        // 添加到团队
        self.agents.write().await.insert(agent_id.clone(), agent);
        self.roles.write().await.insert(agent_id.clone(), role);

        tracing::info!("Agent {} added to crew {}", agent_id, self.name);
        Ok(())
    }

    /// 添加任务
    pub async fn add_task(&self, task: AgentTask) -> Result<()> {
        let task_id = task.id.clone();
        self.tasks.write().await.push(task);
        self.task_queue.write().await.push(task_id);
        tracing::info!("Task added to crew {}", self.name);
        Ok(())
    }

    /// 执行团队任务
    pub async fn kickoff(&self) -> Result<Vec<AgentTask>> {
        tracing::info!(
            "Crew {} starting execution in {:?} mode",
            self.name,
            self.mode
        );

        match self.mode {
            CollaborationMode::Sequential => self.execute_sequential().await,
            CollaborationMode::Parallel => self.execute_parallel().await,
            CollaborationMode::Hierarchical => self.execute_hierarchical().await,
        }
    }

    /// 顺序执行任务
    async fn execute_sequential(&self) -> Result<Vec<AgentTask>> {
        let mut completed_tasks = Vec::new();
        let task_queue = self.task_queue.read().await.clone();

        for task_id in task_queue {
            let task = self.execute_task(&task_id).await?;
            completed_tasks.push(task);
        }

        Ok(completed_tasks)
    }

    /// 并行执行任务
    async fn execute_parallel(&self) -> Result<Vec<AgentTask>> {
        let task_queue = self.task_queue.read().await.clone();
        let mut handles = Vec::new();

        for task_id in task_queue {
            let crew = self.clone_arc();
            let handle = tokio::spawn(async move { crew.execute_task(&task_id).await });
            handles.push(handle);
        }

        let mut completed_tasks = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(task)) => completed_tasks.push(task),
                Ok(Err(e)) => tracing::error!("Task execution failed: {}", e),
                Err(e) => tracing::error!("Task join failed: {}", e),
            }
        }

        Ok(completed_tasks)
    }

    /// 层级执行任务（由管理者协调）
    async fn execute_hierarchical(&self) -> Result<Vec<AgentTask>> {
        // 简化实现：选择第一个 Agent 作为管理者
        let agents = self.agents.read().await;
        let manager_id = agents
            .keys()
            .next()
            .ok_or_else(|| Error::InvalidInput("No agents in crew".to_string()))?
            .clone();
        drop(agents);

        tracing::info!("Using {} as manager for hierarchical execution", manager_id);

        // 管理者分配任务
        self.execute_sequential().await
    }

    /// 执行单个任务
    async fn execute_task(&self, task_id: &str) -> Result<AgentTask> {
        // 获取信号量许可（限制并发）
        let _permit = self
            .concurrency_limit
            .acquire()
            .await
            .map_err(|e| Error::Internal(format!("Failed to acquire semaphore: {e}")))?;

        // 第一步：检查依赖并收集任务信息
        let (description, _dependencies) = {
            let tasks = self.tasks.read().await;
            let task = tasks
                .iter()
                .find(|t| t.id == task_id)
                .ok_or_else(|| Error::NotFound(format!("Task {task_id} not found")))?;

            // 检查依赖
            for dep_id in &task.dependencies {
                let dep_completed = tasks
                    .iter()
                    .find(|t| &t.id == dep_id)
                    .map(|t| t.status == TaskStatus::Completed)
                    .unwrap_or(false);

                if !dep_completed {
                    return Err(Error::InvalidInput(format!(
                        "Dependency task {dep_id} not completed"
                    )));
                }
            }

            (task.description.clone(), task.dependencies.clone())
        };

        // 第二步：标记为进行中并分配 Agent
        let agent_id = {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .iter_mut()
                .find(|t| t.id == task_id)
                .ok_or_else(|| Error::NotFound(format!("Task {task_id} not found")))?;

            task.mark_in_progress();

            // 分配 Agent（如果未分配）
            if task.agent_id.is_none() {
                let agent_id = self.assign_task_to_agent(task).await?;
                task.agent_id = Some(agent_id);
            }

            task.agent_id.clone().unwrap()
        };

        // 执行任务
        tracing::info!("Executing task {} with agent {}", task_id, agent_id);

        // 这里简化实现，实际应该调用 Agent 的 generate 方法
        let result = format!("Task '{description}' completed by agent {agent_id}");

        // 第三步：更新任务状态
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| Error::NotFound(format!("Task {task_id} not found")))?;

        task.mark_completed(result);
        Ok(task.clone())
    }

    /// 分配任务给 Agent（简单的负载均衡）
    async fn assign_task_to_agent(&self, _task: &AgentTask) -> Result<String> {
        let agents = self.agents.read().await;

        // 简单策略：选择第一个可用的 Agent
        // Smart load balancing strategy implementation plan:
        // 1. Track agent workload: current_tasks_count, last_task_completion_time, avg_task_duration
        // 2. Task priority scoring: urgent (9-10), high (7-8), normal (4-6), low (1-3)
        // 3. Agent capability matching: skill_tags, performance_history, success_rate
        // 4. Load balancing algorithms: round_robin, weighted_least_connections, capability_based
        // 5. Dynamic reassignment: monitor task progress and reassign stuck tasks
        agents
            .keys()
            .next()
            .cloned()
            .ok_or_else(|| Error::InvalidInput("No agents available".to_string()))
    }

    /// 克隆 Arc 包装的 self
    fn clone_arc(&self) -> Arc<Self> {
        Arc::new(Self {
            id: self.id.clone(),
            name: self.name.clone(),
            agents: Arc::clone(&self.agents),
            roles: Arc::clone(&self.roles),
            tasks: Arc::clone(&self.tasks),
            mode: self.mode.clone(),
            communication: Arc::clone(&self.communication),
            task_queue: Arc::clone(&self.task_queue),
            concurrency_limit: Arc::clone(&self.concurrency_limit),
            max_concurrent_tasks: self.max_concurrent_tasks,
        })
    }

    /// 获取团队统计信息
    pub async fn get_stats(&self) -> CrewStats {
        let tasks = self.tasks.read().await;
        let agents = self.agents.read().await;

        let total_tasks = tasks.len();
        let completed_tasks = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let failed_tasks = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Failed)
            .count();
        let in_progress_tasks = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count();

        CrewStats {
            total_agents: agents.len(),
            total_tasks,
            completed_tasks,
            failed_tasks,
            in_progress_tasks,
            pending_tasks: total_tasks - completed_tasks - failed_tasks - in_progress_tasks,
        }
    }
}

/// 团队统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewStats {
    pub total_agents: usize,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub in_progress_tasks: usize,
    pub pending_tasks: usize,
}
