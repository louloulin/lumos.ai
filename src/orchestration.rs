//! 简化的Agent编排API
//!
//! 提供简单易用的多Agent协作功能。

use crate::{Result, Error, agent::SimpleAgent};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

// 重导出核心类型
pub use lumosai_core::agent::orchestration::{
    OrchestrationPattern,
    CollaborationTask as CoreCollaborationTask,
    AgentOrchestrator as CoreAgentOrchestrator,
    BasicOrchestrator as CoreBasicOrchestrator,
    AgentExecutionState,
    VotingStrategy,
};

pub use lumosai_core::agent::events::EventBus;

/// 简化的协作任务
#[derive(Clone)]
pub struct CollaborationTask {
    pub name: String,
    pub description: String,
    pub agents: Vec<SimpleAgent>,
    pub pattern: OrchestrationPattern,
    pub input: serde_json::Value,
    pub timeout: Option<u64>,
}

/// Agent编排器
pub type AgentOrchestrator = Arc<dyn CoreAgentOrchestrator>;

/// 基础编排器
pub type BasicOrchestrator = CoreBasicOrchestrator;

/// 编排结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResult {
    pub task_id: String,
    pub results: std::collections::HashMap<String, serde_json::Value>,
    pub execution_time_ms: u64,
    pub status: String,
}

/// 创建协作任务构建器
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let agent1 = lumos::agent::simple("gpt-4", "You are a researcher").await?;
///     let agent2 = lumos::agent::simple("gpt-4", "You are a writer").await?;
///     
///     let task = lumos::orchestration::task()
///         .name("Research and Write")
///         .description("Research a topic and write about it")
///         .agents(vec![agent1, agent2])
///         .pattern(lumos::orchestration::Pattern::Sequential)
///         .input(serde_json::json!({"topic": "AI in healthcare"}))
///         .build();
///     
///     Ok(())
/// }
/// ```
pub fn task() -> TaskBuilder {
    TaskBuilder::new()
}

/// 执行协作任务
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let agent1 = lumos::agent::simple("gpt-4", "You are a researcher").await?;
///     let agent2 = lumos::agent::simple("gpt-4", "You are a writer").await?;
///     
///     let task = lumos::orchestration::task()
///         .name("Research and Write")
///         .agents(vec![agent1, agent2])
///         .pattern(lumos::orchestration::Pattern::Sequential)
///         .build();
///     
///     let result = lumos::orchestration::execute(task).await?;
///     println!("Task completed: {:?}", result);
///     
///     Ok(())
/// }
/// ```
pub async fn execute(task: CollaborationTask) -> Result<OrchestrationResult> {
    let start_time = std::time::Instant::now();

    // 简化实现：模拟协作执行
    let mut results = std::collections::HashMap::new();

    // 根据模式执行
    match task.pattern {
        OrchestrationPattern::Sequential => {
            for (i, agent) in task.agents.iter().enumerate() {
                let agent_id = format!("agent_{}", i);
                let result = format!("Agent {} ({}) processed the task sequentially",
                                   agent.name(), agent_id);
                results.insert(agent_id, serde_json::Value::String(result));
            }
        }
        OrchestrationPattern::Parallel => {
            for (i, agent) in task.agents.iter().enumerate() {
                let agent_id = format!("agent_{}", i);
                let result = format!("Agent {} ({}) processed the task in parallel",
                                   agent.name(), agent_id);
                results.insert(agent_id, serde_json::Value::String(result));
            }
        }
        OrchestrationPattern::Pipeline => {
            for (i, agent) in task.agents.iter().enumerate() {
                let agent_id = format!("agent_{}", i);
                let result = format!("Agent {} ({}) processed the task in pipeline stage {}",
                                   agent.name(), agent_id, i);
                results.insert(agent_id, serde_json::Value::String(result));
            }
        }
        _ => {
            // 简化实现：其他模式暂不支持
            return Err(Error::Agent("Unsupported orchestration pattern".to_string()));
        }
    }

    let execution_time = start_time.elapsed().as_millis() as u64;

    Ok(OrchestrationResult {
        task_id: uuid::Uuid::new_v4().to_string(),
        results,
        execution_time_ms: execution_time,
        status: "completed".to_string(),
    })
}

/// 创建简单的顺序执行任务
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let agents = vec![
///         lumos::agent::simple("gpt-4", "You are a researcher").await?,
///         lumos::agent::simple("gpt-4", "You are a writer").await?,
///     ];
///     
///     let result = lumos::orchestration::sequential(
///         "Research and Write",
///         agents,
///         serde_json::json!({"topic": "AI"})
///     ).await?;
///     
///     Ok(())
/// }
/// ```
pub async fn sequential(
    name: &str,
    agents: Vec<SimpleAgent>,
    input: serde_json::Value,
) -> Result<OrchestrationResult> {
    let task = task()
        .name(name)
        .agents(agents)
        .pattern(OrchestrationPattern::Sequential)
        .input(input)
        .build();
    
    execute(task).await
}

/// 创建简单的并行执行任务
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let agents = vec![
///         lumos::agent::simple("gpt-4", "You are an analyst").await?,
///         lumos::agent::simple("gpt-4", "You are a critic").await?,
///     ];
///     
///     let result = lumos::orchestration::parallel(
///         "Analyze and Critique",
///         agents,
///         serde_json::json!({"content": "Some content to analyze"})
///     ).await?;
///     
///     Ok(())
/// }
/// ```
pub async fn parallel(
    name: &str,
    agents: Vec<SimpleAgent>,
    input: serde_json::Value,
) -> Result<OrchestrationResult> {
    let task = task()
        .name(name)
        .agents(agents)
        .pattern(OrchestrationPattern::Parallel)
        .input(input)
        .build();
    
    execute(task).await
}

/// 任务构建器
pub struct TaskBuilder {
    name: Option<String>,
    description: Option<String>,
    agents: Vec<SimpleAgent>,
    pattern: Option<OrchestrationPattern>,
    input: Option<serde_json::Value>,
    timeout: Option<u64>,
}

impl TaskBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            agents: Vec::new(),
            pattern: None,
            input: None,
            timeout: None,
        }
    }
    
    /// 设置任务名称
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    
    /// 设置任务描述
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// 设置参与的Agent
    pub fn agents(mut self, agents: Vec<SimpleAgent>) -> Self {
        self.agents = agents;
        self
    }
    
    /// 添加单个Agent
    pub fn agent(mut self, agent: SimpleAgent) -> Self {
        self.agents.push(agent);
        self
    }
    
    /// 设置编排模式
    pub fn pattern(mut self, pattern: OrchestrationPattern) -> Self {
        self.pattern = Some(pattern);
        self
    }
    
    /// 设置输入数据
    pub fn input(mut self, input: serde_json::Value) -> Self {
        self.input = Some(input);
        self
    }
    
    /// 设置超时时间（秒）
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// 构建协作任务
    pub fn build(self) -> CollaborationTask {
        CollaborationTask {
            name: self.name.unwrap_or_else(|| "Collaboration Task".to_string()),
            description: self.description.unwrap_or_else(|| "A collaboration task".to_string()),
            agents: self.agents,
            pattern: self.pattern.unwrap_or(OrchestrationPattern::Sequential),
            input: self.input.unwrap_or(serde_json::json!({})),
            timeout: self.timeout,
        }
    }
}

impl Default for TaskBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_builder() {
        let task = task()
            .name("Test Task")
            .description("A test collaboration task")
            .pattern(OrchestrationPattern::Sequential)
            .input(serde_json::json!({"test": "data"}))
            .build();
        
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.description, "A test collaboration task");
        assert!(matches!(task.pattern, OrchestrationPattern::Sequential));
    }
    
    #[test]
    fn test_orchestration_result_serialization() {
        let result = OrchestrationResult {
            task_id: "test_task".to_string(),
            results: std::collections::HashMap::new(),
            execution_time_ms: 1000,
            status: "completed".to_string(),
        };
        
        let json = serde_json::to_string(&result).expect("Failed to serialize");
        let _deserialized: OrchestrationResult = serde_json::from_str(&json).expect("Failed to deserialize");
    }
}
