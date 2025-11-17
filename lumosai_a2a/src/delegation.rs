//! A2A Task Delegation
//!
//! 实现任务委托功能

use crate::types::*;
use crate::{A2AError, A2AResult};

/// 任务委托器
#[derive(Debug)]
pub struct TaskDelegator {
    max_delegation_depth: usize,
}

impl TaskDelegator {
    /// 创建新的任务委托器
    pub fn new(max_delegation_depth: usize) -> Self {
        Self {
            max_delegation_depth,
        }
    }

    /// 委托任务
    pub async fn delegate_task(
        &self,
        task: Task,
        target_agent: &str,
        current_depth: usize,
    ) -> A2AResult<DelegationResult> {
        if current_depth >= self.max_delegation_depth {
            return Err(A2AError::TaskFailed(
                "Maximum delegation depth exceeded".to_string()
            ));
        }

        // 模拟任务委托
        let delegation_record = DelegationRecord {
            id: uuid::Uuid::new_v4().to_string(),
            task_id: task.id.clone(),
            from_agent: "current".to_string(),
            to_agent: target_agent.to_string(),
            timestamp: chrono::Utc::now(),
            status: DelegationStatus::Pending,
        };

        let result = DelegationResult {
            success: true,
            delegation_record,
            artifacts: Vec::new(),
            error: None,
        };

        Ok(result)
    }

    /// 设置最大委托深度
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_delegation_depth = max_depth;
        self
    }
}

/// 委托结果
#[derive(Debug, Clone)]
pub struct DelegationResult {
    pub success: bool,
    pub delegation_record: DelegationRecord,
    pub artifacts: Vec<Artifact>,
    pub error: Option<String>,
}

/// 委托记录
#[derive(Debug, Clone)]
pub struct DelegationRecord {
    pub id: String,
    pub task_id: String,
    pub from_agent: String,
    pub to_agent: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: DelegationStatus,
}

/// 委托状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationStatus {
    Pending,
    Accepted,
    Rejected,
    Completed,
    Failed,
}

impl Default for TaskDelegator {
    fn default() -> Self {
        Self::new(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_delegator() {
        let delegator = TaskDelegator::new(3);
        
        let task = Task::new(
            "Test task".to_string(),
            Message::user_message("Hello".to_string()),
        );

        let result = futures::executor::block_on(
            delegator.delegate_task(task, "target-agent", 0)
        );

        assert!(result.is_ok());
        let delegation_result = result.unwrap();
        assert!(delegation_result.success);
    }

    #[test]
    fn test_max_delegation_depth() {
        let delegator = TaskDelegator::new(1);
        
        let task = Task::new(
            "Test task".to_string(),
            Message::user_message("Hello".to_string()),
        );

        let result = futures::executor::block_on(
            delegator.delegate_task(task, "target-agent", 2)
        );

        assert!(result.is_err());
    }
}