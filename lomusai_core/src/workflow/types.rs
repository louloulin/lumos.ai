use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use crate::error::Error;

/// 工作流步骤的状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    /// 步骤执行成功
    Success,
    /// 步骤执行失败
    Failed,
    /// 步骤被挂起，等待恢复
    Suspended,
    /// 步骤等待条件满足
    Waiting,
    /// 步骤被跳过
    Skipped,
    /// 步骤正在执行
    Running,
    /// 步骤等待执行
    Pending,
}

/// 表示一个工作流步骤的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepResult {
    /// 步骤执行成功，包含输出数据
    Success {
        /// 步骤的输出数据
        output: serde_json::Value,
    },
    /// 步骤执行失败，包含错误信息
    Failed {
        /// 错误详情
        error: String,
    },
    /// 步骤被挂起，等待外部触发恢复
    Suspended {
        /// 挂起时的上下文数据
        suspend_payload: Option<serde_json::Value>,
        /// 可选的部分输出
        output: Option<serde_json::Value>,
    },
    /// 步骤正在等待条件满足
    Waiting,
    /// 步骤被跳过（条件不满足）
    Skipped,
}

/// 条件检查的返回值
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionReturnValue {
    /// 继续执行
    Continue,
    /// 失败后继续
    ContinueFailed,
    /// 中止执行
    Abort,
    /// 挂起等待
    Limbo,
}

/// 步骤执行的上下文
#[derive(Debug, Clone)]
pub struct StepContext {
    /// 运行ID
    pub run_id: String,
    /// 步骤的输入数据
    pub input_data: serde_json::Value,
    /// 工作流触发数据
    pub trigger_data: serde_json::Value,
    /// 步骤的状态记录
    pub steps: HashMap<String, StepResult>,
    /// 步骤重试次数
    pub attempts: HashMap<String, usize>,
}

impl StepContext {
    /// 创建新的步骤上下文
    pub fn new(
        run_id: String,
        input_data: serde_json::Value,
        trigger_data: serde_json::Value,
    ) -> Self {
        Self {
            run_id,
            input_data,
            trigger_data,
            steps: HashMap::new(),
            attempts: HashMap::new(),
        }
    }

    /// 获取步骤的结果
    pub fn get_step_result(&self, step_id: &str) -> Option<&StepResult> {
        self.steps.get(step_id)
    }

    /// 设置步骤的结果
    pub fn set_step_result(&mut self, step_id: String, result: StepResult) {
        self.steps.insert(step_id, result);
    }

    /// 增加步骤的尝试次数
    pub fn increment_attempt(&mut self, step_id: &str) -> usize {
        let count = self.attempts.entry(step_id.to_string()).or_insert(0);
        *count += 1;
        *count
    }
}

/// 工作流运行的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunResult {
    /// 运行ID
    pub run_id: String,
    /// 工作流触发数据
    pub trigger_data: serde_json::Value,
    /// 工作流结果数据
    pub result: Option<serde_json::Value>,
    /// 所有步骤的结果
    pub results: HashMap<String, StepResult>,
    /// 活动路径
    pub active_paths: HashMap<String, ActivePathInfo>,
    /// 时间戳
    pub timestamp: u64,
}

/// 活动路径信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePathInfo {
    /// 步骤状态
    pub status: String,
    /// 挂起载荷（如果有）
    pub suspend_payload: Option<serde_json::Value>,
    /// 步骤路径
    pub step_path: Vec<String>,
}

/// 工作流状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// 步骤状态
    pub steps: HashMap<String, StepStatusInfo>,
    /// 触发数据
    pub trigger_data: serde_json::Value,
    /// 尝试次数
    pub attempts: HashMap<String, usize>,
    /// 活动路径
    pub active_paths: Vec<ActivePathInfo>,
    /// 运行ID
    pub run_id: String,
    /// 时间戳
    pub timestamp: u64,
    /// 子状态
    pub child_states: Option<HashMap<String, WorkflowState>>,
    /// 挂起的步骤
    pub suspended_steps: Option<HashMap<String, String>>,
}

/// 步骤状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStatusInfo {
    /// 状态
    pub status: StepStatus,
    /// 负载数据
    pub payload: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
}

/// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// 尝试次数
    pub attempts: Option<usize>,
    /// 延迟时间（毫秒）
    pub delay: Option<u64>,
}

/// 步骤的依赖检查结果
#[derive(Debug, Clone)]
pub enum DependencyCheckOutput {
    /// 条件满足
    ConditionsMet,
    /// 条件跳过
    ConditionsSkipped,
    /// 条件检查失败
    ConditionFailed { error: String },
    /// 挂起等待
    Suspended,
    /// 等待条件满足
    Waiting,
    /// 条件暂停
    ConditionsLimbo,
}

/// 步骤执行器的输出
#[derive(Debug, Clone)]
pub enum StepExecutorOutput {
    /// 步骤执行成功
    StepSuccess { output: serde_json::Value },
    /// 步骤执行失败
    StepFailed { error: String },
    /// 步骤等待中
    StepWaiting,
}

/// Step trait定义
#[async_trait]
pub trait Step: Send + Sync {
    /// 获取步骤的ID
    fn id(&self) -> &str;
    
    /// 获取步骤的描述
    fn description(&self) -> &str;
    
    /// 执行步骤
    async fn execute(&self, context: StepContext) -> Result<serde_json::Value, Error>;
    
    /// 获取重试配置
    fn retry_config(&self) -> Option<RetryConfig> {
        None
    }
}

/// 步骤条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepCondition {
    /// 引用其他步骤的输出
    Reference {
        /// 步骤ID
        step_id: String,
        /// 路径
        path: String,
        /// 查询条件（JSON格式）
        query: serde_json::Value,
    },
    /// 简单的键值条件
    Simple(HashMap<String, serde_json::Value>),
    /// 与条件
    And(Vec<StepCondition>),
    /// 或条件
    Or(Vec<StepCondition>),
    /// 非条件
    Not(Box<StepCondition>),
} 