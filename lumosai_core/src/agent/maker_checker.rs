//! MakerChecker 协作模式
//!
//! Maker 创建/改进内容，Checker 审核并提供反馈
//!
//! 参考：
//! - Maker-Checker Pattern (金融行业最佳实践)
//! - Two-Phase Commit Pattern

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::Agent;
use crate::error::{Error, Result};

/// 审核状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CheckStatus {
    /// 通过
    Approved,
    /// 拒绝
    Rejected,
    /// 需要修改
    NeedsRevision,
}

/// 审核结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// 审核状态
    pub status: CheckStatus,
    /// 反馈意见
    pub feedback: String,
    /// 审核时间
    pub timestamp: i64,
}

/// MakerChecker 迭代记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakerCheckerIteration {
    /// 迭代次数
    pub iteration: usize,
    /// Maker 创建的内容
    pub created_content: String,
    /// Checker 的审核结果
    pub check_result: CheckResult,
    /// 时间戳
    pub timestamp: i64,
}

/// MakerChecker 执行器
pub struct MakerCheckerExecutor {
    /// Maker Agent
    maker: Arc<dyn Agent>,
    /// Checker Agent
    checker: Arc<dyn Agent>,
    /// 最大迭代次数
    max_iterations: usize,
    /// 迭代历史
    history: Vec<MakerCheckerIteration>,
}

impl MakerCheckerExecutor {
    /// 创建新的 MakerChecker 执行器
    pub fn new(maker: Arc<dyn Agent>, checker: Arc<dyn Agent>, max_iterations: usize) -> Self {
        Self {
            maker,
            checker,
            max_iterations,
            history: Vec::new(),
        }
    }

    /// 执行 MakerChecker 流程
    pub async fn execute(&mut self, initial_request: &str) -> Result<String> {
        tracing::info!("Starting MakerChecker execution");

        let mut current_request = initial_request.to_string();
        let mut approved_content = String::new();

        for iteration in 0..self.max_iterations {
            tracing::debug!(
                "MakerChecker iteration {}/{}",
                iteration + 1,
                self.max_iterations
            );

            // 1. Maker 创建内容
            let created_content = self.maker.generate_simple(&current_request).await?;
            tracing::debug!("Maker created: {}", created_content);

            // 2. Checker 审核内容
            let check_prompt = format!(
                "Please review the following content and provide:\n\
                1. A status: APPROVED, REJECTED, or NEEDS_REVISION\n\
                2. Feedback for improvement (if not approved)\n\n\
                Content:\n{}\n\n\
                Format your response as:\n\
                STATUS: <status>\n\
                FEEDBACK: <your feedback>",
                created_content
            );

            let check_response = self.checker.generate_simple(&check_prompt).await?;
            tracing::debug!("Checker response: {}", check_response);

            // 3. 解析审核结果
            let check_result = self.parse_check_result(&check_response);

            // 4. 记录迭代
            let iteration_record = MakerCheckerIteration {
                iteration: iteration + 1,
                created_content: created_content.clone(),
                check_result: check_result.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            };
            self.history.push(iteration_record);

            // 5. 根据审核结果决定下一步
            match check_result.status {
                CheckStatus::Approved => {
                    tracing::info!("Content approved at iteration {}", iteration + 1);
                    approved_content = created_content;
                    break;
                }
                CheckStatus::Rejected => {
                    tracing::warn!("Content rejected at iteration {}", iteration + 1);
                    // 可以选择继续或停止
                    current_request = format!(
                        "Original request: {}\n\nPrevious attempt was rejected.\nFeedback: {}\n\nPlease create new content addressing the feedback.",
                        initial_request, check_result.feedback
                    );
                }
                CheckStatus::NeedsRevision => {
                    tracing::debug!("Content needs revision at iteration {}", iteration + 1);
                    current_request = format!(
                        "Original request: {}\n\nPrevious attempt:\n{}\n\nFeedback: {}\n\nPlease revise the content based on the feedback.",
                        initial_request, created_content, check_result.feedback
                    );
                }
            }
        }

        // 如果达到最大迭代次数仍未通过，返回最后一次创建的内容
        if approved_content.is_empty() {
            tracing::warn!("Reached max iterations without approval");
            if let Some(last_iteration) = self.history.last() {
                approved_content = last_iteration.created_content.clone();
            }
        }

        Ok(approved_content)
    }

    /// 解析审核结果
    fn parse_check_result(&self, check_response: &str) -> CheckResult {
        let mut status = CheckStatus::NeedsRevision; // 默认状态
        let mut feedback = check_response.to_string();

        // 尝试解析 STATUS: 和 FEEDBACK: 格式
        for line in check_response.lines() {
            if line.starts_with("STATUS:") {
                if let Some(status_str) = line.strip_prefix("STATUS:") {
                    let status_str = status_str.trim().to_uppercase();
                    status = match status_str.as_str() {
                        "APPROVED" => CheckStatus::Approved,
                        "REJECTED" => CheckStatus::Rejected,
                        "NEEDS_REVISION" | "NEEDS REVISION" => CheckStatus::NeedsRevision,
                        _ => CheckStatus::NeedsRevision,
                    };
                }
            } else if line.starts_with("FEEDBACK:") {
                if let Some(feedback_str) = line.strip_prefix("FEEDBACK:") {
                    feedback = feedback_str.trim().to_string();
                }
            }
        }

        CheckResult {
            status,
            feedback,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 获取迭代历史
    pub fn get_history(&self) -> &[MakerCheckerIteration] {
        &self.history
    }

    /// 获取审核通过的迭代
    pub fn get_approved_iteration(&self) -> Option<&MakerCheckerIteration> {
        self.history
            .iter()
            .find(|iter| iter.check_result.status == CheckStatus::Approved)
    }

    /// 清空历史
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// MakerChecker 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakerCheckerStats {
    /// 总迭代次数
    pub total_iterations: usize,
    /// 审核通过次数
    pub approved_count: usize,
    /// 审核拒绝次数
    pub rejected_count: usize,
    /// 需要修改次数
    pub needs_revision_count: usize,
    /// 是否最终通过
    pub final_approved: bool,
}

impl MakerCheckerExecutor {
    /// 获取统计信息
    pub fn get_stats(&self) -> MakerCheckerStats {
        let total_iterations = self.history.len();
        let approved_count = self
            .history
            .iter()
            .filter(|h| h.check_result.status == CheckStatus::Approved)
            .count();
        let rejected_count = self
            .history
            .iter()
            .filter(|h| h.check_result.status == CheckStatus::Rejected)
            .count();
        let needs_revision_count = self
            .history
            .iter()
            .filter(|h| h.check_result.status == CheckStatus::NeedsRevision)
            .count();
        let final_approved = self.get_approved_iteration().is_some();

        MakerCheckerStats {
            total_iterations,
            approved_count,
            rejected_count,
            needs_revision_count,
            final_approved,
        }
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
    fn test_parse_check_result() {
        let executor = MakerCheckerExecutor {
            maker: create_test_agent("maker"),
            checker: create_test_agent("checker"),
            max_iterations: 5,
            history: Vec::new(),
        };

        let response = "STATUS: APPROVED\nFEEDBACK: Looks good!";
        let result = executor.parse_check_result(response);

        assert_eq!(result.status, CheckStatus::Approved);
        assert_eq!(result.feedback, "Looks good!");
    }

    #[test]
    fn test_parse_check_result_needs_revision() {
        let executor = MakerCheckerExecutor {
            maker: create_test_agent("maker"),
            checker: create_test_agent("checker"),
            max_iterations: 5,
            history: Vec::new(),
        };

        let response = "STATUS: NEEDS_REVISION\nFEEDBACK: Please add more details";
        let result = executor.parse_check_result(response);

        assert_eq!(result.status, CheckStatus::NeedsRevision);
        assert_eq!(result.feedback, "Please add more details");
    }

    #[test]
    fn test_maker_checker_stats() {
        let mut executor = MakerCheckerExecutor {
            maker: create_test_agent("maker"),
            checker: create_test_agent("checker"),
            max_iterations: 5,
            history: Vec::new(),
        };

        // 添加一些迭代记录
        executor.history.push(MakerCheckerIteration {
            iteration: 1,
            created_content: "v1".to_string(),
            check_result: CheckResult {
                status: CheckStatus::NeedsRevision,
                feedback: "Needs work".to_string(),
                timestamp: 0,
            },
            timestamp: 0,
        });
        executor.history.push(MakerCheckerIteration {
            iteration: 2,
            created_content: "v2".to_string(),
            check_result: CheckResult {
                status: CheckStatus::Approved,
                feedback: "Good!".to_string(),
                timestamp: 0,
            },
            timestamp: 0,
        });

        let stats = executor.get_stats();
        assert_eq!(stats.total_iterations, 2);
        assert_eq!(stats.approved_count, 1);
        assert_eq!(stats.needs_revision_count, 1);
        assert!(stats.final_approved);
    }
}
