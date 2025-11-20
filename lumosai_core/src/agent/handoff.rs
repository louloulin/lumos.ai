//! Handoff 协作模式
//!
//! Agent 根据内容动态决定是否移交给更合适的 Agent
//!
//! 参考：
//! - AutoGen Handoff Pattern
//! - Azure AI Agent Orchestration (2025)
//! - LangGraph State Graphs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::communication::AgentCommunicationManager;
use super::Agent;
use crate::error::{Error, Result};

/// Handoff 条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandoffCondition {
    /// 基于关键词
    Keyword(Vec<String>),
    /// 基于内容长度
    ContentLength { min: usize, max: usize },
    /// 基于 Agent 能力
    Capability(String),
    /// 自定义条件（由 LLM 判断）
    Custom(String),
}

/// Handoff 规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffRule {
    /// 源 Agent ID
    pub from_agent: String,
    /// 目标 Agent ID
    pub to_agent: String,
    /// 移交条件
    pub condition: HandoffCondition,
    /// 优先级（1-10，10 最高）
    pub priority: u8,
}

impl HandoffRule {
    /// 创建新的移交规则
    pub fn new(from_agent: String, to_agent: String, condition: HandoffCondition) -> Self {
        Self {
            from_agent,
            to_agent,
            condition,
            priority: 5,
        }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }

    /// 检查是否满足移交条件
    pub fn matches(&self, content: &str, agent_capabilities: &[String]) -> bool {
        match &self.condition {
            HandoffCondition::Keyword(keywords) => keywords.iter().any(|kw| content.contains(kw)),
            HandoffCondition::ContentLength { min, max } => {
                let len = content.len();
                len >= *min && len <= *max
            }
            HandoffCondition::Capability(cap) => agent_capabilities.contains(cap),
            HandoffCondition::Custom(_) => {
                // 自定义条件需要 LLM 判断，这里简化为 true
                true
            }
        }
    }
}

/// Handoff 历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffRecord {
    /// 源 Agent ID
    pub from_agent: String,
    /// 目标 Agent ID
    pub to_agent: String,
    /// 移交原因
    pub reason: String,
    /// 移交时间
    pub timestamp: i64,
    /// 移交内容
    pub content: String,
}

/// Handoff 执行器
pub struct HandoffExecutor {
    /// 参与的 Agent
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
    /// 通信管理器
    communication: Arc<AgentCommunicationManager>,
    /// 移交规则
    rules: Vec<HandoffRule>,
    /// 移交历史
    history: Arc<RwLock<Vec<HandoffRecord>>>,
    /// 最大移交次数（防止无限循环）
    max_handoffs: usize,
}

impl HandoffExecutor {
    /// 创建新的 Handoff 执行器
    pub fn new(
        agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
        communication: Arc<AgentCommunicationManager>,
    ) -> Self {
        Self {
            agents,
            communication,
            rules: Vec::new(),
            history: Arc::new(RwLock::new(Vec::new())),
            max_handoffs: 10,
        }
    }

    /// 添加移交规则
    pub fn add_rule(&mut self, rule: HandoffRule) {
        self.rules.push(rule);
        // 按优先级排序
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// 设置最大移交次数
    pub fn with_max_handoffs(mut self, max_handoffs: usize) -> Self {
        self.max_handoffs = max_handoffs;
        self
    }

    /// 执行 Handoff 流程
    pub async fn execute(&self, initial_message: &str) -> Result<String> {
        tracing::info!("Starting Handoff execution");

        let agents = self.agents.read().await;
        if agents.is_empty() {
            return Err(Error::InvalidInput(
                "Handoff requires at least 1 agent".to_string(),
            ));
        }

        // 从第一个 Agent 开始
        let mut current_agent_id = agents.keys().next().unwrap().clone();
        let mut current_message = initial_message.to_string();
        let mut handoff_count = 0;

        loop {
            tracing::debug!(
                "Current agent: {}, handoff count: {}",
                current_agent_id,
                handoff_count
            );

            // 检查是否达到最大移交次数
            if handoff_count >= self.max_handoffs {
                tracing::warn!("Reached max handoffs ({}), stopping", self.max_handoffs);
                break;
            }

            // 获取当前 Agent
            let current_agent = agents
                .get(&current_agent_id)
                .ok_or_else(|| Error::NotFound(format!("Agent {} not found", current_agent_id)))?;

            // 执行当前 Agent
            let response = current_agent.generate_simple(&current_message).await?;
            tracing::debug!("Agent {} response: {}", current_agent_id, response);

            // 检查是否需要移交
            let next_agent_id = self.find_handoff_target(&current_agent_id, &response).await;

            match next_agent_id {
                Some(next_id) if next_id != current_agent_id => {
                    // 记录移交
                    let record = HandoffRecord {
                        from_agent: current_agent_id.clone(),
                        to_agent: next_id.clone(),
                        reason: "Handoff condition matched".to_string(),
                        timestamp: chrono::Utc::now().timestamp(),
                        content: response.clone(),
                    };
                    self.history.write().await.push(record);

                    tracing::info!("Handing off from {} to {}", current_agent_id, next_id);

                    // 更新当前 Agent 和消息
                    current_agent_id = next_id;
                    current_message = response;
                    handoff_count += 1;
                }
                _ => {
                    // 没有移交，返回最终结果
                    tracing::info!("No handoff needed, returning final result");
                    return Ok(response);
                }
            }
        }

        // 达到最大移交次数，返回当前消息
        Ok(current_message)
    }

    /// 查找移交目标
    async fn find_handoff_target(&self, current_agent_id: &str, content: &str) -> Option<String> {
        // 遍历规则，找到第一个匹配的
        for rule in &self.rules {
            if rule.from_agent == current_agent_id {
                // 简化实现：假设所有 Agent 都有空的能力列表
                let capabilities = Vec::new();
                if rule.matches(content, &capabilities) {
                    return Some(rule.to_agent.clone());
                }
            }
        }

        None
    }

    /// 获取移交历史
    pub async fn get_history(&self) -> Vec<HandoffRecord> {
        self.history.read().await.clone()
    }

    /// 清空移交历史
    pub async fn clear_history(&self) {
        self.history.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handoff_rule_keyword() {
        let rule = HandoffRule::new(
            "agent1".to_string(),
            "agent2".to_string(),
            HandoffCondition::Keyword(vec!["urgent".to_string(), "help".to_string()]),
        );

        assert!(rule.matches("This is urgent!", &[]));
        assert!(rule.matches("I need help", &[]));
        assert!(!rule.matches("Normal message", &[]));
    }

    #[test]
    fn test_handoff_rule_content_length() {
        let rule = HandoffRule::new(
            "agent1".to_string(),
            "agent2".to_string(),
            HandoffCondition::ContentLength { min: 10, max: 50 },
        );

        assert!(rule.matches("This is a medium message", &[]));
        assert!(!rule.matches("Short", &[]));
        assert!(!rule.matches(&"x".repeat(100), &[]));
    }

    #[test]
    fn test_handoff_rule_capability() {
        let rule = HandoffRule::new(
            "agent1".to_string(),
            "agent2".to_string(),
            HandoffCondition::Capability("coding".to_string()),
        );

        assert!(rule.matches("Any content", &["coding".to_string()]));
        assert!(!rule.matches("Any content", &["writing".to_string()]));
    }

    #[test]
    fn test_handoff_rule_priority() {
        let rule = HandoffRule::new(
            "agent1".to_string(),
            "agent2".to_string(),
            HandoffCondition::Keyword(vec!["test".to_string()]),
        )
        .with_priority(8);

        assert_eq!(rule.priority, 8);
    }
}
