//! # 任务路由器
//!
//! 智能路由任务到合适的 Agent。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 路由策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// 轮询
    RoundRobin,
    /// 最少连接
    LeastConnections,
    /// 随机
    Random,
    /// 基于能力
    CapabilityBased,
    /// 一致性哈希
    ConsistentHash,
}

/// 负载均衡策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// 平均分配
    Balanced,
    /// 加权分配
    Weighted,
    /// 优先级队列
    Priority,
}

/// 任务路由器
pub struct TaskRouter {
    /// 路由策略
    strategy: RoutingStrategy,
    /// 负载均衡策略
    load_balancing: LoadBalancingStrategy,
    /// 当前轮询索引
    round_robin_index: usize,
    /// Agent 权重
    agent_weights: HashMap<String, f64>,
    /// Agent 连接数
    agent_connections: HashMap<String, usize>,
}

impl TaskRouter {
    /// 创建新的路由器
    pub fn new() -> Self {
        Self {
            strategy: RoutingStrategy::RoundRobin,
            load_balancing: LoadBalancingStrategy::Balanced,
            round_robin_index: 0,
            agent_weights: HashMap::new(),
            agent_connections: HashMap::new(),
        }
    }

    /// 设置路由策略
    pub fn with_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// 设置负载均衡策略
    pub fn with_load_balancing(mut self, load_balancing: LoadBalancingStrategy) -> Self {
        self.load_balancing = load_balancing;
        self
    }

    /// 设置 Agent 权重
    pub fn set_agent_weight(&mut self, agent_id: impl Into<String>, weight: f64) {
        self.agent_weights.insert(agent_id.into(), weight);
    }

    /// 路由任务到指定 Agent
    pub fn route(&mut self, available_agents: &[String], task: &str) -> Option<String> {
        if available_agents.is_empty() {
            return None;
        }

        match self.strategy {
            RoutingStrategy::RoundRobin => self.route_round_robin(available_agents),
            RoutingStrategy::LeastConnections => self.route_least_connections(available_agents),
            RoutingStrategy::Random => self.route_random(available_agents),
            RoutingStrategy::CapabilityBased => self.route_by_capability(available_agents, task),
            RoutingStrategy::ConsistentHash => self.route_consistent_hash(available_agents, task),
        }
    }

    /// 轮询路由
    fn route_round_robin(&mut self, agents: &[String]) -> Option<String> {
        let agent = agents.get(self.round_robin_index % agents.len()).cloned();
        self.round_robin_index += 1;
        agent
    }

    /// 最少连接路由
    fn route_least_connections(&self, agents: &[String]) -> Option<String> {
        agents
            .iter()
            .min_by_key(|id| self.agent_connections.get(id.as_str()).unwrap_or(&0))
            .cloned()
    }

    /// 随机路由
    fn route_random(&self, agents: &[String]) -> Option<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);

        let index = hasher.finish() as usize % agents.len();
        agents.get(index).cloned()
    }

    /// 基于能力路由
    fn route_by_capability(&self, agents: &[String], task: &str) -> Option<String> {
        // 简化实现: 根据任务关键词选择
        if task.contains("math") || task.contains("calculate") {
            agents.iter().find(|a| a.contains("math")).cloned()
        } else if task.contains("write") || task.contains("text") {
            agents.iter().find(|a| a.contains("writer")).cloned()
        } else {
            agents.first().cloned()
        }
    }

    /// 一致性哈希路由
    fn route_consistent_hash(&self, agents: &[String], task: &str) -> Option<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        task.hash(&mut hasher);

        let index = hasher.finish() as usize % agents.len();
        agents.get(index).cloned()
    }

    /// 增加连接数
    pub fn increment_connections(&mut self, agent_id: &str) {
        *self.agent_connections.entry(agent_id.to_string()).or_insert(0) += 1;
    }

    /// 减少连接数
    pub fn decrement_connections(&mut self, agent_id: &str) {
        let count = self.agent_connections.entry(agent_id.to_string()).or_insert(0);
        if *count > 0 {
            *count -= 1;
        }
    }
}

impl Default for TaskRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_robin_routing() {
        let mut router = TaskRouter::new();
        let agents = vec!["agent1".to_string(), "agent2".to_string(), "agent3".to_string()];

        let r1 = router.route(&agents, "task1");
        let r2 = router.route(&agents, "task2");
        let r3 = router.route(&agents, "task3");
        let r4 = router.route(&agents, "task4");

        assert_eq!(r1, Some("agent1".to_string()));
        assert_eq!(r2, Some("agent2".to_string()));
        assert_eq!(r3, Some("agent3".to_string()));
        assert_eq!(r4, Some("agent1".to_string())); // 循环回 agent1
    }

    #[test]
    fn test_least_connections_routing() {
        let mut router = TaskRouter::new();
        let agents = vec!["agent1".to_string(), "agent2".to_string()];

        router.increment_connections("agent1");
        router.increment_connections("agent1");
        router.increment_connections("agent2");

        // agent2 只有 1 个连接,agent1 有 2 个
        let result = router.route(&agents, "task");
        assert_eq!(result, Some("agent2".to_string()));
    }

    #[test]
    fn test_capability_based_routing() {
        let router = TaskRouter::new();
        let agents = vec![
            "math_agent".to_string(),
            "writer_agent".to_string(),
            "general_agent".to_string(),
        ];

        let r1 = router.route(&agents, "Calculate 2+2");
        let r2 = router.route(&agents, "Write a poem");

        assert_eq!(r1, Some("math_agent".to_string()));
        assert_eq!(r2, Some("writer_agent".to_string()));
    }

    #[test]
    fn test_consistent_hash_routing() {
        let router = TaskRouter::new();
        let agents = vec!["agent1".to_string(), "agent2".to_string()];

        // 相同任务应该路由到同一个 Agent
        let r1 = router.route(&agents, "task1");
        let r2 = router.route(&agents, "task1");
        let r3 = router.route(&agents, "task2");

        assert_eq!(r1, r2); // 相同任务
        // r3 可能不同 (哈希不同)
    }

    #[test]
    fn test_empty_agents() {
        let mut router = TaskRouter::new();
        let result = router.route(&[], "task");
        assert!(result.is_none());
    }
}
