//! End-to-End (E2E) Test Framework
//!
//! This module provides a comprehensive E2E testing framework for LumosAI,
//! validating the entire system's functionality from user perspective.
//!
//! # Test Categories
//!
//! - **Agent Tests**: Basic agent creation and conversation
//! - **Tool Tests**: Agent with tool integration
//! - **RAG Tests**: Agent with RAG system
//! - **Multi-Agent Tests**: Multi-agent collaboration
//! - **Workflow Tests**: Workflow orchestration
//! - **Auth Tests**: Authentication and authorization
//! - **Error Tests**: Error handling and recovery
//! - **Concurrency Tests**: Concurrent operations
//! - **Performance Tests**: Performance benchmarks
//! - **Integration Tests**: Full system integration

pub mod test_context;
pub mod test_helpers;
pub mod test_scenarios;

pub use test_context::E2ETestContext;
pub use test_helpers::*;
pub use test_scenarios::*;

use lumosai_core::prelude::*;
use std::time::Duration;

/// E2E test configuration
#[derive(Debug, Clone)]
pub struct E2ETestConfig {
    /// Test timeout duration
    pub timeout: Duration,
    /// Enable verbose logging
    pub verbose: bool,
    /// Use real LLM providers (requires API keys)
    pub use_real_llm: bool,
    /// Maximum retries for flaky tests
    pub max_retries: usize,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            verbose: false,
            use_real_llm: false,
            max_retries: 3,
        }
    }
}

/// E2E test result
#[derive(Debug)]
pub struct E2ETestResult {
    /// Test name
    pub name: String,
    /// Test passed
    pub passed: bool,
    /// Execution duration
    pub duration: Duration,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Test output
    pub output: Option<String>,
}

impl E2ETestResult {
    /// Create a successful test result
    pub fn success(name: String, duration: Duration, output: Option<String>) -> Self {
        Self {
            name,
            passed: true,
            duration,
            error: None,
            output,
        }
    }

    /// Create a failed test result
    pub fn failure(name: String, duration: Duration, error: String) -> Self {
        Self {
            name,
            passed: false,
            duration,
            error: Some(error),
            output: None,
        }
    }
}

/// E2E test runner
pub struct E2ETestRunner {
    config: E2ETestConfig,
    results: Vec<E2ETestResult>,
}

impl E2ETestRunner {
    /// Create a new test runner
    pub fn new(config: E2ETestConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    /// Run a test with timeout and retry
    pub async fn run_test<F, Fut>(&mut self, name: &str, test_fn: F) -> Result<()>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let start = std::time::Instant::now();
        let mut last_error = None;

        for attempt in 0..self.config.max_retries {
            if self.config.verbose {
                println!("🧪 Running test '{}' (attempt {}/{})", name, attempt + 1, self.config.max_retries);
            }

            match tokio::time::timeout(self.config.timeout, test_fn()).await {
                Ok(Ok(())) => {
                    let duration = start.elapsed();
                    self.results.push(E2ETestResult::success(
                        name.to_string(),
                        duration,
                        None,
                    ));
                    if self.config.verbose {
                        println!("✅ Test '{}' passed in {:?}", name, duration);
                    }
                    return Ok(());
                }
                Ok(Err(e)) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries - 1 {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
                Err(_) => {
                    last_error = Some(Error::Timeout(format!(
                        "Test '{}' timed out after {:?}",
                        name, self.config.timeout
                    )));
                    break;
                }
            }
        }

        let duration = start.elapsed();
        let error_msg = last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "Unknown error".to_string());

        self.results.push(E2ETestResult::failure(
            name.to_string(),
            duration,
            error_msg.clone(),
        ));

        if self.config.verbose {
            println!("❌ Test '{}' failed: {}", name, error_msg);
        }

        Err(Error::TestFailed(error_msg))
    }

    /// Get test results
    pub fn results(&self) -> &[E2ETestResult] {
        &self.results
    }

    /// Print test summary
    pub fn print_summary(&self) {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;

        println!("\n{}", "=".repeat(60));
        println!("📊 E2E Test Summary");
        println!("{}", "=".repeat(60));
        println!("Total:  {}", total);
        println!("Passed: {} ✅", passed);
        println!("Failed: {} ❌", failed);
        println!("{}", "=".repeat(60));

        if failed > 0 {
            println!("\n❌ Failed Tests:");
            for result in self.results.iter().filter(|r| !r.passed) {
                println!("  - {}: {}", result.name, result.error.as_ref().unwrap());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_e2e_runner_success() {
        let mut runner = E2ETestRunner::new(E2ETestConfig::default());

        let result = runner
            .run_test("test_success", || async { Ok(()) })
            .await;

        assert!(result.is_ok());
        assert_eq!(runner.results().len(), 1);
        assert!(runner.results()[0].passed);
    }

    #[tokio::test]
    async fn test_e2e_runner_failure() {
        let mut runner = E2ETestRunner::new(E2ETestConfig {
            max_retries: 1,
            ..Default::default()
        });

        let result = runner
            .run_test("test_failure", || async {
                Err(Error::InvalidInput("test error".to_string()))
            })
            .await;

        assert!(result.is_err());
        assert_eq!(runner.results().len(), 1);
        assert!(!runner.results()[0].passed);
    }

    #[tokio::test]
    async fn test_e2e_runner_timeout() {
        let mut runner = E2ETestRunner::new(E2ETestConfig {
            timeout: Duration::from_millis(100),
            max_retries: 1,
            ..Default::default()
        });

        let result = runner
            .run_test("test_timeout", || async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok(())
            })
            .await;

        assert!(result.is_err());
        assert_eq!(runner.results().len(), 1);
        assert!(!runner.results()[0].passed);
    }
}

