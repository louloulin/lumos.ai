//! E2E 测试框架
//!
//! 提供端到端测试的基础设施和工具函数

use lumosai_auth::{AuthService, User};
use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::llm::LlmProvider;
use std::sync::Arc;

/// E2E 测试上下文
///
/// 包含所有测试需要的共享资源
pub struct E2ETestContext {
    pub llm: Arc<dyn LlmProvider>,
    pub auth: AuthService,
}

impl E2ETestContext {
    /// 设置测试环境
    pub async fn setup() -> Result<Self, Box<dyn std::error::Error>> {
        // 创建测试 LLM 提供商
        let llm = create_test_zhipu_provider_arc();

        // 创建 Auth 服务
        let auth = AuthService::with_default_expiration("test-e2e-secret-key-32-bytes".to_string());

        Ok(Self { llm, auth })
    }

    /// 清理测试环境
    pub async fn teardown(self) -> Result<(), Box<dyn std::error::Error>> {
        // 清理资源（如果有需要）
        Ok(())
    }

    /// 创建测试用 Agent
    pub fn create_test_agent(&self, name: &str, instructions: &str) -> Result<lumosai_core::agent::BasicAgent, Box<dyn std::error::Error>> {
        Ok(AgentBuilder::new()
            .name(name)
            .instructions(instructions)
            .model(self.llm.clone())
            .build()?)
    }

    /// 注册测试用户
    pub async fn register_test_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<User, Box<dyn std::error::Error>> {
        Ok(self.auth.register(email, password, None).await?)
    }

    /// 登录测试用户
    pub async fn login_test_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let token = self.auth.authenticate(email, password).await?;
        Ok(token.token)
    }
}

/// E2E 测试辅助宏
#[macro_export]
macro_rules! e2e_test {
    ($name:ident, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let ctx = E2ETestContext::setup().await.expect("Failed to setup context");
            
            let result = $body(ctx.clone()).await;
            
            ctx.teardown().await.expect("Failed to teardown");
            
            result.expect(&format!("E2E test {} failed", stringify!($name)));
        }
    };
}




