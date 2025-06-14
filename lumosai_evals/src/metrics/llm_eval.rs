use std::collections::HashMap;
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use lumosai_core::{
    llm::{LlmOptions, LlmProvider},
    Message, Role,
};

use crate::error::{Error, Result};
use crate::metrics::{Metric, MetricResult};

/// LLM评估指标配置
#[derive(Serialize, Deserialize)]
pub struct LlmEvalMetricConfig {
    /// 系统提示模板
    pub system_prompt_template: String,
    
    /// 用户提示模板
    pub user_prompt_template: String,
    
    /// 分数提取正则表达式
    pub score_regex_pattern: String,
    
    /// 小数点位数
    pub decimal_places: usize,
    
    /// 是否规范化分数到0-1范围
    pub normalize_score: bool,
}

// 手动实现Clone
impl Clone for LlmEvalMetricConfig {
    fn clone(&self) -> Self {
        Self {
            system_prompt_template: self.system_prompt_template.clone(),
            user_prompt_template: self.user_prompt_template.clone(),
            score_regex_pattern: self.score_regex_pattern.clone(),
            decimal_places: self.decimal_places,
            normalize_score: self.normalize_score,
        }
    }
}

impl Default for LlmEvalMetricConfig {
    fn default() -> Self {
        Self {
            system_prompt_template: "You are an expert evaluator of AI responses.".to_string(),
            user_prompt_template: concat!(
                "Please evaluate the following response based on the given input:\n\n",
                "Input: {{input}}\n\n",
                "Response to evaluate: {{output}}\n\n",
                "Score the response from 0 to 10, where 0 is the worst and 10 is the best.\n",
                "Provide your reasoning and then give a score in the format 'Score: X'.",
            ).to_string(),
            score_regex_pattern: r"Score:\s*(\d+(?:\.\d+)?)".to_string(),
            decimal_places: 2,
            normalize_score: true,
        }
    }
}

/// LLM评估指标，使用LLM来评估另一个AI的输出
#[derive(Serialize, Deserialize)]
pub struct LlmEvalMetric {
    /// 指标名称
    pub name: String,
    
    /// 指标描述
    pub description: String,
    
    /// 用于评估的LLM提供者
    #[serde(skip)]
    llm: Option<Box<dyn LlmProvider>>,
    
    /// 评估配置
    pub config: LlmEvalMetricConfig,
}

// 手动实现Clone，避免需要为dyn LlmProvider实现Clone
impl Clone for LlmEvalMetric {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            llm: None, // LLM不能克隆，所以设为None
            config: self.config.clone(),
        }
    }
}

impl Default for LlmEvalMetric {
    fn default() -> Self {
        Self {
            name: "llm_evaluation".to_string(),
            description: "使用LLM评估AI输出的质量".to_string(),
            llm: None,
            config: LlmEvalMetricConfig::default(),
        }
    }
}

impl LlmEvalMetric {
    /// 创建一个新的LLM评估指标
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            ..Default::default()
        }
    }
    
    /// 设置LLM提供者
    pub fn with_llm(mut self, llm: Box<dyn LlmProvider>) -> Self {
        self.llm = Some(llm);
        self
    }
    
    /// 设置评估配置
    pub fn with_config(mut self, config: LlmEvalMetricConfig) -> Self {
        self.config = config;
        self
    }
    
    /// 从LLM响应中提取分数
    fn extract_score_from_response(&self, response: &str) -> Result<f64> {
        let re = Regex::new(&self.config.score_regex_pattern)
            .map_err(|e| Error::MetricCalculation(format!("无效的正则表达式: {}", e)))?;
            
        if let Some(captures) = re.captures(response) {
            if let Some(score_match) = captures.get(1) {
                let score_str = score_match.as_str();
                let score: f64 = score_str.parse()
                    .map_err(|_| Error::MetricCalculation(format!("无法将'{}'解析为数字", score_str)))?;
                    
                // 如果需要规范化分数到0-1范围
                let normalized_score = if self.config.normalize_score {
                    score / 10.0
                } else {
                    score
                };
                
                // 四舍五入到指定小数位
                let factor = 10.0_f64.powi(self.config.decimal_places as i32);
                let rounded_score = (normalized_score * factor).round() / factor;
                
                Ok(rounded_score)
            } else {
                Err(Error::MetricCalculation("未找到分数匹配组".to_string()))
            }
        } else {
            // 如果没有匹配，尝试从文本中提取数字
            let numbers: Vec<f64> = response.split_whitespace()
                .filter_map(|word| {
                    word.trim_matches(|c: char| !c.is_digit(10) && c != '.')
                        .parse::<f64>().ok()
                })
                .collect();
                
            if let Some(&score) = numbers.last() {
                let normalized_score = if self.config.normalize_score {
                    score / 10.0
                } else {
                    score
                };
                
                let factor = 10.0_f64.powi(self.config.decimal_places as i32);
                let rounded_score = (normalized_score * factor).round() / factor;
                
                Ok(rounded_score)
            } else {
                Err(Error::MetricCalculation("无法从响应中提取分数".to_string()))
            }
        }
    }
}

#[async_trait]
impl Metric for LlmEvalMetric {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn measure(&self, input: &str, output: &str) -> Result<MetricResult> {
        let llm = self.llm.as_ref()
            .ok_or_else(|| Error::Configuration("未设置LLM提供者".to_string()))?;
        
        // 准备评估提示
        let system_prompt = self.config.system_prompt_template.clone();
        let user_prompt = self.config.user_prompt_template
            .replace("{{input}}", input)
            .replace("{{output}}", output);
            
        // 发送到LLM进行评估
        let messages = vec![
            Message {
                role: Role::System,
                content: system_prompt,
                metadata: None,
                name: None,
            },
            Message {
                role: Role::User,
                content: user_prompt,
                metadata: None,
                name: None,
            },
        ];
        
        let options = LlmOptions::default();
        let response = llm.generate_with_messages(&messages, &options).await?;
        
        // 提取评估结果
        let score = self.extract_score_from_response(&response)?;
        
        // 创建分析信息
        let mut info = HashMap::new();
        info.insert("full_analysis".to_string(), serde_json::Value::String(response.clone()));
        
        Ok(MetricResult { score, info })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 简单的mock LLM提供者
    struct TestLlmProvider {
        response: Mutex<String>,
    }
    
    impl TestLlmProvider {
        fn new(response: String) -> Self {
            Self { response: Mutex::new(response) }
        }
    }
    
    #[async_trait]
    impl LlmProvider for TestLlmProvider {
        async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> lumosai_core::Result<String> {
            Ok(self.response.lock().unwrap().clone())
        }
        
        async fn generate_with_messages(&self, _messages: &[Message], _options: &LlmOptions) -> lumosai_core::Result<String> {
            Ok(self.response.lock().unwrap().clone())
        }
        
        async fn generate_stream<'a>(&'a self, _prompt: &'a str, _options: &'a LlmOptions) -> lumosai_core::Result<BoxStream<'a, lumosai_core::Result<String>>> {
            let response = self.response.lock().unwrap().clone();
            let stream = stream::once(async move { Ok(response) });
            Ok(Box::pin(stream))
        }
        
        async fn get_embedding(&self, _text: &str) -> lumosai_core::Result<Vec<f32>> {
            Ok(vec![0.1, 0.2, 0.3])
        }
    }
    
    #[tokio::test]
    async fn test_llm_evaluator() {
        // 创建测试LLM提供者
        let test_llm = Box::new(TestLlmProvider::new(
            "The response accurately explains that Rust doesn't have garbage collection. Score: 9.2".to_string()
        ));
            
        // 创建LLM评估指标
        let metric = LlmEvalMetric::default().with_llm(test_llm);
            
        // 测试评估
        let input = "Does Rust have garbage collection?";
        let output = "No, Rust uses a system called ownership with borrowing and lifetimes instead of garbage collection. This allows for memory safety without the overhead of a garbage collector.";
        
        let result = metric.measure(input, output).await;
        
        assert!(result.is_ok());
        let metric_result = result.unwrap();
        assert_eq!(metric_result.score, 0.92);
    }
    
    #[test]
    fn test_extract_score() {
        let metric = LlmEvalMetric::default();
        
        // 测试不同格式的分数提取
        assert_eq!(metric.extract_score_from_response("Score: 7.5").unwrap(), 0.75);
        assert_eq!(metric.extract_score_from_response("Final score: Score: 8").unwrap(), 0.8);
        assert_eq!(metric.extract_score_from_response("I would rate this as Score: 10").unwrap(), 1.0);
        
        // 测试自定义配置
        let custom_config = LlmEvalMetricConfig {
            score_regex_pattern: r"Rating:\s*(\d+(?:\.\d+)?)".to_string(),
            normalize_score: false,
            decimal_places: 1,
            ..LlmEvalMetricConfig::default()
        };
        
        let custom_metric = LlmEvalMetric::default().with_config(custom_config);
        assert_eq!(custom_metric.extract_score_from_response("Rating: 7.56").unwrap(), 7.6);
    }
}