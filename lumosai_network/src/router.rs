//! 消息路由实现

use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use tokio::sync::{RwLock, mpsc};
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::types::AgentId;
use crate::message::Message;
use crate::topology::NetworkTopology;

/// 定义路由策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// 直接路由，发送到指定的Agent
    Direct,
    /// 广播，发送给所有Agent
    Broadcast,
    /// 智能路由，基于能力匹配
    CapabilityBased,
    /// 基于角色路由
    RoleBased,
    /// 随机路由，对于负载均衡有用
    Random,
    /// 最短路径路由
    ShortestPath,
    /// 基于内容路由，根据消息内容选择路由
    ContentBased,
}

/// 定义路由规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// 规则名称
    pub name: String,
    /// 规则优先级
    pub priority: u32,
    /// 源Agent模式
    pub source_pattern: String,
    /// 目标Agent模式
    pub target_pattern: String,
    /// 规则条件
    pub condition: Option<String>,
    /// 路由策略
    pub strategy: RoutingStrategy,
}

/// 路由表项
#[derive(Debug, Clone)]
struct RoutingTableEntry {
    /// 目标Agent
    pub target: AgentId,
    /// 发送通道
    pub sender: mpsc::Sender<Message>,
}

/// 消息路由器特质
#[async_trait]
pub trait MessageRouter: Send + Sync {
    /// 注册Agent
    async fn register(&self, agent_id: AgentId, sender: mpsc::Sender<Message>) -> Result<()>;
    
    /// 注销Agent
    async fn unregister(&self, agent_id: &AgentId) -> Result<()>;
    
    /// 路由消息
    async fn route(&self, message: Message) -> Result<()>;
    
    /// 添加路由规则
    async fn add_rule(&self, rule: RoutingRule) -> Result<()>;
    
    /// 删除路由规则
    async fn remove_rule(&self, rule_name: &str) -> Result<()>;
    
    /// 获取所有规则
    async fn get_rules(&self) -> Result<Vec<RoutingRule>>;
    
    /// 设置网络拓扑
    async fn set_topology(&self, topology: Arc<dyn NetworkTopology>);
    
    /// 获取已注册的Agent数量
    async fn agent_count(&self) -> usize;
}

/// 默认消息路由器实现
pub struct DefaultMessageRouter {
    /// 路由表，将Agent ID映射到其消息通道
    routing_table: RwLock<HashMap<AgentId, mpsc::Sender<Message>>>,
    /// 路由规则
    routing_rules: RwLock<Vec<RoutingRule>>,
    /// 网络拓扑
    topology: RwLock<Option<Arc<dyn NetworkTopology>>>,
}

impl DefaultMessageRouter {
    /// 创建新的默认路由器
    pub fn new() -> Self {
        Self {
            routing_table: RwLock::new(HashMap::new()),
            routing_rules: RwLock::new(Vec::new()),
            topology: RwLock::new(None),
        }
    }
    
    /// 根据规则选择路由策略
    async fn select_strategy(&self, message: &Message) -> RoutingStrategy {
        let rules = self.routing_rules.read().await;
        
        // 尝试找到匹配的规则
        for rule in rules.iter() {
            // 基础匹配逻辑，实际实现可能更复杂
            if rule.source_pattern == "*" || rule.source_pattern == message.sender.as_str() {
                return rule.strategy.clone();
            }
        }
        
        // 默认为直接路由
        RoutingStrategy::Direct
    }
}

#[async_trait]
impl MessageRouter for DefaultMessageRouter {
    async fn register(&self, agent_id: AgentId, sender: mpsc::Sender<Message>) -> Result<()> {
        let mut table = self.routing_table.write().await;
        table.insert(agent_id, sender);
        Ok(())
    }
    
    async fn unregister(&self, agent_id: &AgentId) -> Result<()> {
        let mut table = self.routing_table.write().await;
        table.remove(agent_id);
        Ok(())
    }
    
    async fn route(&self, mut message: Message) -> Result<()> {
        // 标记消息为已发送
        message.mark_as_sent();
        
        // 根据规则选择路由策略
        let strategy = self.select_strategy(&message).await;
        
        // 获取路由表
        let table = self.routing_table.read().await;
        
        match strategy {
            RoutingStrategy::Direct => {
                for receiver in &message.receivers {
                    if let Some(sender) = table.get(receiver) {
                        // 克隆一个消息实例
                        let mut msg_clone = message.clone();
                        // 设置单个接收者
                        msg_clone.receivers = vec![receiver.clone()];
                        
                        if let Err(e) = sender.send(msg_clone).await {
                            return Err(Error::Routing(format!("无法发送消息: {}", e)));
                        }
                    } else {
                        return Err(Error::AgentNotFound(receiver.value()));
                    }
                }
            },
            RoutingStrategy::Broadcast => {
                for (agent_id, sender) in table.iter() {
                    if agent_id != &message.sender { // 不发送给自己
                        // 克隆一个消息实例
                        let mut msg_clone = message.clone();
                        // 设置单个接收者
                        msg_clone.receivers = vec![agent_id.clone()];
                        
                        if let Err(e) = sender.send(msg_clone).await {
                            return Err(Error::Routing(format!("无法发送消息: {}", e)));
                        }
                    }
                }
            },
            // 其他策略实现可以在这里添加
            _ => {
                // 如果没有实现特定策略，使用直接路由
                return self.route(message).await;
            }
        }
        
        Ok(())
    }
    
    async fn add_rule(&self, rule: RoutingRule) -> Result<()> {
        let mut rules = self.routing_rules.write().await;
        rules.push(rule);
        
        // 按优先级排序
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(())
    }
    
    async fn remove_rule(&self, rule_name: &str) -> Result<()> {
        let mut rules = self.routing_rules.write().await;
        let initial_len = rules.len();
        
        rules.retain(|rule| rule.name != rule_name);
        
        if rules.len() == initial_len {
            return Err(Error::Routing(format!("规则 '{}' 不存在", rule_name)));
        }
        
        Ok(())
    }
    
    async fn get_rules(&self) -> Result<Vec<RoutingRule>> {
        let rules = self.routing_rules.read().await;
        Ok(rules.clone())
    }
    
    async fn set_topology(&self, topology: Arc<dyn NetworkTopology>) {
        let mut topo = self.topology.write().await;
        *topo = Some(topology);
    }
    
    async fn agent_count(&self) -> usize {
        let table = self.routing_table.read().await;
        table.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use crate::message::MessageType;
    use crate::types::AgentId;
    
    #[tokio::test]
    async fn test_router_registration() {
        let router = DefaultMessageRouter::new();
        
        let agent1 = AgentId::from_str("agent1");
        let (tx1, _rx1) = mpsc::channel(10);
        
        let agent2 = AgentId::from_str("agent2");
        let (tx2, _rx2) = mpsc::channel(10);
        
        // 注册两个Agent
        router.register(agent1.clone(), tx1).await.unwrap();
        router.register(agent2.clone(), tx2).await.unwrap();
        
        // 验证注册数量
        assert_eq!(router.agent_count().await, 2);
        
        // 注销一个Agent
        router.unregister(&agent1).await.unwrap();
        
        // 验证注销后的数量
        assert_eq!(router.agent_count().await, 1);
    }
    
    #[tokio::test]
    async fn test_routing_rules() {
        let router = DefaultMessageRouter::new();
        
        // 添加规则
        let rule1 = RoutingRule {
            name: "rule1".to_string(),
            priority: 10,
            source_pattern: "*".to_string(),
            target_pattern: "*".to_string(),
            condition: None,
            strategy: RoutingStrategy::Broadcast,
        };
        
        let rule2 = RoutingRule {
            name: "rule2".to_string(),
            priority: 20, // 更高优先级
            source_pattern: "agent1".to_string(),
            target_pattern: "agent2".to_string(),
            condition: None,
            strategy: RoutingStrategy::Direct,
        };
        
        router.add_rule(rule1.clone()).await.unwrap();
        router.add_rule(rule2.clone()).await.unwrap();
        
        // 获取规则列表
        let rules = router.get_rules().await.unwrap();
        
        // 验证规则数量
        assert_eq!(rules.len(), 2);
        
        // 验证规则排序（按优先级降序）
        assert_eq!(rules[0].name, "rule2");
        assert_eq!(rules[1].name, "rule1");
        
        // 删除规则
        router.remove_rule("rule1").await.unwrap();
        
        // 验证删除后的规则数量
        let rules = router.get_rules().await.unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "rule2");
    }
    
    #[tokio::test]
    async fn test_message_routing() {
        let router = DefaultMessageRouter::new();
        
        let agent1 = AgentId::from_str("agent1");
        let (tx1, mut rx1) = mpsc::channel(10);
        
        let agent2 = AgentId::from_str("agent2");
        let (tx2, mut rx2) = mpsc::channel(10);
        
        // 注册Agent
        router.register(agent1.clone(), tx1).await.unwrap();
        router.register(agent2.clone(), tx2).await.unwrap();
        
        // 创建消息
        let message = Message::new(
            agent1.clone(),
            vec![agent2.clone()],
            MessageType::Text,
            "Hello from Agent 1!"
        );
        
        // 路由消息
        router.route(message).await.unwrap();
        
        // agent2应该收到消息
        let received = rx2.recv().await.unwrap();
        assert_eq!(received.sender, agent1);
        assert_eq!(received.content, serde_json::json!("Hello from Agent 1!"));
        
        // agent1不应该收到消息
        assert!(rx1.try_recv().is_err());
    }
} 