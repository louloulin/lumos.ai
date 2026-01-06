//! # 编排模式
//!
//! 定义多种 Multi-Agent 编排模式。

use serde::{Deserialize, Serialize};

/// 编排模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrchestrationPattern {
    /// 层级模式 (Manager-Agent)
    ///
    /// 一个 Manager Agent 协调多个 Worker Agents
    Hierarchical,

    /// 平等模式 (Flat Collaboration)
    ///
    /// 所有 Agent 平等协作,共同决策
    Flat,

    /// 流水线模式 (Pipeline)
    ///
    /// Agent 按顺序执行,每个 Agent 的输出是下一个的输入
    Pipeline,

    /// 图模式 (Graph)
    ///
    /// 复杂的依赖关系和分支执行
    Graph,
}

impl OrchestrationPattern {
    /// 获取模式描述
    pub fn description(&self) -> &str {
        match self {
            OrchestrationPattern::Hierarchical => {
                "层级模式: Manager 协调多个 Workers"
            }
            OrchestrationPattern::Flat => {
                "平等模式: 所有 Agent 平等协作"
            }
            OrchestrationPattern::Pipeline => {
                "流水线模式: 顺序执行,传递结果"
            }
            OrchestrationPattern::Graph => {
                "图模式: 复杂依赖和分支"
            }
        }
    }
}

/// 模式执行器 trait
pub trait PatternExecutor: Send + Sync {
    /// 执行编排模式
    fn execute(&self, input: &str) -> Result<String, String>;

    /// 获取模式名称
    fn pattern_name(&self) -> &str;
}

/// 层级模式实现
pub struct HierarchicalPattern {
    manager_id: String,
    worker_ids: Vec<String>,
}

impl HierarchicalPattern {
    pub fn new(manager_id: impl Into<String>, worker_ids: Vec<String>) -> Self {
        Self {
            manager_id: manager_id.into(),
            worker_ids,
        }
    }
}

impl PatternExecutor for HierarchicalPattern {
    fn execute(&self, input: &str) -> Result<String, String> {
        // 简化实现: 实际需要集成真实的 Agent 调用
        Ok(format!(
            "Manager {} coordinating workers {:?} for task: {}",
            self.manager_id, self.worker_ids, input
        ))
    }

    fn pattern_name(&self) -> &str {
        "Hierarchical"
    }
}

/// 平等模式实现
pub struct FlatPattern {
    agent_ids: Vec<String>,
}

impl FlatPattern {
    pub fn new(agent_ids: Vec<String>) -> Self {
        Self { agent_ids }
    }
}

impl PatternExecutor for FlatPattern {
    fn execute(&self, input: &str) -> Result<String, String> {
        Ok(format!(
            "Flat collaboration among agents {:?} for task: {}",
            self.agent_ids, input
        ))
    }

    fn pattern_name(&self) -> &str {
        "Flat"
    }
}

/// 流水线模式实现
pub struct PipelinePattern {
    stages: Vec<String>,
}

impl PipelinePattern {
    pub fn new(stages: Vec<String>) -> Self {
        Self { stages }
    }
}

impl PatternExecutor for PipelinePattern {
    fn execute(&self, input: &str) -> Result<String, String> {
        Ok(format!(
            "Pipeline through stages {:?} with input: {}",
            self.stages, input
        ))
    }

    fn pattern_name(&self) -> &str {
        "Pipeline"
    }
}

/// 图模式实现
pub struct GraphPattern {
    nodes: Vec<String>,
    edges: Vec<(String, String)>,
}

impl GraphPattern {
    pub fn new(nodes: Vec<String>, edges: Vec<(String, String)>) -> Self {
        Self { nodes, edges }
    }
}

impl PatternExecutor for GraphPattern {
    fn execute(&self, input: &str) -> Result<String, String> {
        Ok(format!(
            "Graph execution with nodes {:?}, edges {:?}, input: {}",
            self.nodes, self.edges, input
        ))
    }

    fn pattern_name(&self) -> &str {
        "Graph"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_descriptions() {
        assert!(OrchestrationPattern::Hierarchical.description().contains("Manager"));
        assert!(OrchestrationPattern::Flat.description().contains("平等"));
        assert!(OrchestrationPattern::Pipeline.description().contains("流水线"));
        assert!(OrchestrationPattern::Graph.description().contains("图"));
    }

    #[test]
    fn test_hierarchical_pattern() {
        let pattern = HierarchicalPattern::new("manager1", vec!["worker1".into(), "worker2".into()]);
        let result = pattern.execute("test task").unwrap();
        assert!(result.contains("manager1"));
        assert!(result.contains("test task"));
    }

    #[test]
    fn test_flat_pattern() {
        let pattern = FlatPattern::new(vec!["agent1".into(), "agent2".into()]);
        let result = pattern.execute("test task").unwrap();
        assert!(result.contains("agent1"));
        assert!(result.contains("agent2"));
    }

    #[test]
    fn test_pipeline_pattern() {
        let pattern = PipelinePattern::new(vec!["stage1".into(), "stage2".into()]);
        let result = pattern.execute("test input").unwrap();
        assert!(result.contains("stage1"));
        assert!(result.contains("test input"));
    }

    #[test]
    fn test_graph_pattern() {
        let pattern = GraphPattern::new(
            vec!["node1".into(), "node2".into()],
            vec![("node1".into(), "node2".into())],
        );
        let result = pattern.execute("test task").unwrap();
        assert!(result.contains("node1"));
        assert!(result.contains("node2"));
    }
}
