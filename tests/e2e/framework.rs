//! E2E 测试框架
//!
//! 提供端到端测试的基础设施和工具函数

use lumosai_core::agent::{AgentBuilder};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::llm::LlmProvider;
use lumosai_core::tool::Tool;
use lumosai_core::vector::MemoryVectorStorage;
use std::sync::Arc;

// 注意：lumosai_auth 需要在 Cargo.toml 中配置为 dev-dependency
// 如果没有配置，这些功能将无法使用
#[cfg(feature = "auth_tests")]
use lumosai_auth::{AuthService, User};

/// E2E 测试上下文
///
/// 包含所有测试需要的共享资源
pub struct E2ETestContext {
    pub llm: Arc<dyn LlmProvider>,
    #[cfg(feature = "auth_tests")]
    pub auth: lumosai_auth::AuthService,
    pub vector_storage: Arc<MemoryVectorStorage>,
}

impl E2ETestContext {
    /// 设置测试环境
    pub async fn setup() -> Result<Self, Box<dyn std::error::Error>> {
        // 初始化日志（可选）
        #[cfg(feature = "tracing")]
        {
            let _ = tracing_subscriber::fmt()
                .with_test_writer()
                .try_init();
        }

        // 创建测试 LLM 提供商
        let llm = create_test_zhipu_provider_arc();

        // 创建 Auth 服务（仅当启用 auth_tests feature 时）
        #[cfg(feature = "auth_tests")]
        let auth = lumosai_auth::AuthService::with_default_expiration("test-e2e-secret-key-32-bytes".to_string());

        // 创建向量存储
        let vector_storage = Arc::new(MemoryVectorStorage::new(384, None));

        Ok(Self {
            llm,
            #[cfg(feature = "auth_tests")]
            auth,
            vector_storage,
        })
    }

    /// 清理测试环境
    pub async fn teardown(self) -> Result<(), Box<dyn std::error::Error>> {
        // 清理资源（如果有需要）
        Ok(())
    }

    /// 创建测试用 Agent
    pub fn create_test_agent(
        &self,
        name: &str,
        instructions: &str,
    ) -> Result<lumosai_core::agent::BasicAgent, Box<dyn std::error::Error>> {
        Ok(AgentBuilder::new()
            .name(name)
            .instructions(instructions)
            .model(self.llm.clone())
            .build()?)
    }

    /// 创建带工具的测试 Agent
    pub fn create_agent_with_tools(
        &self,
        name: &str,
        instructions: &str,
        tools: Vec<Box<dyn Tool>>,
    ) -> Result<lumosai_core::agent::BasicAgent, Box<dyn std::error::Error>> {
        let mut builder = AgentBuilder::new()
            .name(name)
            .instructions(instructions)
            .model(self.llm.clone());

        for tool in tools {
            builder = builder.tool(tool);
        }

        Ok(builder.build()?)
    }

    /// 注册测试用户（仅当启用 auth_tests feature 时可用）
    #[cfg(feature = "auth_tests")]
    pub async fn register_test_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<lumosai_auth::User, Box<dyn std::error::Error>> {
        Ok(self.auth.register(email, password, None).await?)
    }

    /// 登录测试用户（仅当启用 auth_tests feature 时可用）
    #[cfg(feature = "auth_tests")]
    pub async fn login_test_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let token = self.auth.authenticate(email, password).await?;
        Ok(token.token)
    }
}

// E2E 测试辅助宏已移除，请直接使用 #[tokio::test] 编写测试

/// 测试断言辅助函数
pub struct E2EAssertions;

impl E2EAssertions {
    /// 断言响应不为空
    pub fn assert_non_empty_response(response: &str) {
        assert!(
            !response.is_empty(),
            "Response should not be empty, got: '{}'",
            response
        );
    }

    /// 断言响应包含关键词
    pub fn assert_contains(response: &str, keyword: &str) {
        assert!(
            response.to_lowercase().contains(&keyword.to_lowercase()),
            "Response '{}' should contain '{}'",
            response,
            keyword
        );
    }

    /// 断言响应时间合理（< 30秒）
    pub fn assert_reasonable_response_time(duration: std::time::Duration) {
        assert!(
            duration.as_secs() < 30,
            "Response time {:?} exceeded 30 seconds",
            duration
        );
    }
}




