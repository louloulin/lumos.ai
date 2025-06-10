// LumosAI Test Suite
// Comprehensive testing framework for the LumosAI project

// Test configuration and utilities
pub mod test_config;

// Unit tests
pub mod unit;

// Integration tests
pub mod integration;

// Performance tests
pub mod performance;

// Coverage utilities
pub mod coverage;

// Test automation
pub mod automation;

// Re-export commonly used test utilities
pub use test_config::*;

// Test suite runner
use automation::test_runner::TestRunner;

/// Main test suite entry point
pub async fn run_test_suite() -> bool {
    let mut runner = TestRunner::new();
    runner.run_all().await
}

/// Run specific test category
pub async fn run_test_category(category: &str) -> bool {
    let mut runner = TestRunner::new();
    
    if let Some(config) = runner.config.get(category).cloned() {
        let result = runner.run_suite(category, &config).await;
        result.passed
    } else {
        eprintln!("Unknown test category: {}", category);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_framework_initialization() {
        init_test_env();
        
        // Test that the test framework itself works
        let config = TestConfig::default();
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.mock_llm);
        assert!(config.use_memory_storage);
    }
    
    #[tokio::test]
    async fn test_utilities_work() {
        init_test_env();
        
        // Test that test utilities function correctly
        let docs = TestUtils::generate_test_documents(5);
        assert_eq!(docs.len(), 5);
        
        for (i, doc) in docs.iter().enumerate() {
            assert!(doc.contains(&format!("Test document {}", i)));
        }
    }
    
    #[tokio::test]
    async fn test_performance_utils() {
        init_test_env();
        
        // Test performance measurement utilities
        let (result, duration) = PerformanceTestUtils::measure_time(|| async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            "test result"
        }).await;
        
        assert_eq!(result, "test result");
        assert!(duration >= std::time::Duration::from_millis(10));
        assert!(duration < std::time::Duration::from_millis(100));
    }
}
