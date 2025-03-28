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
        let test_llm = TestLlmProvider::new(
            "分析：查询和回复非常相关，回复准确地提供了关于Rust的垃圾回收机制的信息。\n分数：0.92".to_string()
        );
            
        // 创建LLM评估指标
        let metric = LlmEvaluator::default()
            .with_llm(Box::new(test_llm));
            
        // 测试评估
        let result = metric.measure(
            "Rust中有垃圾回收吗？", 
            "Rust不使用传统的垃圾回收机制。它使用所有权系统和借用检查器在编译时管理内存，无需运行时垃圾回收器。"
        ).await;
        
        assert!(result.is_ok());
        let metric_result = result.unwrap();
        assert_eq!(metric_result.score, 0.92);
    }
} 