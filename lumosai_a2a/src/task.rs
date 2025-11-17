//! A2A Task Management
//!
//! 实现任务管理功能

use crate::types::*;
use crate::{A2AError, A2AResult};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

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

    /// 创建新任务并返回 ID（用于服务器接口）
    pub fn create_task_with_id(&mut self, agent_id: &str, message: Message, _artifacts: Option<Vec<Artifact>>) -> A2AResult<String> {
        let task = Task {
            id: Uuid::new_v4().to_string(),
            description: format!("Task for agent {}", agent_id),
            input_message: message,
            status: TaskStatus::new(TaskState::Submitted),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deadline: None,
            assigned_agent: Some(agent_id.to_string()),
            parameters: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        let task_id = task.id.clone();
        self.tasks.insert(task_id.clone(), task);
        Ok(task_id)
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

    /// 直接设置任务状态（用于服务器接口）
    pub fn set_task_status(&mut self, task_id: &str, status: TaskStatus) -> A2AResult<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = status;
            task.updated_at = chrono::Utc::now();
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

    /// 添加任务工件
    pub fn add_artifact(&mut self, task_id: &str, artifact: Artifact) -> A2AResult<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.artifacts.push(artifact);
            task.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(A2AError::TaskNotFound(task_id.to_string()))
        }
    }

    /// 添加任务工件（别名方法）
    pub fn add_task_artifact(&mut self, task_id: &str, artifact: Artifact) -> A2AResult<()> {
        self.add_artifact(task_id, artifact)
    }

    /// 添加任务消息
    pub fn add_task_message(&mut self, task_id: &str, message: Message) -> A2AResult<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            // 可以选择存储消息到任务中或更新状态消息
            task.status.message = Some(message);
            task.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(A2AError::TaskNotFound(task_id.to_string()))
        }
    }

    /// 按状态列出任务
    pub fn list_tasks_by_state(&self, state: TaskState) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.status.state == state)
            .collect()
    }

    /// 按Agent列出任务
    pub fn list_tasks_by_agent(&self, agent_id: &str) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.assigned_agent.as_ref().map_or(false, |id| id == agent_id))
            .collect()
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

    #[test]
    fn test_task_manager_error_handling() {
        let mut manager = TaskManager::new();
        
        // 测试获取不存在的任务
        let result = manager.get_task("nonexistent-id");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), A2AError::TaskNotFound(_)));
        
        // 测试更新不存在的任务状态
        let result = manager.update_task_status("nonexistent-id", TaskState::Working, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), A2AError::TaskNotFound(_)));
        
        // 测试为不存在的任务分配Agent
        let result = manager.assign_task_to_agent("nonexistent-id", "test-agent".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), A2AError::TaskNotFound(_)));
        
        // 测试为不存在的任务添加工件
        let artifact = Artifact::from_text("Test result".to_string());
        let result = manager.add_task_artifact("nonexistent-id", artifact);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), A2AError::TaskNotFound(_)));
    }

    #[test]
    fn test_task_artifact_management() {
        let mut manager = TaskManager::new();
        
        let task = manager.create_task("Artifact test".to_string(), Message::user_message("Test".to_string()));
        
        // 添加工件
        let artifact1 = Artifact::from_text("Result 1".to_string()).with_name("result1.txt".to_string());
        let artifact2 = Artifact::from_text("Result 2".to_string()).with_name("result2.json".to_string());
        
        manager.add_task_artifact(&task.id, artifact1.clone()).unwrap();
        manager.add_task_artifact(&task.id, artifact2.clone()).unwrap();
        
        let retrieved_task = manager.get_task(&task.id).unwrap();
        assert_eq!(retrieved_task.artifacts.len(), 2);
        assert_eq!(retrieved_task.artifacts[0].name, Some("result1.txt".to_string()));
        assert_eq!(retrieved_task.artifacts[1].name, Some("result2.json".to_string()));
    }

    #[test]
    fn test_task_list_filtering() {
        let mut manager = TaskManager::new();
        
        let task1 = manager.create_task("Task 1".to_string(), Message::user_message("Hello".to_string()));
        let task2 = manager.create_task("Task 2".to_string(), Message::user_message("World".to_string()));
        let task3 = manager.create_task("Task 3".to_string(), Message::user_message("Test".to_string()));

        // 分配不同的Agent
        manager.assign_task_to_agent(&task1.id, "agent-alpha".to_string()).unwrap();
        manager.assign_task_to_agent(&task2.id, "agent-beta".to_string()).unwrap();
        manager.assign_task_to_agent(&task3.id, "agent-alpha".to_string()).unwrap();

        // 更新不同状态
        manager.update_task_status(&task1.id, TaskState::Completed, None).unwrap();
        manager.update_task_status(&task2.id, TaskState::Working, None).unwrap();
        manager.update_task_status(&task3.id, TaskState::Failed, None).unwrap();

        // 测试按Agent过滤
        let alpha_tasks = manager.list_tasks_by_agent("agent-alpha");
        assert_eq!(alpha_tasks.len(), 2);

        let beta_tasks = manager.list_tasks_by_agent("agent-beta");
        assert_eq!(beta_tasks.len(), 1);

        let gamma_tasks = manager.list_tasks_by_agent("agent-gamma");
        assert_eq!(gamma_tasks.len(), 0);

        // 测试按状态过滤
        let completed_tasks = manager.list_tasks_by_state(TaskState::Completed);
        assert_eq!(completed_tasks.len(), 1);

        let working_tasks = manager.list_tasks_by_state(TaskState::Working);
        assert_eq!(working_tasks.len(), 1);

        let failed_tasks = manager.list_tasks_by_state(TaskState::Failed);
        assert_eq!(failed_tasks.len(), 1);
    }

    #[test]
    fn test_task_message_addition() {
        let mut manager = TaskManager::new();
        
        let task = manager.create_task("Message test".to_string(), Message::user_message("Initial".to_string()));
        
        let user_message = Message::user_message("User response".to_string());
        let assistant_message = Message::assistant_message("Assistant reply".to_string());
        
        manager.add_task_message(&task.id, user_message).unwrap();
        manager.add_task_message(&task.id, assistant_message).unwrap();
        
        let retrieved_task = manager.get_task(&task.id).unwrap();
        
        // 检查历史记录
        assert_eq!(retrieved_task.status.history.len(), 2);
        assert_eq!(retrieved_task.status.history[0].message.role, MessageRole::User);
        assert_eq!(retrieved_task.status.history[1].message.role, MessageRole::Assistant);
    }

    #[test]
    fn test_task_cancellation() {
        let mut manager = TaskManager::new();
        
        let task = manager.create_task("Cancellation test".to_string(), Message::user_message("Test".to_string()));
        
        // 更新为工作中状态
        manager.update_task_status(&task.id, TaskState::Working, None).unwrap();
        
        // 取消任务
        manager.update_task_status(&task.id, TaskState::Canceled, Some(Message::agent_message("User cancelled".to_string()))).unwrap();
        
        let retrieved_task = manager.get_task(&task.id).unwrap();
        assert_eq!(retrieved_task.status.state, TaskState::Canceled);
        assert_eq!(retrieved_task.status.message.as_ref().map(|m| &m.parts[0]), Some(&"User cancelled".to_string()));
    }

    #[test]
    fn test_task_input_required_state() {
        let mut manager = TaskManager::new();
        
        let task = manager.create_task("Input test".to_string(), Message::user_message("Need input".to_string()));
        
        // 设置为需要输入状态
        manager.update_task_status(&task.id, TaskState::InputRequired, Some(Message::agent_message("Please provide more details".to_string()))).unwrap();
        
        let retrieved_task = manager.get_task(&task.id).unwrap();
        assert_eq!(retrieved_task.status.state, TaskState::InputRequired);
        assert_eq!(retrieved_task.status.message.as_ref().map(|m| &m.parts[0]), Some(&"Please provide more details".to_string()));
    }

    #[test]
    fn test_task_stats_zero_tasks() {
        let manager = TaskManager::new();
        let stats = manager.get_task_stats();
        
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.completed_tasks, 0);
        assert_eq!(stats.failed_tasks, 0);
        assert_eq!(stats.working_tasks, 0);
        assert_eq!(stats.completion_rate(), 0.0);
        assert_eq!(stats.failure_rate(), 0.0);
    }

    #[test]
    fn test_task_multiple_status_updates() {
        let mut manager = TaskManager::new();
        
        let task = manager.create_task("Multi-update test".to_string(), Message::user_message("Test".to_string()));
        
        // 连续更新状态
        manager.update_task_status(&task.id, TaskState::Working, Some("Starting".to_string())).unwrap();
        manager.update_task_status(&task.id, TaskState::InputRequired, Some("Need input".to_string())).unwrap();
        manager.update_task_status(&task.id, TaskState::Working, Some("Processing".to_string())).unwrap();
        manager.update_task_status(&task.id, TaskState::Completed, Some("Done".to_string())).unwrap();
        
        let retrieved_task = manager.get_task(&task.id).unwrap();
        assert_eq!(retrieved_task.status.state, TaskState::Completed);
        assert_eq!(retrieved_task.status.message.as_ref().map(|m| &m.parts[0]), Some(&"Done".to_string()));
        
        // 检查历史记录
        assert!(retrieved_task.status.history.len() >= 4);
    }

    #[test]
    fn test_task_agent_reassignment() {
        let mut manager = TaskManager::new();
        
        let task = manager.create_task("Reassignment test".to_string(), Message::user_message("Test".to_string()));
        
        // 初始分配
        manager.assign_task_to_agent(&task.id, "agent1".to_string()).unwrap();
        let retrieved = manager.get_task(&task.id).unwrap();
        assert_eq!(retrieved.assigned_agent, Some("agent1".to_string()));
        
        // 重新分配
        manager.assign_task_to_agent(&task.id, "agent2".to_string()).unwrap();
        let reassigned = manager.get_task(&task.id).unwrap();
        assert_eq!(reassigned.assigned_agent, Some("agent2".to_string()));
    }

    #[test]
    fn test_task_unique_ids() {
        let mut manager = TaskManager::new();
        
        let task1 = manager.create_task("Task 1".to_string(), Message::user_message("Hello".to_string()));
        let task2 = manager.create_task("Task 2".to_string(), Message::user_message("World".to_string()));
        let task3 = manager.create_task("Task 3".to_string(), Message::user_message("Test".to_string()));
        
        // 验证ID唯一性
        assert_ne!(task1.id, task2.id);
        assert_ne!(task2.id, task3.id);
        assert_ne!(task1.id, task3.id);
        
        // 验证ID格式
        assert!(task1.id.starts_with("task_"));
        assert!(task2.id.starts_with("task_"));
        assert!(task3.id.starts_with("task_"));
    }

    #[test]
    fn test_task_artifact_types() {
        let mut manager = TaskManager::new();
        
        let task = manager.create_task("Mixed artifacts test".to_string(), Message::user_message("Test".to_string()));
        
        // 添加不同类型的工件
        let text_artifact = Artifact::from_text("Plain text result".to_string());
        let file_content = FileContent::from_base64("hello.txt".to_string(), "SGVsbG8gV29ybGQ=".to_string(), "text/plain".to_string());
        let file_artifact = Artifact::from_file("hello.txt".to_string(), file_content);
        
        manager.add_task_artifact(&task.id, text_artifact).unwrap();
        manager.add_task_artifact(&task.id, file_artifact).unwrap();
        
        let retrieved_task = manager.get_task(&task.id).unwrap();
        assert_eq!(retrieved_task.artifacts.len(), 2);
        
        // 验证工件类型
        assert!(matches!(&retrieved_task.artifacts[0].parts[0], Part::Text(_)));
        assert!(matches!(&retrieved_task.artifacts[1].parts[0], Part::File(_)));
    }
}