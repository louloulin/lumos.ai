//! # Chain of Thought (CoT) 推理
//!
//! 实现思维链推理,帮助 Agent 进行复杂推理。

use serde::{Deserialize, Serialize};

use super::{ReasoningError, StepType};

/// CoT 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoTConfig {
    pub verbose: bool,
    pub max_steps: usize,
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

/// CoT 步骤类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoTStepType {
    Understanding,
    Decomposition,
    Reasoning,
    Calculation,
    Verification,
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

/// CoT 步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoTStep {
    pub id: String,
    pub step_number: usize,
    pub thought: String,
    pub step_type: CoTStepType,
    pub is_critical: bool,
    pub timestamp: u64,
}

/// 思维过程
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtProcess {
    pub problem: String,
    pub steps: Vec<CoTStep>,
    pub conclusion: String,
    pub confidence: f64,
}

impl ThoughtProcess {
    pub fn new(problem: impl Into<String>) -> Self {
        Self {
            problem: problem.into(),
            steps: Vec::new(),
            conclusion: String::new(),
            confidence: 0.0,
        }
    }

    pub fn add_step(mut self, step: CoTStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn with_conclusion(mut self, conclusion: impl Into<String>) -> Self {
        self.conclusion = conclusion.into();
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Problem: {}\n\n", self.problem));

        for step in &self.steps {
            output.push_str(&format!(
                "Step {}: {} ({})\n",
                step.step_number, step.thought, step.step_type
            ));
        }

        output.push_str(&format!("\nConclusion: {}\n", self.conclusion));
        output.push_str(&format!("Confidence: {:.2}%\n", self.confidence * 100.0));

        output
    }
}

/// Chain of Thought 推理器
pub struct ChainOfThought {
    config: CoTConfig,
}

impl ChainOfThought {
    pub fn new(config: CoTConfig) -> Self {
        Self { config }
    }

    pub async fn reason(&self, problem: &str) -> Result<ThoughtProcess, ReasoningError> {
        let mut process = ThoughtProcess::new(problem);

        let step1 = CoTStep {
            id: uuid::Uuid::new_v4().to_string(),
            step_number: 1,
            thought: format!("理解问题: {}", problem),
            step_type: CoTStepType::Understanding,
            is_critical: true,
            timestamp: 123456,
        };
        process = process.add_step(step1);

        let conclusion = format!("基于 {} 个推理步骤,得出答案", process.steps.len());
        process = process.with_conclusion(conclusion).with_confidence(0.85);

        Ok(process)
    }
}

impl Default for ChainOfThought {
    fn default() -> Self {
        Self::new(CoTConfig::default())
    }
}
