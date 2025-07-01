use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::metrics::{Metric, MetricResult};
use lumosai_core::llm::{LlmProvider, LlmOptions, Message, Role};

/// 连贯性评估指标，用于评估输出的逻辑连贯性和一致性
#[derive(Serialize, Deserialize)]
pub struct CoherenceMetric {
    /// 指标名称
    pub name: String,
    
    /// 指标描述
    pub description: String,
    
    /// 用于评估的LLM提供者
    #[serde(skip)]
    llm: Option<Box<dyn LlmProvider>>,
    
    /// 用于评估的提示模板
    pub prompt_template: String,
}

// 手动实现Clone
impl Clone for CoherenceMetric {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            llm: None, // LLM不能克隆，所以设为None
            prompt_template: self.prompt_template.clone(),
        }
    }
}

impl Default for CoherenceMetric {
    fn default() -> Self {
        Self {
            name: "coherence".to_string(),
            description: "评估输出的逻辑连贯性和一致性".to_string(),
            llm: None,
            prompt_template: concat!(
                "你是一名专业的文本连贯性评估专家。你需要评估下面的文本是否连贯、一致、有条理。\n\n",
                "文本内容: {{output}}\n\n",
                "请分析这段文本的连贯性，评估它是否内部一致、逻辑清晰、组织有序。首先详细分析文本的连贯性，",
                "然后给出0到1之间的分数，其中1表示完全连贯和一致，0表示完全不连贯和自相矛盾。\n\n",
                "分析结果格式如下：\n",
                "分析: <分析文本>\n",
                "分数: <0到1之间的分数>"
            ).to_string(),
        }
    }
}

impl CoherenceMetric {
    /// 创建一个新的连贯性指标
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
impl Metric for CoherenceMetric {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn measure(&self, _input: &str, output: &str) -> Result<MetricResult> {
        let llm = self.llm.as_ref()
            .ok_or_else(|| Error::Configuration("未设置LLM提供者".to_string()))?;
        
        // 准备评估提示
        let prompt = self.prompt_template.replace("{{output}}", output);
            
        // 发送到LLM进行评估
        let messages = vec![
            Message {
                role: Role::System,
                content: "你是一个专业的文本连贯性评估专家。".to_string(),
                metadata: None,
                name: None,
            },
            Message {
                role: Role::User,
                content: prompt,
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
        info.insert("full_analysis".to_string(), serde_json::Value::String(response));
        
        Ok(MetricResult { score, info })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::{self, BoxStream};
    use std::sync::Mutex;
    
    // 简单的mock LLM提供者，避免使用mockall
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
        fn name(&self) -> &str {
            "test-llm-provider"
        }

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
    async fn test_coherence_metric() {
        let test_llm = TestLlmProvider::new(
            "分析: 文本逻辑清晰，段落之间有良好的衔接，没有自相矛盾的地方。\n分数: 0.85".to_string()
        );
            
        // 创建连贯性指标
        let metric = CoherenceMetric::default()
            .with_llm(Box::new(test_llm));
            
        // 测试评估
        let result = metric.measure(
            "", // 输入不使用
            "Rust语言以其安全性和性能著称。它通过所有权系统保证内存安全，同时没有垃圾回收带来的性能开销。Rust的特性使得开发人员可以在编译时捕获大量错误，而不是在运行时。"
        ).await;
        
        assert!(result.is_ok());
        let metric_result = result.unwrap();
        assert_eq!(metric_result.score, 0.85);
    }
} 