//! 工作流编排引擎模块
//!
//! 提供强大的工作流编排能力，支持条件分支、并行执行和复杂的工作流模式。

use crate::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{
    agent::{Agent, AgentConfig},
    tools::{ToolRegistry, Tool},
};

/// 工作流步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStep {
    /// Agent调用
    AgentCall {
        id: String,
        agent_id: String,
        input_template: String,
        output_variable: String,
        timeout_seconds: Option<u64>,
        retry_count: Option<u32>,
    },
    
    /// 工具调用
    ToolCall {
        id: String,
        tool_name: String,
        parameters: serde_json::Value,
        output_variable: String,
        timeout_seconds: Option<u64>,
        retry_count: Option<u32>,
    },
    
    /// 条件分支
    Condition {
        id: String,
        condition_expression: String,
        true_branch: Vec<WorkflowStep>,
        false_branch: Vec<WorkflowStep>,
    },
    
    /// 并行执行
    Parallel {
        id: String,
        branches: Vec<Vec<WorkflowStep>>,
        wait_for_all: Option<bool>, // true: 等待所有分支完成, false: 任何一个完成即可
    },
    
    /// 循环
    Loop {
        id: String,
        loop_variable: String,
        start_value: serde_json::Value,
        end_condition: String,
        step_value: serde_json::Value,
        steps: Vec<WorkflowStep>,
        max_iterations: Option<u32>,
    },
    
    /// 等待
    Delay {
        id: String,
        duration_seconds: u64,
    },
    
    /// 设置变量
    SetVariable {
        id: String,
        variable_name: String,
        value: serde_json::Value,
    },
    
    /// 记录日志
    Log {
        id: String,
        level: LogLevel,
        message_template: String,
        include_context: Option<bool>,
    },
}

/// 日志级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<WorkflowStep>,
    pub variables: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
    pub timeout_seconds: Option<u64>,
    pub retry_policy: Option<RetryPolicy>,
}

/// 重试策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub retry_on_error_types: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            retry_on_error_types: vec!["NetworkError".to_string(), "TimeoutError".to_string()],
        }
    }
}

/// 工作流执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub workflow_id: String,
    pub execution_id: String,
    pub status: WorkflowStatus,
    pub result_variables: HashMap<String, serde_json::Value>,
    pub execution_log: Vec<LogEntry>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
}

/// 工作流状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub step_id: Option<String>,
    pub level: LogLevel,
    pub message: String,
    pub context: HashMap<String, serde_json::Value>,
}

/// 工作流执行引擎
pub struct WorkflowEngine {
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
    tool_registry: Arc<ToolRegistry>,
    execution_history: Arc<RwLock<HashMap<String, WorkflowResult>>>,
    template_engine: TemplateEngine,
}

/// 模板引擎
pub struct TemplateEngine;

impl TemplateEngine {
    pub fn render_template(
        &self,
        template: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let mut result = template.to_string();
        
        // 简单的变量替换 {{variable}}
        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
        
        Ok(result)
    }
    
    pub fn evaluate_condition(
        &self,
        condition: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        // 简单的条件表达式评估
        // 支持格式: variable == "value", variable != null, variable > 10 等
        
        let condition = condition.trim();
        
        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let left = self.get_variable_value(parts[0].trim(), context)?;
                let right = self.parse_value(parts[1].trim())?;
                return Ok(left == right);
            }
        }
        
        if condition.contains("!=") {
            let parts: Vec<&str> = condition.split("!=").collect();
            if parts.len() == 2 {
                let left = self.get_variable_value(parts[0].trim(), context)?;
                let right = self.parse_value(parts[1].trim())?;
                return Ok(left != right);
            }
        }
        
        if condition.contains(">") {
            let parts: Vec<&str> = condition.split(">").collect();
            if parts.len() == 2 {
                let left = self.get_numeric_value(parts[0].trim(), context)?;
                let right = self.parse_numeric_value(parts[1].trim())?;
                return Ok(left > right);
            }
        }
        
        if condition.contains("<") {
            let parts: Vec<&str> = condition.split("<").collect();
            if parts.len() == 2 {
                let left = self.get_numeric_value(parts[0].trim(), context)?;
                let right = self.parse_numeric_value(parts[1].trim())?;
                return Ok(left < right);
            }
        }
        
        // 检查变量是否存在且不为null
        if let Some(var_name) = condition.strip_prefix("!") {
            let value = self.get_variable_value(var_name.trim(), context)?;
            return Ok(value != serde_json::Value::Null);
        }
        
        // 默认检查变量是否存在
        let value = self.get_variable_value(condition, context)?;
        Ok(value != serde_json::Value::Null)
    }
    
    fn get_variable_value(
        &self,
        var_name: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let var_name = var_name.trim_matches('"').trim();
        Ok(context.get(var_name)
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }
    
    fn get_numeric_value(
        &self,
        expr: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<f64> {
        let value = self.get_variable_value(expr, context)?;
        match value {
            serde_json::Value::Number(n) => Ok(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(s) => s.parse().map_err(|_| {
                Error::Other(format!("Cannot parse '{}' as number", s))
            }),
            _ => Err(Error::Other(format!("Invalid numeric value: {:?}", value))),
        }
    }
    
    fn parse_value(&self, value_str: &str) -> Result<serde_json::Value> {
        let value_str = value_str.trim().trim_matches('"');
        
        // 尝试解析为数字
        if let Ok(num) = value_str.parse::<f64>() {
            return Ok(serde_json::Value::Number(serde_json::Number::from_f64(num).unwrap()));
        }
        
        // 尝试解析为布尔值
        match value_str.to_lowercase().as_str() {
            "true" => return Ok(serde_json::Value::Bool(true)),
            "false" => return Ok(serde_json::Value::Bool(false)),
            "null" => return Ok(serde_json::Value::Null),
            _ => {}
        }
        
        // 默认作为字符串
        Ok(serde_json::Value::String(value_str.to_string()))
    }
    
    fn parse_numeric_value(&self, value_str: &str) -> Result<f64> {
        let value_str = value_str.trim().trim_matches('"');
        value_str.parse::<f64>().map_err(|_| {
            Error::Other(format!("Cannot parse '{}' as number", value_str))
        })
    }
}

impl WorkflowEngine {
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tool_registry,
            execution_history: Arc::new(RwLock::new(HashMap::new())),
            template_engine: TemplateEngine,
        }
    }
    
    /// 注册Agent
    pub async fn register_agent(&self, agent_id: String, agent: Arc<dyn Agent>) -> Result<()> {
        let mut agents = self.agents.write().await;
        agents.insert(agent_id, agent);
        Ok(())
    }
    
    /// 执行工作流
    pub async fn execute_workflow(&self, workflow: &Workflow) -> Result<WorkflowResult> {
        let execution_id = uuid::Uuid::new_v4().to_string();
        let start_time = chrono::Utc::now();
        
        let mut result = WorkflowResult {
            workflow_id: workflow.id.clone(),
            execution_id: execution_id.clone(),
            status: WorkflowStatus::Running,
            result_variables: workflow.variables.clone(),
            execution_log: Vec::new(),
            started_at: start_time,
            completed_at: None,
            error_message: None,
        };
        
        // 记录开始日志
        self.add_log_entry(&mut result, None, LogLevel::Info, 
            &format!("Starting workflow execution: {}", workflow.name));
        
        // 执行工作流步骤
        let execution_result = self.execute_steps(&workflow.steps, &mut result).await;
        
        // 更新最终状态
        let completion_time = chrono::Utc::now();
        result.completed_at = Some(completion_time);
        
        match execution_result {
            Ok(_) => {
                result.status = WorkflowStatus::Completed;
                self.add_log_entry(&mut result, None, LogLevel::Info, "Workflow completed successfully");
            }
            Err(e) => {
                result.status = WorkflowStatus::Failed;
                result.error_message = Some(e.to_string());
                self.add_log_entry(&mut result, None, LogLevel::Error, 
                    &format!("Workflow failed: {}", e));
            }
        }
        
        // 保存执行历史
        {
            let mut history = self.execution_history.write().await;
            history.insert(execution_id.clone(), result.clone());
        }
        
        Ok(result)
    }
    
    /// 执行工作流步骤
    async fn execute_steps(
        &self,
        steps: &[WorkflowStep],
        result: &mut WorkflowResult,
    ) -> Result<()> {
        for step in steps {
            if let Err(e) = self.execute_step(step, result).await {
                self.add_log_entry(result, Some(&self.get_step_id(step)), LogLevel::Error, 
                    &format!("Step execution failed: {}", e));
                return Err(e);
            }
        }
        Ok(())
    }
    
    /// 执行单个步骤
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        result: &mut WorkflowResult,
    ) -> Result<()> {
        let step_id = self.get_step_id(step);
        
        match step {
            WorkflowStep::AgentCall { id, agent_id, input_template, output_variable, timeout_seconds, retry_count } => {
                self.add_log_entry(result, Some(id), LogLevel::Info, 
                    &format!("Executing agent call: {}", agent_id));
                
                let agent = {
                    let agents = self.agents.read().await;
                    agents.get(agent_id)
                        .ok_or_else(|| Error::Other(format!("Agent '{}' not found", agent_id)))?
                        .clone()
                };
                
                // 渲染输入模板
                let input = self.template_engine.render_template(input_template, &result.result_variables)?;
                
                // 执行Agent调用（带超时）
                let response = match timeout_seconds {
                    Some(seconds) => {
                        tokio::time::timeout(
                            std::time::Duration::from_secs(*seconds),
                            agent.process(&input)
                        ).await
                        .map_err(|_| Error::Other(format!("Agent call timed out after {} seconds", seconds)))?
                    }
                    None => agent.process(&input).await
                }?;
                
                // 保存输出结果
                result.result_variables.insert(output_variable.clone(), 
                    serde_json::Value::String(response));
                
                self.add_log_entry(result, Some(id), LogLevel::Info, "Agent call completed");
            }
            
            WorkflowStep::ToolCall { id, tool_name, parameters, output_variable, timeout_seconds, retry_count } => {
                self.add_log_entry(result, Some(id), LogLevel::Info, 
                    &format!("Executing tool call: {}", tool_name));
                
                // 渲染参数模板
                let rendered_parameters = self.render_json_template(parameters, &result.result_variables)?;
                
                // 执行工具调用（带超时）
                let tool_result = match timeout_seconds {
                    Some(seconds) => {
                        tokio::time::timeout(
                            std::time::Duration::from_secs(*seconds),
                            self.tool_registry.call_tool(tool_name, rendered_parameters, &[])
                        ).await
                        .map_err(|_| Error::Other(format!("Tool call timed out after {} seconds", seconds)))?
                    }
                    None => self.tool_registry.call_tool(tool_name, rendered_parameters, &[]).await
                }?;
                
                // 保存输出结果
                result.result_variables.insert(output_variable.clone(), tool_result);
                
                self.add_log_entry(result, Some(id), LogLevel::Info, "Tool call completed");
            }
            
            WorkflowStep::Condition { id, condition_expression, true_branch, false_branch } => {
                self.add_log_entry(result, Some(id), LogLevel::Info, 
                    &format!("Evaluating condition: {}", condition_expression));
                
                let condition_result = self.template_engine.evaluate_condition(
                    condition_expression, 
                    &result.result_variables
                )?;
                
                let branch = if condition_result { true_branch } else { false_branch };
                
                self.add_log_entry(result, Some(id), LogLevel::Debug, 
                    &format!("Condition evaluated to: {}, executing {} branch", 
                        condition_result, if condition_result { "true" } else { "false" }));
                
                self.execute_steps(branch, result).await?;
            }
            
            WorkflowStep::Parallel { id, branches, wait_for_all } => {
                self.add_log_entry(result, Some(id), LogLevel::Info, 
                    &format!("Executing {} parallel branches", branches.len()));
                
                let wait_for_all = wait_for_all.unwrap_or(true);
                
                if wait_for_all {
                    // 等待所有分支完成
                    let mut tasks = Vec::new();
                    for branch in branches {
                        let mut branch_result = result.clone();
                        let task = tokio::spawn(async move {
                            self.execute_steps(branch, &mut branch_result).await
                                .map(|_| branch_result.result_variables)
                        });
                        tasks.push(task);
                    }
                    
                    for task in tasks {
                        let branch_variables = task.await
                            .map_err(|e| Error::Other(format!("Parallel task join error: {}", e)))??;
                        // 合并分支结果变量
                        result.result_variables.extend(branch_variables);
                    }
                } else {
                    // 等待任何一个分支完成
                    let mut tasks = Vec::new();
                    for branch in branches {
                        let mut branch_result = result.clone();
                        let task = tokio::spawn(async move {
                            self.execute_steps(branch, &mut branch_result).await
                                .map(|_| branch_result.result_variables)
                        });
                        tasks.push(task);
                    }
                    
                    // 使用select!等待第一个完成的任务
                    let (result_variables, _, remaining) = futures::future::select_all(tasks).await;
                    result.result_variables.extend(result_variables.map_err(|e| {
                        Error::Other(format!("Parallel task error: {}", e))
                    })?);
                    
                    // 取消剩余任务
                    for task in remaining {
                        task.abort();
                    }
                }
                
                self.add_log_entry(result, Some(id), LogLevel::Info, "Parallel execution completed");
            }
            
            WorkflowStep::Loop { id, loop_variable, start_value, end_condition, step_value, steps, max_iterations } => {
                self.add_log_entry(result, Some(id), LogLevel::Info, 
                    &format!("Starting loop with variable: {}", loop_variable));
                
                let mut current_value = start_value.clone();
                let mut iterations = 0;
                let max_iterations = max_iterations.unwrap_or(100);
                
                loop {
                    iterations += 1;
                    if iterations > max_iterations {
                        return Err(Error::Other(format!("Loop exceeded maximum iterations: {}", max_iterations)));
                    }
                    
                    // 设置循环变量
                    result.result_variables.insert(loop_variable.clone(), current_value.clone());
                    
                    // 检查结束条件
                    let should_continue = !self.template_engine.evaluate_condition(
                        end_condition, 
                        &result.result_variables
                    )?;
                    
                    if !should_continue {
                        break;
                    }
                    
                    // 执行循环体
                    self.execute_steps(steps, result).await?;
                    
                    // 更新循环变量
                    current_value = self.apply_step_operation(&current_value, step_value)?;
                }
                
                self.add_log_entry(result, Some(id), LogLevel::Info, 
                    &format!("Loop completed after {} iterations", iterations));
            }
            
            WorkflowStep::Delay { id, duration_seconds } => {
                self.add_log_entry(result, Some(id), LogLevel::Info, 
                    &format!("Delaying for {} seconds", duration_seconds));
                
                tokio::time::sleep(std::time::Duration::from_secs(*duration_seconds)).await;
            }
            
            WorkflowStep::SetVariable { id, variable_name, value } => {
                let rendered_value = self.render_json_template(value, &result.result_variables)?;
                result.result_variables.insert(variable_name.clone(), rendered_value);
                
                self.add_log_entry(result, Some(id), LogLevel::Debug, 
                    &format!("Set variable: {} = {}", variable_name, value));
            }
            
            WorkflowStep::Log { id, level, message_template, include_context } => {
                let message = self.template_engine.render_template(message_template, &result.result_variables)?;
                
                let mut context = HashMap::new();
                if include_context.unwrap_or(false) {
                    context = result.result_variables.clone();
                }
                
                self.add_log_entry(result, Some(id), level.clone(), &message);
            }
        }
        
        Ok(())
    }
    
    /// 渲染JSON模板
    fn render_json_template(
        &self,
        json_value: &serde_json::Value,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        match json_value {
            serde_json::Value::String(s) => {
                let rendered = self.template_engine.render_template(s, context)?;
                Ok(serde_json::Value::String(rendered))
            }
            serde_json::Value::Object(map) => {
                let mut result = serde_json::Map::new();
                for (key, value) in map {
                    result.insert(key.clone(), self.render_json_template(value, context)?);
                }
                Ok(serde_json::Value::Object(result))
            }
            serde_json::Value::Array(arr) => {
                let mut result = Vec::new();
                for value in arr {
                    result.push(self.render_json_template(value, context)?);
                }
                Ok(serde_json::Value::Array(result))
            }
            other => Ok(other.clone()),
        }
    }
    
    /// 应用步骤操作（用于循环变量递增/递减）
    fn apply_step_operation(&self, current: &serde_json::Value, step: &serde_json::Value) -> Result<serde_json::Value> {
        match (current, step) {
            (serde_json::Value::Number(curr), serde_json::Value::Number(step_val)) => {
                let curr_f64 = curr.as_f64().unwrap_or(0.0);
                let step_f64 = step_val.as_f64().unwrap_or(0.0);
                let new_value = curr_f64 + step_f64;
                
                if new_value.fract() == 0.0 {
                    Ok(serde_json::Value::Number(serde_json::Number::from(new_value as i64)))
                } else {
                    Ok(serde_json::Value::Number(
                        serde_json::Number::from_f64(new_value).unwrap()
                    ))
                }
            }
            _ => Err(Error::Other("Invalid step operation: both values must be numbers".to_string())),
        }
    }
    
    /// 获取步骤ID
    fn get_step_id(&self, step: &WorkflowStep) -> String {
        match step {
            WorkflowStep::AgentCall { id, .. } => id.clone(),
            WorkflowStep::ToolCall { id, .. } => id.clone(),
            WorkflowStep::Condition { id, .. } => id.clone(),
            WorkflowStep::Parallel { id, .. } => id.clone(),
            WorkflowStep::Loop { id, .. } => id.clone(),
            WorkflowStep::Delay { id, .. } => id.clone(),
            WorkflowStep::SetVariable { id, .. } => id.clone(),
            WorkflowStep::Log { id, .. } => id.clone(),
        }
    }
    
    /// 添加日志条目
    fn add_log_entry(
        &self,
        result: &mut WorkflowResult,
        step_id: Option<&str>,
        level: LogLevel,
        message: &str,
    ) {
        let entry = LogEntry {
            timestamp: chrono::Utc::now(),
            step_id: step_id.map(|s| s.to_string()),
            level,
            message: message.to_string(),
            context: result.result_variables.clone(),
        };
        
        result.execution_log.push(entry);
    }
    
    /// 获取工作流执行历史
    pub async fn get_execution_history(&self, execution_id: &str) -> Option<WorkflowResult> {
        let history = self.execution_history.read().await;
        history.get(execution_id).cloned()
    }
    
    /// 获取所有工作流执行历史
    pub async fn get_all_executions(&self) -> HashMap<String, WorkflowResult> {
        let history = self.execution_history.read().await;
        history.clone()
    }
}

/// 工作流构建器，用于方便地创建工作流
pub struct WorkflowBuilder {
    workflow: Workflow,
}

impl WorkflowBuilder {
    pub fn new(id: String, name: String) -> Self {
        Self {
            workflow: Workflow {
                id,
                name,
                description: None,
                steps: Vec::new(),
                variables: HashMap::new(),
                metadata: HashMap::new(),
                timeout_seconds: None,
                retry_policy: None,
            },
        }
    }
    
    pub fn description(mut self, description: String) -> Self {
        self.workflow.description = Some(description);
        self
    }
    
    pub fn variable(mut self, name: String, value: serde_json::Value) -> Self {
        self.workflow.variables.insert(name, value);
        self
    }
    
    pub fn agent_call(
        mut self,
        id: String,
        agent_id: String,
        input_template: String,
        output_variable: String,
    ) -> Self {
        self.workflow.steps.push(WorkflowStep::AgentCall {
            id,
            agent_id,
            input_template,
            output_variable,
            timeout_seconds: None,
            retry_count: None,
        });
        self
    }
    
    pub fn tool_call(
        mut self,
        id: String,
        tool_name: String,
        parameters: serde_json::Value,
        output_variable: String,
    ) -> Self {
        self.workflow.steps.push(WorkflowStep::ToolCall {
            id,
            tool_name,
            parameters,
            output_variable,
            timeout_seconds: None,
            retry_count: None,
        });
        self
    }
    
    pub fn condition(
        mut self,
        id: String,
        condition_expression: String,
        true_branch: Vec<WorkflowStep>,
        false_branch: Vec<WorkflowStep>,
    ) -> Self {
        self.workflow.steps.push(WorkflowStep::Condition {
            id,
            condition_expression,
            true_branch,
            false_branch,
        });
        self
    }
    
    pub fn parallel(mut self, id: String, branches: Vec<Vec<WorkflowStep>>) -> Self {
        self.workflow.steps.push(WorkflowStep::Parallel {
            id,
            branches,
            wait_for_all: Some(true),
        });
        self
    }
    
    pub fn delay(mut self, id: String, duration_seconds: u64) -> Self {
        self.workflow.steps.push(WorkflowStep::Delay { id, duration_seconds });
        self
    }
    
    pub fn set_variable(mut self, id: String, variable_name: String, value: serde_json::Value) -> Self {
        self.workflow.steps.push(WorkflowStep::SetVariable {
            id,
            variable_name,
            value,
        });
        self
    }
    
    pub fn log(mut self, id: String, level: LogLevel, message_template: String) -> Self {
        self.workflow.steps.push(WorkflowStep::Log {
            id,
            level,
            message_template,
            include_context: Some(false),
        });
        self
    }
    
    pub fn build(self) -> Workflow {
        self.workflow
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::SimpleAgent;
    use crate::llm::MockLlmProvider;
    use builtin_tools::register_builtin_tools;

    #[tokio::test]
    async fn test_template_engine() {
        let engine = TemplateEngine;
        let mut context = HashMap::new();
        context.insert("name".to_string(), serde_json::Value::String("World".to_string()));
        context.insert("count".to_string(), serde_json::Value::Number(serde_json::Number::from(42)));
        
        let result = engine.render_template("Hello, {{name}}! Count: {{count}}", &context).unwrap();
        assert_eq!(result, "Hello, World! Count: 42");
        
        // 测试条件评估
        assert!(engine.evaluate_condition("count == 42", &context).unwrap());
        assert!(engine.evaluate_condition("name == \"World\"", &context).unwrap());
        assert!(engine.evaluate_condition("count > 40", &context).unwrap());
        assert!(engine.evaluate_condition("count < 50", &context).unwrap());
        assert!(!engine.evaluate_condition("count == 100", &context).unwrap());
    }
    
    #[tokio::test]
    async fn test_workflow_builder() {
        let workflow = WorkflowBuilder::new(
            "test-workflow".to_string(),
            "Test Workflow".to_string(),
        )
        .description("A simple test workflow".to_string())
        .variable("input_text".to_string(), serde_json::Value::String("Hello".to_string()))
        .agent_call(
            "step1".to_string(),
            "test-agent".to_string(),
            "{{input_text}}".to_string(),
            "agent_response".to_string(),
        )
        .log(
            "log-step".to_string(),
            LogLevel::Info,
            "Agent responded: {{agent_response}}".to_string(),
        )
        .build();
        
        assert_eq!(workflow.id, "test-workflow");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.steps.len(), 2);
        assert!(workflow.variables.contains_key("input_text"));
    }
    
    #[tokio::test]
    async fn test_simple_workflow_execution() {
        let tool_registry = Arc::new(ToolRegistry::new());
        register_builtin_tools(&tool_registry).await.unwrap();
        
        let engine = WorkflowEngine::new(tool_registry);
        
        // 创建一个简单的工作流
        let workflow = WorkflowBuilder::new(
            "simple-workflow".to_string(),
            "Simple Test Workflow".to_string(),
        )
        .tool_call(
            "calc-step".to_string(),
            "calculator".to_string(),
            serde_json::json!({
                "expression": "10 + 5"
            }),
            "calc_result".to_string(),
        )
        .log(
            "log-step".to_string(),
            LogLevel::Info,
            "Calculation result: {{calc_result.result}}".to_string(),
        )
        .build();
        
        let result = engine.execute_workflow(&workflow).await.unwrap();
        
        assert_eq!(result.status, WorkflowStatus::Completed);
        assert!(result.result_variables.contains_key("calc_result"));
        assert_eq!(result.execution_log.len(), 2);
    }
}