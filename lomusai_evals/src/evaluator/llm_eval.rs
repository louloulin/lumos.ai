//! 基于LLM的评估器实现

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::evaluator::{Evaluator, EvalOptions, EvalResult, TestInfo};
use lomusai_core::llm::{LlmProvider, LlmOptions, Message, Role};

/// 配置LLM评估器的选项
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LlmEvaluatorConfig {
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

impl Default for LlmEvaluatorConfig {
    fn default() -> Self {
        Self {
            system_prompt_template: "你是一个专业的评估专家，负责评估AI回答的质量。".to_string(),
            user_prompt_template: concat!(
                "请评估以下AI回答的质量。\n\n",
                "人类问题：{{input}}\n\n",
                "AI回答：{{output}}\n\n",
                "请从准确性、相关性和连贯性等方面对AI回答进行分析，给出1-10分的评分。\n",
                "首先提供详细的分析，然后在最后一行给出分数，格式为"分数：X"。"
            ).to_string(),
            score_pattern: r"(?:分数|Score)\s*(?::|：)\s*(\d+(?:\.\d+)?)".to_string(),
            decimal_places: Some(2),
            normalize_score: true,
        }
    }
}

/// 使用LLM进行评估的评估器
pub struct LlmEvaluator {
    /// 评估器名称
    name: String,
    
    /// LLM提供者
    llm: Arc<dyn LlmProvider>,
    
    /// 评估配置
    config: LlmEvaluatorConfig,
}

impl LlmEvaluator {
    /// 创建一个新的LLM评估器
    pub fn new(name: impl Into<String>, llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            name: name.into(),
            llm,
            config: LlmEvaluatorConfig::default(),
        }
    }
    
    /// 设置评估配置
    pub fn with_config(mut self, config: LlmEvaluatorConfig) -> Self {
        self.config = config;
        self
    }
    
    /// 从LLM响应中提取分数
    fn extract_score(&self, response: &str) -> Result<f64> {
        // 使用正则表达式提取分数
        let re = Regex::new(&self.config.score_pattern)
            .map_err(|e| Error::MetricCalculation(format!("正则表达式错误: {}", e)))?;
            
        if let Some(caps) = re.captures(response) {
            if let Some(score_match) = caps.get(1) {
                let score_str = score_match.as_str();
                let mut score: f64 = score_str.parse()
                    .map_err(|_| Error::MetricCalculation(format!("无法解析分数: {}", score_str)))?;
                    
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
                
                return Ok(score);
            }
        }
        
        // 如果无法提取分数，返回错误
        Err(Error::MetricCalculation("无法从LLM响应中提取分数".to_string()))
    }
}

#[async_trait]
impl Evaluator for LlmEvaluator {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn evaluate(&self, input: &str, output: &str, options: &EvalOptions) -> Result<EvalResult> {
        // 准备评估提示
        let system_prompt = self.config.system_prompt_template.clone();
        let user_prompt = self.config.user_prompt_template
            .replace("{{input}}", input)
            .replace("{{output}}", output);
            
        // 构建消息
        let messages = vec![
            Message::new(Role::System, system_prompt),
            Message::new(Role::User, user_prompt),
        ];
        
        // 向LLM发送评估请求
        let llm_options = LlmOptions::default();
        let response = self.llm.generate_with_messages(&messages, &llm_options).await
            .map_err(|e| Error::Llm(e.to_string()))?;
            
        // 提取分数
        let score = self.extract_score(&response)?;
        
        // 创建元数据
        let mut metadata = HashMap::new();
        metadata.insert("full_response".to_string(), serde_json::Value::String(response));
        
        if let Some(test_name) = &options.test_name {
            metadata.insert("test_name".to_string(), serde_json::Value::String(test_name.clone()));
        }
        
        // 创建评估结果
        let result = EvalResult {
            id: Uuid::new_v4().to_string(),
            evaluator_id: self.name.clone(),
            input: input.to_string(),
            output: output.to_string(),
            score,
            timestamp: Utc::now(),
            test_info: options.test_name.clone().map(|name| TestInfo {
                name,
                description: options.test_description.clone().unwrap_or_default(),
            }),
            metadata,
        };
        
        Ok(result)
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
    async fn test_llm_evaluator() {
        // 创建模拟LLM提供者
        let mut mock_llm = MockLlmProviderMock::new();
        
        // 设置模拟行为
        mock_llm.expect_generate_with_messages()
            .returning(|_, _| {
                Ok("这个回答非常准确，提供了相关信息，且组织良好。分数：8.5".to_string())
            });
            
        // 创建LLM评估器
        let evaluator = LlmEvaluator::new("llm_eval", Arc::new(mock_llm));
        
        // 测试评估
        let options = EvalOptions {
            test_name: Some("test_accuracy".to_string()),
            test_description: Some("测试回答准确性".to_string()),
        };
        
        let result = evaluator.evaluate(
            "如何在Rust中处理错误？",
            "Rust提供了Result和Option类型来处理错误。Result用于可恢复错误，而Option用于可能为空的值。",
            &options
        ).await;
        
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert_eq!(eval_result.score, 0.85); // 规范化后应该是8.5/10 = 0.85
        assert_eq!(eval_result.evaluator_id, "llm_eval");
        assert!(eval_result.test_info.is_some());
        let test_info = eval_result.test_info.unwrap();
        assert_eq!(test_info.name, "test_accuracy");
    }
    
    #[test]
    fn test_extract_score() {
        let llm = MockLlmProviderMock::new();
        let evaluator = LlmEvaluator::new("test", Arc::new(llm));
        
        // 测试标准格式
        let result = evaluator.extract_score("分析结果很好。分数：7.5");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.75); // 规范化后
        
        // 测试英文格式
        let result = evaluator.extract_score("Good analysis. Score: 8");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.8); // 规范化后
        
        // 测试无法提取的情况
        let result = evaluator.extract_score("这里没有分数");
        assert!(result.is_err());
    }
} 