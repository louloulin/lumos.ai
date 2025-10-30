//! 多 Agent 协作模块
//!
//! 实现 Agent 团队协作、任务分配和编排，对标 CrewAI 的多 Agent 能力

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

use super::communication::{AgentCommunicationManager, AgentInfo, AgentLoadInfo, AgentStatus};
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

    // ===== SOP 执行模式（深度融合） =====
    /// SOP React 模式：事件驱动，Agent 根据消息反应
    ///
    /// 特点：
    /// - Agent 通过 `sop_watch()` 订阅感兴趣的消息类型
    /// - 收到消息后通过 `sop_think()` 决定行动
    /// - 通过 `sop_act()` 执行行动并发送新消息
    /// - 适用于动态、响应式的协作场景
    SopReact,

    /// SOP ByOrder 模式：按预定义顺序执行
    ///
    /// 特点：
    /// - Agent 按照添加顺序依次执行
    /// - 每个 Agent 完成后传递消息给下一个
    /// - 适用于流水线式的协作场景
    SopByOrder,

    /// SOP PlanAndAct 模式：先规划后执行
    ///
    /// 特点：
    /// - 第一阶段：所有 Agent 通过 `sop_think()` 生成计划
    /// - 第二阶段：根据计划依次执行 `sop_act()`
    /// - 适用于需要全局协调的复杂任务
    SopPlanAndAct,
}

/// Agent 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// 当前任务数量
    pub current_tasks_count: usize,
    /// 最后任务完成时间
    pub last_task_completion_time: Option<i64>,
    /// 平均任务持续时间（毫秒）
    pub avg_task_duration: f64,
    /// 成功率（0.0-1.0）
    pub success_rate: f64,
    /// 总完成任务数
    pub total_completed_tasks: usize,
    /// 总失败任务数
    pub total_failed_tasks: usize,
    /// 技能标签
    pub skill_tags: Vec<String>,
    /// 性能历史（最近10个任务的完成时间）
    pub performance_history: Vec<i64>,
}

impl AgentMetrics {
    /// 创建新的性能指标
    pub fn new() -> Self {
        Self {
            current_tasks_count: 0,
            last_task_completion_time: None,
            avg_task_duration: 1000.0, // 默认1秒
            success_rate: 1.0,
            total_completed_tasks: 0,
            total_failed_tasks: 0,
            skill_tags: Vec::new(),
            performance_history: Vec::new(),
        }
    }

    /// 更新任务开始
    pub fn start_task(&mut self) {
        self.current_tasks_count += 1;
    }

    /// 更新任务完成
    pub fn complete_task(&mut self, duration_ms: i64) {
        self.current_tasks_count = self.current_tasks_count.saturating_sub(1);
        self.last_task_completion_time = Some(chrono::Utc::now().timestamp());
        self.total_completed_tasks += 1;

        // 更新性能历史
        self.performance_history.push(duration_ms);
        if self.performance_history.len() > 10 {
            self.performance_history.remove(0);
        }

        // 重新计算平均持续时间
        if !self.performance_history.is_empty() {
            self.avg_task_duration = self.performance_history.iter().sum::<i64>() as f64
                / self.performance_history.len() as f64;
        }

        // 重新计算成功率
        let total_tasks = self.total_completed_tasks + self.total_failed_tasks;
        if total_tasks > 0 {
            self.success_rate = self.total_completed_tasks as f64 / total_tasks as f64;
        }
    }

    /// 更新任务失败
    pub fn fail_task(&mut self) {
        self.current_tasks_count = self.current_tasks_count.saturating_sub(1);
        self.total_failed_tasks += 1;

        // 重新计算成功率
        let total_tasks = self.total_completed_tasks + self.total_failed_tasks;
        if total_tasks > 0 {
            self.success_rate = self.total_completed_tasks as f64 / total_tasks as f64;
        }
    }

    /// 计算负载分数（分数越高，负载越低）
    pub fn load_score(&self) -> f64 {
        // 负载分数 = 成功率权重 * (1 - 当前任务比例) + 性能权重 * (1 / 平均时间)
        let task_load_factor = if self.max_concurrent_tasks() > 0 {
            self.current_tasks_count as f64 / self.max_concurrent_tasks() as f64
        } else {
            0.0
        };

        let performance_factor = 1.0 / (self.avg_task_duration / 1000.0).max(0.1);

        self.success_rate * (1.0 - task_load_factor) * 0.6 + performance_factor * 0.4
    }

    /// 获取最大并发任务数
    fn max_concurrent_tasks(&self) -> usize {
        // 基于成功率和性能动态调整
        let base_capacity = 5;
        let performance_multiplier = (self.success_rate * 2.0).min(3.0);
        (base_capacity as f64 * performance_multiplier) as usize
    }
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
    /// 技能标签
    pub skills: Vec<String>,
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
            skills: Vec::new(),
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

    /// 添加技能标签
    pub fn with_skills(mut self, skills: Vec<String>) -> Self {
        self.skills = skills;
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
    /// Agent 性能指标（Agent ID -> Metrics）
    metrics: Arc<RwLock<HashMap<String, AgentMetrics>>>,
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

impl Clone for Crew {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            agents: Arc::clone(&self.agents),
            roles: Arc::clone(&self.roles),
            metrics: Arc::clone(&self.metrics),
            tasks: Arc::clone(&self.tasks),
            mode: self.mode.clone(),
            communication: Arc::clone(&self.communication),
            task_queue: Arc::clone(&self.task_queue),
            concurrency_limit: Arc::clone(&self.concurrency_limit),
            max_concurrent_tasks: self.max_concurrent_tasks,
        }
    }
}

impl Crew {
    /// 创建新团队
    pub fn new(name: String, mode: CollaborationMode, max_concurrent_tasks: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            agents: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
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
        // 创建AgentInfo
        let agent_info = AgentInfo {
            agent_id: agent_id.clone(),
            name: format!("Agent {}", agent_id),
            status: AgentStatus::Active,
            capabilities: vec![format!("{:?}", role).to_string()],
            load_info: AgentLoadInfo {
                cpu_usage: 20.0,
                memory_usage: 30.0,
                current_tasks: 0,
                max_tasks: 10,
                avg_response_time: 100.0,
            },
            metadata: std::collections::HashMap::new(),
            registered_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        };

        // 注册 Agent 到通信管理器
        self.communication
            .register_agent(agent_id.clone(), agent_info)
            .await?;

        // 添加到团队
        self.agents.write().await.insert(agent_id.clone(), agent);
        self.roles
            .write()
            .await
            .insert(agent_id.clone(), role.clone());

        // 初始化性能指标
        let mut metrics = AgentMetrics::new();
        metrics.skill_tags = role.skills.clone();
        self.metrics.write().await.insert(agent_id.clone(), metrics);

        tracing::info!(
            "Agent {} added to crew {} with {} skills",
            agent_id,
            self.name,
            role.skills.len()
        );
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

            // SOP 执行模式（深度融合）
            CollaborationMode::SopReact => self.execute_sop_react().await,
            CollaborationMode::SopByOrder => self.execute_sop_by_order().await,
            CollaborationMode::SopPlanAndAct => self.execute_sop_plan_and_act().await,
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

    // ===== SOP 执行模式实现（深度融合） =====

    /// SOP React 模式：事件驱动执行
    ///
    /// Agent 通过 watch-think-act 循环响应消息
    async fn execute_sop_react(&self) -> Result<Vec<AgentTask>> {
        use super::sop_environment::SopEnvironment;
        use super::sop_types::SopExecutionMode;

        tracing::info!("Executing SOP React mode for crew {}", self.name);

        // 创建 SOP 环境（复用当前 Crew）
        let sop_env = SopEnvironment::from_crew(Arc::new(self.clone()), SopExecutionMode::React);

        // 执行 SOP 流程
        // 注意：这里暂时返回空任务列表，因为 SOP 使用不同的任务模型
        // 未来可以将 SOP 的执行结果转换为 AgentTask
        sop_env.run(None).await?;

        // 返回当前任务列表
        Ok(self.tasks.read().await.clone())
    }

    /// SOP ByOrder 模式：按顺序执行
    ///
    /// Agent 按照添加顺序依次执行
    async fn execute_sop_by_order(&self) -> Result<Vec<AgentTask>> {
        use super::sop_environment::SopEnvironment;
        use super::sop_types::SopExecutionMode;

        tracing::info!("Executing SOP ByOrder mode for crew {}", self.name);

        // 创建 SOP 环境（复用当前 Crew）
        let sop_env = SopEnvironment::from_crew(Arc::new(self.clone()), SopExecutionMode::ByOrder);

        // 执行 SOP 流程
        sop_env.run(None).await?;

        // 返回当前任务列表
        Ok(self.tasks.read().await.clone())
    }

    /// SOP PlanAndAct 模式：先规划后执行
    ///
    /// 第一阶段：所有 Agent 生成计划
    /// 第二阶段：根据计划执行
    async fn execute_sop_plan_and_act(&self) -> Result<Vec<AgentTask>> {
        use super::sop_environment::SopEnvironment;
        use super::sop_types::SopExecutionMode;

        tracing::info!("Executing SOP PlanAndAct mode for crew {}", self.name);

        // 创建 SOP 环境（复用当前 Crew）
        let sop_env =
            SopEnvironment::from_crew(Arc::new(self.clone()), SopExecutionMode::PlanAndAct);

        // 执行 SOP 流程
        sop_env.run(None).await?;

        // 返回当前任务列表
        Ok(self.tasks.read().await.clone())
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

        // 执行任务并监控性能
        tracing::info!("Executing task {} with agent {}", task_id, agent_id);
        let start_time = std::time::Instant::now();

        // 更新 Agent 指标 - 任务开始
        {
            let mut metrics = self.metrics.write().await;
            if let Some(agent_metrics) = metrics.get_mut(&agent_id) {
                agent_metrics.start_task();
            }
        }

        // 实际执行任务 - 调用 Agent 的 generate 方法
        let agents = self.agents.read().await;
        let result = if let Some(agent) = agents.get(&agent_id) {
            // 获取任务完整信息用于构建提示词
            let task_info = {
                let tasks = self.tasks.read().await;
                tasks.iter().find(|t| t.id == task_id).cloned()
            };

            let task_prompt = if let Some(task) = task_info {
                format!(
                    "任务描述: {}\n期望输出: {}\n优先级: {}\n元数据: {:#?}",
                    description,
                    task.expected_output.as_deref().unwrap_or("无"),
                    task.priority,
                    task.metadata
                )
            } else {
                format!(
                    "任务描述: {}\n期望输出: 无\n优先级: 5\n元数据: {{}}",
                    description
                )
            };

            use crate::agent::types::AgentGenerateOptions;
            use crate::llm::{Message, Role};

            // 构建任务消息
            let task_message = Message::new(Role::User, task_prompt.clone(), None, None);
            let messages = vec![task_message];
            let options = AgentGenerateOptions::default();

            match agent.generate(&messages, &options).await {
                Ok(response) => {
                    let duration = start_time.elapsed().as_millis() as i64;

                    // 更新 Agent 指标 - 任务成功完成
                    {
                        let mut metrics = self.metrics.write().await;
                        if let Some(agent_metrics) = metrics.get_mut(&agent_id) {
                            agent_metrics.complete_task(duration);
                        }
                    }

                    tracing::info!("Task {} completed successfully in {}ms", task_id, duration);
                    response.response.clone()
                }
                Err(e) => {
                    let duration = start_time.elapsed().as_millis() as i64;

                    // 更新 Agent 指标 - 任务失败
                    {
                        let mut metrics = self.metrics.write().await;
                        if let Some(agent_metrics) = metrics.get_mut(&agent_id) {
                            agent_metrics.fail_task();
                        }
                    }

                    tracing::error!("Task {} failed in {}ms: {}", task_id, duration, e);
                    return Err(Error::Agent(format!(
                        "Agent {} failed to execute task: {}",
                        agent_id, e
                    )));
                }
            }
        } else {
            return Err(Error::NotFound(format!("Agent {} not found", agent_id)));
        };

        // 第三步：更新任务状态
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| Error::NotFound(format!("Task {task_id} not found")))?;

        task.mark_completed(result);
        Ok(task.clone())
    }

    /// 分配任务给 Agent（智能负载均衡）
    async fn assign_task_to_agent(&self, task: &AgentTask) -> Result<String> {
        let agents = self.agents.read().await;
        let roles = self.roles.read().await;
        let metrics = self.metrics.read().await;

        // 智能负载均衡策略：基于性能指标和任务需求
        let mut best_agent_id: Option<String> = None;
        let mut best_score = -1.0;

        for agent_id in agents.keys() {
            // 获取Agent的性能指标
            if let Some(agent_metrics) = metrics.get(agent_id) {
                let agent_role = roles.get(agent_id);

                // 1. 计算负载分数（分数越高，越适合分配任务）
                let load_score = agent_metrics.load_score();

                // 2. 任务优先级评分
                let priority_score = self.calculate_priority_score(task.priority);

                // 3. 技能匹配评分
                let skill_match_score =
                    self.calculate_skill_match_score(&task.metadata, agent_role, agent_metrics);

                // 4. 综合评分
                let total_score =
                    load_score * 0.4 + priority_score * 0.3 + skill_match_score as f64 * 0.3;

                // 5. 检查Agent是否可用（没有超过最大并发数）
                let is_available =
                    agent_metrics.current_tasks_count < agent_metrics.max_concurrent_tasks();

                if is_available && total_score > best_score {
                    best_score = total_score;
                    best_agent_id = Some(agent_id.clone());
                }
            }
        }

        best_agent_id
            .ok_or_else(|| Error::InvalidInput("No suitable agent available for task".to_string()))
    }

    /// 计算任务优先级分数
    fn calculate_priority_score(&self, priority: u8) -> f64 {
        // urgent (9-10): 1.0 分
        // high (7-8): 0.8 分
        // normal (4-6): 0.6 分
        // low (1-3): 0.4 分
        match priority {
            9..=10 => 1.0,
            7..=8 => 0.8,
            4..=6 => 0.6,
            1..=3 => 0.4,
            _ => 0.2,
        }
    }

    /// 计算技能匹配分数
    fn calculate_skill_match_score(
        &self,
        task_metadata: &HashMap<String, serde_json::Value>,
        agent_role: Option<&AgentRole>,
        agent_metrics: &AgentMetrics,
    ) -> f32 {
        let agent_skills = if let Some(role) = agent_role {
            &role.skills
        } else {
            &agent_metrics.skill_tags
        };

        if agent_skills.is_empty() {
            return 0.5; // 中等匹配度
        }

        // 检查任务要求的技能（从metadata中提取）
        let required_skills = self.extract_required_skills(task_metadata);

        if required_skills.is_empty() {
            return 0.7; // 如果任务没有技能要求，给予中等分数
        }

        // 计算技能匹配度
        let mut match_count = 0;
        for skill in &required_skills {
            if agent_skills.contains(skill) {
                match_count += 1;
            }
        }

        match_count as f32 / required_skills.len() as f32
    }

    /// 从任务元数据中提取所需技能
    fn extract_required_skills(
        &self,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Vec<String> {
        let mut skills = Vec::new();

        // 从元数据中查找技能相关的字段
        if let Some(skill_value) = metadata.get("required_skills") {
            if let Ok(skill_array) = serde_json::from_value::<Vec<String>>(skill_value.clone()) {
                skills.extend(skill_array);
            }
        }

        // 从元数据中查找其他可能的技能字段
        if let Some(abilities) = metadata.get("abilities") {
            if let Ok(ability_array) = serde_json::from_value::<Vec<String>>(abilities.clone()) {
                skills.extend(ability_array);
            }
        }

        // 如果有domain字段，可以作为技能要求
        if let Some(domain) = metadata.get("domain") {
            if let Some(domain_str) = domain.as_str() {
                skills.push(domain_str.to_string());
            }
        }

        skills
    }

    /// 克隆 Arc 包装的 self
    fn clone_arc(&self) -> Arc<Self> {
        Arc::new(Self {
            id: self.id.clone(),
            name: self.name.clone(),
            agents: Arc::clone(&self.agents),
            roles: Arc::clone(&self.roles),
            metrics: Arc::clone(&self.metrics),
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

    // ===== SOP 辅助方法（用于 SopEnvironment 访问） =====

    /// 获取所有 Agent（用于 SOP 环境）
    ///
    /// 返回当前 Crew 中的所有 Agent 实例
    pub async fn get_agents(&self) -> Vec<Arc<dyn Agent>> {
        self.agents.read().await.values().cloned().collect()
    }

    /// 获取 Agent ID 列表（用于 SOP 环境）
    ///
    /// 返回当前 Crew 中的所有 Agent ID
    pub async fn get_agent_ids(&self) -> Vec<String> {
        self.agents.read().await.keys().cloned().collect()
    }

    /// 获取通信管理器（用于 SOP 环境）
    ///
    /// 返回当前 Crew 的通信管理器，用于 SOP 消息路由
    pub fn get_communication(&self) -> Arc<AgentCommunicationManager> {
        Arc::clone(&self.communication)
    }

    /// 获取 Agent 性能指标
    pub async fn get_agent_metrics(&self, agent_id: &str) -> Option<AgentMetrics> {
        let metrics = self.metrics.read().await;
        metrics.get(agent_id).cloned()
    }

    /// 获取所有 Agent 的性能指标
    pub async fn get_all_agent_metrics(&self) -> HashMap<String, AgentMetrics> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// 重置 Agent 性能指标
    pub async fn reset_agent_metrics(&self, agent_id: &str) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(agent_metrics) = metrics.get_mut(agent_id) {
            *agent_metrics = AgentMetrics::new();
            tracing::info!("Reset metrics for agent {}", agent_id);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Agent {} not found", agent_id)))
        }
    }
}

/// 任务复杂度等级
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskComplexity {
    /// 简单任务（单步完成）
    Simple,
    /// 中等任务（2-5个子任务）
    Medium,
    /// 复杂任务（5-20个子任务）
    Complex,
    /// 超复杂任务（20+个子任务）
    UltraComplex,
}

/// 任务分解策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecompositionStrategy {
    /// 顺序分解（按步骤分解）
    Sequential,
    /// 并行分解（可并行执行的子任务）
    Parallel,
    /// 混合分解（顺序+并行）
    Hybrid,
    /// 智能分解（基于任务类型自动选择）
    Intelligent,
}

/// 子任务关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTaskRelation {
    /// 父任务ID
    pub parent_id: String,
    /// 子任务ID
    pub child_id: String,
    /// 关系类型
    pub relation_type: TaskRelationType,
    /// 依赖权重（1-10）
    pub weight: u8,
}

/// 任务关系类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskRelationType {
    /// 强依赖（必须完成）
    StrongDependency,
    /// 弱依赖（建议完成）
    WeakDependency,
    /// 可选依赖
    OptionalDependency,
    /// 互斥依赖
    MutexDependency,
}

/// 依赖图节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    /// 任务ID
    pub task_id: String,
    /// 入度（依赖的任务数）
    pub in_degree: usize,
    /// 出度（依赖此任务的任务数）
    pub out_degree: usize,
    /// 邻接节点（此任务依赖的节点）
    pub dependencies: Vec<String>,
    /// 被依赖节点（依赖此任务的节点）
    pub dependents: Vec<String>,
}

/// 任务依赖图
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// 节点映射
    nodes: HashMap<String, DependencyNode>,
    /// 边集合
    edges: Vec<SubTaskRelation>,
    /// 拓扑排序结果
    topological_order: Vec<String>,
    /// 是否存在循环依赖
    has_cycle: bool,
}

impl DependencyGraph {
    /// 创建新的依赖图
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            topological_order: Vec::new(),
            has_cycle: false,
        }
    }

    /// 添加任务节点
    pub fn add_task(&mut self, task_id: String) {
        if !self.nodes.contains_key(&task_id) {
            self.nodes.insert(
                task_id.clone(),
                DependencyNode {
                    task_id: task_id.clone(),
                    in_degree: 0,
                    out_degree: 0,
                    dependencies: Vec::new(),
                    dependents: Vec::new(),
                },
            );
        }
    }

    /// 添加依赖关系
    pub fn add_dependency(&mut self, relation: SubTaskRelation) -> Result<()> {
        // 添加节点（如果不存在）
        self.add_task(relation.parent_id.clone());
        self.add_task(relation.child_id.clone());

        // 检查是否会创建循环依赖
        if self.would_create_cycle(&relation.parent_id, &relation.child_id) {
            return Err(Error::InvalidInput(format!(
                "Adding dependency would create a cycle: {} -> {}",
                relation.parent_id, relation.child_id
            )));
        }

        // 更新节点信息
        if let Some(parent_node) = self.nodes.get_mut(&relation.parent_id) {
            parent_node.dependencies.push(relation.child_id.clone());
            parent_node.out_degree += 1;
        }

        if let Some(child_node) = self.nodes.get_mut(&relation.child_id) {
            child_node.dependents.push(relation.parent_id.clone());
            child_node.in_degree += 1;
        }

        self.edges.push(relation);
        Ok(())
    }

    /// 检查添加依赖是否会创建循环
    fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        // 使用DFS检测从to是否能到达from
        let mut visited = std::collections::HashSet::new();
        self.has_path_dfs(to, from, &mut visited)
    }

    /// DFS检测路径
    fn has_path_dfs(
        &self,
        current: &str,
        target: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> bool {
        if current == target {
            return true;
        }

        if visited.contains(current) {
            return false;
        }

        visited.insert(current.to_string());

        if let Some(node) = self.nodes.get(current) {
            for dependent in &node.dependents {
                if self.has_path_dfs(dependent, target, visited) {
                    return true;
                }
            }
        }

        false
    }

    /// 执行拓扑排序
    pub fn topological_sort(&mut self) -> Result<Vec<String>> {
        if self.nodes.is_empty() {
            return Ok(Vec::new());
        }

        // Kahn算法实现拓扑排序
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();

        // 初始化入度表
        for (task_id, node) in &self.nodes {
            in_degree.insert(task_id.clone(), node.in_degree);
            if node.in_degree == 0 {
                queue.push_back(task_id.clone());
            }
        }

        let mut result = Vec::new();

        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            // 更新依赖此任务的所有节点的入度
            if let Some(node) = self.nodes.get(&current) {
                for dependent in &node.dependents {
                    if let Some(deg) = in_degree.get_mut(dependent) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        // 检查是否有循环依赖
        if result.len() != self.nodes.len() {
            self.has_cycle = true;
            return Err(Error::InvalidInput(
                "Cycle detected in task dependencies".to_string(),
            ));
        }

        self.topological_order = result.clone();
        Ok(result)
    }

    /// 获取关键路径
    pub fn get_critical_path(&self) -> Vec<String> {
        // 简化实现：返回最长的依赖链
        let mut longest_path = Vec::new();
        let mut max_length = 0;

        for task_id in self.nodes.keys() {
            if let Some(path) = self.find_longest_path(task_id) {
                if path.len() > max_length {
                    max_length = path.len();
                    longest_path = path;
                }
            }
        }

        longest_path
    }

    /// 查找从指定节点开始的最长路径
    fn find_longest_path(&self, start: &str) -> Option<Vec<String>> {
        let mut memo = HashMap::new();
        self.dfs_longest_path(start, &mut memo)
    }

    /// DFS查找最长路径
    fn dfs_longest_path(
        &self,
        current: &str,
        memo: &mut HashMap<String, Option<Vec<String>>>,
    ) -> Option<Vec<String>> {
        if let Some(result) = memo.get(current) {
            return result.clone();
        }

        let mut best_path = vec![current.to_string()];

        if let Some(node) = self.nodes.get(current) {
            for dependent in &node.dependents {
                if let Some(mut sub_path) = self.dfs_longest_path(dependent, memo) {
                    if sub_path.len() + 1 > best_path.len() {
                        sub_path.insert(0, current.to_string());
                        best_path = sub_path;
                    }
                }
            }
        }

        memo.insert(current.to_string(), Some(best_path.clone()));
        Some(best_path)
    }

    /// 获取可以立即执行的任务
    pub fn get_ready_tasks(&self) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.in_degree == 0)
            .map(|(task_id, _)| task_id.clone())
            .collect()
    }

    /// 标记任务为完成，更新依赖关系
    pub fn mark_task_completed(&mut self, task_id: &str) {
        let dependents: Vec<String> = if let Some(node) = self.nodes.get(task_id) {
            node.dependents.clone()
        } else {
            return;
        };

        for dependent in dependents {
            if let Some(dep_node) = self.nodes.get_mut(&dependent) {
                dep_node.in_degree = dep_node.in_degree.saturating_sub(1);
            }
        }
    }

    /// 获取图统计信息
    pub fn get_stats(&self) -> GraphStats {
        GraphStats {
            total_nodes: self.nodes.len(),
            total_edges: self.edges.len(),
            has_cycle: self.has_cycle,
            max_depth: self.calculate_max_depth(),
            critical_path_length: self.get_critical_path().len(),
        }
    }

    /// 计算图的最大深度
    fn calculate_max_depth(&self) -> usize {
        let mut max_depth = 0;
        for task_id in self.nodes.keys() {
            max_depth =
                max_depth.max(self.calculate_depth(task_id, &mut std::collections::HashSet::new()));
        }
        max_depth
    }

    /// 计算节点的深度
    fn calculate_depth(
        &self,
        current: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> usize {
        if visited.contains(current) {
            return 0; // 避免循环
        }

        visited.insert(current.to_string());

        let mut max_child_depth = 0;
        if let Some(node) = self.nodes.get(current) {
            for dependent in &node.dependents {
                max_child_depth = max_child_depth.max(self.calculate_depth(dependent, visited));
            }
        }

        visited.remove(current);
        1 + max_child_depth
    }
}

/// 图统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub has_cycle: bool,
    pub max_depth: usize,
    pub critical_path_length: usize,
}

/// 任务分解器接口
#[async_trait]
pub trait TaskDecomposer: Send + Sync {
    /// 分解复杂任务
    async fn decompose_task(&self, task: &AgentTask) -> Result<Vec<AgentTask>>;

    /// 分析任务复杂度
    fn analyze_complexity(&self, task: &AgentTask) -> TaskComplexity;

    /// 选择分解策略
    fn select_strategy(
        &self,
        task: &AgentTask,
        complexity: TaskComplexity,
    ) -> DecompositionStrategy;
}

/// 智能任务分解器
pub struct IntelligentTaskDecomposer {
    /// LLM提供商，用于分析任务
    llm_provider: Arc<dyn crate::llm::LlmProvider>,
    /// 分解规则
    decomposition_rules: HashMap<String, DecompositionPattern>,
}

/// 分解模式
#[derive(Debug, Clone)]
struct DecompositionPattern {
    /// 任务关键词
    keywords: Vec<String>,
    /// 分解策略
    strategy: DecompositionStrategy,
    /// 子任务模板
    subtask_templates: Vec<String>,
    /// 依赖关系模板
    dependency_templates: Vec<(usize, usize, TaskRelationType)>,
}

impl IntelligentTaskDecomposer {
    /// 创建新的智能分解器
    pub fn new(llm_provider: Arc<dyn crate::llm::LlmProvider>) -> Self {
        let mut decomposer = Self {
            llm_provider,
            decomposition_rules: HashMap::new(),
        };

        decomposer.init_default_patterns();
        decomposer
    }

    /// 初始化默认分解模式
    fn init_default_patterns(&mut self) {
        // 数据分析任务模式
        self.decomposition_rules.insert(
            "数据分析".to_string(),
            DecompositionPattern {
                keywords: vec!["分析".to_string(), "数据".to_string(), "报告".to_string()],
                strategy: DecompositionStrategy::Sequential,
                subtask_templates: vec![
                    "收集和清理数据".to_string(),
                    "探索性数据分析".to_string(),
                    "深度分析和建模".to_string(),
                    "生成分析报告".to_string(),
                    "验证结果".to_string(),
                ],
                dependency_templates: vec![
                    (0, 1, TaskRelationType::StrongDependency),
                    (1, 2, TaskRelationType::StrongDependency),
                    (2, 3, TaskRelationType::StrongDependency),
                    (3, 4, TaskRelationType::WeakDependency),
                ],
            },
        );

        // 软件开发任务模式
        self.decomposition_rules.insert(
            "软件开发".to_string(),
            DecompositionPattern {
                keywords: vec![
                    "开发".to_string(),
                    "编程".to_string(),
                    "实现".to_string(),
                    "代码".to_string(),
                ],
                strategy: DecompositionStrategy::Hybrid,
                subtask_templates: vec![
                    "需求分析和技术设计".to_string(),
                    "环境搭建和配置".to_string(),
                    "核心功能开发".to_string(),
                    "单元测试编写".to_string(),
                    "集成测试".to_string(),
                    "文档编写".to_string(),
                    "代码审查和优化".to_string(),
                    "部署准备".to_string(),
                ],
                dependency_templates: vec![
                    (0, 2, TaskRelationType::StrongDependency),
                    (1, 2, TaskRelationType::StrongDependency),
                    (2, 3, TaskRelationType::StrongDependency),
                    (2, 4, TaskRelationType::WeakDependency),
                    (3, 5, TaskRelationType::WeakDependency),
                    (4, 6, TaskRelationType::WeakDependency),
                    (5, 7, TaskRelationType::WeakDependency),
                    (6, 7, TaskRelationType::StrongDependency),
                ],
            },
        );

        // 研究任务模式
        self.decomposition_rules.insert(
            "研究".to_string(),
            DecompositionPattern {
                keywords: vec![
                    "研究".to_string(),
                    "调研".to_string(),
                    "分析".to_string(),
                    "探索".to_string(),
                ],
                strategy: DecompositionStrategy::Parallel,
                subtask_templates: vec![
                    "文献综述".to_string(),
                    "市场调研".to_string(),
                    "技术调研".to_string(),
                    "竞品分析".to_string(),
                    "用户调研".to_string(),
                    "专家访谈".to_string(),
                    "综合分析和总结".to_string(),
                ],
                dependency_templates: vec![
                    (0, 6, TaskRelationType::WeakDependency),
                    (1, 6, TaskRelationType::WeakDependency),
                    (2, 6, TaskRelationType::WeakDependency),
                    (3, 6, TaskRelationType::WeakDependency),
                    (4, 6, TaskRelationType::WeakDependency),
                    (5, 6, TaskRelationType::StrongDependency),
                ],
            },
        );
    }

    /// 匹配分解模式
    fn find_matching_pattern(&self, task: &AgentTask) -> Option<&DecompositionPattern> {
        let task_text = format!(
            "{} {}",
            task.description,
            task.expected_output.as_deref().unwrap_or("")
        );

        for (_pattern_name, pattern) in &self.decomposition_rules {
            for keyword in &pattern.keywords {
                if task_text.contains(keyword) {
                    return Some(pattern);
                }
            }
        }

        None
    }

    /// 基于模板创建子任务
    fn create_subtasks_from_template(
        &self,
        parent_task: &AgentTask,
        pattern: &DecompositionPattern,
    ) -> Vec<AgentTask> {
        let mut subtasks = Vec::new();

        for (index, template) in pattern.subtask_templates.iter().enumerate() {
            let subtask_description = format!(
                "{} - 子任务{}: {}",
                parent_task.description,
                index + 1,
                template
            );

            let mut subtask = AgentTask::new(subtask_description)
                .with_expected_output(format!("完成{}的成果", template))
                .with_priority(parent_task.priority)
                .with_dependency(parent_task.id.clone());

            // 继承父任务的相关元数据
            subtask.metadata = parent_task.metadata.clone();
            subtask.metadata.insert(
                "parent_task_id".to_string(),
                serde_json::Value::String(parent_task.id.clone()),
            );
            subtask.metadata.insert(
                "subtask_index".to_string(),
                serde_json::Value::Number(index.into()),
            );

            subtasks.push(subtask);
        }

        subtasks
    }

    /// 创建依赖关系
    fn create_dependencies_from_template(
        &self,
        subtasks: &[AgentTask],
        pattern: &DecompositionPattern,
    ) -> Vec<SubTaskRelation> {
        let mut dependencies = Vec::new();

        for (from_idx, to_idx, relation_type) in &pattern.dependency_templates {
            if *from_idx < subtasks.len() && *to_idx < subtasks.len() {
                dependencies.push(SubTaskRelation {
                    parent_id: subtasks[*from_idx].id.clone(),
                    child_id: subtasks[*to_idx].id.clone(),
                    relation_type: relation_type.clone(),
                    weight: 5, // 默认权重
                });
            }
        }

        dependencies
    }
}

#[async_trait]
impl TaskDecomposer for IntelligentTaskDecomposer {
    async fn decompose_task(&self, task: &AgentTask) -> Result<Vec<AgentTask>> {
        let complexity = self.analyze_complexity(task);

        // 简单任务不需要分解
        if matches!(complexity, TaskComplexity::Simple) {
            return Ok(vec![task.clone()]);
        }

        // 尝试匹配已知模式
        if let Some(pattern) = self.find_matching_pattern(task) {
            let subtasks = self.create_subtasks_from_template(task, pattern);
            tracing::info!(
                "Task {} decomposed into {} subtasks using pattern",
                task.id,
                subtasks.len()
            );
            return Ok(subtasks);
        }

        // 使用LLM进行智能分解
        self.llm_decompose_task(task, complexity).await
    }

    fn analyze_complexity(&self, task: &AgentTask) -> TaskComplexity {
        let description = &task.description;
        let expected_output = task.expected_output.as_deref().unwrap_or("");
        let full_text = format!("{} {}", description, expected_output);

        // 基于关键词和长度判断复杂度
        let word_count = full_text.split_whitespace().count();
        let complexity_indicators = [
            "分析", "设计", "开发", "实现", "研究", "调研", "报告", "系统", "架构", "优化", "重构",
            "测试", "部署", "集成", "迁移", "转换",
        ];

        let indicator_count = complexity_indicators
            .iter()
            .filter(|&&indicator| full_text.contains(indicator))
            .count();

        match (word_count, indicator_count) {
            (0..=50, 0..=1) => TaskComplexity::Simple,
            (51..=200, 2..=4) => TaskComplexity::Medium,
            (201..=500, 5..=8) => TaskComplexity::Complex,
            _ => TaskComplexity::UltraComplex,
        }
    }

    fn select_strategy(
        &self,
        task: &AgentTask,
        complexity: TaskComplexity,
    ) -> DecompositionStrategy {
        // 如果有匹配的模式，使用模式的策略
        if let Some(pattern) = self.find_matching_pattern(task) {
            return pattern.strategy.clone();
        }

        // 基于复杂度选择策略
        match complexity {
            TaskComplexity::Simple => DecompositionStrategy::Sequential,
            TaskComplexity::Medium => DecompositionStrategy::Hybrid,
            TaskComplexity::Complex => DecompositionStrategy::Parallel,
            TaskComplexity::UltraComplex => DecompositionStrategy::Intelligent,
        }
    }
}

impl IntelligentTaskDecomposer {
    /// 使用LLM进行任务分解
    async fn llm_decompose_task(
        &self,
        task: &AgentTask,
        complexity: TaskComplexity,
    ) -> Result<Vec<AgentTask>> {
        let _prompt = format!(
            "请将以下任务分解为具体的子任务：\n\n任务描述: {}\n期望输出: {}\n优先级: {}\n\n复杂度等级: {:?}\n\n请提供：\n1. 3-8个子任务\n2. 每个子任务的简要描述\n3. 子任务之间的依赖关系\n\n请以JSON格式回复，包含subtasks和dependencies两个数组。\n\n例如：\n{{\n  \"subtasks\": [\n    {{\"description\": \"子任务1描述\", \"priority\": 8}},\n    {{\"description\": \"子任务2描述\", \"priority\": 7}}\n  ],\n  \"dependencies\": [\n    {{\"from\": 0, \"to\": 1, \"type\": \"strong\"}}\n  ]\n}}",
            task.description,
            task.expected_output.as_deref().unwrap_or("无"),
            task.priority,
            complexity
        );

        // 这里简化实现，实际应该调用LLM
        // 为了演示，我们返回一个基本的分解
        let basic_subtasks = vec![
            AgentTask::new(format!("{} - 步骤1: 准备和分析", task.description))
                .with_expected_output("分析和准备工作的结果".to_string())
                .with_priority(task.priority),
            AgentTask::new(format!("{} - 步骤2: 核心执行", task.description))
                .with_expected_output("核心任务执行结果".to_string())
                .with_priority(task.priority),
            AgentTask::new(format!("{} - 步骤3: 验证和完善", task.description))
                .with_expected_output("最终完成的结果".to_string())
                .with_priority(task.priority),
        ];

        tracing::info!("Task {} decomposed using basic template", task.id);
        Ok(basic_subtasks)
    }
}

/// 高级任务调度器
#[derive(Debug)]
pub struct AdvancedScheduler {
    /// 依赖图
    dependency_graph: Arc<RwLock<DependencyGraph>>,
    /// 任务队列（按优先级排序）
    task_queue: Arc<RwLock<VecDeque<String>>>,
    /// 正在执行的任务
    running_tasks: Arc<RwLock<HashSet<String>>>,
    /// 已完成的任务
    completed_tasks: Arc<RwLock<HashSet<String>>>,
    /// 调度策略
    scheduling_strategy: SchedulingStrategy,
}

/// 调度策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulingStrategy {
    /// 先进先出
    FIFO,
    /// 优先级调度
    Priority,
    /// 关键路径优先
    CriticalPath,
    /// 最短执行时间优先
    ShortestJobFirst,
    /// 负载均衡
    LoadBalanced,
}

impl AdvancedScheduler {
    /// 创建新的调度器
    pub fn new(strategy: SchedulingStrategy) -> Self {
        Self {
            dependency_graph: Arc::new(RwLock::new(DependencyGraph::new())),
            task_queue: Arc::new(RwLock::new(VecDeque::new())),
            running_tasks: Arc::new(RwLock::new(HashSet::new())),
            completed_tasks: Arc::new(RwLock::new(HashSet::new())),
            scheduling_strategy: strategy,
        }
    }

    /// 添加任务到调度器
    pub async fn add_task(&self, task: &AgentTask) -> Result<()> {
        let mut graph = self.dependency_graph.write().await;
        graph.add_task(task.id.clone());

        // 添加依赖关系
        for dep_id in &task.dependencies {
            let relation = SubTaskRelation {
                parent_id: dep_id.clone(),
                child_id: task.id.clone(),
                relation_type: TaskRelationType::StrongDependency,
                weight: 5,
            };
            graph.add_dependency(relation)?;
        }

        // 重新计算拓扑排序
        drop(graph);
        self.rebuild_task_queue().await?;

        tracing::info!("Task {} added to scheduler", task.id);
        Ok(())
    }

    /// 重建任务队列
    async fn rebuild_task_queue(&self) -> Result<()> {
        let mut graph = self.dependency_graph.write().await;
        let topological_order = graph.topological_sort()?;
        drop(graph);

        let mut queue = self.task_queue.write().await;
        queue.clear();

        match self.scheduling_strategy {
            SchedulingStrategy::FIFO => {
                for task_id in topological_order {
                    queue.push_back(task_id);
                }
            }
            SchedulingStrategy::Priority => {
                // 这里需要任务信息来进行优先级排序，简化实现
                for task_id in topological_order {
                    queue.push_back(task_id);
                }
            }
            SchedulingStrategy::CriticalPath => {
                let graph = self.dependency_graph.read().await;
                let critical_path = graph.get_critical_path();
                drop(graph);

                // 优先安排关键路径上的任务
                for task_id in &critical_path {
                    if topological_order.contains(task_id) {
                        queue.push_back(task_id.clone());
                    }
                }

                // 然后安排其他任务
                for task_id in topological_order {
                    if !critical_path.contains(&task_id) {
                        queue.push_back(task_id);
                    }
                }
            }
            _ => {
                // 其他策略的简化实现
                for task_id in topological_order {
                    queue.push_back(task_id);
                }
            }
        }

        Ok(())
    }

    /// 获取下一个可执行的任务
    pub async fn get_next_task(&self) -> Option<String> {
        let graph = self.dependency_graph.read().await;
        let ready_tasks = graph.get_ready_tasks();
        drop(graph);

        let running_tasks = self.running_tasks.read().await;
        let completed_tasks = self.completed_tasks.read().await;

        // 找到不在运行中且未完成的就绪任务
        for task_id in ready_tasks {
            if !running_tasks.contains(&task_id) && !completed_tasks.contains(&task_id) {
                return Some(task_id);
            }
        }

        None
    }

    /// 标记任务开始执行
    pub async fn mark_task_started(&self, task_id: &str) {
        let mut running_tasks = self.running_tasks.write().await;
        running_tasks.insert(task_id.to_string());
        tracing::debug!("Task {} marked as started", task_id);
    }

    /// 标记任务完成
    pub async fn mark_task_completed(&self, task_id: &str) {
        let mut running_tasks = self.running_tasks.write().await;
        running_tasks.remove(task_id);

        let mut completed_tasks = self.completed_tasks.write().await;
        completed_tasks.insert(task_id.to_string());

        let mut graph = self.dependency_graph.write().await;
        graph.mark_task_completed(task_id);

        tracing::debug!("Task {} marked as completed", task_id);
    }

    /// 检查所有任务是否完成
    pub async fn is_all_completed(&self) -> bool {
        let graph = self.dependency_graph.read().await;
        let completed_tasks = self.completed_tasks.read().await;
        graph.nodes.len() == completed_tasks.len()
    }

    /// 获取调度器统计信息
    pub async fn get_stats(&self) -> SchedulerStats {
        let graph = self.dependency_graph.read().await;
        let running_tasks = self.running_tasks.read().await;
        let completed_tasks = self.completed_tasks.read().await;
        let queue = self.task_queue.read().await;

        SchedulerStats {
            total_tasks: graph.nodes.len(),
            completed_tasks: completed_tasks.len(),
            running_tasks: running_tasks.len(),
            pending_tasks: graph.nodes.len() - completed_tasks.len() - running_tasks.len(),
            queue_length: queue.len(),
            graph_stats: graph.get_stats(),
        }
    }
}

/// 调度器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub running_tasks: usize,
    pub pending_tasks: usize,
    pub queue_length: usize,
    pub graph_stats: GraphStats,
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
