//! 日志记录模块
//!
//! 提供统一的日志记录接口和实现

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

/// 日志记录器 trait
#[async_trait]
pub trait Logger: Send + Sync {
    /// 记录日志
    async fn log(&self, level: LogLevel, message: &str, metadata: Option<Value>);

    /// 记录 trace 级别日志
    async fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message, None).await;
    }

    /// 记录 debug 级别日志
    async fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message, None).await;
    }

    /// 记录 info 级别日志
    async fn info(&self, message: &str) {
        self.log(LogLevel::Info, message, None).await;
    }

    /// 记录 warn 级别日志
    async fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message, None).await;
    }

    /// 记录 error 级别日志
    async fn error(&self, message: &str) {
        self.log(LogLevel::Error, message, None).await;
    }
}

/// 默认的控制台日志记录器
#[derive(Debug, Default)]
pub struct ConsoleLogger {
    min_level: LogLevel,
}

impl ConsoleLogger {
    /// 创建新的控制台日志记录器
    pub fn new(min_level: LogLevel) -> Self {
        Self { min_level }
    }
}

#[async_trait]
impl Logger for ConsoleLogger {
    async fn log(&self, level: LogLevel, message: &str, metadata: Option<Value>) {
        if level as u8 >= self.min_level as u8 {
            let level_str = match level {
                LogLevel::Trace => "TRACE",
                LogLevel::Debug => "DEBUG",
                LogLevel::Info => "INFO",
                LogLevel::Warn => "WARN",
                LogLevel::Error => "ERROR",
            };

            if let Some(meta) = metadata {
                println!("[{level_str}] {message} | {meta}");
            } else {
                println!("[{level_str}] {message}");
            }
        }
    }
}

/// 空日志记录器（用于测试）
#[derive(Debug, Default)]
pub struct NoopLogger;

#[async_trait]
impl Logger for NoopLogger {
    async fn log(&self, _level: LogLevel, _message: &str, _metadata: Option<Value>) {
        // 不做任何操作
    }
}

/// 创建默认的日志记录器
pub fn default_logger() -> Arc<dyn Logger> {
    Arc::new(ConsoleLogger::new(LogLevel::Info))
}
