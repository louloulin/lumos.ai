//! DAG (Directed Acyclic Graph) 并行调度器
//!
//! 提供基于依赖关系的智能并行调度，支持：
//! - 自动依赖分析
//! - 拓扑排序
//! - 最大并行度优化
//! - 工作窃取算法

use super::enhanced::{StepFlowEntry, WorkflowStep};
use crate::agent::types::RuntimeContext;
use crate::{Error, Result};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinSet;

/// DAG 节点
#[derive(Debug, Clone)]
pub struct DagNode {
    /// 节点 ID
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 依赖的节点 ID 列表
    pub dependencies: Vec<String>,
    /// 步骤引用
    pub step: WorkflowStep,
}

/// DAG 图
#[derive(Debug, Clone)]
pub struct Dag {
    /// 所有节点
    nodes: HashMap<String, DagNode>,
    /// 邻接表（节点 -> 依赖它的节点列表）
    adjacency: HashMap<String, Vec<String>>,
    /// 入度表（节点 -> 入度数）
    in_degree: HashMap<String, usize>,
}

impl Dag {
    /// 创建新的 DAG
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            adjacency: HashMap::new(),
            in_degree: HashMap::new(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, node: DagNode) -> Result<()> {
        let node_id = node.id.clone();

        // 检查是否已存在
        if self.nodes.contains_key(&node_id) {
            return Err(Error::Workflow(format!("Node {} already exists", node_id)));
        }

        // 初始化入度
        self.in_degree.insert(node_id.clone(), node.dependencies.len());

        // 更新邻接表
        for dep_id in &node.dependencies {
            self.adjacency
                .entry(dep_id.clone())
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        }

        // 添加节点
        self.nodes.insert(node_id, node);

        Ok(())
    }

    /// 检测环
    pub fn detect_cycle(&self) -> Result<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                if self.detect_cycle_util(node_id, &mut visited, &mut rec_stack)? {
                    return Err(Error::Workflow(format!(
                        "Cycle detected in DAG involving node: {}",
                        node_id
                    )));
                }
            }
        }

        Ok(())
    }

    /// 环检测辅助函数
    fn detect_cycle_util(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<bool> {
        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());

        // 检查所有依赖此节点的节点
        if let Some(dependents) = self.adjacency.get(node_id) {
            for dependent_id in dependents {
                if !visited.contains(dependent_id) {
                    if self.detect_cycle_util(dependent_id, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(dependent_id) {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(node_id);
        Ok(false)
    }

    /// 拓扑排序（Kahn 算法）
    pub fn topological_sort(&self) -> Result<Vec<Vec<String>>> {
        // 检测环
        self.detect_cycle()?;

        let mut in_degree = self.in_degree.clone();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut levels: Vec<Vec<String>> = Vec::new();

        // 找到所有入度为 0 的节点（起始节点）
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        // 按层级处理
        while !queue.is_empty() {
            let level_size = queue.len();
            let mut current_level = Vec::new();

            for _ in 0..level_size {
                if let Some(node_id) = queue.pop_front() {
                    current_level.push(node_id.clone());

                    // 更新依赖此节点的节点的入度
                    if let Some(dependents) = self.adjacency.get(&node_id) {
                        for dependent_id in dependents {
                            if let Some(degree) = in_degree.get_mut(dependent_id) {
                                *degree -= 1;
                                if *degree == 0 {
                                    queue.push_back(dependent_id.clone());
                                }
                            }
                        }
                    }
                }
            }

            if !current_level.is_empty() {
                levels.push(current_level);
            }
        }

        // 检查是否所有节点都被处理
        if levels.iter().map(|level| level.len()).sum::<usize>() != self.nodes.len() {
            return Err(Error::Workflow(
                "Failed to process all nodes - possible cycle".to_string(),
            ));
        }

        Ok(levels)
    }

    /// 获取节点
    pub fn get_node(&self, node_id: &str) -> Option<&DagNode> {
        self.nodes.get(node_id)
    }

    /// 获取所有节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for Dag {
    fn default() -> Self {
        Self::new()
    }
}

/// DAG 调度器
#[derive(Clone)]
pub struct DagScheduler {
    /// 最大并发度
    max_concurrency: usize,
    /// 是否启用工作窃取
    enable_work_stealing: bool,
}

impl DagScheduler {
    /// 创建新的 DAG 调度器
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            max_concurrency,
            enable_work_stealing: true,
        }
    }

    /// 启用/禁用工作窃取
    pub fn with_work_stealing(mut self, enable: bool) -> Self {
        self.enable_work_stealing = enable;
        self
    }

    /// 执行 DAG
    pub async fn execute(
        &self,
        dag: &Dag,
        initial_input: Value,
        context: &RuntimeContext,
    ) -> Result<HashMap<String, Value>> {
        // 拓扑排序获取执行层级
        let levels = dag.topological_sort()?;

        tracing::info!(
            "DAG execution started: {} nodes in {} levels",
            dag.node_count(),
            levels.len()
        );

        // 存储每个节点的执行结果
        let results: Arc<RwLock<HashMap<String, Value>>> =
            Arc::new(RwLock::new(HashMap::new()));

        // 按层级执行
        for (level_idx, level) in levels.iter().enumerate() {
            tracing::debug!(
                "Executing level {}/{}: {} nodes",
                level_idx + 1,
                levels.len(),
                level.len()
            );

            // 并行执行当前层级的所有节点
            self.execute_level(dag, level, &results, initial_input.clone(), context)
                .await?;
        }

        // 返回所有结果
        let final_results = results.read().await.clone();
        Ok(final_results)
    }

    /// 执行一个层级的节点
    async fn execute_level(
        &self,
        dag: &Dag,
        level: &[String],
        results: &Arc<RwLock<HashMap<String, Value>>>,
        initial_input: Value,
        context: &RuntimeContext,
    ) -> Result<()> {
        // 控制并发度
        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));
        let mut join_set: JoinSet<Result<(String, Value)>> = JoinSet::new();

        for node_id in level {
            let node = dag
                .get_node(node_id)
                .ok_or_else(|| Error::Workflow(format!("Node not found: {}", node_id)))?
                .clone();

            let results_clone = Arc::clone(results);
            let semaphore_clone = Arc::clone(&semaphore);
            let context_clone = context.clone();
            let initial_input_clone = initial_input.clone();

            join_set.spawn(async move {
                // 获取信号量许可
                let _permit = semaphore_clone.acquire().await.map_err(|e| {
                    Error::Workflow(format!("Failed to acquire semaphore: {}", e))
                })?;

                // 准备输入：合并初始输入和依赖节点的输出
                let input = Self::prepare_input(
                    &node,
                    &results_clone,
                    initial_input_clone,
                ).await?;

                // 执行节点
                tracing::debug!("Executing node: {} ({})", node.id, node.name);
                let start = std::time::Instant::now();

                let output = node.step.execute.execute(input, &context_clone).await?;

                let elapsed = start.elapsed();
                tracing::debug!(
                    "Node {} completed in {:?}",
                    node.id,
                    elapsed
                );

                Ok((node.id.clone(), output))
            });
        }

        // 收集所有结果
        while let Some(result) = join_set.join_next().await {
            let (node_id, output) = result.map_err(|e| {
                Error::Workflow(format!("Task join error: {}", e))
            })??;

            results.write().await.insert(node_id, output);
        }

        Ok(())
    }

    /// 准备节点输入
    async fn prepare_input(
        node: &DagNode,
        results: &Arc<RwLock<HashMap<String, Value>>>,
        initial_input: Value,
    ) -> Result<Value> {
        let results_read = results.read().await;

        // 如果没有依赖，使用初始输入
        if node.dependencies.is_empty() {
            return Ok(initial_input);
        }

        // 合并所有依赖节点的输出
        let mut merged_input = serde_json::Map::new();

        // 添加初始输入
        if let Value::Object(obj) = initial_input {
            for (k, v) in obj {
                merged_input.insert(k, v);
            }
        }

        // 添加依赖节点的输出
        for dep_id in &node.dependencies {
            if let Some(dep_output) = results_read.get(dep_id) {
                merged_input.insert(dep_id.clone(), dep_output.clone());
            }
        }

        Ok(Value::Object(merged_input))
    }
}

