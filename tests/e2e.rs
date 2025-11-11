//! LumosAI E2E 测试套件
//!
//! 端到端测试，验证整个系统的功能
//!
//! 运行方式:
//! ```bash
//! cargo test --test e2e
//! ```

// 包含测试模块
#[path = "e2e/framework.rs"]
mod framework;

#[path = "e2e/agent_tests.rs"]
mod agent_tests;

#[path = "e2e/auth_tests.rs"]
mod auth_tests;

#[path = "e2e/integration_tests.rs"]
mod integration_tests;

