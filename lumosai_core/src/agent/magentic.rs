//! Magentic 协作模式
//!
//! Manager 动态构建任务清单，Worker 执行具体任务
//!
//! 参考：
//! - AutoGen Magentic-One (Microsoft Research)
//! - Dynamic Task Planning (2025)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::Agent;
use crate::error::Result;

/// 任务状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskState {
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

/// Magentic 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagenticTask {
    /// 任务 ID
    pub id: String,
    /// 任务描述
    pub description: String,
    /// 分配的 Worker ID
    pub assigned_worker: Option<String>,
    /// 任务状态
    pub state: TaskState,
    /// 任务结果
    pub result: Option<String>,
    /// 创建时间
    pub created_at: i64,
    /// 完成时间
    pub completed_at: Option<i64>,
}

impl MagenticTask {
    /// 创建新任务
    pub fn new(description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description,
            assigned_worker: None,
            state: TaskState::Pending,
            result: None,
            created_at: chrono::Utc::now().timestamp(),
            completed_at: None,
        }
    }

    /// 分配给 Worker
    pub fn assign_to(&mut self, worker_id: String) {
        self.assigned_worker = Some(worker_id);
        self.state = TaskState::InProgress;
    }

    /// 标记为完成
    pub fn mark_completed(&mut self, result: String) {
        self.state = TaskState::Completed;
        self.result = Some(result);
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }

    /// 标记为失败
    pub fn mark_failed(&mut self) {
        self.state = TaskState::Failed;
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }
}

/// 任务清单（Ledger）
#[derive(Debug, Clone)]
pub struct TaskLedger {
    /// 任务列表
    tasks: Arc<RwLock<HashMap<String, MagenticTask>>>,
}

impl TaskLedger {
    /// 创建新的任务清单
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加任务
    pub async fn add_task(&self, task: MagenticTask) {
        let task_id = task.id.clone();
        self.tasks.write().await.insert(task_id, task);
    }

    /// 获取待处理任务
    pub async fn get_pending_tasks(&self) -> Vec<MagenticTask> {
        self.tasks
            .read()
            .await
            .values()
            .filter(|t| t.state == TaskState::Pending)
            .cloned()
            .collect()
    }

    /// 获取任务
    pub async fn get_task(&self, task_id: &str) -> Option<MagenticTask> {
        self.tasks.read().await.get(task_id).cloned()
    }

    /// 更新任务
    pub async fn update_task(&self, task: MagenticTask) {
        let task_id = task.id.clone();
        self.tasks.write().await.insert(task_id, task);
    }

    /// 获取所有任务
    pub async fn get_all_tasks(&self) -> Vec<MagenticTask> {
        self.tasks.read().await.values().cloned().collect()
    }

    /// 获取任务总数
    pub async fn total_tasks(&self) -> usize {
        self.tasks.read().await.len()
    }

    /// 获取已完成任务数
    pub async fn completed_tasks(&self) -> usize {
        self.tasks
            .read()
            .await
            .values()
            .filter(|t| t.state == TaskState::Completed)
            .count()
    }

    /// 是否所有任务都已完成
    pub async fn is_all_completed(&self) -> bool {
        let tasks = self.tasks.read().await;
        !tasks.is_empty() && tasks.values().all(|t| t.state == TaskState::Completed)
    }
}

impl Default for TaskLedger {
    fn default() -> Self {
        Self::new()
    }
}

/// Magentic 执行器
pub struct MagenticExecutor {
    /// Manager Agent
    manager: Arc<dyn Agent>,
    /// Worker Agents
    workers: HashMap<String, Arc<dyn Agent>>,
    /// 任务清单
    ledger: TaskLedger,
    /// 最大迭代次数
    max_iterations: usize,
}

impl MagenticExecutor {
    /// 创建新的 Magentic 执行器
    pub fn new(
        manager: Arc<dyn Agent>,
        workers: HashMap<String, Arc<dyn Agent>>,
        max_iterations: usize,
    ) -> Self {
        Self {
            manager,
            workers,
            ledger: TaskLedger::new(),
            max_iterations,
        }
    }

    /// 执行 Magentic 流程
    pub async fn execute(&mut self, initial_goal: &str) -> Result<(String, TaskLedger)> {
        tracing::info!("Starting Magentic execution with goal: {}", initial_goal);

        let mut iteration = 0;
        let mut final_result = String::new();

        while iteration < self.max_iterations {
            tracing::debug!(
                "Magentic iteration {}/{}",
                iteration + 1,
                self.max_iterations
            );

            // 1. Manager 规划任务
            let planning_prompt = self.build_planning_prompt(initial_goal).await;
            let plan = self.manager.generate_simple(&planning_prompt).await?;
            tracing::debug!("Manager plan: {}", plan);

            // 2. 解析任务清单
            let new_tasks = self.parse_tasks(&plan);
            for task in new_tasks {
                self.ledger.add_task(task).await;
            }

            // 3. 执行待处理任务
            let pending_tasks = self.ledger.get_pending_tasks().await;
            if pending_tasks.is_empty() {
                tracing::info!("No pending tasks, execution complete");
                break;
            }

            for mut task in pending_tasks {
                // 分配给第一个可用的 Worker
                if let Some((worker_id, worker)) = self.workers.iter().next() {
                    task.assign_to(worker_id.clone());
                    self.ledger.update_task(task.clone()).await;

                    // 执行任务
                    match worker.generate_simple(&task.description).await {
                        Ok(result) => {
                            task.mark_completed(result.clone());
                            final_result = result;
                            tracing::debug!("Task {} completed: {}", task.id, final_result);
                        }
                        Err(e) => {
                            task.mark_failed();
                            tracing::warn!("Task {} failed: {}", task.id, e);
                        }
                    }

                    self.ledger.update_task(task).await;
                }
            }

            // 4. 检查是否所有任务都已完成
            if self.ledger.is_all_completed().await {
                tracing::info!("All tasks completed");
                break;
            }

            iteration += 1;
        }

        Ok((final_result, self.ledger.clone()))
    }

    /// 构建规划提示
    async fn build_planning_prompt(&self, goal: &str) -> String {
        let completed = self.ledger.completed_tasks().await;
        let total = self.ledger.total_tasks().await;

        format!(
            "You are a task planning manager. Your goal is: {}\n\n\
            Current progress: {}/{} tasks completed\n\n\
            Please create a list of tasks needed to achieve this goal.\n\
            Format each task on a new line starting with 'TASK:'",
            goal, completed, total
        )
    }

    /// 解析任务清单
    fn parse_tasks(&self, plan: &str) -> Vec<MagenticTask> {
        let mut tasks = Vec::new();

        for line in plan.lines() {
            if let Some(task_desc) = line.strip_prefix("TASK:") {
                let task = MagenticTask::new(task_desc.trim().to_string());
                tasks.push(task);
            }
        }

        tasks
    }

    /// 获取任务清单
    pub fn get_ledger(&self) -> &TaskLedger {
        &self.ledger
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_ledger() {
        let ledger = TaskLedger::new();

        let task1 = MagenticTask::new("Task 1".to_string());
        let task2 = MagenticTask::new("Task 2".to_string());

        ledger.add_task(task1.clone()).await;
        ledger.add_task(task2.clone()).await;

        assert_eq!(ledger.total_tasks().await, 2);
        assert_eq!(ledger.completed_tasks().await, 0);
        assert!(!ledger.is_all_completed().await);

        let mut task1_updated = task1.clone();
        task1_updated.mark_completed("Done".to_string());
        ledger.update_task(task1_updated).await;

        assert_eq!(ledger.completed_tasks().await, 1);
    }

    #[test]
    fn test_parse_tasks() {
        use crate::agent::AgentBuilder;
        use crate::llm::test_helpers::create_test_zhipu_provider_arc;

        let llm = create_test_zhipu_provider_arc();
        let manager = Arc::new(
            AgentBuilder::new()
                .name("manager")
                .instructions("You are a task manager")
                .model(llm)
                .build()
                .expect("Failed to build agent"),
        );

        let executor = MagenticExecutor {
            manager,
            workers: HashMap::new(),
            ledger: TaskLedger::new(),
            max_iterations: 10,
        };

        let plan = "TASK: First task\nTASK: Second task\nSome other text\nTASK: Third task";
        let tasks = executor.parse_tasks(plan);

        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].description, "First task");
        assert_eq!(tasks[1].description, "Second task");
        assert_eq!(tasks[2].description, "Third task");
    }
}
