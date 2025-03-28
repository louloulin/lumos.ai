use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::agent::{Agent, AgentGenerateOptions};
use crate::error::Result;
use crate::llm::{Message, Role};

/// 工作流步骤执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// 步骤名称
    pub step_name: String,
    /// 是否成功完成
    pub success: bool,
    /// 步骤输出
    pub output: Value,
    /// 错误消息（如果失败）
    pub error: Option<String>,
    /// 执行耗时（毫秒）
    pub execution_time_ms: u64,
}

/// 工作流接口
#[async_trait]
pub trait Workflow: Send + Sync {
    /// 执行工作流
    async fn execute(&self, input: Value) -> Result<Value>;
    
    /// 获取工作流名称
    fn name(&self) -> &str;
    
    /// 获取工作流描述
    fn description(&self) -> Option<&str>;
    
    /// 获取工作流步骤列表
    fn steps(&self) -> Vec<String>;
}

/// 工作流步骤条件
pub enum StepCondition {
    /// 指定步骤完成后执行
    StepCompleted(String),
    /// 指定步骤失败后执行
    StepFailed(String),
    /// 指定步骤输出包含特定值时执行
    OutputContains { step: String, key: String, value: Value },
    /// 同时满足多个条件
    And(Vec<StepCondition>),
    /// 满足任一条件
    Or(Vec<StepCondition>),
    /// 总是执行
    Always,
}

impl StepCondition {
    /// 检查条件是否满足
    pub fn is_satisfied(&self, results: &HashMap<String, StepResult>) -> bool {
        match self {
            StepCondition::StepCompleted(step) => {
                results.get(step).map_or(false, |result| result.success)
            },
            StepCondition::StepFailed(step) => {
                results.get(step).map_or(false, |result| !result.success)
            },
            StepCondition::OutputContains { step, key, value } => {
                if let Some(result) = results.get(step) {
                    if result.success {
                        if let Some(output) = result.output.as_object() {
                            if let Some(actual) = output.get(key) {
                                return actual == value;
                            }
                        }
                    }
                }
                false
            },
            StepCondition::And(conditions) => {
                conditions.iter().all(|cond| cond.is_satisfied(results))
            },
            StepCondition::Or(conditions) => {
                conditions.iter().any(|cond| cond.is_satisfied(results))
            },
            StepCondition::Always => true,
        }
    }
}

/// 工作流步骤
pub struct WorkflowStep {
    /// 步骤名称
    pub name: String,
    /// 执行步骤的代理
    pub agent: Arc<dyn Agent>,
    /// 步骤指令
    pub instructions: String,
    /// 执行条件
    pub condition: StepCondition,
    /// 超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    /// 重试次数
    pub retry_count: Option<u32>,
}

/// 基本工作流实现
pub struct BasicWorkflow {
    /// 工作流名称
    name: String,
    /// 工作流描述
    description: Option<String>,
    /// 工作流步骤
    steps: Vec<WorkflowStep>,
}

impl BasicWorkflow {
    /// 创建新的基本工作流
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            steps: Vec::new(),
        }
    }
    
    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// 添加步骤
    pub fn add_step(&mut self, step: WorkflowStep) {
        self.steps.push(step);
    }
}

#[async_trait]
impl Workflow for BasicWorkflow {
    async fn execute(&self, input: Value) -> Result<Value> {
        let mut step_results = HashMap::new();
        let mut final_output = input.clone();
        
        for step in &self.steps {
            if step.condition.is_satisfied(&step_results) {
                println!("执行步骤: {}", step.name);
                
                // 准备步骤输入
                let step_input = serde_json::json!({
                    "workflow_input": input,
                    "current_output": final_output,
                    "step_results": step_results,
                    "instructions": step.instructions
                });
                
                // 创建用户消息
                let user_message = Message {
                    role: Role::User,
                    content: step_input.to_string(),
                    name: None,
                    metadata: None,
                };
                
                // 执行步骤
                let start_time = std::time::Instant::now();
                let step_output = match step.agent.generate(&[user_message], &AgentGenerateOptions::default()).await {
                    Ok(output) => {
                        // 尝试解析JSON输出
                        match serde_json::from_str::<Value>(&output.response) {
                            Ok(json) => json,
                            Err(_) => serde_json::json!({ "text": output.response }),
                        }
                    },
                    Err(e) => {
                        // 记录错误并继续
                        let step_result = StepResult {
                            step_name: step.name.clone(),
                            success: false,
                            output: Value::Null,
                            error: Some(e.to_string()),
                            execution_time_ms: start_time.elapsed().as_millis() as u64,
                        };
                        step_results.insert(step.name.clone(), step_result);
                        continue;
                    }
                };
                
                // 更新结果
                let step_result = StepResult {
                    step_name: step.name.clone(),
                    success: true,
                    output: step_output.clone(),
                    error: None,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                };
                
                step_results.insert(step.name.clone(), step_result);
                final_output = step_output;
            }
        }
        
        Ok(final_output)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    fn steps(&self) -> Vec<String> {
        self.steps.iter().map(|step| step.name.clone()).collect()
    }
}

/// 创建基本的工作流
pub fn create_basic_workflow(name: impl Into<String>) -> BasicWorkflow {
    BasicWorkflow::new(name)
} 