// Test configuration and utilities for LumosAI framework
use std::sync::Once;
use tokio::runtime::Runtime;
use lumosai::prelude::*;

static INIT: Once = Once::new();

/// Initialize test environment
pub fn init_test_env() {
    INIT.call_once(|| {
        // Initialize logging for tests
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    });
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub mock_llm: bool,
    pub use_memory_storage: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            retry_attempts: 3,
            mock_llm: true,
            use_memory_storage: true,
        }
    }
}

/// Mock LLM provider for testing
pub struct MockLlmProvider {
    pub responses: Vec<String>,
    pub current_index: std::sync::atomic::AtomicUsize,
}

impl MockLlmProvider {
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            current_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn with_single_response(response: &str) -> Self {
        Self::new(vec![response.to_string()])
    }
}

/// Test utilities
pub struct TestUtils;

impl TestUtils {
    /// Create a test agent with mock LLM
    pub async fn create_test_agent(name: &str) -> Result<Agent> {
        Agent::builder()
            .name(name)
            .model("mock-model")
            .system_prompt("You are a test assistant")
            .build()
            .await
    }

    /// Create test vector storage
    pub async fn create_test_vector_storage() -> Result<VectorStorage> {
        VectorStorage::memory().await
    }

    /// Create test RAG system
    pub async fn create_test_rag() -> Result<RagSystem> {
        let storage = Self::create_test_vector_storage().await?;
        RagSystem::builder()
            .storage(storage)
            .embedding_provider("mock")
            .build()
            .await
    }

    /// Generate test documents
    pub fn generate_test_documents(count: usize) -> Vec<String> {
        (0..count)
            .map(|i| format!("Test document {} with content about topic {}", i, i % 5))
            .collect()
    }

    /// Assert response contains expected content
    pub fn assert_response_contains(response: &str, expected: &str) {
        assert!(
            response.contains(expected),
            "Response '{}' does not contain expected content '{}'",
            response,
            expected
        );
    }

    /// Assert response is not empty
    pub fn assert_response_not_empty(response: &str) {
        assert!(!response.is_empty(), "Response should not be empty");
    }

    /// Create test session
    pub async fn create_test_session(agent_name: &str) -> Result<Session> {
        Session::create(agent_name, Some("test-user")).await
    }
}

/// Test macros
#[macro_export]
macro_rules! async_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            use crate::test_config::{init_test_env, TestConfig};
            
            init_test_env();
            let _config = TestConfig::default();
            
            $test_body.await;
        }
    };
}

#[macro_export]
macro_rules! timeout_test {
    ($test_name:ident, $timeout_secs:expr, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            use crate::test_config::{init_test_env, TestConfig};
            use tokio::time::{timeout, Duration};
            
            init_test_env();
            let _config = TestConfig::default();
            
            let result = timeout(Duration::from_secs($timeout_secs), $test_body).await;
            assert!(result.is_ok(), "Test timed out after {} seconds", $timeout_secs);
        }
    };
}

/// Performance test utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    /// Measure execution time
    pub async fn measure_time<F, Fut, T>(f: F) -> (T, std::time::Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = std::time::Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Assert execution time is within bounds
    pub fn assert_execution_time_within(
        duration: std::time::Duration,
        max_duration: std::time::Duration,
    ) {
        assert!(
            duration <= max_duration,
            "Execution took {:?}, expected <= {:?}",
            duration,
            max_duration
        );
    }

    /// Benchmark function execution
    pub async fn benchmark<F, Fut, T>(
        name: &str,
        iterations: usize,
        f: F,
    ) -> Vec<std::time::Duration>
    where
        F: Fn() -> Fut + Clone,
        Fut: std::future::Future<Output = T>,
    {
        let mut durations = Vec::new();
        
        for i in 0..iterations {
            let (_, duration) = Self::measure_time(f.clone()).await;
            durations.push(duration);
            
            if i % 10 == 0 {
                println!("Benchmark '{}': iteration {}/{}", name, i + 1, iterations);
            }
        }
        
        let avg_duration = durations.iter().sum::<std::time::Duration>() / iterations as u32;
        let min_duration = durations.iter().min().unwrap();
        let max_duration = durations.iter().max().unwrap();
        
        println!(
            "Benchmark '{}' results: avg={:?}, min={:?}, max={:?}",
            name, avg_duration, min_duration, max_duration
        );
        
        durations
    }
}

/// Integration test utilities
pub struct IntegrationTestUtils;

impl IntegrationTestUtils {
    /// Setup test environment for integration tests
    pub async fn setup_integration_env() -> Result<IntegrationTestEnv> {
        let agent = TestUtils::create_test_agent("integration-agent").await?;
        let storage = TestUtils::create_test_vector_storage().await?;
        let rag = TestUtils::create_test_rag().await?;
        let session = TestUtils::create_test_session("integration-agent").await?;
        
        Ok(IntegrationTestEnv {
            agent,
            storage,
            rag,
            session,
        })
    }
}

/// Integration test environment
pub struct IntegrationTestEnv {
    pub agent: Agent,
    pub storage: VectorStorage,
    pub rag: RagSystem,
    pub session: Session,
}

/// Test result assertions
pub struct TestAssertions;

impl TestAssertions {
    /// Assert agent response is valid
    pub fn assert_valid_agent_response(response: &str) {
        assert!(!response.is_empty(), "Agent response should not be empty");
        assert!(response.len() > 5, "Agent response should be meaningful");
    }

    /// Assert vector search results
    pub fn assert_valid_search_results(results: &[SearchResult], min_count: usize) {
        assert!(
            results.len() >= min_count,
            "Expected at least {} search results, got {}",
            min_count,
            results.len()
        );
        
        for result in results {
            assert!(result.score >= 0.0 && result.score <= 1.0, "Score should be between 0 and 1");
            assert!(!result.content.is_empty(), "Result content should not be empty");
        }
    }

    /// Assert session state
    pub fn assert_valid_session_state(session: &Session) {
        assert!(!session.id().is_empty(), "Session ID should not be empty");
        assert!(!session.agent_name().is_empty(), "Agent name should not be empty");
    }
}
