//! Group Chat 协作模式
//!
//! 多个 Agent 在共享对话线程中讨论，通过轮次控制达成共识
//!
//! 参考：
//! - AutoGen Group Chat
//! - Azure AI Agent Orchestration Patterns (2025)
//! - Consensus-LLM (2025)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::communication::AgentCommunicationManager;
use super::Agent;
use crate::error::{Error, Result};
use crate::llm::{Message, Role};

/// Group Chat 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 发送者 Agent ID
    pub sender_id: String,
    /// 消息内容
    pub content: String,
    /// 消息时间戳
    pub timestamp: i64,
    /// 消息轮次
    pub round: usize,
}

/// Group Chat 对话线程
#[derive(Debug, Clone)]
pub struct ChatThread {
    /// 线程 ID
    pub id: String,
    /// 消息历史
    pub messages: Vec<ChatMessage>,
    /// 当前轮次
    pub current_round: usize,
    /// 最大轮次
    pub max_rounds: usize,
}

impl ChatThread {
    /// 创建新的对话线程
    pub fn new(max_rounds: usize) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            messages: Vec::new(),
            current_round: 0,
            max_rounds,
        }
    }

    /// 添加消息
    pub fn add_message(&mut self, sender_id: String, content: String) {
        let message = ChatMessage {
            sender_id,
            content,
            timestamp: chrono::Utc::now().timestamp(),
            round: self.current_round,
        };
        self.messages.push(message);
    }

    /// 进入下一轮
    pub fn next_round(&mut self) {
        self.current_round += 1;
    }

    /// 是否达到最大轮次
    pub fn is_finished(&self) -> bool {
        self.current_round >= self.max_rounds
    }

    /// 获取对话历史摘要
    pub fn get_summary(&self) -> String {
        let mut summary = String::new();
        for msg in &self.messages {
            summary.push_str(&format!(
                "[Round {}] {}: {}\n",
                msg.round, msg.sender_id, msg.content
            ));
        }
        summary
    }
}

/// Group Chat 执行器
pub struct GroupChatExecutor {
    /// 参与的 Agent
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
    /// 通信管理器
    communication: Arc<AgentCommunicationManager>,
    /// 最大轮次
    max_rounds: usize,
}

impl GroupChatExecutor {
    /// 创建新的 Group Chat 执行器
    pub fn new(
        agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
        communication: Arc<AgentCommunicationManager>,
        max_rounds: usize,
    ) -> Self {
        Self {
            agents,
            communication,
            max_rounds,
        }
    }

    /// 执行 Group Chat
    pub async fn execute(&self, initial_message: &str) -> Result<String> {
        println!(
            "  🟢 [GroupChatExecutor] Starting with {} max rounds",
            self.max_rounds
        );
        tracing::info!("Starting Group Chat with {} max rounds", self.max_rounds);

        let mut thread = ChatThread::new(self.max_rounds);
        let agents = self.agents.read().await;

        println!("  🟢 [GroupChatExecutor] Agents count: {}", agents.len());

        if agents.is_empty() {
            return Err(Error::InvalidInput(
                "Group Chat requires at least 1 agent".to_string(),
            ));
        }

        // 添加初始消息
        thread.add_message("system".to_string(), initial_message.to_string());
        println!("  🟢 [GroupChatExecutor] Initial message added");

        // 执行多轮对话
        while !thread.is_finished() {
            println!(
                "  🟡 [GroupChatExecutor] Round {}/{}",
                thread.current_round + 1,
                self.max_rounds
            );
            tracing::debug!(
                "Group Chat round {}/{}",
                thread.current_round + 1,
                self.max_rounds
            );

            // 每个 Agent 依次发言
            for (agent_id, agent) in agents.iter() {
                println!(
                    "    🔹 [GroupChatExecutor] Agent {} is generating response...",
                    agent_id
                );

                // 构建上下文消息
                let context = thread.get_summary();
                let prompt = format!(
                    "You are participating in a group discussion. Here is the conversation so far:\n\n{}\n\nPlease provide your response:",
                    context
                );

                // 调用 Agent 生成响应
                match agent.generate_simple(&prompt).await {
                    Ok(response) => {
                        thread.add_message(agent_id.clone(), response.clone());
                        println!(
                            "    ✅ [GroupChatExecutor] Agent {} responded (length: {})",
                            agent_id,
                            response.len()
                        );
                        tracing::debug!("Agent {} responded: {}", agent_id, response);
                    }
                    Err(e) => {
                        println!(
                            "    ❌ [GroupChatExecutor] Agent {} failed: {}",
                            agent_id, e
                        );
                        tracing::warn!("Agent {} failed to respond: {}", agent_id, e);
                        thread.add_message(agent_id.clone(), format!("[Error: {}]", e));
                    }
                }
            }

            thread.next_round();
            println!(
                "  🟡 [GroupChatExecutor] Round {} completed",
                thread.current_round
            );
        }

        println!("  ✅ [GroupChatExecutor] All rounds completed, generating summary");

        // 返回对话摘要
        Ok(thread.get_summary())
    }

    /// 执行带共识检测的 Group Chat
    pub async fn execute_with_consensus(
        &self,
        initial_message: &str,
        consensus_threshold: f32,
    ) -> Result<String> {
        tracing::info!(
            "Starting Group Chat with consensus detection (threshold: {})",
            consensus_threshold
        );

        let mut thread = ChatThread::new(self.max_rounds);
        let agents = self.agents.read().await;

        if agents.is_empty() {
            return Err(Error::InvalidInput(
                "Group Chat requires at least 1 agent".to_string(),
            ));
        }

        // 添加初始消息
        thread.add_message("system".to_string(), initial_message.to_string());

        // 执行多轮对话，直到达成共识或达到最大轮次
        while !thread.is_finished() {
            tracing::debug!(
                "Group Chat round {}/{}",
                thread.current_round + 1,
                self.max_rounds
            );

            let mut round_responses = Vec::new();

            // 每个 Agent 依次发言
            for (agent_id, agent) in agents.iter() {
                let context = thread.get_summary();
                let prompt = format!(
                    "You are participating in a group discussion. Here is the conversation so far:\n\n{}\n\nPlease provide your response:",
                    context
                );

                match agent.generate_simple(&prompt).await {
                    Ok(response) => {
                        thread.add_message(agent_id.clone(), response.clone());
                        round_responses.push(response);
                    }
                    Err(e) => {
                        tracing::warn!("Agent {} failed to respond: {}", agent_id, e);
                    }
                }
            }

            // 检测共识（简化实现：检查响应相似度）
            if self.check_consensus(&round_responses, consensus_threshold) {
                tracing::info!("Consensus reached at round {}", thread.current_round + 1);
                break;
            }

            thread.next_round();
        }

        Ok(thread.get_summary())
    }

    /// 检测是否达成共识（简化实现）
    fn check_consensus(&self, responses: &[String], threshold: f32) -> bool {
        if responses.len() < 2 {
            return true;
        }

        // 简化实现：检查响应长度的相似度
        let avg_len =
            responses.iter().map(|r| r.len()).sum::<usize>() as f32 / responses.len() as f32;
        let variance = responses
            .iter()
            .map(|r| {
                let diff = r.len() as f32 - avg_len;
                diff * diff
            })
            .sum::<f32>()
            / responses.len() as f32;

        let std_dev = variance.sqrt();
        let coefficient_of_variation = std_dev / avg_len;

        // 如果变异系数小于阈值，认为达成共识
        coefficient_of_variation < (1.0 - threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_thread() {
        let mut thread = ChatThread::new(3);
        assert_eq!(thread.current_round, 0);
        assert!(!thread.is_finished());

        thread.add_message("agent1".to_string(), "Hello".to_string());
        thread.next_round();
        assert_eq!(thread.current_round, 1);

        thread.next_round();
        thread.next_round();
        assert!(thread.is_finished());
    }

    #[tokio::test]
    async fn test_consensus_detection() {
        use crate::agent::communication::CommunicationConfig;

        let executor = GroupChatExecutor {
            agents: Arc::new(RwLock::new(HashMap::new())),
            communication: Arc::new(AgentCommunicationManager::new(
                CommunicationConfig::default(),
            )),
            max_rounds: 10,
        };

        // 相似的响应应该达成共识
        let similar_responses = vec![
            "This is a response".to_string(),
            "This is a response too".to_string(),
            "This is also a response".to_string(),
        ];
        assert!(executor.check_consensus(&similar_responses, 0.8));

        // 差异很大的响应不应该达成共识
        let different_responses = vec![
            "Short".to_string(),
            "This is a much longer response with more content".to_string(),
            "Medium length response".to_string(),
        ];
        assert!(!executor.check_consensus(&different_responses, 0.8));
    }
}
