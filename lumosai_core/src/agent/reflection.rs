//! Reflection 协作模式
//!
//! Generator 生成初步结果，Critic 提供反馈，迭代改进
//!
//! 参考：
//! - LangGraph Reflection Pattern
//! - AutoGen Reflection
//! - Self-Refine (2023)

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::Agent;
use crate::error::Result;

/// Reflection 迭代记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionIteration {
    /// 迭代次数
    pub iteration: usize,
    /// 生成的内容
    pub generated_content: String,
    /// 批评反馈
    pub critique: String,
    /// 质量分数（0.0-1.0）
    pub quality_score: f32,
    /// 时间戳
    pub timestamp: i64,
}

/// Reflection 执行器
pub struct ReflectionExecutor {
    /// Generator Agent
    generator: Arc<dyn Agent>,
    /// Critic Agent
    critic: Arc<dyn Agent>,
    /// 最大迭代次数
    max_iterations: usize,
    /// 质量阈值（0.0-1.0）
    quality_threshold: f32,
    /// 迭代历史
    history: Vec<ReflectionIteration>,
}

impl ReflectionExecutor {
    /// 创建新的 Reflection 执行器
    pub fn new(
        generator: Arc<dyn Agent>,
        critic: Arc<dyn Agent>,
        max_iterations: usize,
        quality_threshold: f32,
    ) -> Self {
        Self {
            generator,
            critic,
            max_iterations,
            quality_threshold: quality_threshold.clamp(0.0, 1.0),
            history: Vec::new(),
        }
    }

    /// 执行 Reflection 循环
    pub async fn execute(&mut self, initial_prompt: &str) -> Result<String> {
        tracing::info!(
            "Starting Reflection with max_iterations={}, quality_threshold={}",
            self.max_iterations,
            self.quality_threshold
        );

        let mut current_prompt = initial_prompt.to_string();
        let mut best_content = String::new();
        let mut best_score = 0.0;

        for iteration in 0..self.max_iterations {
            tracing::debug!(
                "Reflection iteration {}/{}",
                iteration + 1,
                self.max_iterations
            );

            // 1. Generator 生成内容
            let generated_content = self.generator.generate_simple(&current_prompt).await?;
            tracing::debug!("Generated content: {}", generated_content);

            // 2. Critic 评估内容
            let critique_prompt = format!(
                "Please evaluate the following content and provide:\n\
                1. A quality score from 0.0 to 1.0\n\
                2. Specific feedback for improvement\n\n\
                Content:\n{}\n\n\
                Format your response as:\n\
                SCORE: <number>\n\
                FEEDBACK: <your feedback>",
                generated_content
            );

            let critique_response = self.critic.generate_simple(&critique_prompt).await?;
            tracing::debug!("Critique response: {}", critique_response);

            // 3. 解析质量分数和反馈
            let (quality_score, critique) = self.parse_critique(&critique_response);

            // 4. 记录迭代
            let iteration_record = ReflectionIteration {
                iteration: iteration + 1,
                generated_content: generated_content.clone(),
                critique: critique.clone(),
                quality_score,
                timestamp: chrono::Utc::now().timestamp(),
            };
            self.history.push(iteration_record);

            // 5. 更新最佳结果
            if quality_score > best_score {
                best_score = quality_score;
                best_content = generated_content.clone();
            }

            // 6. 检查是否达到质量阈值
            if quality_score >= self.quality_threshold {
                tracing::info!(
                    "Quality threshold reached at iteration {} (score: {})",
                    iteration + 1,
                    quality_score
                );
                return Ok(generated_content);
            }

            // 7. 准备下一轮的提示（包含反馈）
            current_prompt = format!(
                "Original task: {}\n\n\
                Previous attempt:\n{}\n\n\
                Feedback:\n{}\n\n\
                Please improve the content based on the feedback above.",
                initial_prompt, generated_content, critique
            );
        }

        // 达到最大迭代次数，返回最佳结果
        tracing::info!(
            "Reached max iterations ({}), returning best result (score: {})",
            self.max_iterations,
            best_score
        );
        Ok(best_content)
    }

    /// 解析 Critic 的反馈
    fn parse_critique(&self, critique_response: &str) -> (f32, String) {
        let mut score = 0.5; // 默认分数
        let mut feedback = critique_response.to_string();

        // 尝试解析 SCORE: 和 FEEDBACK: 格式
        for line in critique_response.lines() {
            if line.starts_with("SCORE:") {
                if let Some(score_str) = line.strip_prefix("SCORE:") {
                    if let Ok(parsed_score) = score_str.trim().parse::<f32>() {
                        score = parsed_score.clamp(0.0, 1.0);
                    }
                }
            } else if line.starts_with("FEEDBACK:") {
                if let Some(feedback_str) = line.strip_prefix("FEEDBACK:") {
                    feedback = feedback_str.trim().to_string();
                }
            }
        }

        (score, feedback)
    }

    /// 获取迭代历史
    pub fn get_history(&self) -> &[ReflectionIteration] {
        &self.history
    }

    /// 获取最佳迭代
    pub fn get_best_iteration(&self) -> Option<&ReflectionIteration> {
        self.history
            .iter()
            .max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap())
    }

    /// 清空历史
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// Reflection 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionStats {
    /// 总迭代次数
    pub total_iterations: usize,
    /// 最终质量分数
    pub final_score: f32,
    /// 平均质量分数
    pub avg_score: f32,
    /// 最佳质量分数
    pub best_score: f32,
    /// 是否达到阈值
    pub threshold_reached: bool,
}

impl ReflectionExecutor {
    /// 获取统计信息
    pub fn get_stats(&self) -> ReflectionStats {
        if self.history.is_empty() {
            return ReflectionStats {
                total_iterations: 0,
                final_score: 0.0,
                avg_score: 0.0,
                best_score: 0.0,
                threshold_reached: false,
            };
        }

        let total_iterations = self.history.len();
        let final_score = self.history.last().unwrap().quality_score;
        let avg_score =
            self.history.iter().map(|h| h.quality_score).sum::<f32>() / total_iterations as f32;
        let best_score = self
            .history
            .iter()
            .map(|h| h.quality_score)
            .fold(0.0, f32::max);
        let threshold_reached = best_score >= self.quality_threshold;

        ReflectionStats {
            total_iterations,
            final_score,
            avg_score,
            best_score,
            threshold_reached,
        }
    }
}

// 测试已移除 - 需要真实的 Agent 实现才能进行完整测试
// 可以在集成测试中使用 BasicAgent 进行测试
