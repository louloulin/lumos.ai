use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use tokio::time::sleep;
use tokio::sync::Mutex;
use std::time::Duration;

use crate::error::Error;
use super::types::{
    Step, StepResult, StepContext, WorkflowRunResult, WorkflowState, 
    StepCondition, DependencyCheckOutput, StepExecutorOutput, RetryConfig,
    ActivePathInfo, StepStatus, StepStatusInfo
};

/// 工作流图节点
#[derive(Clone)]
struct StepNode {
    /// 步骤
    step: Arc<dyn Step>,
    /// 执行前条件
    when: Option<StepCondition>,
    /// 步骤配置数据
    data: serde_json::Value,
}

/// 工作流图
#[derive(Clone, Default)]
struct WorkflowGraph {
    /// 初始节点
    initial: Vec<String>,
    /// 节点之间的连接关系
    edges: HashMap<String, Vec<String>>,
    /// 节点信息
    nodes: HashMap<String, StepNode>,
}

/// 工作流实例
pub struct WorkflowInstance {
    /// 工作流ID
    id: String,
    /// 工作流实例ID
    run_id: String,
    /// 节点图
    graph: Arc<WorkflowGraph>,
    /// 触发数据
    trigger_data: serde_json::Value,
    /// 工作流状态
    state: Arc<Mutex<WorkflowState>>,
    /// 重试配置
    retry_config: Option<RetryConfig>,
}

impl WorkflowInstance {
    /// 创建新的工作流实例
    pub fn new(
        id: String,
        graph: Arc<WorkflowGraph>,
        trigger_data: serde_json::Value,
        retry_config: Option<RetryConfig>,
    ) -> Self {
        let run_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().timestamp() as u64;
        
        let state = Arc::new(Mutex::new(WorkflowState {
            steps: HashMap::new(),
            trigger_data: trigger_data.clone(),
            attempts: HashMap::new(),
            active_paths: Vec::new(),
            run_id: run_id.clone(),
            timestamp,
            child_states: None,
            suspended_steps: None,
        }));
        
        Self {
            id,
            run_id,
            graph,
            trigger_data,
            state,
            retry_config,
        }
    }
    
    /// 开始执行工作流
    pub async fn run(&self) -> Result<WorkflowRunResult, Error> {
        // 使用层次遍历处理工作流步骤
        let mut pending_steps: Vec<String> = self.graph.initial.clone();
        let mut processed_steps: HashSet<String> = HashSet::new();
        
        // 处理所有步骤，直到没有更多步骤或所有步骤都已处理
        while !pending_steps.is_empty() {
            let step_id = pending_steps.remove(0);
            
            // 如果步骤已处理过，跳过
            if processed_steps.contains(&step_id) {
                continue;
            }
            
            // 执行步骤
            if let Err(e) = self.execute_step(&step_id).await {
                eprintln!("执行步骤 {} 失败: {}", step_id, e);
            }
            
            // 标记为已处理
            processed_steps.insert(step_id.clone());
            
            // 添加后续步骤到待处理列表
            if let Some(next_steps) = self.graph.edges.get(&step_id) {
                for next_step in next_steps {
                    if !processed_steps.contains(next_step) {
                        pending_steps.push(next_step.clone());
                    }
                }
            }
        }
        
        // 等待所有步骤执行完成或失败
        self.wait_for_completion().await?;
        
        // 返回工作流结果
        Ok(self.get_result().await)
    }
    
    /// 等待工作流执行完成
    async fn wait_for_completion(&self) -> Result<(), Error> {
        // 简单实现：轮询检查是否所有步骤都完成了
        // 实际实现可能需要更复杂的事件通知机制
        let mut retry_count = 0;
        let max_retries = 100; // 防止无限循环
        
        while retry_count < max_retries {
            let is_completed = {
                let state = self.state.lock().await;
                let active_steps = state.active_paths.len();
                active_steps == 0 || state.suspended_steps.is_some()
            };
            
            if is_completed {
                return Ok(());
            }
            
            // 等待一段时间再检查
            sleep(Duration::from_millis(100)).await;
            retry_count += 1;
        }
        
        Err(Error::Workflow(format!("工作流执行超时: {}", self.id)))
    }
    
    /// 执行单个步骤
    async fn execute_step(&self, step_id: &str) -> Result<StepResult, Error> {
        // 获取步骤节点
        let node = match self.graph.nodes.get(step_id) {
            Some(node) => node.clone(),
            None => {
                return Err(Error::Workflow(format!("未找到步骤: {}", step_id)));
            }
        };
        
        // 首先检查条件是否满足
        let check_result = self.check_dependencies(&node).await?;
        
        match check_result {
            DependencyCheckOutput::ConditionsMet => {
                // 条件满足，执行步骤
                let executor_output = self.run_step(step_id, &node).await?;
                
                match executor_output {
                    StepExecutorOutput::StepSuccess { output } => {
                        // 更新状态
                        let result = StepResult::Success { output: output.clone() };
                        self.update_step_result(step_id, result.clone()).await;
                        Ok(result)
                    },
                    StepExecutorOutput::StepFailed { error } => {
                        // 更新状态
                        let result = StepResult::Failed { error: error.clone() };
                        self.update_step_result(step_id, result.clone()).await;
                        Ok(result)
                    },
                    StepExecutorOutput::StepWaiting => {
                        // 步骤等待中
                        let result = StepResult::Waiting;
                        self.update_step_result(step_id, result.clone()).await;
                        Ok(result)
                    }
                }
            },
            DependencyCheckOutput::ConditionsSkipped => {
                // 条件不满足，跳过步骤
                let result = StepResult::Skipped;
                self.update_step_result(step_id, result.clone()).await;
                Ok(result)
            },
            DependencyCheckOutput::ConditionFailed { error } => {
                // 条件检查失败
                let result = StepResult::Failed { error };
                self.update_step_result(step_id, result.clone()).await;
                Ok(result)
            },
            DependencyCheckOutput::Suspended => {
                // 步骤挂起
                let result = StepResult::Suspended { 
                    suspend_payload: None, 
                    output: None 
                };
                self.update_step_result(step_id, result.clone()).await;
                Ok(result)
            },
            DependencyCheckOutput::Waiting => {
                // 步骤等待
                let result = StepResult::Waiting;
                self.update_step_result(step_id, result.clone()).await;
                Ok(result)
            },
            DependencyCheckOutput::ConditionsLimbo => {
                // 步骤暂停
                let result = StepResult::Waiting;
                self.update_step_result(step_id, result.clone()).await;
                Ok(result)
            }
        }
    }
    
    /// 检查步骤依赖条件
    async fn check_dependencies(&self, node: &StepNode) -> Result<DependencyCheckOutput, Error> {
        if let Some(condition) = &node.when {
            // 获取当前状态
            let state = self.state.lock().await;
            
            // 构建步骤上下文
            let context = StepContext {
                run_id: self.run_id.clone(),
                input_data: node.data.clone(),
                trigger_data: self.trigger_data.clone(),
                steps: state.steps.iter().map(|(k, v)| {
                    let result = match v.status {
                        StepStatus::Success => StepResult::Success { 
                            output: v.payload.clone().unwrap_or(serde_json::Value::Null) 
                        },
                        StepStatus::Failed => StepResult::Failed { 
                            error: v.error.clone().unwrap_or_else(|| "Unknown error".to_string()) 
                        },
                        StepStatus::Suspended => StepResult::Suspended { 
                            suspend_payload: v.payload.clone(), 
                            output: None 
                        },
                        StepStatus::Waiting => StepResult::Waiting,
                        StepStatus::Skipped => StepResult::Skipped,
                        _ => StepResult::Waiting,
                    };
                    (k.clone(), result)
                }).collect(),
                attempts: state.attempts.clone(),
            };
            
            // 根据条件类型进行检查
            match self.evaluate_condition(condition, &context) {
                true => Ok(DependencyCheckOutput::ConditionsMet),
                false => Ok(DependencyCheckOutput::ConditionsSkipped),
            }
        } else {
            // 没有条件，直接满足
            Ok(DependencyCheckOutput::ConditionsMet)
        }
    }
    
    /// 评估条件
    fn evaluate_condition(&self, condition: &StepCondition, context: &StepContext) -> bool {
        match condition {
            StepCondition::Reference { step_id, path, query } => {
                // 获取步骤结果
                if let Some(step_result) = context.get_step_result(step_id) {
                    match step_result {
                        StepResult::Success { output } => {
                            // 使用path获取输出中的特定字段
                            let value = self.get_value_at_path(output, path);
                            
                            // 使用query条件进行检查
                            self.check_query_condition(&value, query)
                        },
                        _ => false, // 非成功状态不满足条件
                    }
                } else {
                    false // 步骤结果不存在
                }
            },
            StepCondition::Simple(map) => {
                // 简单键值条件，所有条件都必须匹配
                map.iter().all(|(key, value)| {
                    let parts: Vec<&str> = key.split('.').collect();
                    if parts.len() != 2 {
                        return false;
                    }
                    
                    let step_id = parts[0];
                    let path = parts[1];
                    
                    if let Some(step_result) = context.get_step_result(step_id) {
                        match step_result {
                            StepResult::Success { output } => {
                                let result_value = self.get_value_at_path(output, path);
                                result_value == *value
                            },
                            _ => false,
                        }
                    } else if step_id == "trigger" {
                        let trigger_value = self.get_value_at_path(&context.trigger_data, path);
                        trigger_value == *value
                    } else {
                        false
                    }
                })
            },
            StepCondition::And(conditions) => {
                // 与条件，所有子条件都必须为真
                conditions.iter().all(|c| self.evaluate_condition(c, context))
            },
            StepCondition::Or(conditions) => {
                // 或条件，任一子条件为真即可
                conditions.iter().any(|c| self.evaluate_condition(c, context))
            },
            StepCondition::Not(condition) => {
                // 非条件，子条件为假时为真
                !self.evaluate_condition(condition, context)
            }
        }
    }
    
    /// 根据路径获取值
    fn get_value_at_path(&self, data: &serde_json::Value, path: &str) -> serde_json::Value {
        if path.is_empty() || path == "." {
            return data.clone();
        }
        
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = data;
        
        for part in parts {
            match current {
                serde_json::Value::Object(obj) => {
                    if let Some(value) = obj.get(part) {
                        current = value;
                    } else {
                        return serde_json::Value::Null;
                    }
                },
                serde_json::Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        if index < arr.len() {
                            current = &arr[index];
                        } else {
                            return serde_json::Value::Null;
                        }
                    } else {
                        return serde_json::Value::Null;
                    }
                },
                _ => return serde_json::Value::Null,
            }
        }
        
        current.clone()
    }
    
    /// 检查查询条件
    fn check_query_condition(&self, value: &serde_json::Value, query: &serde_json::Value) -> bool {
        // 简单实现：只支持$eq操作符
        if let serde_json::Value::Object(obj) = query {
            if let Some(eq_value) = obj.get("$eq") {
                return value == eq_value;
            }
        }
        
        false
    }
    
    /// 运行步骤
    async fn run_step(&self, step_id: &str, node: &StepNode) -> Result<StepExecutorOutput, Error> {
        // 获取状态
        let mut state_guard = self.state.lock().await;
        
        // 增加尝试次数
        let attempts = {
            let count = state_guard.attempts.entry(step_id.to_string()).or_insert(0);
            *count += 1;
            *count
        };
        
        // 检查是否超过最大重试次数
        let max_attempts = node.step.retry_config()
            .and_then(|c| c.attempts)
            .or_else(|| self.retry_config.as_ref().and_then(|c| c.attempts))
            .unwrap_or(1);
        
        if attempts > max_attempts {
            return Ok(StepExecutorOutput::StepFailed { 
                error: format!("超过最大重试次数: {}", max_attempts) 
            });
        }
        
        // 更新步骤状态为运行中
        state_guard.steps.insert(step_id.to_string(), StepStatusInfo {
            status: StepStatus::Running,
            payload: None,
            error: None,
        });
        
        // 添加到活动路径
        state_guard.active_paths.push(ActivePathInfo {
            status: "running".to_string(),
            suspend_payload: None,
            step_path: vec![step_id.to_string()],
        });
        
        // 构建步骤上下文
        let context = {
            StepContext {
                run_id: self.run_id.clone(),
                input_data: node.data.clone(),
                trigger_data: self.trigger_data.clone(),
                steps: state_guard.steps.iter().map(|(k, v)| {
                    let result = match v.status {
                        StepStatus::Success => StepResult::Success { 
                            output: v.payload.clone().unwrap_or(serde_json::Value::Null) 
                        },
                        StepStatus::Failed => StepResult::Failed { 
                            error: v.error.clone().unwrap_or_else(|| "Unknown error".to_string()) 
                        },
                        StepStatus::Suspended => StepResult::Suspended { 
                            suspend_payload: v.payload.clone(), 
                            output: None 
                        },
                        StepStatus::Waiting => StepResult::Waiting,
                        StepStatus::Skipped => StepResult::Skipped,
                        _ => StepResult::Waiting,
                    };
                    (k.clone(), result)
                }).collect(),
                attempts: state_guard.attempts.clone(),
            }
        };
        
        // 释放锁，以便执行步骤时避免死锁
        drop(state_guard);
        
        // 执行步骤
        match node.step.execute(context).await {
            Ok(output) => {
                // 更新状态为成功
                let mut state = self.state.lock().await;
                state.steps.insert(step_id.to_string(), StepStatusInfo {
                    status: StepStatus::Success,
                    payload: Some(output.clone()),
                    error: None,
                });
                
                // 更新活动路径
                state.active_paths.retain(|p| p.step_path != vec![step_id.to_string()]);
                
                Ok(StepExecutorOutput::StepSuccess { output })
            },
            Err(err) => {
                // 更新状态为失败
                let mut state = self.state.lock().await;
                state.steps.insert(step_id.to_string(), StepStatusInfo {
                    status: StepStatus::Failed,
                    payload: None,
                    error: Some(err.to_string()),
                });
                
                // 更新活动路径
                state.active_paths.retain(|p| p.step_path != vec![step_id.to_string()]);
                
                Ok(StepExecutorOutput::StepFailed { error: err.to_string() })
            }
        }
    }
    
    /// 更新步骤结果
    async fn update_step_result(&self, step_id: &str, result: StepResult) {
        let mut state = self.state.lock().await;
        
        let status_info = match &result {
            StepResult::Success { output } => StepStatusInfo {
                status: StepStatus::Success,
                payload: Some(output.clone()),
                error: None,
            },
            StepResult::Failed { error } => StepStatusInfo {
                status: StepStatus::Failed,
                payload: None,
                error: Some(error.clone()),
            },
            StepResult::Suspended { suspend_payload, output } => StepStatusInfo {
                status: StepStatus::Suspended,
                payload: output.clone().or(suspend_payload.clone()),
                error: None,
            },
            StepResult::Waiting => StepStatusInfo {
                status: StepStatus::Waiting,
                payload: None,
                error: None,
            },
            StepResult::Skipped => StepStatusInfo {
                status: StepStatus::Skipped,
                payload: None,
                error: None,
            },
        };
        
        state.steps.insert(step_id.to_string(), status_info);
    }
    
    /// 获取工作流结果
    async fn get_result(&self) -> WorkflowRunResult {
        let state = self.state.lock().await;
        
        // 转换结果
        let results = state.steps.iter().map(|(step_id, status_info)| {
            let result = match status_info.status {
                StepStatus::Success => StepResult::Success { 
                    output: status_info.payload.clone().unwrap_or(serde_json::Value::Null) 
                },
                StepStatus::Failed => StepResult::Failed { 
                    error: status_info.error.clone().unwrap_or_else(|| "Unknown error".to_string()) 
                },
                StepStatus::Suspended => StepResult::Suspended { 
                    suspend_payload: status_info.payload.clone(), 
                    output: None 
                },
                StepStatus::Waiting => StepResult::Waiting,
                StepStatus::Skipped => StepResult::Skipped,
                _ => StepResult::Waiting,
            };
            
            (step_id.clone(), result)
        }).collect();
        
        // 转换活动路径
        let active_paths = state.active_paths.iter().map(|path| {
            let path_info = ActivePathInfo {
                status: path.status.clone(),
                suspend_payload: path.suspend_payload.clone(),
                step_path: path.step_path.clone(),
            };
            
            (path.step_path[0].clone(), path_info)
        }).collect();
        
        WorkflowRunResult {
            run_id: self.run_id.clone(),
            trigger_data: self.trigger_data.clone(),
            result: None, // 这里可以添加结果映射逻辑
            results,
            active_paths,
            timestamp: state.timestamp,
        }
    }
}

/// 工作流定义
#[derive(Clone)]
pub struct WorkflowDefinition {
    /// 图定义
    graph: Arc<WorkflowGraph>,
    /// 重试配置
    retry_config: Option<RetryConfig>,
}

impl WorkflowDefinition {
    /// 创建新的工作流定义
    pub fn new(retry_config: Option<RetryConfig>) -> Self {
        Self {
            graph: Arc::new(WorkflowGraph::default()),
            retry_config,
        }
    }
}

/// 工作流构建器
pub struct WorkflowBuilder {
    /// 工作流ID
    id: String,
    /// 工作流名称
    name: String,
    /// 步骤集
    steps: HashMap<String, Arc<dyn Step>>,
    /// 步骤条件
    conditions: HashMap<String, StepCondition>,
    /// 步骤输入数据
    step_data: HashMap<String, serde_json::Value>,
    /// 步骤关系
    dependencies: HashMap<String, HashSet<String>>,
    /// 重试配置
    retry_config: Option<RetryConfig>,
}

impl WorkflowBuilder {
    /// 创建新的工作流构建器
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            steps: HashMap::new(),
            conditions: HashMap::new(),
            step_data: HashMap::new(),
            dependencies: HashMap::new(),
            retry_config: None,
        }
    }
    
    /// 添加步骤
    pub fn add_step<S: Step + 'static>(
        mut self,
        step: S,
        input_data: Option<serde_json::Value>,
        condition: Option<StepCondition>,
    ) -> Self {
        let step_id = step.id().to_string();
        self.steps.insert(step_id.clone(), Arc::new(step));
        
        if let Some(data) = input_data {
            self.step_data.insert(step_id.clone(), data);
        } else {
            self.step_data.insert(step_id.clone(), serde_json::json!({}));
        }
        
        if let Some(cond) = condition {
            self.conditions.insert(step_id, cond);
        }
        
        self
    }
    
    /// 添加依赖
    pub fn add_dependency(mut self, from_step_id: &str, to_step_id: &str) -> Self {
        self.dependencies
            .entry(from_step_id.to_string())
            .or_default()
            .insert(to_step_id.to_string());
        self
    }
    
    /// 设置重试配置
    pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = Some(retry_config);
        self
    }
    
    /// 构建工作流
    pub fn build(self) -> Workflow {
        // 构建图
        let mut graph = WorkflowGraph::default();
        
        // 添加节点
        for (step_id, step) in &self.steps {
            let condition = self.conditions.get(step_id).cloned();
            let data = self.step_data.get(step_id).cloned().unwrap_or(serde_json::json!({}));
            
            graph.nodes.insert(step_id.clone(), StepNode {
                step: Arc::clone(step),
                when: condition,
                data,
            });
        }
        
        // 添加边
        for (from_id, to_ids) in &self.dependencies {
            let edges = graph.edges.entry(from_id.clone()).or_insert_with(Vec::new);
            for to_id in to_ids {
                edges.push(to_id.clone());
            }
        }
        
        // 确定初始节点（没有其他节点指向的节点）
        let mut has_incoming = HashSet::new();
        for to_ids in graph.edges.values() {
            for to_id in to_ids {
                has_incoming.insert(to_id.clone());
            }
        }
        
        for step_id in self.steps.keys() {
            if !has_incoming.contains(step_id) {
                graph.initial.push(step_id.clone());
            }
        }
        
        // 创建工作流
        Workflow {
            id: self.id,
            name: self.name,
            definition: WorkflowDefinition {
                graph: Arc::new(graph),
                retry_config: self.retry_config,
            },
        }
    }
}

/// 工作流
pub struct Workflow {
    /// 工作流ID
    pub id: String,
    /// 工作流名称
    pub name: String,
    /// 工作流定义
    definition: WorkflowDefinition,
}

impl Workflow {
    /// 创建新的工作流
    pub fn new(id: String, name: String) -> WorkflowBuilder {
        WorkflowBuilder::new(id, name)
    }
    
    /// 创建运行实例
    pub fn create_run(&self, trigger_data: serde_json::Value) -> WorkflowInstance {
        WorkflowInstance::new(
            self.id.clone(),
            Arc::clone(&self.definition.graph),
            trigger_data,
            self.definition.retry_config.clone(),
        )
    }
    
    /// 获取工作流ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// 获取工作流名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// 恢复工作流
pub async fn resume_workflow(
    workflow: &Workflow,
    _run_id: &str,
    _step_id: &str,
    _context: Option<serde_json::Value>,
) -> Result<WorkflowRunResult, Error> {
    // 这里应该有一个存储工作流状态的地方
    // 简化实现，实际上需要从存储中恢复工作流状态
    
    Err(Error::Workflow(format!("恢复工作流尚未实现: {}", workflow.id)))
} 