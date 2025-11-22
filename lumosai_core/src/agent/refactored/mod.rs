//! BasicAgent 重构模块
//!
//! 这个模块包含了 BasicAgent 重构后的组件，将原来的 BasicAgent 拆分为多个专门的组件：
//!
//! - `core::AgentCore` - Agent 核心配置和 LLM 提供者
//! - `executor::AgentExecutor` - Agent 执行器，包含工具和内存
//! - `generator::AgentGenerator` - Agent 生成器，包含工具解析器和内存解析器
//!
//! # 设计目标
//!
//! 1. **降低复杂度**: 将 2300+ 行的 BasicAgent 拆分为多个小模块
//! 2. **提高可测试性**: 每个组件可以独立测试
//! 3. **提高可维护性**: 每个组件职责单一，易于理解和修改
//! 4. **保持兼容性**: 重构后的 API 保持与 BasicAgent 兼容

pub mod core;
pub mod executor;
pub mod generator;

pub use core::AgentCore;
pub use executor::AgentExecutor;
pub use generator::AgentGenerator;

