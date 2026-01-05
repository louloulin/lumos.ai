//! # Chain of Thought (CoT) 推理
//!
//! 实现思维链推理,帮助 Agent 进行复杂推理。

use serde::{Deserialize, Serialize};

use super::{ReasoningError, StepType};

/// CoT 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoTConfig {
    /// 是否显示详细推理过程
    pub verbose: bool,
    /// 最大思维步骤数
    pub max_steps: usize,
    /// 是否启用自我验证
    pub enable_self_verification: bool,
}

impl Default for CoTConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            max_steps: 20,
            enable_self_verification: true,
        }
    }
}

/// CoT 步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoTStep {
    /// 步骤 ID
    pub id: String,
    /// 步骤序号
    pub step_number: usize,
    /// 思维内容
    pub thought: String,
    /// 步骤类型
    pub step_type: CoTStepType,
    /// 是否为关键步骤
    pub is_critical: bool,
    /// 时间戳
    pub timestamp: u64,
}

/// CoT 步骤类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoTStepType {
    /// 初始理解
    Understanding,
    /// 分解
    Decomposition,
    /// 推理
    Reasoning,
    /// 计算
    Calculation,
    /// 验证
    Verification,
    /// 结论
    Conclusion,
}

impl std::fmt::Display for CoTStepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoTStepType::Understanding => write!(f, "Understanding"),
            CoTStepType::Decomposition => write!(f, "Decomposition"),
            CoTStepType::Reasoning => write!(f, "Reasoning"),
            CoTStepType::Calculation => write!(f, "Calculation"),
            CoTStepType::Verification => write!(f, "Verification"),
            CoTStepType::Conclusion => write!(f, "Conclusion"),
        }
    }
}

/// 思维过程
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtProcess {
    /// 问题
    pub problem: String,
    /// 思维步骤
    pub steps: Vec<CoTStep>,
    /// 最终结论
    pub conclusion: String,
    /// 置信度 (0-1)
    pub confidence: f64,
}

impl ThoughtProcess {
    /// 创建新的思维过程
    pub fn new(problem: impl Into<String>) -> Self {
        Self {
            problem: problem.into(),
            steps: Vec::new(),
            conclusion: String::new(),
            confidence: 0.0,
        }
    }

    /// 添加步骤
    pub fn add_step(mut self, step: CoTStep) -> Self {
        self.steps.push(step);
        self
    }

    /// 设置结论
    pub fn with_conclusion(mut self, conclusion: impl Into<String>) -> Self {
        self.conclusion = conclusion.into();
        self
    }

    /// 设置置信度
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    /// 格式化为可读文本
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(output.push_str(&format!format!("{}", ("Problem: {}\n\n", self.problem));

        for step in &self.steps {
            output.push_str(output.push_str(&format!format!("{}", (
                "Step {}: {} ({})\n",
                step.step_number, step.thought, step.step_type
            ));
        }

        output.push_str(output.push_str(&format!format!("{}", ("\nConclusion: {}\n", self.conclusion));
        output.push_str(output.push_str(&format!format!("{}", ("Confidence: {:.2}%\n", self.confidence * 100.0));

        output
    }
}

/// Chain of Thought 推理器
pub struct ChainOfThought {
    /// 配置
    config: CoTConfig,
}

impl ChainOfThought {
    /// 创建新的 CoT 推理器
    pub fn new(config: CoTConfig) -> Self {
        Self { config }
    }

    /// 推理
    pub async fn reason(&self, problem: &str) -> Result<ThoughtProcess, ReasoningError> {
        let mut process = ThoughtProcess::new(problem);

        // 步骤 1: 理解问题
        let step1 = self.step_understanding(problem).await?;
        process = process.add_step(step1);

        // 步骤 2: 分解问题
        let sub_problems = self.step_decomposition(problem).await?;
        let step2 = CoTStep {
            id: uuid::Uuid::new_v4().to_string(),
            step_number: 2,
            thought: format!("分解为 {} 个子问题", sub_problems.len()),
            step_type: CoTStepType::Decomposition,
            is_critical: true,
            timestamp: self.current_timestamp(),
        };
        process = process.add_step(step2);

        // 步骤 3: 逐步推理
        for (i, sub_problem) in sub_problems.iter().enumerate() {
            let reasoning_step = self.step_reasoning(sub_problem).await?;
            let step = CoTStep {
                id: uuid::Uuid::new_v4().to_string(),
                step_number: 3 + i,
                thought: reasoning_step,
                step_type: CoTStepType::Reasoning,
                is_critical: false,
                timestamp: self.current_timestamp(),
            };
            process = process.add_step(step);
        }

        // 步骤 4: 验证 (如果启用)
        if self.config.enable_self_verification {
            let verification = self.step_verification(&process).await?;
            let step = CoTStep {
                id: uuid::Uuid::new_v4().to_string(),
                step_number: 3 + sub_problems.len() + 1,
                thought: verification,
                step_type: CoTStepType::Verification,
                is_critical: true,
                timestamp: self.current_timestamp(),
            };
            process = process.add_step(step);
        }

        // 步骤 5: 结论
        let conclusion = self.step_conclusion(&process).await?;
        process = process
            .with_conclusion(conclusion.clone())
            .with_confidence(0.85); // 简化: 固定置信度

        Ok(process)
    }

    /// 理解问题
    async fn step_understanding(&self, problem: &str) -> Result<CoTStep, ReasoningError> {
        Ok(CoTStep {
            id: uuid::Uuid::new_v4().to_string(),
            step_number: 1,
            thought: format!("理解问题: {}", problem),
            step_type: CoTStepType::Understanding,
            is_critical: true,
            timestamp: self.current_timestamp(),
        })
    }

    /// 分解问题
    async fn step_decomposition(&self, _problem: &str) -> Result<Vec<String>, ReasoningError> {
        // 简化实现: 返回子问题列表
        Ok(vec!["分析输入".to_string(), "处理逻辑".to_string(), "验证结果".to_string()])
    }

    /// 推理步骤
    async fn step_reasoning(&self, sub_problem: &str) -> Result<String, ReasoningError> {
        Ok(format!("对 '{}' 进行推理分析", sub_problem))
    }

    /// 验证步骤
    async fn step_verification(&self, _process: &ThoughtProcess) -> Result<String, ReasoningError> {
        Ok("验证推理逻辑和结果的一致性".to_string())
    }

    /// 结论步骤
    async fn step_conclusion(&self, process: &ThoughtProcess) -> Result<String, ReasoningError> {
        Ok(format!(
            "基于 {} 个推理步骤,得出最终答案",
            process.steps.len()
        ))
    }

    /// 获取当前时间戳
    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Default for ChainOfThought {
    fn default() -> Self {
        Self::new(CoTConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thought_process_creation() {
        let process = ThoughtProcess::new("What is 2+2?")
            .with_conclusion("The answer is 4")
            .with_confidence(0.95);

        assert_eq!(process.problem, "What is 2+2?");
        assert_eq!(process.conclusion, "The answer is 4");
        assert_eq!(process.confidence, 0.95);
    }

    #[test]
    fn test_cot_config() {
        let config = CoTConfig::default();
        assert_eq!(config.max_steps, 20);
        assert!(config.enable_self_verification);
    }

    #[tokio::test]
    async fn test_chain_of_thought() {
        let cot = ChainOfThought::new(CoTConfig::default());
        let result = cot.reason("Solve: 2+2=?").await;

        assert!(result.is_ok());
        let process = result.unwrap();
        assert!(!process.steps.is_empty());
        assert!(!process.conclusion.is_empty());
    }

    #[test]
    fn test_thought_process_format() {
        let process = ThoughtProcess::new("Test problem")
            .add_step(CoTStep {
                id: "1".to_string(),
                step_number: 1,
                thought: "Initial thought".to_string(),
                step_type: CoTStepType::Understanding,
                is_critical: true,
                timestamp: 123456,
            })
            .with_conclusion("Test conclusion")
            .with_confidence(0.9);

        let formatted = process.format();
        assert!(formatted.contains("Test problem"));
        assert!(formatted.contains("Initial thought"));
        assert!(formatted.contains("Test conclusion"));
        assert!(formatted.contains("90.00%"));
    }
}
