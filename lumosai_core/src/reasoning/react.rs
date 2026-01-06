//! # ReAct (Reasoning + Acting) Agent
//!
//! 实现 ReAct 循环: Thought → Action → Observation

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ReasoningError, ReasoningStep, StepType};

/// ReAct 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReActConfig {
    /// 最大迭代次数
    pub max_iterations: usize,
    /// 是否启用详细日志
    pub verbose: bool,
    /// 超时时间 (秒)
    pub timeout_seconds: u64,
}

impl Default for ReActConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            verbose: false,
            timeout_seconds: 300,
        }
    }
}

/// ReAct 步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReActStep {
    /// 步骤序号
    pub iteration: usize,
    /// 思考内容
    pub thought: String,
    /// 行动内容
    pub action: Option<String>,
    /// 观察结果
    pub observation: Option<String>,
    /// 是否完成
    pub finished: bool,
}

impl ReActStep {
    /// 创建新步骤
    pub fn new(iteration: usize) -> Self {
        Self {
            iteration,
            thought: String::new(),
            action: None,
            observation: None,
            finished: false,
        }
    }

    /// 设置思考
    pub fn with_thought(mut self, thought: impl Into<String>) -> Self {
        self.thought = thought.into();
        self
    }

    /// 设置行动
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// 设置观察
    pub fn with_observation(mut self, observation: impl Into<String>) -> Self {
        self.observation = Some(observation.into());
        self
    }

    /// 标记为完成
    pub fn mark_finished(mut self) -> Self {
        self.finished = true;
        self
    }
}

/// ReAct 执行结果
#[derive(Debug, Clone)]
pub struct ReActResult {
    /// 最终答案
    pub answer: String,
    /// 执行的所有步骤
    pub steps: Vec<ReActStep>,
    /// 总迭代次数
    pub total_iterations: usize,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// ReAct Agent
pub struct ReActAgent {
    /// 配置
    config: ReActConfig,
    /// 可用工具列表
    tools: Arc<RwLock<Vec<String>>>,
    /// 执行步骤记录
    steps: Arc<RwLock<Vec<ReActStep>>>,
}

impl ReActAgent {
    /// 创建新的 ReAct Agent
    pub fn new(config: ReActConfig) -> Self {
        Self {
            config,
            tools: Arc::new(RwLock::new(Vec::new())),
            steps: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 设置最大迭代次数
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.config.max_iterations = max;
        self
    }

    /// 添加工具
    pub async fn add_tool(&self, tool: impl Into<String>) {
        let mut tools = self.tools.write().await;
        tools.push(tool.into());
    }

    /// 设置工具列表
    pub async fn with_tools(mut self, tools: Vec<String>) -> Self {
        *self.tools.write().await = tools;
        self
    }

    /// 执行 ReAct 循环
    pub async fn execute(&self, query: &str) -> Result<ReActResult, ReasoningError> {
        let mut steps = Vec::new();
        let mut current_answer = String::new();

        for iteration in 0..self.config.max_iterations {
            // 1. Thought - 思考当前状态
            let thought = self.generate_thought(query, &current_answer, iteration).await?;

            if self.config.verbose {
                println!("Iteration {} - Thought: {}", iteration + 1, thought);
            }

            // 检查是否已经完成
            if thought.contains("Final Answer") || thought.contains("完成") {
                let answer = self.extract_final_answer(&thought)?;
                steps.push(
                    ReActStep::new(iteration)
                        .with_thought(thought)
                        .mark_finished(),
                );
                return Ok(ReActResult {
                    answer,
                    steps,
                    total_iterations: iteration + 1,
                    success: true,
                    error: None,
                });
            }

            // 2. Action - 决定下一步行动
            let action = self.decide_action(&thought).await?;

            if self.config.verbose {
                println!("Iteration {} - Action: {}", iteration + 1, action);
            }

            // 3. Observation - 执行行动并观察结果
            let observation = self.execute_action(&action).await?;

            if self.config.verbose {
                println!("Iteration {} - Observation: {}", iteration + 1, observation);
            }

            // 更新当前答案
            current_answer = self.update_answer(&current_answer, &observation);

            // 记录步骤
            steps.push(
                ReActStep::new(iteration)
                    .with_thought(thought)
                    .with_action(action)
                    .with_observation(observation),
            );
        }

        // 达到最大迭代次数
        Ok(ReActResult {
            answer: current_answer,
            steps,
            total_iterations: self.config.max_iterations,
            success: false,
            error: Some("达到最大迭代次数".to_string()),
        })
    }

    /// 生成思考
    async fn generate_thought(
        &self,
        query: &str,
        current_answer: &str,
        iteration: usize,
    ) -> Result<String, ReasoningError> {
        // 简化实现: 实际应该调用 LLM
        if iteration == 0 {
            Ok(format!("I need to answer: {}", query))
        } else if current_answer.is_empty() {
            Ok("I haven't found the answer yet. Let me try an action.".to_string())
        } else {
            Ok(format!(
                "Current answer: {}. Let me verify this.",
                current_answer
            ))
        }
    }

    /// 决定行动
    async fn decide_action(&self, thought: &str) -> Result<String, ReasoningError> {
        let tools = self.tools.read().await;

        // 简化实现: 根据思考内容选择工具
        if thought.contains("calculate") || thought.contains("2+2") {
            Ok("Calculator[2+2]".to_string())
        } else if thought.contains("search") {
            Ok("Search[query]".to_string())
        } else if !tools.is_empty() {
            Ok(format!("{}[input]", tools[0]))
        } else {
            Ok("Think[continue]".to_string())
        }
    }

    /// 执行行动
    async fn execute_action(&self, action: &str) -> Result<String, ReasoningError> {
        // 简化实现: 模拟工具执行
        if action.contains("Calculator[2+2]") {
            Ok("The result is 4".to_string())
        } else if action.contains("Search") {
            Ok("Found relevant information".to_string())
        } else {
            Ok("Action completed".to_string())
        }
    }

    /// 更新答案
    fn update_answer(&self, current: &str, observation: &str) -> String {
        if current.is_empty() {
            observation.to_string()
        } else {
            format!("{} | {}", current, observation)
        }
    }

    /// 提取最终答案
    fn extract_final_answer(&self, thought: &str) -> Result<String, ReasoningError> {
        // 简化实现: 从思考中提取答案
        if thought.contains("Final Answer:") {
            Ok(thought
                .split("Final Answer:")
                .last()
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| thought.to_string()))
        } else {
            Ok(thought.to_string())
        }
    }

    /// 获取配置
    pub fn config(&self) -> &ReActConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_react_step() {
        let step = ReActStep::new(0)
            .with_thought("I need to calculate")
            .with_action("Calculator[2+2]")
            .with_observation("The result is 4")
            .mark_finished();

        assert_eq!(step.iteration, 0);
        assert_eq!(step.thought, "I need to calculate");
        assert_eq!(step.action, Some("Calculator[2+2]".to_string()));
        assert!(step.finished);
    }

    #[tokio::test]
    async fn test_react_config() {
        let config = ReActConfig::default();
        assert_eq!(config.max_iterations, 10);
        assert_eq!(config.timeout_seconds, 300);
    }

    #[tokio::test]
    async fn test_react_agent_creation() {
        let agent = ReActAgent::new(ReActConfig::default());
        assert_eq!(agent.config().max_iterations, 10);
    }

    #[tokio::test]
    async fn test_react_execution_simple() {
        let agent = ReActAgent::new(ReActConfig {
            max_iterations: 3,
            verbose: false,
            timeout_seconds: 30,
        });

        // 简化测试: 实际需要更完整的实现
        let result = agent.execute("2+2=?").await;

        // 由于实现是简化的,结果可能不完美
        assert!(result.is_ok());
        let react_result = result.unwrap();
        assert!(!react_result.steps.is_empty());
    }
}
