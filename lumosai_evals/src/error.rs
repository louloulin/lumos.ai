use thiserror::Error;

/// 评估框架中可能出现的错误
#[derive(Error, Debug)]
pub enum Error {
    /// 评估配置错误
    #[error("配置错误: {0}")]
    Configuration(String),
    
    /// 评估执行过程中的错误
    #[error("评估执行错误: {0}")]
    Execution(String),
    
    /// 指标计算错误
    #[error("指标计算错误: {0}")]
    MetricCalculation(String),
    
    /// LLM错误
    #[error("LLM错误: {0}")]
    Llm(#[from] lumosai_core::Error),
    
    /// 序列化/反序列化错误
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// I/O错误
    #[error("I/O错误: {0}")]
    Io(#[from] std::io::Error),
    
    /// 其他未分类错误
    #[error("其他错误: {0}")]
    Other(String),
}

/// 评估操作的结果类型
pub type Result<T> = std::result::Result<T, Error>; 