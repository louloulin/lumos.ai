//! 基于LLM的评估器实现

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::types::{EvalOptions, EvalResult};
use crate::evaluator::Evaluator;
use lomusai_core::llm::{LlmProvider, LlmOptions, Message, Role};
use crate::{
    metrics::{LlmEvalMetric, Metric, MetricResult},
    TestInfo,
};

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
                "首先提供详细的分析，然后在最后一行给出分数，格式为\"分数:X\"。"
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
        // 将内部操作封装到一个异步块中，可以简化异步到同步的转换
        // 提前克隆必要的数据，避免生命周期问题
        let name = self.name.clone();
        let system_prompt = self.config.system_prompt_template.clone();
        let user_prompt_template = self.config.user_prompt_template.clone();
        let llm = self.llm.clone();
        let _score_pattern = self.config.score_pattern.clone();
        let _normalize_score = self.config.normalize_score;
        let _decimal_places = self.config.decimal_places;
        
        // 准备评估提示
        let user_prompt = user_prompt_template
            .replace("{{input}}", input)
            .replace("{{output}}", output);
                
        // 构建消息
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
            
        // 向LLM发送评估请求
        let llm_options = LlmOptions::default();
        let response = llm.generate_with_messages(&messages, &llm_options).await
            .map_err(|e| Error::Llm(e))?;
                
        // 提取分数
        let score = self.extract_score(&response)?;
            
        // 创建分数详情
        let mut score_details = HashMap::new();
        score_details.insert("full_response".to_string(), serde_json::Value::String(response));
            
        // 创建评估结果
        let global_run_id = options.global_run_id.clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
                
        let run_id = options.run_id.clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
                
        let result = EvalResult {
            id: Uuid::new_v4().to_string(),
            global_run_id,
            run_id,
            input: input.to_string(),
            output: output.to_string(),
            score,
            score_details,
            created_at: Utc::now(),
            evaluator_name: name,
            metric_name: "llm".to_string(), // 使用LLM作为指标名称
            target_name: options.target_name.clone(),
            test_info: options.test_info.clone(),
            instructions: options.instructions.clone(),
        };
            
        Ok(result)
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
    async fn test_llm_evaluator() {
        // 创建测试LLM提供者
        let test_llm = TestLlmProvider::new(
            "这个回答非常准确，提供了相关信息，且组织良好。分数:8.5".to_string()
        );
            
        // 创建LLM评估器
        let evaluator = LlmEvaluator::new("llm_eval", Arc::new(test_llm));
        
        // 测试评估
        let test_info = TestInfo {
            test_name: Some("test_accuracy".to_string()),
            test_path: None,
            tags: vec![],
            description: Some("测试回答准确性".to_string()),
        };
        
        let options = EvalOptions {
            test_info: Some(test_info),
            ..Default::default()
        };
        
        let result = evaluator.evaluate(
            "如何在Rust中处理错误？",
            "Rust提供了Result和Option类型来处理错误。Result用于可恢复错误，而Option用于可能为空的值。",
            &options
        ).await;
        
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert_eq!(eval_result.score, 0.85); // 规范化后应该是8.5/10 = 0.85
        assert_eq!(eval_result.evaluator_name, "llm_eval");
        assert!(eval_result.test_info.is_some());
    }
    
    #[test]
    fn test_extract_score() {
        let test_llm = TestLlmProvider::new("".to_string());
        let evaluator = LlmEvaluator::new("test", Arc::new(test_llm));
        
        // 测试标准格式
        let result = evaluator.extract_score("分析结果很好。分数:7.5");
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