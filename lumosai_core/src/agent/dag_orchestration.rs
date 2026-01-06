//! Agent DAG Orchestration
//!
//! 使用 DAG 调度器编排多个 Agent 的协作执行

use super::trait_def::Agent;
use super::types::RuntimeContext;
use crate::workflow::dag_scheduler::{Dag, DagNode, DagScheduler};
use crate::workflow::{StepExecutor, WorkflowStep};
use crate::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent DAG 编排器
pub struct AgentDagOrchestrator {
    /// DAG 调度器
    scheduler: DagScheduler,
    /// DAG 图
    dag: Arc<RwLock<Dag>>,
    /// Agent 映射
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
}

/// Agent 执行器（将 Agent 包装为 StepExecutor）
struct AgentExecutor {
    agent_id: String,
    agent: Arc<dyn Agent>,
}

#[async_trait]
impl StepExecutor for AgentExecutor {
    async fn execute(&self, input: Value, _context: &RuntimeContext) -> Result<Value> {
        tracing::info!("Executing agent: {}", self.agent_id);

        // 从输入中提取消息
        let message = if let Some(msg) = input.get("message") {
            msg.as_str().unwrap_or("").to_string()
        } else {
            input.to_string()
        };

        // 执行 Agent（使用 generate_simple 方法）
        let response = self.agent.generate_simple(&message).await?;

        // 返回结果
        Ok(json!({
            "agent_id": self.agent_id,
            "message": message,
            "response": response,
            "executed_at": chrono::Utc::now().to_rfc3339()
        }))
    }
}

impl AgentDagOrchestrator {
    /// 创建新的 Agent DAG 编排器
    pub fn new() -> Self {
        Self {
            scheduler: DagScheduler::new(num_cpus::get().max(4)),
            dag: Arc::new(RwLock::new(Dag::new())),
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建新的 Agent DAG 编排器（自定义并发度）
    pub fn with_concurrency(max_concurrency: usize) -> Self {
        Self {
            scheduler: DagScheduler::new(max_concurrency),
            dag: Arc::new(RwLock::new(Dag::new())),
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加 Agent 节点
    pub async fn add_agent(
        &self,
        agent_id: String,
        agent: Arc<dyn Agent>,
        dependencies: Vec<String>,
    ) -> Result<()> {
        // 存储 Agent
        self.agents
            .write()
            .await
            .insert(agent_id.clone(), Arc::clone(&agent));

        // 创建 Agent 执行器
        let executor = Arc::new(AgentExecutor {
            agent_id: agent_id.clone(),
            agent: Arc::clone(&agent),
        });

        // 创建 WorkflowStep
        let step = WorkflowStep {
            id: agent_id.clone(),
            description: Some(format!("Agent: {}", agent_id)),
            step_type: crate::workflow::StepType::Agent,
            execute: executor,
            input_schema: None,
            output_schema: None,
        };

        // 创建 DAG 节点
        let node = DagNode {
            id: agent_id.clone(),
            name: agent_id.clone(),
            dependencies,
            step,
        };

        // 添加到 DAG
        let mut dag = self.dag.write().await;
        dag.add_node(node)?;

        Ok(())
    }

    /// 添加根 Agent（无依赖）
    pub async fn add_root_agent(&self, agent_id: String, agent: Arc<dyn Agent>) -> Result<()> {
        self.add_agent(agent_id, agent, vec![]).await
    }

    /// 验证 DAG（检查环）
    pub async fn validate(&self) -> Result<()> {
        let dag = self.dag.read().await;
        dag.detect_cycle()?;
        Ok(())
    }

    /// 获取拓扑排序层级
    pub async fn get_execution_levels(&self) -> Result<Vec<Vec<String>>> {
        let dag = self.dag.read().await;
        dag.topological_sort()
    }

    /// 获取 Agent 数量
    pub async fn agent_count(&self) -> usize {
        let dag = self.dag.read().await;
        dag.node_count()
    }

    /// 执行 Agent DAG
    pub async fn execute(
        &self,
        input: Value,
        context: &RuntimeContext,
    ) -> Result<HashMap<String, Value>> {
        // 验证 DAG
        self.validate().await?;

        // 执行 DAG
        let dag = self.dag.read().await;
        let results = self.scheduler.execute(&dag, input, context).await?;

        Ok(results)
    }

    /// 执行并返回最终结果（最后一个节点的输出）
    pub async fn execute_and_get_final(
        &self,
        input: Value,
        context: &RuntimeContext,
    ) -> Result<Value> {
        let results = self.execute(input, context).await?;

        // 获取拓扑排序
        let levels = self.get_execution_levels().await?;

        // 获取最后一层的最后一个节点
        if let Some(last_level) = levels.last() {
            if let Some(last_node_id) = last_level.last() {
                if let Some(result) = results.get(last_node_id) {
                    return Ok(result.clone());
                }
            }
        }

        // 如果没有找到，返回所有结果
        Ok(json!(results))
    }

    /// 获取 Agent
    pub async fn get_agent(&self, agent_id: &str) -> Option<Arc<dyn Agent>> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// 列出所有 Agent
    pub async fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }
}

impl Default for AgentDagOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent DAG 编排器构建器
pub struct AgentDagOrchestratorBuilder {
    max_concurrency: Option<usize>,
}

impl AgentDagOrchestratorBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            max_concurrency: None,
        }
    }

    /// 设置最大并发度
    pub fn max_concurrency(mut self, max_concurrency: usize) -> Self {
        self.max_concurrency = Some(max_concurrency);
        self
    }

    /// 构建编排器
    pub fn build(self) -> AgentDagOrchestrator {
        if let Some(max_concurrency) = self.max_concurrency {
            AgentDagOrchestrator::with_concurrency(max_concurrency)
        } else {
            AgentDagOrchestrator::new()
        }
    }
}

impl Default for AgentDagOrchestratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent 链（简化的 DAG，用于线性 Agent 链）
pub struct AgentChain {
    orchestrator: AgentDagOrchestrator,
    agent_ids: Vec<String>,
}

impl AgentChain {
    /// 创建新的 Agent 链
    pub fn new() -> Self {
        Self {
            orchestrator: AgentDagOrchestrator::new(),
            agent_ids: Vec::new(),
        }
    }

    /// 添加 Agent 到链
    pub async fn add_agent(&mut self, agent_id: String, agent: Arc<dyn Agent>) -> Result<()> {
        let dependencies = if let Some(last_agent_id) = self.agent_ids.last() {
            vec![last_agent_id.clone()]
        } else {
            vec![]
        };

        self.orchestrator
            .add_agent(agent_id.clone(), agent, dependencies)
            .await?;
        self.agent_ids.push(agent_id);

        Ok(())
    }

    /// 执行 Agent 链
    pub async fn execute(&self, input: Value, context: &RuntimeContext) -> Result<Value> {
        self.orchestrator
            .execute_and_get_final(input, context)
            .await
    }

    /// 获取链长度
    pub fn len(&self) -> usize {
        self.agent_ids.len()
    }

    /// 检查链是否为空
    pub fn is_empty(&self) -> bool {
        self.agent_ids.is_empty()
    }
}

impl Default for AgentChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试将在单独的测试文件中实现
}
