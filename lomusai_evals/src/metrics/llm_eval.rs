use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::metrics::{Metric, MetricResult};
use lomusai_core::llm::{LlmProvider, LlmOptions, Message, Role};

/// LLM评估指标配置
#[derive(Clone, Serialize, Deserialize)]
pub struct LlmEvalMetricConfig {
    /// 系统提示模板
    pub system_prompt_template: String,
    
    /// 用户提示模板
    pub user_prompt_template: String,
    
    /// 用于从LLM响应中提取分数的正则表达式
    pub score_pattern: String,
    
    /// 分数的小数位数
    pub decimal_places: Option<usize>,
    
    /// 是否规范化分数到0-1范围
    pub normalize_score: bool,
}

impl Default for LlmEvalMetricConfig {
    fn default() -> Self {
        Self {
            system_prompt_template: "你是一个专业的评估专家，负责评估AI回答的质量。".to_string(),
            user_prompt_template: concat!(
                "请评估以下AI回答的质量。\n\n",
                "人类问题：{{input}}\n\n",
                "AI回答：{{output}}\n\n",
                "请从准确性、相关性和连贯性等方面对AI回答进行分析，给出1-10分的评分。\n",
                "首先提供详细的分析，然后在最后一行给出分数，格式为\"分数:X\"。"
            ).to_string(),
            score_pattern: r"(?:分数|Score)\s*(?::|：)\s*(\d+(?:\.\d+)?)".to_string(),
            decimal_places: Some(2),
            normalize_score: true,
        }
    }
}

/// 使用LLM进行评估的通用指标
#[derive(Serialize, Deserialize)]
pub struct LlmEvalMetric {
    /// 指标名称
    pub name: String,
    
    /// 指标描述
    pub description: String,
    
    /// 用于评估的LLM提供者
    #[serde(skip)]
    llm: Option<Box<dyn LlmProvider>>,
    
    /// 指标配置
    pub config: LlmEvalMetricConfig,
}

// 手动实现Clone
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
            name: "llm_eval".to_string(),
            description: "使用LLM评估AI回答的质量".to_string(),
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
    
    /// 设置指标配置
    pub fn with_config(mut self, config: LlmEvalMetricConfig) -> Self {
        self.config = config;
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
                
            let mut score: f64 = score_str.parse()
                .map_err(|_| Error::MetricCalculation(format!("无法将'{}'解析为数字", score_str)))?;
                
            // 规范化分数到0-1范围（如果配置为true）
            if self.config.normalize_score && score > 1.0 {
                // 假设分数是1-10范围内，规范化到0-1
                score = score / 10.0;
            }
            
            // 四舍五入到指定小数位
            if let Some(places) = self.config.decimal_places {
                let factor = 10.0_f64.powi(places as i32);
                score = (score * factor).round() / factor;
            }
            
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
                        } else if self.config.normalize_score && num > 1.0 && num <= 10.0 {
                            Some(num / 10.0)
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
                // 如果还是找不到，返回错误
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
        let user_prompt = self.config.user_prompt_template
            .replace("{{input}}", input)
            .replace("{{output}}", output);
                
        // 构建消息
        let messages = vec![
            Message::new(Role::System, self.config.system_prompt_template.clone()),
            Message::new(Role::User, user_prompt),
        ];
            
        // 向LLM发送评估请求
        let llm_options = LlmOptions::default();
        let response = llm.generate_with_messages(&messages, &llm_options).await?;
                
        // 提取分数
        let score = self.extract_score_from_response(&response)?;
            
        // 创建分数详情
        let mut info = HashMap::new();
        info.insert("full_analysis".to_string(), serde_json::Value::String(response));
            
        Ok(MetricResult { score, info })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::{self, BoxStream};
    use std::sync::Mutex;
    
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
        async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> lomusai_core::Result<String> {
            Ok(self.response.lock().unwrap().clone())
        }
        
        async fn generate_with_messages(&self, _messages: &[Message], _options: &LlmOptions) -> lomusai_core::Result<String> {
            Ok(self.response.lock().unwrap().clone())
        }
        
        async fn generate_stream<'a>(&'a self, _prompt: &'a str, _options: &'a LlmOptions) -> lomusai_core::Result<BoxStream<'a, lomusai_core::Result<String>>> {
            let response = self.response.lock().unwrap().clone();
            let stream = stream::once(async move { Ok(response) });
            Ok(Box::pin(stream))
        }
        
        async fn get_embedding(&self, _text: &str) -> lomusai_core::Result<Vec<f32>> {
            Ok(vec![0.1, 0.2, 0.3])
        }
    }
    
    #[tokio::test]
    async fn test_llm_eval_metric() {
        let test_llm = Box::new(TestLlmProvider::new(
            "分析：查询和回复非常相关，回复准确地提供了关于Rust的垃圾回收机制的信息。\n分数：0.92".to_string()
        ));
            
        // 创建LLM评估指标
        let metric = LlmEvalMetric::default().with_llm(test_llm);
            
        // 测试评估
        let result = metric.measure(
            "Rust中有垃圾回收吗？", 
            "Rust不使用传统的垃圾回收机制。它使用所有权系统和借用检查器在编译时管理内存，无需运行时垃圾回收器。"
        ).await;
        
        assert!(result.is_ok());
        let metric_result = result.unwrap();
        assert_eq!(metric_result.score, 0.92);
    }
    
    #[test]
    fn test_extract_score() {
        let metric = LlmEvalMetric::default();
        
        // 测试格式化分数
        let response = "分析：详细分析内容...\n分数：0.85";
        let score = metric.extract_score_from_response(response).unwrap();
        assert_eq!(score, 0.85);
        
        // 测试需要规范化的分数
        let response = "分析：详细分析内容...\n分数：8.5";
        let score = metric.extract_score_from_response(response).unwrap();
        assert_eq!(score, 0.85);
        
        // 测试英文格式
        let response = "Analysis: detailed content...\nScore: 0.7";
        let score = metric.extract_score_from_response(response).unwrap();
        assert_eq!(score, 0.7);
    }
} 