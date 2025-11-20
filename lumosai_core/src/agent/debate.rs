//! Debate 协作模式
//!
//! 正反方 Agent 辩论，Judge Agent 评判
//!
//! 参考：
//! - Multi-Agent Debate Strategies (2025)
//! - Debating with More Persuasive LLMs (2024)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::Agent;
use crate::error::{Error, Result};

/// 辩论立场
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DebatePosition {
    /// 正方
    Proposer,
    /// 反方
    Opposer,
    /// 裁判
    Judge,
}

/// 辩论轮次
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateRound {
    /// 轮次编号
    pub round: usize,
    /// 正方论点
    pub proposer_argument: String,
    /// 反方论点
    pub opposer_argument: String,
    /// 时间戳
    pub timestamp: i64,
}

/// 辩论结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateResult {
    /// 获胜方
    pub winner: DebatePosition,
    /// 裁判评语
    pub judgment: String,
    /// 辩论轮次历史
    pub rounds: Vec<DebateRound>,
    /// 总轮次数
    pub total_rounds: usize,
}

/// Debate 执行器
pub struct DebateExecutor {
    /// 正方 Agent
    proposer: Arc<dyn Agent>,
    /// 反方 Agent
    opposer: Arc<dyn Agent>,
    /// 裁判 Agent
    judge: Arc<dyn Agent>,
    /// 最大辩论轮次
    max_rounds: usize,
    /// 辩论历史
    rounds: Vec<DebateRound>,
}

impl DebateExecutor {
    /// 创建新的 Debate 执行器
    pub fn new(
        proposer: Arc<dyn Agent>,
        opposer: Arc<dyn Agent>,
        judge: Arc<dyn Agent>,
        max_rounds: usize,
    ) -> Self {
        Self {
            proposer,
            opposer,
            judge,
            max_rounds,
            rounds: Vec::new(),
        }
    }

    /// 执行辩论
    pub async fn execute(&mut self, topic: &str) -> Result<String> {
        tracing::info!("Starting debate on topic: {}", topic);

        let mut proposer_context = format!(
            "You are debating in favor of: {}\nPresent your opening argument.",
            topic
        );
        let mut opposer_context = format!(
            "You are debating against: {}\nPresent your opening argument.",
            topic
        );

        // 执行多轮辩论
        for round in 0..self.max_rounds {
            tracing::debug!("Debate round {}/{}", round + 1, self.max_rounds);

            // 正方发言
            let proposer_arg = self.proposer.generate_simple(&proposer_context).await?;
            tracing::debug!("Proposer: {}", proposer_arg);

            // 反方发言
            let opposer_arg = self.opposer.generate_simple(&opposer_context).await?;
            tracing::debug!("Opposer: {}", opposer_arg);

            // 记录本轮辩论
            let debate_round = DebateRound {
                round: round + 1,
                proposer_argument: proposer_arg.clone(),
                opposer_argument: opposer_arg.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            };
            self.rounds.push(debate_round);

            // 更新上下文（包含对方的论点）
            proposer_context = format!(
                "Previous round:\nYour argument: {}\nOpponent's argument: {}\n\nRespond to your opponent and strengthen your position.",
                proposer_arg, opposer_arg
            );
            opposer_context = format!(
                "Previous round:\nYour argument: {}\nOpponent's argument: {}\n\nRespond to your opponent and strengthen your position.",
                opposer_arg, proposer_arg
            );
        }

        // 裁判评判
        let judgment = self.get_judgment(topic).await?;
        tracing::info!("Debate concluded. Judgment: {}", judgment);

        Ok(judgment)
    }

    /// 获取裁判评判
    async fn get_judgment(&self, topic: &str) -> Result<String> {
        let mut debate_summary = format!("Topic: {}\n\nDebate rounds:\n", topic);

        for round in &self.rounds {
            debate_summary.push_str(&format!(
                "\nRound {}:\nProposer: {}\nOpposer: {}\n",
                round.round, round.proposer_argument, round.opposer_argument
            ));
        }

        let judge_prompt = format!(
            "{}\n\nAs a judge, please evaluate both sides and declare a winner.\n\
            Format your response as:\n\
            WINNER: [Proposer/Opposer]\n\
            REASONING: [Your reasoning]",
            debate_summary
        );

        self.judge.generate_simple(&judge_prompt).await
    }

    /// 获取辩论结果
    pub fn get_result(&self, judgment: &str) -> DebateResult {
        let winner = if judgment.contains("WINNER: Proposer") {
            DebatePosition::Proposer
        } else if judgment.contains("WINNER: Opposer") {
            DebatePosition::Opposer
        } else {
            DebatePosition::Judge // 平局
        };

        DebateResult {
            winner,
            judgment: judgment.to_string(),
            rounds: self.rounds.clone(),
            total_rounds: self.rounds.len(),
        }
    }

    /// 获取辩论历史
    pub fn get_rounds(&self) -> &[DebateRound] {
        &self.rounds
    }

    /// 清空辩论历史
    pub fn clear_rounds(&mut self) {
        self.rounds.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentBuilder;
    use crate::llm::test_helpers::create_test_zhipu_provider_arc;

    fn create_test_agent(name: &str) -> Arc<dyn Agent> {
        let llm = create_test_zhipu_provider_arc();
        let agent = AgentBuilder::new()
            .name(name)
            .instructions(&format!("You are {}", name))
            .model(llm)
            .build()
            .expect("Failed to build agent");
        Arc::new(agent)
    }

    #[test]
    fn test_debate_result_parsing() {
        let executor = DebateExecutor {
            proposer: create_test_agent("proposer"),
            opposer: create_test_agent("opposer"),
            judge: create_test_agent("judge"),
            max_rounds: 3,
            rounds: vec![DebateRound {
                round: 1,
                proposer_argument: "Arg 1".to_string(),
                opposer_argument: "Counter 1".to_string(),
                timestamp: 0,
            }],
        };

        let judgment = "WINNER: Proposer\nREASONING: Better arguments";
        let result = executor.get_result(judgment);

        assert_eq!(result.winner, DebatePosition::Proposer);
        assert_eq!(result.total_rounds, 1);
    }

    #[test]
    fn test_debate_rounds() {
        let mut executor = DebateExecutor {
            proposer: create_test_agent("proposer"),
            opposer: create_test_agent("opposer"),
            judge: create_test_agent("judge"),
            max_rounds: 3,
            rounds: Vec::new(),
        };

        assert_eq!(executor.get_rounds().len(), 0);

        executor.rounds.push(DebateRound {
            round: 1,
            proposer_argument: "Test".to_string(),
            opposer_argument: "Counter".to_string(),
            timestamp: 0,
        });

        assert_eq!(executor.get_rounds().len(), 1);

        executor.clear_rounds();
        assert_eq!(executor.get_rounds().len(), 0);
    }
}
