//! # 规划模块
//!
//! 任务分解和规划能力。

use serde::{Deserialize, Serialize};

use super::ReasoningError;

/// 规划配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningConfig {
    /// 最大子任务数
    pub max_subtasks: usize,
    /// 是否启用并行执行
    pub enable_parallel: bool,
}

impl Default for PlanningConfig {
    fn default() -> Self {
        Self {
            max_subtasks: 10,
            enable_parallel: true,
        }
    }
}

/// 任务依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    /// 依赖的任务 ID
    pub task_id: String,
    /// 依赖类型
    pub dependency_type: DependencyType,
}

/// 依赖类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// 强依赖 (必须完成)
    Strong,
    /// 弱依赖 (可选)
    Weak,
}

/// 子任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    /// 任务 ID
    pub id: String,
    /// 任务描述
    pub description: String,
    /// 依赖的任务
    pub dependencies: Vec<TaskDependency>,
    /// 预估时间 (秒)
    pub estimated_duration: u64,
    /// 是否完成
    pub completed: bool,
}

/// 任务计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    /// 主任务
    pub main_task: String,
    /// 子任务列表
    pub subtasks: Vec<SubTask>,
    /// 总预估时间
    pub total_duration: u64,
}

/// 规划器
pub struct Planner {
    config: PlanningConfig,
}

impl Planner {
    pub fn new(config: PlanningConfig) -> Self {
        Self { config }
    }

    pub async fn plan(&self, task: &str) -> Result<TaskPlan, ReasoningError> {
        // 简化实现: 自动分解任务
        let subtasks = vec![
            SubTask {
                id: "1".to_string(),
                description: format!("分析任务: {}", task),
                dependencies: vec![],
                estimated_duration: 60,
                completed: false,
            },
            SubTask {
                id: "2".to_string(),
                description: "制定执行方案".to_string(),
                dependencies: vec![TaskDependency {
                    task_id: "1".to_string(),
                    dependency_type: DependencyType::Strong,
                }],
                estimated_duration: 120,
                completed: false,
            },
            SubTask {
                id: "3".to_string(),
                description: "执行并验证".to_string(),
                dependencies: vec![TaskDependency {
                    task_id: "2".to_string(),
                    dependency_type: DependencyType::Strong,
                }],
                estimated_duration: 300,
                completed: false,
            },
        ];

        let total_duration: u64 = subtasks.iter().map(|t| t.estimated_duration).sum();

        Ok(TaskPlan {
            main_task: task.to_string(),
            subtasks,
            total_duration,
        })
    }
}

impl Default for Planner {
    fn default() -> Self {
        Self::new(PlanningConfig::default())
    }
}
