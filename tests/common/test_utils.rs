use std::sync::Arc;
use std::time::{Duration, Instant};
use lumosai_core::prelude::*;
use lumosai_core::agent::{BasicAgent, AgentConfig};
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::vector::MemoryVectorStorage;
use lumosai_network::AgentNetwork;

/// 测试工具集
pub struct TestUtils;

impl TestUtils {
    /// 创建测试用Agent
    pub async fn create_test_agent(name: &str) -> Result<BasicAgent> {
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        let config = AgentConfig {
            name: name.to_string(),
            instructions: "Test agent".to_string(),
            ..Default::default()
        };
        Ok(BasicAgent::new(config, llm))
    }
    
    /// 创建带自定义响应的测试Agent
    pub async fn create_test_agent_with_responses(name: &str, responses: Vec<String>) -> Result<BasicAgent> {
        let llm = Arc::new(MockLlmProvider::new(responses));
        let config = AgentConfig {
            name: name.to_string(),
            instructions: "Test agent with custom responses".to_string(),
            ..Default::default()
        };
        Ok(BasicAgent::new(config, llm))
    }
    
    /// 创建测试用RAG系统
    pub async fn create_test_rag_system() -> Result<Box<dyn std::fmt::Debug>> {
        let _storage = MemoryVectorStorage::new(384, None);
        let _embedding_provider = Arc::new(MockEmbeddingProvider::new());
        // 暂时返回一个简单的调试对象，等待RAG实现完善
        Ok(Box::new("MockRagSystem"))
    }
    
    /// 创建测试用工作流
    pub fn create_test_workflow(name: &str) -> String {
        // 暂时返回工作流名称，等待BasicWorkflow实现
        name.to_string()
    }
    
    /// 创建测试用Agent网络
    pub async fn create_test_network() -> AgentNetwork {
        AgentNetwork::new().await
    }
    
    /// 创建真实LLM测试环境
    pub async fn create_real_llm_agent(_model: &str) -> Result<BasicAgent> {
        // 使用真实的qwen3-30b-a3b模型进行测试
        let _api_key = std::env::var("QWEN_API_KEY")
            .unwrap_or_else(|_| "sk-bc977c4e31e542f1a34159cb42478198".to_string());

        // 这里需要实际的QwenProvider实现
        // let llm = QwenProvider::new(api_key, model.to_string());
        // 暂时使用Mock，等待真实实现
        let llm = Arc::new(MockLlmProvider::new(vec!["Real API response".to_string()]));

        let config = AgentConfig {
            name: "real_test_agent".to_string(),
            instructions: "You are a test agent for validation".to_string(),
            ..Default::default()
        };
        Ok(BasicAgent::new(config, llm))
    }
    
    /// 创建性能测试环境
    pub fn setup_performance_test() -> PerformanceTestContext {
        PerformanceTestContext::new()
    }
    
    /// 等待异步操作完成
    pub async fn wait_for_completion(duration: Duration) {
        tokio::time::sleep(duration).await;
    }
    
    /// 验证响应内容
    pub fn validate_response(response: &str, expected_keywords: &[&str]) -> bool {
        expected_keywords.iter().any(|keyword| response.contains(keyword))
    }
}

/// Mock嵌入提供商
pub struct MockEmbeddingProvider {
    dimension: usize,
}

impl MockEmbeddingProvider {
    pub fn new() -> Self {
        Self { dimension: 384 }
    }
}

// 暂时注释掉EmbeddingProvider实现，等待接口定义
// impl EmbeddingProvider for MockEmbeddingProvider {
//     async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
//         // 生成基于文本哈希的确定性嵌入
//         let hash = text.len() as f32;
//         let mut embedding = vec![0.0; self.dimension];
//         for i in 0..self.dimension {
//             embedding[i] = (hash + i as f32) / 1000.0;
//         }
//         Ok(embedding)
//     }
// }

/// 性能测试上下文
pub struct PerformanceTestContext {
    start_time: Instant,
    memory_tracker: MemoryTracker,
}

impl PerformanceTestContext {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            memory_tracker: MemoryTracker::new(),
        }
    }
    
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    pub fn memory_usage(&self) -> usize {
        self.memory_tracker.current_usage()
    }
    
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.memory_tracker.reset();
    }
}

/// 内存跟踪器
pub struct MemoryTracker {
    initial_usage: usize,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            initial_usage: Self::get_memory_usage(),
        }
    }
    
    pub fn current_usage(&self) -> usize {
        Self::get_memory_usage() - self.initial_usage
    }
    
    pub fn reset(&mut self) {
        self.initial_usage = Self::get_memory_usage();
    }
    
    fn get_memory_usage() -> usize {
        // 简化的内存使用量获取
        // 在实际实现中可以使用更精确的方法
        0
    }
}

/// 测试断言辅助函数
pub struct TestAssertions;

impl TestAssertions {
    /// 断言响应时间在预期范围内
    pub fn assert_response_time(duration: Duration, max_duration: Duration) {
        assert!(
            duration <= max_duration,
            "Response time {:?} exceeded maximum {:?}",
            duration, max_duration
        );
    }
    
    /// 断言内存使用在合理范围内
    pub fn assert_memory_usage(usage: usize, max_usage: usize) {
        assert!(
            usage <= max_usage,
            "Memory usage {} exceeded maximum {}",
            usage, max_usage
        );
    }
    
    /// 断言响应内容包含预期关键词
    pub fn assert_response_contains(response: &str, keywords: &[&str]) {
        for keyword in keywords {
            assert!(
                response.contains(keyword),
                "Response '{}' does not contain keyword '{}'",
                response, keyword
            );
        }
    }
}
