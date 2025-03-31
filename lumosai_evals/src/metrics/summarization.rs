use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::metrics::{Metric, MetricResult};
use lumosai_core::llm::{LlmProvider, LlmOptions, Message, Role};

/// 摘要评估指标，用于评估摘要的质量
/// 
/// 评估两个关键方面：
/// 1. 忠实度（Faithfulness）：摘要是否忠实于原文，不包含虚假信息
/// 2. 信息覆盖度（Coverage）：摘要是否包含原文的关键信息
#[derive(Serialize, Deserialize)]
pub struct SummarizationMetric {
    /// 任意名称
    pub name: String,
    
    /// 该指标的描述
    pub description: String,
    
    /// 用于评估的LLM提供者（可选）
    #[serde(skip)]
    llm: Option<Box<dyn LlmProvider>>,
    
    /// 用于评估的提示模板
    pub prompt_template: String,
}

// 手动实现Clone
impl Clone for SummarizationMetric {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            llm: None, // LLM不能克隆，所以设为None
            prompt_template: self.prompt_template.clone(),
        }
    }
}

impl Default for SummarizationMetric {
    fn default() -> Self {
        Self {
            name: "summarization".to_string(),
            description: "评估摘要的质量，包括忠实度和信息覆盖度".to_string(),
            llm: None,
            prompt_template: concat!(
                "你是一名评估摘要质量的专家。你需要评估下面的摘要质量。\n\n",
                "原文: {{input}}\n\n",
                "摘要: {{output}}\n\n",
                "请从两个方面评估摘要质量：\n",
                "1. 忠实度：摘要是否忠实于原文，不包含虚假信息或错误解读\n",
                "2. 信息覆盖度：摘要是否包含原文的关键信息，没有遗漏重要内容\n\n",
                "针对每个方面给出0-1的分数，然后计算总分（两者的较小值）。\n\n",
                "分析结果格式如下：\n",
                "忠实度分析: <分析文本>\n",
                "忠实度分数: <0到1之间的分数>\n\n",
                "信息覆盖度分析: <分析文本>\n",
                "信息覆盖度分数: <0到1之间的分数>\n\n",
                "总分: <忠实度与信息覆盖度的较小值>"
            ).to_string(),
        }
    }
}

impl SummarizationMetric {
    /// 创建一个新的摘要评估指标
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
    fn extract_scores_from_response(&self, response: &str) -> Result<(f64, f64, f64)> {
        // 提取忠实度分数
        let faithfulness_score = Self::extract_named_score(response, "忠实度分数")
            .or_else(|_| Self::extract_named_score(response, "Faithfulness Score"))?;
            
        // 提取信息覆盖度分数
        let coverage_score = Self::extract_named_score(response, "信息覆盖度分数")
            .or_else(|_| Self::extract_named_score(response, "Coverage Score"))?;
            
        // 提取总分
        let total_score = Self::extract_named_score(response, "总分")
            .or_else(|_| Self::extract_named_score(response, "Total Score"))
            .unwrap_or_else(|_| f64::min(faithfulness_score, coverage_score));
            
        Ok((faithfulness_score, coverage_score, total_score))
    }
    
    /// 提取指定名称的分数
    fn extract_named_score(response: &str, score_name: &str) -> Result<f64> {
        for line in response.lines() {
            let line = line.trim();
            if line.starts_with(score_name) && (line.contains(':') || line.contains('：')) {
                let score_part = line.split(':').nth(1)
                    .or_else(|| line.split('：').nth(1))
                    .ok_or_else(|| Error::MetricCalculation(format!("无法解析{}行", score_name)))?
                    .trim();
                    
                let score: f64 = score_part.parse()
                    .map_err(|_| Error::MetricCalculation(format!("无法将'{}'解析为数字", score_part)))?;
                    
                if score < 0.0 || score > 1.0 {
                    return Err(Error::MetricCalculation(format!("分数'{}'超出0-1范围", score)));
                }
                
                return Ok(score);
            }
        }
        
        Err(Error::MetricCalculation(format!("无法找到{}", score_name)))
    }
}

#[async_trait]
impl Metric for SummarizationMetric {
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
            Message {
                role: Role::System,
                content: "你是一个专业的摘要评估专家，负责评估摘要的质量。".to_string(),
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
        let (faithfulness_score, coverage_score, total_score) = self.extract_scores_from_response(&response)?;
        
        // 创建分析信息
        let mut info = HashMap::new();
        info.insert("full_analysis".to_string(), serde_json::Value::String(response.clone()));
        info.insert("faithfulness_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(faithfulness_score).unwrap()));
        info.insert("coverage_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(coverage_score).unwrap()));
        
        Ok(MetricResult { score: total_score, info })
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
    async fn test_summarization_metric() {
        // 准备测试数据
        let original_text = "人工智能（AI）是计算机科学的一个分支，致力于创建能够模拟人类智能的系统。它包括机器学习、自然语言处理、计算机视觉等多个领域。近年来，深度学习技术的突破使AI取得了显著进展，在图像识别、语音识别、自动翻译等任务上达到或超过人类水平。";
        let good_summary = "AI是计算机科学分支，致力于模拟人类智能，包括机器学习等多个领域。深度学习突破使AI在图像识别等任务上达到或超过人类水平。";
        
        // 创建测试响应
        let test_response = r#"
忠实度分析: 摘要准确反映了原文内容，没有添加不存在的信息或错误解读原文。摘要提到AI是计算机科学分支，致力于模拟人类智能，包括机器学习等领域，以及深度学习带来的进展，这些都与原文一致。

忠实度分数: 0.95

信息覆盖度分析: 摘要包含了原文的关键信息，包括AI的定义、所属领域、组成部分以及近年来的发展。不过略微简化了一些细节，如具体应用领域（自然语言处理、计算机视觉）只概括为"多个领域"，但保留了核心例子（图像识别）。

信息覆盖度分数: 0.85

总分: 0.85
        "#;
        
        // 创建测试LLM提供者
        let llm = Box::new(TestLlmProvider::new(test_response.to_string()));
        
        // 创建并配置摘要评估指标
        let metric = SummarizationMetric::default().with_llm(llm);
        
        // 执行测试
        let result = metric.measure(original_text, good_summary).await.unwrap();
        
        // 验证结果
        assert_eq!(result.score, 0.85);
        assert!(result.info.contains_key("faithfulness_score"));
        assert!(result.info.contains_key("coverage_score"));
        assert!(result.info.contains_key("full_analysis"));
        
        if let Some(serde_json::Value::Number(faithfulness)) = result.info.get("faithfulness_score") {
            assert_eq!(faithfulness.as_f64().unwrap(), 0.95);
        } else {
            panic!("缺少忠实度分数");
        }
        
        if let Some(serde_json::Value::Number(coverage)) = result.info.get("coverage_score") {
            assert_eq!(coverage.as_f64().unwrap(), 0.85);
        } else {
            panic!("缺少信息覆盖度分数");
        }
    }
    
    #[test]
    fn test_extract_scores() {
        let metric = SummarizationMetric::default();
        
        // 测试正常响应
        let response = r#"
忠实度分析: 分析内容...
忠实度分数: 0.9
信息覆盖度分析: 分析内容...
信息覆盖度分数: 0.8
总分: 0.8
        "#;
        
        let (faithfulness, coverage, total) = metric.extract_scores_from_response(response).unwrap();
        assert_eq!(faithfulness, 0.9);
        assert_eq!(coverage, 0.8);
        assert_eq!(total, 0.8);
        
        // 测试英文响应
        let en_response = r#"
Faithfulness Analysis: analysis...
Faithfulness Score: 0.9
Coverage Analysis: analysis...
Coverage Score: 0.8
Total Score: 0.8
        "#;
        
        let (faithfulness, coverage, total) = metric.extract_scores_from_response(en_response).unwrap();
        assert_eq!(faithfulness, 0.9);
        assert_eq!(coverage, 0.8);
        assert_eq!(total, 0.8);
        
        // 测试无总分响应（应取较小值）
        let no_total_response = r#"
忠实度分析: 分析内容...
忠实度分数: 0.9
信息覆盖度分析: 分析内容...
信息覆盖度分数: 0.7
        "#;
        
        let (faithfulness, coverage, total) = metric.extract_scores_from_response(no_total_response).unwrap();
        assert_eq!(faithfulness, 0.9);
        assert_eq!(coverage, 0.7);
        assert_eq!(total, 0.7); // 应取较小值
    }
} 