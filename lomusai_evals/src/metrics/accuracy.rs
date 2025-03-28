use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::metrics::{Metric, MetricResult};
use lomusai_core::llm::{LlmProvider, LlmOptions, Message, Role};

/// 准确性评估指标，用于评估输出在事实上的准确性
#[derive(Clone, Serialize, Deserialize)]
pub struct AccuracyMetric {
    /// 任意名称
    pub name: String,
    
    /// 该指标的描述
    pub description: String,
    
    /// 用于评估的LLM提供者
    #[serde(skip)]
    llm: Option<Box<dyn LlmProvider>>,
    
    /// 用于评估的提示模板
    pub prompt_template: String,
}

impl Default for AccuracyMetric {
    fn default() -> Self {
        Self {
            name: "accuracy".to_string(),
            description: "评估输出在事实上的准确性".to_string(),
            llm: None,
            prompt_template: concat!(
                "你是一名评估AI响应准确性的专家。你需要评估下面的回答是否在事实上准确。\n\n",
                "输入问题: {{input}}\n\n",
                "AI回答: {{output}}\n\n",
                "请分析AI回答中的事实准确性，判断是否存在事实错误。首先详细分析各个陈述的准确性，",
                "然后给出0到1之间的分数，其中1表示完全准确，0表示完全不准确。\n\n",
                "分析结果格式如下：\n",
                "分析: <分析文本>\n",
                "分数: <0到1之间的分数>"
            ).to_string(),
        }
    }
}

impl AccuracyMetric {
    /// 创建一个新的准确性指标
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
    
    /// 设置评估提示模板
    pub fn with_prompt_template(mut self, template: impl Into<String>) -> Self {
        self.prompt_template = template.into();
        self
    }
    
    /// 从LLM响应中提取分数
    fn extract_score_from_response(&self, response: &str) -> Result<f64> {
        // 尝试从格式化响应中提取分数
        if let Some(score_line) = response.lines()
            .find(|line| line.trim().starts_with("分数:") || line.trim().starts_with("分数：") || 
                 line.trim().starts_with("Score:"))
        {
            let score_str = score_line.split(':').nth(1)
                .or_else(|| score_line.split('：').nth(1))
                .ok_or_else(|| Error::MetricCalculation("无法解析分数行".to_string()))?
                .trim();
                
            let score: f64 = score_str.parse()
                .map_err(|_| Error::MetricCalculation(format!("无法将'{}'解析为数字", score_str)))?;
                
            if score < 0.0 || score > 1.0 {
                return Err(Error::MetricCalculation(format!("分数'{}'超出0-1范围", score)));
            }
            
            Ok(score)
        } else {
            // 如果没有找到明确的分数行，尝试从文本中提取数字
            let numbers: Vec<f64> = response.split_whitespace()
                .filter_map(|word| {
                    if let Ok(num) = word.parse::<f64>() {
                        if num >= 0.0 && num <= 1.0 {
                            Some(num)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
                
            if let Some(&score) = numbers.last() {
                Ok(score)
            } else {
                // 如果还是找不到，返回默认分数
                Err(Error::MetricCalculation("无法从响应中提取分数".to_string()))
            }
        }
    }
}

#[async_trait]
impl Metric for AccuracyMetric {
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
        let prompt = self.prompt_template.replace("{{input}}", input)
            .replace("{{output}}", output);
            
        // 发送到LLM进行评估
        let messages = vec![
            Message::new(Role::System, "你是一个专业的AI回答评估专家，负责评估回答的准确性。"),
            Message::new(Role::User, prompt),
        ];
        
        let options = LlmOptions::default();
        let response = llm.generate_with_messages(&messages, &options).await?;
        
        // 提取评估结果
        let score = self.extract_score_from_response(&response)?;
        
        // 创建分析信息
        let mut info = HashMap::new();
        info.insert("full_analysis".to_string(), serde_json::Value::String(response));
        
        Ok(MetricResult { score, info })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;
    use futures::stream::BoxStream;
    
    mock! {
        LlmProviderMock {}
        
        #[async_trait]
        impl LlmProvider for LlmProviderMock {
            async fn generate(&self, prompt: &str, options: &LlmOptions) -> lomusai_core::Result<String>;
            async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> lomusai_core::Result<String>;
            async fn generate_stream<'a>(&'a self, prompt: &'a str, options: &'a LlmOptions) -> lomusai_core::Result<BoxStream<'a, lomusai_core::Result<String>>>;
            async fn get_embedding(&self, text: &str) -> lomusai_core::Result<Vec<f32>>;
        }
    }
    
    #[tokio::test]
    async fn test_accuracy_metric() {
        let mut mock_llm = MockLlmProviderMock::new();
        
        // 设置模拟LLM的行为
        mock_llm.expect_generate_with_messages()
            .returning(|_, _| {
                Ok("分析: 回答提供的信息是准确的，没有明显的事实错误。\n分数: 0.95".to_string())
            });
            
        // 创建准确性指标
        let metric = AccuracyMetric::default()
            .with_llm(Box::new(mock_llm));
            
        // 测试评估
        let result = metric.measure(
            "什么是Rust语言？", 
            "Rust是一门系统编程语言，专注于安全，特别是并发安全，支持函数式和命令式以及泛型等编程范式的多范式语言。"
        ).await;
        
        assert!(result.is_ok());
        let metric_result = result.unwrap();
        assert_eq!(metric_result.score, 0.95);
    }
    
    #[test]
    fn test_extract_score() {
        let metric = AccuracyMetric::default();
        
        let response1 = "分析: 很好\n分数: 0.8";
        assert_eq!(metric.extract_score_from_response(response1).unwrap(), 0.8);
        
        let response2 = "分析结果：回答非常准确\n分数：0.95";
        assert_eq!(metric.extract_score_from_response(response2).unwrap(), 0.95);
        
        let response3 = "This answer is excellent and factually accurate. Score: 0.9";
        assert_eq!(metric.extract_score_from_response(response3).unwrap(), 0.9);
    }
} 