use async_trait::async_trait;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::error::Error;
use super::types::{Step, StepContext, RetryConfig};

/// 步骤的配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    /// 步骤ID
    pub id: String,
    /// 步骤描述
    pub description: String,
    /// 重试配置
    pub retry_config: Option<RetryConfig>,
}

/// 基本步骤实现
pub struct BasicStep {
    /// 步骤配置
    config: StepConfig,
    /// 步骤执行函数
    execute_fn: Arc<dyn Fn(StepContext) -> futures::future::BoxFuture<'static, Result<serde_json::Value, Error>> + Send + Sync>,
}

impl BasicStep {
    /// 创建新的步骤
    pub fn new<F, Fut>(
        id: String,
        description: String,
        execute_fn: F,
        retry_config: Option<RetryConfig>,
    ) -> Self
    where
        F: Fn(StepContext) -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = Result<serde_json::Value, Error>> + Send + 'static,
    {
        let execute_fn = Arc::new(move |ctx: StepContext| {
            let future = execute_fn(ctx);
            Box::pin(future) as futures::future::BoxFuture<'static, _>
        });

        Self {
            config: StepConfig {
                id,
                description,
                retry_config,
            },
            execute_fn,
        }
    }
    
    /// 创建一个简单的步骤
    pub fn create_simple<F>(id: String, description: String, f: F) -> Self
    where
        F: Fn(serde_json::Value) -> Result<serde_json::Value, Error> + Send + Sync + 'static,
    {
        Self::new(
            id,
            description,
            move |ctx: StepContext| {
                let input_data = ctx.input_data.clone();
                let f = f.clone();
                async move { f(input_data) }
            },
            None,
        )
    }
}

#[async_trait]
impl Step for BasicStep {
    fn id(&self) -> &str {
        &self.config.id
    }
    
    fn description(&self) -> &str {
        &self.config.description
    }
    
    async fn execute(&self, context: StepContext) -> Result<serde_json::Value, Error> {
        (self.execute_fn)(context).await
    }
    
    fn retry_config(&self) -> Option<RetryConfig> {
        self.config.retry_config.clone()
    }
}

/// 步骤构建器
pub struct StepBuilder {
    id: String,
    description: String,
    retry_config: Option<RetryConfig>,
}

impl StepBuilder {
    /// 创建新的步骤构建器
    pub fn new(id: String) -> Self {
        Self {
            id,
            description: String::from(""),
            retry_config: None,
        }
    }
    
    /// 设置步骤描述
    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
    
    /// 设置重试配置
    pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = Some(retry_config);
        self
    }
    
    /// 构建步骤
    pub fn build<F, Fut>(self, execute_fn: F) -> BasicStep
    where
        F: Fn(StepContext) -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = Result<serde_json::Value, Error>> + Send + 'static,
    {
        BasicStep::new(
            self.id,
            self.description,
            execute_fn,
            self.retry_config,
        )
    }
} 