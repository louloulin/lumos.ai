//! 统一的 Agent 错误处理和恢复系统
//!
//! 对标 Mastra 的错误处理设计，提供：
//! - 统一的错误类型分类
//! - 智能重试策略
//! - 错误恢复机制
//! - 错误上下文和追踪

use crate::error::{Error, Result};
use crate::agent::types::RuntimeContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Agent 错误类型分类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentErrorType {
    /// LLM 相关错误
    LlmError,
    /// 工具执行错误
    ToolError,
    /// 内存错误
    MemoryError,
    /// 配置错误
    ConfigError,
    /// 网络错误
    NetworkError,
    /// 超时错误
    TimeoutError,
    /// 验证错误
    ValidationError,
    /// 不可重试的错误（如权限错误）
    NonRetryableError,
    /// 未知错误
    Unknown,
}

impl AgentErrorType {
    /// 从 Error 推断错误类型
    pub fn from_error(error: &Error) -> Self {
        match error {
            Error::Llm(_) | Error::LlmProvider(_) => AgentErrorType::LlmError,
            Error::Tool(_) => AgentErrorType::ToolError,
            Error::Memory(_) => AgentErrorType::MemoryError,
            Error::Config(_) | Error::ConfigError { .. } | Error::Configuration(_) => {
                AgentErrorType::ConfigError
            }
            Error::Http(_) | Error::Network(_) | Error::NetworkError { .. } => {
                AgentErrorType::NetworkError
            }
            Error::Timeout(_) => AgentErrorType::TimeoutError,
            Error::ValidationError(_) | Error::Validation { .. } => AgentErrorType::ValidationError,
            Error::AccessDenied(_) | Error::Authentication(_) => AgentErrorType::NonRetryableError,
            _ => AgentErrorType::Unknown,
        }
    }

    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AgentErrorType::LlmError
                | AgentErrorType::NetworkError
                | AgentErrorType::TimeoutError
                | AgentErrorType::ToolError
        )
    }
}

/// 回退策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// 固定延迟
    Fixed { delay_ms: u64 },
    /// 线性回退
    Linear { initial_delay_ms: u64, increment_ms: u64 },
    /// 指数回退
    Exponential { initial_delay_ms: u64, multiplier: f64 },
    /// 指数回退（带抖动）
    ExponentialJitter { initial_delay_ms: u64, multiplier: f64, max_delay_ms: u64 },
}

impl BackoffStrategy {
    /// 计算延迟时间
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        match self {
            BackoffStrategy::Fixed { delay_ms } => Duration::from_millis(*delay_ms),
            BackoffStrategy::Linear {
                initial_delay_ms,
                increment_ms,
            } => Duration::from_millis(initial_delay_ms + (attempt as u64 * increment_ms)),
            BackoffStrategy::Exponential {
                initial_delay_ms,
                multiplier,
            } => {
                let delay = (*initial_delay_ms as f64 * multiplier.powi(attempt as i32)) as u64;
                Duration::from_millis(delay)
            }
            BackoffStrategy::ExponentialJitter {
                initial_delay_ms,
                multiplier,
                max_delay_ms,
            } => {
                let base_delay = (*initial_delay_ms as f64 * multiplier.powi(attempt as i32)) as u64;
                let delay = base_delay.min(*max_delay_ms);
                // 添加随机抖动（简化实现，使用固定抖动）
                let jitter = delay / 10;
                Duration::from_millis(delay + jitter)
            }
        }
    }
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        BackoffStrategy::Exponential {
            initial_delay_ms: 100,
            multiplier: 2.0,
        }
    }
}

/// 重试策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStrategy {
    /// 最大重试次数
    pub max_retries: u32,
    /// 回退策略
    pub backoff: BackoffStrategy,
    /// 可重试的错误类型
    pub retryable_errors: Vec<AgentErrorType>,
    /// 最大延迟时间（毫秒）
    pub max_delay_ms: Option<u64>,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff: BackoffStrategy::default(),
            retryable_errors: vec![
                AgentErrorType::LlmError,
                AgentErrorType::NetworkError,
                AgentErrorType::TimeoutError,
            ],
            max_delay_ms: Some(5000),
        }
    }
}

/// 错误恢复动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// 重试操作
    Retry,
    /// 使用降级方案
    Fallback,
    /// 跳过当前步骤
    Skip,
    /// 中止执行
    Abort,
    /// 等待后重试
    WaitAndRetry { delay_ms: u64 },
}

/// 错误恢复器 Trait
#[async_trait::async_trait]
pub trait ErrorRecovery: Send + Sync {
    /// 尝试恢复错误
    async fn recover(
        &self,
        error: &Error,
        error_type: &AgentErrorType,
        context: &RuntimeContext,
        attempt: u32,
    ) -> Result<RecoveryAction>;
}

/// 默认错误恢复器
pub struct DefaultErrorRecovery {
    strategy: RetryStrategy,
}

impl DefaultErrorRecovery {
    pub fn new(strategy: RetryStrategy) -> Self {
        Self { strategy }
    }

    pub fn default() -> Self {
        Self {
            strategy: RetryStrategy::default(),
        }
    }
}

#[async_trait::async_trait]
impl ErrorRecovery for DefaultErrorRecovery {
    async fn recover(
        &self,
        _error: &Error,
        error_type: &AgentErrorType,
        _context: &RuntimeContext,
        attempt: u32,
    ) -> Result<RecoveryAction> {
        // 检查是否可重试
        if !error_type.is_retryable() {
            return Ok(RecoveryAction::Abort);
        }

        // 检查是否在可重试的错误类型列表中
        if !self.strategy.retryable_errors.contains(error_type) {
            return Ok(RecoveryAction::Abort);
        }

        // 检查是否超过最大重试次数
        if attempt >= self.strategy.max_retries {
            return Ok(RecoveryAction::Abort);
        }

        // 计算延迟时间
        let delay = self.strategy.backoff.calculate_delay(attempt);
        let delay_ms = delay.as_millis() as u64;

        // 应用最大延迟限制
        let final_delay = if let Some(max_delay) = self.strategy.max_delay_ms {
            delay_ms.min(max_delay)
        } else {
            delay_ms
        };

        Ok(RecoveryAction::WaitAndRetry {
            delay_ms: final_delay,
        })
    }
}

/// 带重试的操作执行器
pub struct RetryExecutor {
    strategy: RetryStrategy,
    recovery: Box<dyn ErrorRecovery>,
}

impl RetryExecutor {
    pub fn new(strategy: RetryStrategy, recovery: Box<dyn ErrorRecovery>) -> Self {
        Self { strategy, recovery }
    }

    pub fn with_default_recovery(strategy: RetryStrategy) -> Self {
        Self {
            strategy,
            recovery: Box::new(DefaultErrorRecovery::default()),
        }
    }

    pub fn default() -> Self {
        Self {
            strategy: RetryStrategy::default(),
            recovery: Box::new(DefaultErrorRecovery::default()),
        }
    }

    /// 执行带重试的操作
    pub async fn execute<F, Fut, T>(
        &self,
        mut operation: F,
        context: &RuntimeContext,
    ) -> Result<T>
    where
        F: FnMut() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        tracing::debug!("Operation succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    let error_type = AgentErrorType::from_error(&error);

                    // 尝试恢复
                    let recovery_action = self.recovery.recover(&error, &error_type, context, attempt).await?;
                    
                    match recovery_action {
                        RecoveryAction::Retry => {
                            attempt += 1;
                            if attempt > self.strategy.max_retries {
                                return Err(error);
                            }
                            // 使用默认延迟
                            let delay = self.strategy.backoff.calculate_delay(attempt);
                            tracing::debug!(
                                "Retrying operation after {:?} (attempt {}/{})",
                                delay,
                                attempt,
                                self.strategy.max_retries
                            );
                            tokio::time::sleep(delay).await;
                        }
                        RecoveryAction::WaitAndRetry { delay_ms } => {
                            attempt += 1;
                            if attempt > self.strategy.max_retries {
                                return Err(error);
                            }
                            tracing::debug!(
                                "Retrying operation after {}ms (attempt {}/{})",
                                delay_ms,
                                attempt,
                                self.strategy.max_retries
                            );
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        }
                        RecoveryAction::Fallback => {
                            tracing::warn!("Fallback not implemented, aborting");
                            return Err(error);
                        }
                        RecoveryAction::Skip => {
                            tracing::warn!("Skipping operation due to error");
                            return Err(error);
                        }
                        RecoveryAction::Abort => {
                            tracing::error!("Aborting operation due to non-retryable error");
                            return Err(error);
                        }
                    }
                }
            }
        }
    }
}

/// 错误上下文信息
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// 错误类型
    pub error_type: AgentErrorType,
    /// 错误消息
    pub message: String,
    /// 发生时间
    pub timestamp: Instant,
    /// 重试次数
    pub retry_count: u32,
    /// 附加上下文信息
    pub metadata: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(error: &Error, retry_count: u32) -> Self {
        Self {
            error_type: AgentErrorType::from_error(error),
            message: error.to_string(),
            timestamp: Instant::now(),
            retry_count,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn test_error_type_classification() {
        let llm_error = Error::Llm("Test error".to_string());
        assert_eq!(
            AgentErrorType::from_error(&llm_error),
            AgentErrorType::LlmError
        );

        let tool_error = Error::Tool("Tool failed".to_string());
        assert_eq!(
            AgentErrorType::from_error(&tool_error),
            AgentErrorType::ToolError
        );

        let timeout_error = Error::Timeout("Operation timed out".to_string());
        assert_eq!(
            AgentErrorType::from_error(&timeout_error),
            AgentErrorType::TimeoutError
        );
    }

    #[test]
    fn test_retryable_errors() {
        assert!(AgentErrorType::LlmError.is_retryable());
        assert!(AgentErrorType::NetworkError.is_retryable());
        assert!(AgentErrorType::TimeoutError.is_retryable());
        assert!(!AgentErrorType::NonRetryableError.is_retryable());
        assert!(!AgentErrorType::ConfigError.is_retryable());
    }

    #[test]
    fn test_backoff_strategy() {
        let fixed = BackoffStrategy::Fixed { delay_ms: 100 };
        assert_eq!(fixed.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(fixed.calculate_delay(5), Duration::from_millis(100));

        let exponential = BackoffStrategy::Exponential {
            initial_delay_ms: 100,
            multiplier: 2.0,
        };
        assert_eq!(exponential.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(exponential.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(exponential.calculate_delay(2), Duration::from_millis(400));
    }

    #[tokio::test]
    async fn test_retry_executor_success() {
        let executor = RetryExecutor::default();
        let context = RuntimeContext::default();

        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let call_count_clone = call_count.clone();
        let result: Result<String> = executor
            .execute(
                move || {
                    let count = call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    async move {
                        if count < 1 {
                            Err(Error::Network("Temporary network error".to_string()))
                        } else {
                            Ok("Success".to_string())
                        }
                    }
                },
                &context,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_executor_max_retries() {
        let executor = RetryExecutor::default();
        let context = RuntimeContext::default();

        let result: Result<String> = executor
            .execute(
                || async move { Err(Error::Network("Persistent error".to_string())) },
                &context,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_executor_non_retryable() {
        let executor = RetryExecutor::default();
        let context = RuntimeContext::default();

        let result: Result<String> = executor
            .execute(
                || async move { Err(Error::AccessDenied("Permission denied".to_string())) },
                &context,
            )
            .await;

        assert!(result.is_err());
    }
}

