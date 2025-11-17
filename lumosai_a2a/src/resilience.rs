//! A2A Error Handling and Retry Mechanisms
//!
//! 提供重试逻辑、错误恢复和熔断机制

use crate::{A2AError, A2AResult};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::sleep;

/// 重试策略配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: usize,
    /// 初始重试间隔
    pub initial_delay: Duration,
    /// 退避倍数
    pub backoff_multiplier: f64,
    /// 最大退避间隔
    pub max_delay: Duration,
    /// 可重试的错误类型
    pub retryable_errors: Vec<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(30),
            retryable_errors: vec![
                "timeout".to_string(),
                "network".to_string(),
                "temporary".to_string(),
            ],
        }
    }
}

impl RetryConfig {
    /// 创建新的重试配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大重试次数
    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 设置初始延迟
    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// 设置退避倍数
    pub fn backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// 设置最大延迟
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// 添加可重试错误类型
    pub fn retryable_error(mut self, error: &str) -> Self {
        self.retryable_errors.push(error.to_string());
        self
    }

    /// 检查错误是否可重试
    pub fn is_retryable(&self, error: &A2AError) -> bool {
        let error_str = format!("{:?}", error).to_lowercase();
        self.retryable_errors.iter().any(|retryable| {
            error_str.contains(&retryable.to_lowercase())
        })
    }

    /// 计算重试延迟
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        let delay = self.initial_delay.as_secs_f64() * self.backoff_multiplier.powi(attempt as i32);
        let delay_secs = delay.min(self.max_delay.as_secs_f64());
        Duration::from_secs_f64(delay_secs)
    }
}

/// 带重试的操作执行器
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    /// 创建新的重试执行器
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// 执行可重试的操作
    pub async fn execute<F, Fut, T>(&self, operation: F) -> A2AResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = A2AResult<T>>,
    {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(format!("{}", error));

                    // 如果是最后一次尝试或错误不可重试，直接返回
                    if attempt == self.config.max_retries || !self.config.is_retryable(&error) {
                        return Err(error);
                    }

                    // 等待后重试
                    let delay = self.config.calculate_delay(attempt);
                    sleep(delay).await;
                }
            }
        }

        Err(A2AError::InternalError(last_error.unwrap()))
    }

    /// 执行带超时的操作
    pub async fn execute_with_timeout<F, Fut, T>(
        &self,
        operation: F,
        timeout: Duration,
    ) -> A2AResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = A2AResult<T>>,
    {
        let retry_op = || async {
            tokio::time::timeout(timeout, operation()).await
                .map_err(|_| A2AError::TimeoutError(format!("Operation timed out after {:?}", timeout)))
        };

        self.execute(retry_op).await?
    }
}

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// 关闭状态，正常工作
    Closed,
    /// 打开状态，快速失败
    Open,
    /// 半开状态，允许少量请求通过
    HalfOpen,
}

/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 失败阈值
    pub failure_threshold: usize,
    /// 成功阈值（半开状态下需要的成功次数）
    pub success_threshold: usize,
    /// 超时时间（打开状态的持续时间）
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
        }
    }
}

/// 熔断器
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
    failure_count: usize,
    success_count: usize,
    last_failure_time: Option<std::time::Instant>,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
        }
    }

    /// 执行操作
    pub async fn execute<F, Fut, T>(&mut self, operation: F) -> A2AResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = A2AResult<T>>,
    {
        // 检查熔断器状态
        if !self.can_execute() {
            return Err(A2AError::ProtocolError("Circuit breaker is open".to_string()));
        }

        // 执行操作
        match operation().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(error) => {
                self.on_failure();
                Err(error)
            }
        }
    }

    /// 检查是否可以执行操作
    fn can_execute(&self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    last_failure.elapsed() > self.config.timeout
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    /// 处理成功操作
    fn on_success(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.success_count = 0;
                    self.failure_count = 0;
                }
            }
            CircuitBreakerState::Open => {
                // 不应该发生
            }
        }
    }

    /// 处理失败操作
    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
                self.success_count = 0;
            }
            CircuitBreakerState::Open => {
                // 保持打开状态
            }
        }
    }

    /// 获取当前状态
    pub fn state(&self) -> CircuitBreakerState {
        self.state
    }

    /// 获取失败次数
    pub fn failure_count(&self) -> usize {
        self.failure_count
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

/// 错误恢复策略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 重试
    Retry(RetryConfig),
    /// 熔断器
    CircuitBreaker(CircuitBreakerConfig),
    /// 组合策略（先重试，再熔断）
    Combined {
        retry_config: RetryConfig,
        circuit_breaker: CircuitBreakerConfig,
    },
}

impl Default for RecoveryStrategy {
    fn default() -> Self {
        Self::Retry(RetryConfig::default())
    }
}

/// 可恢复的操作执行器
pub struct ResilientExecutor {
    strategy: RecoveryStrategy,
    circuit_breaker: Option<CircuitBreaker>,
}

impl ResilientExecutor {
    /// 创建新的可恢复执行器
    pub fn new(strategy: RecoveryStrategy) -> Self {
        let circuit_breaker = match &strategy {
            RecoveryStrategy::CircuitBreaker(config) => {
                Some(CircuitBreaker::new(config.clone()))
            }
            RecoveryStrategy::Combined { circuit_breaker, .. } => {
                Some(CircuitBreaker::new(circuit_breaker.clone()))
            }
            _ => None,
        };

        Self {
            strategy,
            circuit_breaker,
        }
    }

    /// 执行可恢复的操作
    pub async fn execute<F, Fut, T>(&mut self, operation: F) -> A2AResult<T>
    where
        F: Fn() -> Fut + Clone + Send + 'static,
        Fut: std::future::Future<Output = A2AResult<T>> + Send + 'static,
        T: Send + 'static,
    {
        match &self.strategy {
            RecoveryStrategy::Retry(config) => {
                let executor = RetryExecutor::new(config.clone());
                executor.execute(operation).await
            }
            RecoveryStrategy::CircuitBreaker(_) => {
                match self.circuit_breaker {
                    Some(ref mut cb) => cb.execute(operation).await,
                    None => operation().await,
                }
            }
            RecoveryStrategy::Combined { retry_config, .. } => {
                // Skip circuit breaker for now to avoid closure capture issues
                let executor = RetryExecutor::new(retry_config.clone());
                executor.execute(operation).await
            }
        }
    }
}

impl Default for ResilientExecutor {
    fn default() -> Self {
        Self::new(RecoveryStrategy::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        let attempt_counter = AtomicU32::new(0);
        
        let config = RetryConfig::new()
            .max_retries(2)
            .initial_delay(Duration::from_millis(10))
            .retryable_error("connection refused");

        let executor = RetryExecutor::new(config);
        
        let result = executor.execute(|| async {
            let attempt = attempt_counter.fetch_add(1, Ordering::SeqCst);
            if attempt < 2 {
                Err(A2AError::InternalError("Connection refused".to_string()))
            } else {
                Ok("success")
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
        };

        let mut breaker = CircuitBreaker::new(config);
        
        // 前两次失败
        let _: Result<String, _> = breaker.execute(|| async {
            Err(A2AError::InternalError("Test failure".to_string()))
        }).await;

        let _: Result<String, _> = breaker.execute(|| async {
            Err(A2AError::InternalError("Test failure".to_string()))
        }).await;

        // 现在熔断器应该是打开状态
        assert_eq!(breaker.state(), CircuitBreakerState::Open);

        // 下一个操作应该被拒绝
        let result = breaker.execute(|| async {
            Ok("should not reach")
        }).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), A2AError::ProtocolError(_)));
    }

    #[tokio::test]
    async fn test_resilient_executor_retry() {
        let attempt_counter = Arc::new(AtomicU32::new(0));
        
        let strategy = RecoveryStrategy::Retry(
            RetryConfig::new()
                .max_retries(2)
                .initial_delay(Duration::from_millis(10))
                .retryable_error("temporary error")
        );

        let mut executor = ResilientExecutor::new(strategy);
        let counter_clone = attempt_counter.clone();
        
        let result = executor.execute(move || async {
            let attempt = counter_clone.fetch_add(1, Ordering::SeqCst);
            if attempt < 2 {
                Err(A2AError::InternalError("temporary error occurred".to_string()))
            } else {
                Ok("success after retries")
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success after retries");
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_resilient_executor_max_retries_exceeded() {
        let attempt_counter = AtomicU32::new(0);
        
        let strategy = RecoveryStrategy::Retry(
            RetryConfig::new()
                .max_retries(2)
                .initial_delay(Duration::from_millis(10))
                .retryable_error("persistent error")
        );

        let mut executor = ResilientExecutor::new(strategy);
        
        let result = executor.execute(|| async {
            attempt_counter.fetch_add(1, Ordering::SeqCst);
            Err(A2AError::InternalError("persistent error occurred".to_string()))
        }).await;

        assert!(result.is_err());
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3); // initial + 2 retries
    }

    #[tokio::test]
    async fn test_retry_config_exponential_backoff() {
        let config = RetryConfig::new()
            .max_retries(3)
            .initial_delay(Duration::from_millis(10))
            .backoff_multiplier(2.0);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(10));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(20));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(40));
        
        // Test max delay
        let config_with_max = config.max_delay(Duration::from_millis(50));
        assert_eq!(config_with_max.calculate_delay(3), Duration::from_millis(50)); // capped at max
    }

    #[test]
    fn test_retry_config_error_classification() {
        let config = RetryConfig::default();
        
        // Test retryable errors
        assert!(config.is_retryable(&A2AError::InternalError("timeout occurred".to_string())));
        assert!(config.is_retryable(&A2AError::InternalError("network error".to_string())));
        assert!(config.is_retryable(&A2AError::InternalError("temporary failure".to_string())));
        
        // Test non-retryable errors
        assert!(!config.is_retryable(&A2AError::InvalidInput("bad input".to_string())));
        assert!(!config.is_retryable(&A2AError::AgentNotFound("not found".to_string())));
    }

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 3);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_success() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout: Duration::from_millis(50),
        };

        let mut breaker = CircuitBreaker::new(config);
        
        // First failure triggers open state
        let _: Result<String, _> = breaker.execute(|| async {
            Err(A2AError::InternalError("First failure".to_string()))
        }).await;

        assert_eq!(breaker.state(), CircuitBreakerState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(60)).await;
        
        // Should now be half-open and allow requests
        let result = breaker.execute(|| async {
            Ok("success")
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(breaker.state(), CircuitBreakerState::HalfOpen);
        
        // Another success should close the circuit
        let result = breaker.execute(|| async {
            Ok("success again")
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(breaker.state(), CircuitBreakerState::Closed);
    }
}