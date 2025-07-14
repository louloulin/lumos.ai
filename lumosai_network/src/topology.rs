//! 网络拓扑实现

use std::collections::HashMap;
use async_trait::async_trait;
use parking_lot::RwLock;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::dijkstra;
use petgraph::visit::{EdgeRef};
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::types::{AgentId, AgentLocation};

/// 网络拓扑类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TopologyType {
    /// 全连接网络
    FullyConnected,
    /// 星型网络
    Star,
    /// 环形网络
    Ring,
    /// 网格网络
    Mesh,
    /// 树形网络
    Tree,
    /// 分层网络
    Hierarchical,
    /// 自定义拓扑
    Custom,
}

/// 边属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeAttributes {
    /// 边权重（可用于路由距离计算）
    pub weight: f64,
    /// 带宽 (可选)
    pub bandwidth: Option<f64>,
    /// 延迟 (可选)
    pub latency: Option<f64>,
    /// 边标签
    pub label: Option<String>,
}

impl Default for EdgeAttributes {
    fn default() -> Self {
        Self {
            weight: 1.0,
            bandwidth: None,
            latency: None,
            label: None,
        }
    }
}

/// 节点属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAttributes {
    /// 节点ID
    pub id: AgentId,
    /// 节点位置
    pub location: Option<AgentLocation>,
    /// 节点标签
    pub label: Option<String>,
}

/// 网络拓扑接口
#[async_trait]
pub trait NetworkTopology: Send + Sync {
    /// 添加节点
    async fn add_node(&self, id: AgentId, attrs: Option<NodeAttributes>) -> Result<()>;
    
    /// 移除节点
    async fn remove_node(&self, id: &AgentId) -> Result<()>;
    
    /// 添加边
    async fn add_edge(&self, from: &AgentId, to: &AgentId, attrs: Option<EdgeAttributes>) -> Result<()>;
    
    /// 移除边
    async fn remove_edge(&self, from: &AgentId, to: &AgentId) -> Result<()>;
    
    /// 检查连接
    async fn is_connected(&self, from: &AgentId, to: &AgentId) -> Result<bool>;
    
    /// 获取所有邻居
    async fn get_neighbors(&self, id: &AgentId) -> Result<Vec<AgentId>>;
    
    /// 获取两点之间的最短路径
    async fn get_shortest_path(&self, from: &AgentId, to: &AgentId) -> Result<Vec<AgentId>>;
    
    /// 获取拓扑类型
    fn get_topology_type(&self) -> TopologyType;
    
    /// 获取所有节点
    async fn get_all_nodes(&self) -> Result<Vec<AgentId>>;
    
    /// 获取节点数量
    async fn node_count(&self) -> usize;
    
    /// 获取边数量
    async fn edge_count(&self) -> usize;
}

/// 使用petgraph实现的网络拓扑
pub struct GraphTopology {
    /// 拓扑图
    graph: RwLock<DiGraph<NodeAttributes, EdgeAttributes>>,
    /// 节点索引映射
    node_indices: RwLock<HashMap<AgentId, NodeIndex>>,
    /// 拓扑类型
    topology_type: TopologyType,
}

impl GraphTopology {
    /// 创建新的图拓扑
    pub fn new(topology_type: TopologyType) -> Self {
        Self {
            graph: RwLock::new(DiGraph::new()),
            node_indices: RwLock::new(HashMap::new()),
            topology_type,
        }
    }
    
    /// 创建全连接拓扑
    pub fn fully_connected() -> Self {
        Self::new(TopologyType::FullyConnected)
    }
    
    /// 创建星型拓扑
    pub fn star() -> Self {
        Self::new(TopologyType::Star)
    }
    
    /// 创建环形拓扑
    pub fn ring() -> Self {
        Self::new(TopologyType::Ring)
    }
    
    /// 获取节点索引
    fn get_node_index(&self, id: &AgentId) -> Option<NodeIndex> {
        let indices = self.node_indices.read();
        indices.get(id).copied()
    }
}

#[async_trait]
impl NetworkTopology for GraphTopology {
    async fn add_node(&self, id: AgentId, attrs: Option<NodeAttributes>) -> Result<()> {
        // 创建节点属性
        let node_attrs = match attrs {
            Some(a) => a,
            None => NodeAttributes {
                id: id.clone(),
                location: None,
                label: None,
            },
        };
        
        // 添加节点到图
        let mut graph = self.graph.write();
        let node_idx = graph.add_node(node_attrs.clone());
        
        // 更新索引映射
        let mut indices = self.node_indices.write();
        indices.insert(id, node_idx);
        
        // 为全连接拓扑添加边
        if self.topology_type == TopologyType::FullyConnected {
            // 获取所有现有节点
            let existing_nodes: Vec<(AgentId, NodeIndex)> = indices.iter()
                    .filter(|(agent_id, _)| agent_id.as_str() != node_attrs.id.as_str())
                    .map(|(agent_id, idx)| (agent_id.clone(), *idx))
                    .collect();
            
            // 添加双向边
            for (_other_id, other_idx) in existing_nodes {
                // 添加到新节点的边
                graph.add_edge(
                    node_idx,
                    other_idx,
                    EdgeAttributes::default()
                );
                
                // 添加从新节点的边
                graph.add_edge(
                    other_idx,
                    node_idx,
                    EdgeAttributes::default()
                );
                
                // 记录连接
                // log::debug!("建立连接: {} <-> {}", id, other_id);
            }
        }
        
        Ok(())
    }
    
    async fn remove_node(&self, id: &AgentId) -> Result<()> {
        let node_idx = self.get_node_index(id)
            .ok_or_else(|| Error::Topology(format!("节点不存在: {}", id)))?;
        
        // 移除节点及其连接的边
        let mut graph = self.graph.write();
        graph.remove_node(node_idx);
        
        // 更新索引映射
        let mut indices = self.node_indices.write();
        indices.remove(id);
        
        Ok(())
    }
    
    async fn add_edge(&self, from: &AgentId, to: &AgentId, attrs: Option<EdgeAttributes>) -> Result<()> {
        let from_idx = self.get_node_index(from)
            .ok_or_else(|| Error::Topology(format!("源节点不存在: {}", from)))?;
        
        let to_idx = self.get_node_index(to)
            .ok_or_else(|| Error::Topology(format!("目标节点不存在: {}", to)))?;
        
        // 添加边
        let mut graph = self.graph.write();
        graph.add_edge(from_idx, to_idx, attrs.unwrap_or_default());
        
        Ok(())
    }
    
    async fn remove_edge(&self, from: &AgentId, to: &AgentId) -> Result<()> {
        let from_idx = self.get_node_index(from)
            .ok_or_else(|| Error::Topology(format!("源节点不存在: {}", from)))?;
        
        let to_idx = self.get_node_index(to)
            .ok_or_else(|| Error::Topology(format!("目标节点不存在: {}", to)))?;
        
        // 移除边
        let mut graph = self.graph.write();
        let edge_idx = graph.find_edge(from_idx, to_idx)
            .ok_or_else(|| Error::Topology(format!("边不存在: {} -> {}", from, to)))?;
        
        graph.remove_edge(edge_idx);
        
        Ok(())
    }
    
    async fn is_connected(&self, from: &AgentId, to: &AgentId) -> Result<bool> {
        let from_idx = self.get_node_index(from)
            .ok_or_else(|| Error::Topology(format!("源节点不存在: {}", from)))?;
        
        let to_idx = self.get_node_index(to)
            .ok_or_else(|| Error::Topology(format!("目标节点不存在: {}", to)))?;
        
        // 检查边是否存在
        let graph = self.graph.read();
        let is_connected = graph.find_edge(from_idx, to_idx).is_some();
        
        Ok(is_connected)
    }
    
    async fn get_neighbors(&self, id: &AgentId) -> Result<Vec<AgentId>> {
        let node_idx = self.get_node_index(id)
            .ok_or_else(|| Error::Topology(format!("节点不存在: {}", id)))?;
        
        let graph = self.graph.read();
        let indices = self.node_indices.read();
        
        // 获取所有邻居
        let mut neighbors = Vec::new();
        
        for neighbor_idx in graph.neighbors(node_idx) {
            // 查找邻居ID
            for (agent_id, idx) in indices.iter() {
                if *idx == neighbor_idx {
                    neighbors.push(agent_id.clone());
                    break;
                }
            }
        }
        
        Ok(neighbors)
    }
    
    async fn get_shortest_path(&self, from: &AgentId, to: &AgentId) -> Result<Vec<AgentId>> {
        let from_idx = self.get_node_index(from)
            .ok_or_else(|| Error::Topology(format!("源节点不存在: {}", from)))?;
        
        let to_idx = self.get_node_index(to)
            .ok_or_else(|| Error::Topology(format!("目标节点不存在: {}", to)))?;
        
        let graph = self.graph.read();
        
        // 使用Dijkstra算法计算最短路径
        let path = dijkstra(&*graph, from_idx, Some(to_idx), |e| {
            let edge_idx = graph.find_edge(e.source(), e.target()).unwrap();
            graph.edge_weight(edge_idx).unwrap().weight
        });
        
        // 没有找到路径
        if !path.contains_key(&to_idx) {
            return Err(Error::Topology(format!("无法找到从 {} 到 {} 的路径", from, to)));
        }
        
        // 重建路径
        let mut current = to_idx;
        let mut path_indices = vec![current];
        
        // 从路径映射中重建完整路径
        while current != from_idx {
            // 获取前一个节点
            // 这里假设Dijkstra返回的路径是正确的，每个节点都有前驱节点
            let prev_edges = graph.edges_directed(current, petgraph::Direction::Incoming);
            
            let mut found_prev = false;
            for edge in prev_edges {
                let source = edge.source();
                let edge_weight = graph.edge_weight(graph.find_edge(source, current).unwrap()).unwrap();
                
                if path.contains_key(&source) && path[&source] + edge_weight.weight == path[&current] {
                    path_indices.push(source);
                    current = source;
                    found_prev = true;
                    break;
                }
            }
            
            if !found_prev {
                return Err(Error::Topology("路径重建失败".to_string()));
            }
        }
        
        // 反转路径（从source到target）
        path_indices.reverse();
        
        // 将节点索引转换为AgentId
        let indices = self.node_indices.read();
        let mut agent_path = Vec::new();
        
        for idx in path_indices {
            for (agent_id, node_idx) in indices.iter() {
                if *node_idx == idx {
                    agent_path.push(agent_id.clone());
                    break;
                }
            }
        }
        
        Ok(agent_path)
    }
    
    fn get_topology_type(&self) -> TopologyType {
        self.topology_type.clone()
    }
    
    async fn get_all_nodes(&self) -> Result<Vec<AgentId>> {
        let indices = self.node_indices.read();
        Ok(indices.keys().cloned().collect())
    }
    
    async fn node_count(&self) -> usize {
        let graph = self.graph.read();
        graph.node_count()
    }
    
    async fn edge_count(&self) -> usize {
        let graph = self.graph.read();
        graph.edge_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fully_connected_topology() {
        let topology = GraphTopology::fully_connected();
        
        // 添加三个节点
        let node1 = AgentId::from_str("node1");
        let node2 = AgentId::from_str("node2");
        let node3 = AgentId::from_str("node3");
        
        topology.add_node(node1.clone(), None).await.unwrap();
        topology.add_node(node2.clone(), None).await.unwrap();
        topology.add_node(node3.clone(), None).await.unwrap();
        
        // 验证节点数量
        assert_eq!(topology.node_count().await, 3);
        
        // 全连接网络应有 n*(n-1) 条边
        assert_eq!(topology.edge_count().await, 6);
        
        // 验证连接性
        assert!(topology.is_connected(&node1, &node2).await.unwrap());
        assert!(topology.is_connected(&node2, &node1).await.unwrap());
        assert!(topology.is_connected(&node1, &node3).await.unwrap());
        assert!(topology.is_connected(&node3, &node1).await.unwrap());
        assert!(topology.is_connected(&node2, &node3).await.unwrap());
        assert!(topology.is_connected(&node3, &node2).await.unwrap());
        
        // 获取邻居
        let neighbors = topology.get_neighbors(&node1).await.unwrap();
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&node2));
        assert!(neighbors.contains(&node3));
    }
    
    #[tokio::test]
    async fn test_custom_topology() {
        let topology = GraphTopology::new(TopologyType::Custom);
        
        // 添加节点
        let node1 = AgentId::from_str("node1");
        let node2 = AgentId::from_str("node2");
        let node3 = AgentId::from_str("node3");
        let node4 = AgentId::from_str("node4");
        
        topology.add_node(node1.clone(), None).await.unwrap();
        topology.add_node(node2.clone(), None).await.unwrap();
        topology.add_node(node3.clone(), None).await.unwrap();
        topology.add_node(node4.clone(), None).await.unwrap();
        
        // 创建自定义连接
        topology.add_edge(&node1, &node2, None).await.unwrap();
        topology.add_edge(&node2, &node3, None).await.unwrap();
        topology.add_edge(&node3, &node4, None).await.unwrap();
        topology.add_edge(&node4, &node1, None).await.unwrap();
        
        // 验证连接性
        assert!(topology.is_connected(&node1, &node2).await.unwrap());
        assert!(topology.is_connected(&node2, &node3).await.unwrap());
        assert!(topology.is_connected(&node3, &node4).await.unwrap());
        assert!(topology.is_connected(&node4, &node1).await.unwrap());
        
        // 节点1到节点3的最短路径应该是 node1 -> node2 -> node3
        let path = topology.get_shortest_path(&node1, &node3).await.unwrap();
        assert_eq!(path, vec![node1.clone(), node2.clone(), node3.clone()]);
        
        // 移除一条边
        topology.remove_edge(&node2, &node3).await.unwrap();
        
        // 现在应该无法从node1到node3
        assert!(topology.get_shortest_path(&node1, &node3).await.is_err());
        
        // 添加直接连接
        topology.add_edge(&node1, &node3, None).await.unwrap();
        
        // 现在最短路径是直接连接
        let path = topology.get_shortest_path(&node1, &node3).await.unwrap();
        assert_eq!(path, vec![node1.clone(), node3.clone()]);
    }
    
    #[tokio::test]
    async fn test_edge_weights() {
        let topology = GraphTopology::new(TopologyType::Custom);
        
        // 添加节点
        let node1 = AgentId::from_str("node1");
        let node2 = AgentId::from_str("node2");
        let node3 = AgentId::from_str("node3");
        
        topology.add_node(node1.clone(), None).await.unwrap();
        topology.add_node(node2.clone(), None).await.unwrap();
        topology.add_node(node3.clone(), None).await.unwrap();
        
        // 创建带权重的边
        let edge_attrs1 = EdgeAttributes {
            weight: 1.0,
            ..Default::default()
        };
        
        let edge_attrs2 = EdgeAttributes {
            weight: 10.0, // 更高的权重
            ..Default::default()
        };
        
        // node1 -> node2 (权重=1)
        topology.add_edge(&node1, &node2, Some(edge_attrs1.clone())).await.unwrap();
        
        // node1 -> node3 (权重=10)
        topology.add_edge(&node1, &node3, Some(edge_attrs2)).await.unwrap();
        
        // node2 -> node3 (权重=1)
        topology.add_edge(&node2, &node3, Some(edge_attrs1)).await.unwrap();
        
        // 从node1到node3的最短路径应该是 node1 -> node2 -> node3
        // 因为总权重为2，而直接路径权重为10
        let path = topology.get_shortest_path(&node1, &node3).await.unwrap();
        assert_eq!(path, vec![node1.clone(), node2.clone(), node3.clone()]);
    }
} 