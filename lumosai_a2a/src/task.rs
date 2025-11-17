//! A2A Task Management
//!
//! 实现任务管理功能

use crate::types::*;
use crate::{A2AError, A2AResult};
use std::collections::HashMap;
use std::time::Duration;

/// 任务管理器
#[derive(Debug)]
pub struct TaskManager {
    tasks: HashMap<String, Task>,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    /// 创建新任务
    pub fn create_task(&mut self, description: String, input_message: Message) -> Task {
        let task = Task::new(description, input_message);
        let task_id = task.id.clone();
        self.tasks.insert(task_id, task.clone());
        task
    }

    /// 获取任务
    pub fn get_task(&self, task_id: &str) -> A2AResult<&Task> {
        self.tasks
            .get(task_id)
            .ok_or_else(|| A2AError::TaskNotFound(task_id.to_string()))
    }

    /// 获取可变的任务
    pub fn get_task_mut(&mut self, task_id: &str) -> A2AResult<&mut Task> {
        self.tasks
            .get_mut(task_id)
            .ok_or_else(|| A2AError::TaskNotFound(task_id.to_string()))
    }

    /// 更新任务状态
    pub fn update_task_status(&mut self, task_id: &str, new_state: TaskState, message: Option<Message>) -> A2AResult<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.update_status(new_state, message);
            Ok(())
        } else {
            Err(A2AError::TaskNotFound(task_id.to_string()))
        }
    }

    /// 分配任务给 Agent
    pub fn assign_task_to_agent(&mut self, task_id: &str, agent_id: String) -> A2AResult<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.assign_to_agent(agent_id);
            Ok(())
        } else {
            Err(A2AError::TaskNotFound(task_id.to_string()))
        }
    }

    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    /// 根据状态获取任务
    pub fn get_tasks_by_state(&self, state: TaskState) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.status.state == state)
            .collect()
    }

    /// 根据 Agent 获取任务
    pub fn get_tasks_by_agent(&self, agent_id: &str) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.assigned_agent.as_ref().map(|id| id == agent_id).unwrap_or(false))
            .collect()
    }

    /// 删除任务
    pub fn delete_task(&mut self, task_id: &str) -> A2AResult<Task> {
        self.tasks
            .remove(task_id)
            .ok_or_else(|| A2AError::TaskNotFound(task_id.to_string()))
    }

    /// 清理已完成的任务
    pub fn cleanup_completed_tasks(&mut self, older_than: Duration) -> Vec<Task> {
        let now = chrono::Utc::now();
        let cutoff = now - chrono::Duration::from_std(older_than).unwrap();

        let mut to_remove = Vec::new();
        for (id, task) in &self.tasks {
            if task.status.state == TaskState::Completed && task.status.timestamp < cutoff {
                to_remove.push(id.clone());
            }
        }

        let mut removed_tasks = Vec::new();
        for id in to_remove {
            if let Some(task) = self.tasks.remove(&id) {
                removed_tasks.push(task);
            }
        }

        removed_tasks
    }

    /// 获取任务统计信息
    pub fn get_task_stats(&self) -> TaskStats {
        let mut stats = TaskStats::default();

        for task in self.tasks.values() {
            stats.total_tasks += 1;
            match task.status.state {
                TaskState::Submitted => stats.submitted_tasks += 1,
                TaskState::Working => stats.working_tasks += 1,
                TaskState::InputRequired => stats.input_required_tasks += 1,
                TaskState::Completed => stats.completed_tasks += 1,
                TaskState::Canceled => stats.canceled_tasks += 1,
                TaskState::Failed => stats.failed_tasks += 1,
                TaskState::Unknown => {}
            }
        }

        stats
    }
}

/// 任务统计信息
#[derive(Debug, Clone, Default)]
pub struct TaskStats {
    pub total_tasks: usize,
    pub submitted_tasks: usize,
    pub working_tasks: usize,
    pub input_required_tasks: usize,
    pub completed_tasks: usize,
    pub canceled_tasks: usize,
    pub failed_tasks: usize,
}

impl TaskStats {
    /// 计算完成率
    pub fn completion_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.completed_tasks as f64 / self.total_tasks as f64
        }
    }

    /// 计算失败率
    pub fn failure_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.failed_tasks as f64 / self.total_tasks as f64
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_manager() {
        let mut manager = TaskManager::new();
        
        let task = manager.create_task(
            "Test task".to_string(),
            Message::user_message("Hello".to_string()),
        );

        assert_eq!(task.status.state, TaskState::Submitted);

        // 测试获取任务
        let retrieved = manager.get_task(&task.id).unwrap();
        assert_eq!(retrieved.id, task.id);

        // 测试更新状态
        manager.update_task_status(&task.id, TaskState::Working, None).unwrap();
        let updated = manager.get_task(&task.id).unwrap();
        assert_eq!(updated.status.state, TaskState::Working);

        // 测试分配给 Agent
        manager.assign_task_to_agent(&task.id, "test-agent".to_string()).unwrap();
        let assigned = manager.get_task(&task.id).unwrap();
        assert_eq!(assigned.assigned_agent, Some("test-agent".to_string()));
    }

    #[test]
    fn test_task_stats() {
        let mut manager = TaskManager::new();
        
        // 创建一些测试任务
        let task1 = manager.create_task("Task 1".to_string(), Message::user_message("Hello".to_string()));
        let task2 = manager.create_task("Task 2".to_string(), Message::user_message("World".to_string()));
        let task3 = manager.create_task("Task 3".to_string(), Message::user_message("Test".to_string()));

        // 更新一些任务状态
        manager.update_task_status(&task1.id, TaskState::Completed, None).unwrap();
        manager.update_task_status(&task2.id, TaskState::Failed, None).unwrap();
        manager.update_task_status(&task3.id, TaskState::Working, None).unwrap();

        let stats = manager.get_task_stats();
        assert_eq!(stats.total_tasks, 3);
        assert_eq!(stats.completed_tasks, 1);
        assert_eq!(stats.failed_tasks, 1);
        assert_eq!(stats.working_tasks, 1);
        assert_eq!(stats.completion_rate(), 1.0 / 3.0);
        assert_eq!(stats.failure_rate(), 1.0 / 3.0);
    }
}