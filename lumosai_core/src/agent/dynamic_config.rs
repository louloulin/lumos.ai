//! 动态配置系统 - 对标 Mastra 的 DynamicArgument
//!
//! 实现运行时上下文感知的配置系统，支持静态值和动态闭包配置
//! 
//! # 设计目标
//! - 对标 Mastra 的 DynamicArgument<T> 类型
//! - 支持运行时上下文感知的配置解析
//! - 提供类型安全的动态配置 API
//! - 保持向后兼容性

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::llm::{LlmProvider, Message};
use crate::tool::Tool;

/// 增强的运行时上下文 - 对标 Mastra RuntimeContext
///
/// 提供类型安全的上下文变量管理和动态配置解析能力
#[derive(Debug, Clone)]
pub struct EnhancedRuntimeContext {
    /// 基础上下文变量
    pub variables: HashMap<String, Value>,
    /// 请求特定的元数据
    pub metadata: HashMap<String, String>,
    /// 执行时间戳
    pub timestamp: std::time::SystemTime,
    /// 会话ID
    pub session_id: String,
    /// 用户ID（可选）
    pub user_id: Option<String>,
    /// 用户角色
    pub user_role: Option<String>,
    /// 应用领域
    pub domain: Option<String>,
    /// 复杂度级别
    pub complexity: ComplexityLevel,
    /// 当前消息历史
    pub messages: Vec<Message>,
    /// 可用工具列表
    pub available_tools: Vec<String>,
}

/// 复杂度级别枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    /// 简单任务
    Simple,
    /// 复杂任务
    Complex,
    /// 专家级任务
    Expert,
}

/// 动态参数类型 - 对标 Mastra 的 DynamicArgument<T>
///
/// 支持静态值或运行时闭包配置
pub enum DynamicArgument<T> {
    /// 静态值
    Static(T),
    /// 动态闭包 - 基于运行时上下文计算值
    Dynamic(Box<dyn Fn(&EnhancedRuntimeContext) -> Pin<Box<dyn Future<Output = Result<T>> + Send>> + Send + Sync>),
}

impl<T> std::fmt::Debug for DynamicArgument<T> 
where 
    T: std::fmt::Debug 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DynamicArgument::Static(value) => f.debug_tuple("Static").field(value).finish(),
            DynamicArgument::Dynamic(_) => f.debug_tuple("Dynamic").field(&"<closure>").finish(),
        }
    }
}

impl<T: Clone> Clone for DynamicArgument<T> {
    fn clone(&self) -> Self {
        match self {
            DynamicArgument::Static(value) => DynamicArgument::Static(value.clone()),
            DynamicArgument::Dynamic(_) => {
                // 对于闭包，我们无法直接克隆，这里返回一个错误闭包
                DynamicArgument::Dynamic(Box::new(|_| {
                    Box::pin(async {
                        Err(Error::Internal("Cannot clone dynamic closure".to_string()))
                    })
                }))
            }
        }
    }
}

/// 动态配置解析器
pub struct DynamicConfigResolver;

impl DynamicConfigResolver {
    /// 解析动态参数
    pub async fn resolve<T>(&self, arg: &DynamicArgument<T>, context: &EnhancedRuntimeContext) -> Result<T>
    where
        T: Clone,
    {
        match arg {
            DynamicArgument::Static(value) => Ok(value.clone()),
            DynamicArgument::Dynamic(closure) => {
                let future = closure(context);
                future.await
            }
        }
    }

    /// 解析动态字符串参数
    pub async fn resolve_string(&self, arg: &DynamicArgument<String>, context: &EnhancedRuntimeContext) -> Result<String> {
        self.resolve(arg, context).await
    }

    /// 解析动态模型参数
    pub async fn resolve_model(&self, arg: &DynamicArgument<String>, context: &EnhancedRuntimeContext) -> Result<String> {
        self.resolve(arg, context).await
    }

    /// 解析动态工具列表
    pub async fn resolve_tools(&self, arg: &DynamicArgument<Vec<String>>, context: &EnhancedRuntimeContext) -> Result<Vec<String>> {
        self.resolve(arg, context).await
    }
}

impl EnhancedRuntimeContext {
    /// 创建新的增强运行时上下文
    pub fn new(session_id: String) -> Self {
        Self {
            variables: HashMap::new(),
            metadata: HashMap::new(),
            timestamp: std::time::SystemTime::now(),
            session_id,
            user_id: None,
            user_role: None,
            domain: None,
            complexity: ComplexityLevel::Simple,
            messages: Vec::new(),
            available_tools: Vec::new(),
        }
    }

    /// 设置用户ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// 设置用户角色
    pub fn with_user_role(mut self, role: String) -> Self {
        self.user_role = Some(role);
        self
    }

    /// 设置应用领域
    pub fn with_domain(mut self, domain: String) -> Self {
        self.domain = Some(domain);
        self
    }

    /// 设置复杂度级别
    pub fn with_complexity(mut self, complexity: ComplexityLevel) -> Self {
        self.complexity = complexity;
        self
    }

    /// 设置消息历史
    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    /// 设置可用工具
    pub fn with_available_tools(mut self, tools: Vec<String>) -> Self {
        self.available_tools = tools;
        self
    }

    /// 设置上下文变量
    pub fn set_variable(&mut self, key: String, value: Value) {
        self.variables.insert(key, value);
    }

    /// 获取上下文变量
    pub fn get_variable(&self, key: &str) -> Option<&Value> {
        self.variables.get(key)
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// 获取用户工具列表（基于用户角色和权限）
    pub fn get_user_tools(&self) -> Vec<String> {
        match self.user_role.as_deref() {
            Some("admin") => vec![
                "web_search".to_string(),
                "calculator".to_string(),
                "file_manager".to_string(),
                "code_executor".to_string(),
                "system_monitor".to_string(),
            ],
            Some("developer") => vec![
                "web_search".to_string(),
                "calculator".to_string(),
                "code_executor".to_string(),
                "file_manager".to_string(),
            ],
            Some("analyst") => vec![
                "web_search".to_string(),
                "calculator".to_string(),
                "data_analyzer".to_string(),
            ],
            _ => vec![
                "web_search".to_string(),
                "calculator".to_string(),
            ],
        }
    }

    /// 获取内存配置（基于复杂度和用户类型）
    pub fn get_memory_config(&self) -> HashMap<String, Value> {
        let mut config = HashMap::new();
        
        match self.complexity {
            ComplexityLevel::Simple => {
                config.insert("type".to_string(), Value::String("basic".to_string()));
                config.insert("capacity".to_string(), Value::Number(serde_json::Number::from(100)));
            },
            ComplexityLevel::Complex => {
                config.insert("type".to_string(), Value::String("semantic".to_string()));
                config.insert("capacity".to_string(), Value::Number(serde_json::Number::from(500)));
            },
            ComplexityLevel::Expert => {
                config.insert("type".to_string(), Value::String("hybrid".to_string()));
                config.insert("capacity".to_string(), Value::Number(serde_json::Number::from(1000)));
                config.insert("enable_semantic".to_string(), Value::Bool(true));
            },
        }
        
        config
    }
}

/// 便利函数：创建静态动态参数
pub fn static_arg<T>(value: T) -> DynamicArgument<T> {
    DynamicArgument::Static(value)
}

/// 便利函数：创建动态参数
pub fn dynamic_arg<T, F, Fut>(f: F) -> DynamicArgument<T>
where
    F: Fn(&EnhancedRuntimeContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: 'static,
{
    DynamicArgument::Dynamic(Box::new(move |ctx| Box::pin(f(ctx))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Role, Message};

    #[tokio::test]
    async fn test_enhanced_runtime_context() {
        let mut context = EnhancedRuntimeContext::new("session_123".to_string())
            .with_user_role("developer".to_string())
            .with_domain("ai_development".to_string())
            .with_complexity(ComplexityLevel::Complex);

        context.set_variable("project".to_string(), Value::String("lumosai".to_string()));
        
        assert_eq!(context.session_id, "session_123");
        assert_eq!(context.user_role, Some("developer".to_string()));
        assert_eq!(context.complexity, ComplexityLevel::Complex);
        assert_eq!(context.get_variable("project"), Some(&Value::String("lumosai".to_string())));
    }

    #[tokio::test]
    async fn test_dynamic_argument_static() {
        let context = EnhancedRuntimeContext::new("test".to_string());
        let resolver = DynamicConfigResolver;
        
        let static_arg = static_arg("test_value".to_string());
        let result = resolver.resolve_string(&static_arg, &context).await.unwrap();
        
        assert_eq!(result, "test_value");
    }

    #[tokio::test]
    async fn test_dynamic_argument_dynamic() {
        let context = EnhancedRuntimeContext::new("test".to_string())
            .with_user_role("admin".to_string())
            .with_complexity(ComplexityLevel::Expert);
        
        let resolver = DynamicConfigResolver;
        
        let dynamic_instructions = dynamic_arg(|ctx: &EnhancedRuntimeContext| async move {
            Ok(format!("You are a {} assistant for {}", 
                ctx.user_role.as_deref().unwrap_or("general"),
                ctx.domain.as_deref().unwrap_or("general tasks")
            ))
        });
        
        let result = resolver.resolve_string(&dynamic_instructions, &context).await.unwrap();
        assert_eq!(result, "You are a admin assistant for general tasks");
    }

    #[tokio::test]
    async fn test_context_based_model_selection() {
        let resolver = DynamicConfigResolver;
        
        // 简单任务使用轻量模型
        let simple_context = EnhancedRuntimeContext::new("test".to_string())
            .with_complexity(ComplexityLevel::Simple);
        
        let model_selector = dynamic_arg(|ctx: &EnhancedRuntimeContext| async move {
            Ok(match ctx.complexity {
                ComplexityLevel::Simple => "gpt-3.5-turbo".to_string(),
                ComplexityLevel::Complex => "gpt-4".to_string(),
                ComplexityLevel::Expert => "claude-3-opus".to_string(),
            })
        });
        
        let result = resolver.resolve_model(&model_selector, &simple_context).await.unwrap();
        assert_eq!(result, "gpt-3.5-turbo");
        
        // 专家级任务使用高级模型
        let expert_context = EnhancedRuntimeContext::new("test".to_string())
            .with_complexity(ComplexityLevel::Expert);
        
        let result = resolver.resolve_model(&model_selector, &expert_context).await.unwrap();
        assert_eq!(result, "claude-3-opus");
    }
}
