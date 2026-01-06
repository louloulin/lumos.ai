//! # 任务队列
//!
//! 管理 Multi-Agent 系统的任务队列。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// 任务状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 等待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub id: String,
    /// 任务类型
    pub task_type: String,
    /// 任务输入
    pub input: String,
    /// 优先级
    pub priority: TaskPriority,
    /// 状态
    pub status: TaskStatus,
    /// 分配的 Agent ID
    pub assigned_agent: Option<String>,
    /// 创建时间
    pub created_at: u64,
    /// 开始时间
    pub started_at: Option<u64>,
    /// 完成时间
    pub completed_at: Option<u64>,
    /// 结果
    pub result: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Task {
    /// 创建新任务
    pub fn new(task_type: impl Into<String>, input: impl Into<String>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_type: task_type.into(),
            input: input.into(),
            priority: TaskPriority::Normal,
            status: TaskStatus::Pending,
            assigned_agent: None,
            created_at: timestamp,
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 分配 Agent
    pub fn assign(&mut self, agent_id: impl Into<String>) {
        self.assigned_agent = Some(agent_id.into());
        self.status = TaskStatus::Running;
        self.started_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    /// 完成任务
    pub fn complete(&mut self, result: impl Into<String>) {
        self.status = TaskStatus::Completed;
        self.result = Some(result.into());
        self.completed_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    /// 标记失败
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = TaskStatus::Failed;
        self.error = Some(error.into());
        self.completed_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    /// 取消任务
    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.completed_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    /// 计算执行时长 (毫秒)
    pub fn duration_ms(&self) -> Option<u64> {
        if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            Some((completed - started) * 1000)
        } else {
            None
        }
    }
}

/// 任务队列
pub struct TaskQueue {
    /// 待处理任务发送器
    pending_sender: mpsc::Sender<Task>,
    /// 任务映射 (task_id -> task)
    tasks: RwLock<HashMap<String, Task>>,
}

impl TaskQueue {
    /// 创建新的任务队列
    pub fn new() -> Self {
        let (pending_sender, _) = mpsc::channel(1000);

        Self {
            pending_sender,
            tasks: RwLock::new(HashMap::new()),
        }
    }

    /// 添加任务
    pub async fn enqueue(&self, task: Task) -> Result<(), String> {
        // 存储任务
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());

        // 发送到待处理队列
        self.pending_sender
            .send(task)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 获取任务
    pub async fn get(&self, task_id: &str) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// 更新任务
    pub async fn update(&self, task: Task) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task);
        Ok(())
    }

    /// 订阅待处理任务
    pub fn subscribe_pending(&self) -> mpsc::Receiver<Task> {
        let (tx, rx) = mpsc::channel(100);
        // 注意: 实际实现需要更复杂的订阅机制
        // 这里简化为创建新通道
        rx
    }

    /// 获取所有任务
    pub async fn list_all(&self) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// 按状态获取任务
    pub async fn list_by_status(&self, status: TaskStatus) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks
            .values()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }

    /// 获取队列统计
    pub async fn stats(&self) -> TaskQueueStats {
        let tasks = self.tasks.read().await;

        let total = tasks.len();
        let pending = tasks.values().filter(|t| t.status == TaskStatus::Pending).count();
        let running = tasks.values().filter(|t| t.status == TaskStatus::Running).count();
        let completed = tasks.values().filter(|t| t.status == TaskStatus::Completed).count();
        let failed = tasks.values().filter(|t| t.status == TaskStatus::Failed).count();

        TaskQueueStats {
            total,
            pending,
            running,
            completed,
            failed,
        }
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// 任务队列统计
#[derive(Debug, Clone)]
pub struct TaskQueueStats {
    /// 总任务数
    pub total: usize,
    /// 待处理
    pub pending: usize,
    /// 执行中
    pub running: usize,
    /// 已完成
    pub completed: usize,
    /// 失败
    pub failed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("test_type", "test input")
            .with_priority(TaskPriority::High)
            .with_metadata("key", "value");

        assert_eq!(task.task_type, "test_type");
        assert_eq!(task.input, "test input");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = Task::new("test", "input");

        assert_eq!(task.status, TaskStatus::Pending);

        task.assign("agent1");
        assert_eq!(task.status, TaskStatus::Running);
        assert_eq!(task.assigned_agent, Some("agent1".to_string()));

        task.complete("result");
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.result, Some("result".to_string()));
        assert!(task.duration_ms().is_some());
    }

    #[test]
    fn test_task_failure() {
        let mut task = Task::new("test", "input");
        task.assign("agent1");
        task.fail("Error occurred");

        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.error, Some("Error occurred".to_string()));
    }

    #[tokio::test]
    async fn test_task_queue() {
        let queue = TaskQueue::new();
        let task = Task::new("test", "input");

        queue.enqueue(task.clone()).await.unwrap();

        let retrieved = queue.get(&task.id).await.unwrap();
        assert_eq!(retrieved.id, task.id);
    }

    #[tokio::test]
    async fn test_queue_stats() {
        let queue = TaskQueue::new();

        let mut task1 = Task::new("test1", "input1");
        task1.assign("agent1");
        task1.complete("result1");

        let task2 = Task::new("test2", "input2");

        queue.update(task1).await.unwrap();
        queue.enqueue(task2).await.unwrap();

        let stats = queue.stats().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.pending, 1);
    }
}
