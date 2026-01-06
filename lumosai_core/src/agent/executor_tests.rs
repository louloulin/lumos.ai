//! BasicAgent 单元测试
//!
//! 测试 BasicAgent 的核心功能，包括：
//! - Agent 创建和配置
//! - Tool 注册和管理
//! - Memory 集成
//! - 状态管理
//! - 错误处理

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::agent::config::AgentConfig;
    use crate::agent::executor::BasicAgent;
    use crate::agent::trait_def::Agent;
    use crate::llm::test_helpers::create_test_zhipu_provider_arc;
    use crate::agent::types::{AgentGenerateOptions, AgentStatus};
    use crate::base::Base;
    use crate::compat::Component;
    use crate::llm::{LlmOptions};
    use crate::llm::test_helpers::{create_test_zhipu_provider, create_test_zhipu_provider_arc};
    use crate::tool::Tool;
    use serde_json::{json, Value};
    use std::sync::Arc;

    /// 测试 Agent 基本创建
    #[test]
    fn test_basic_agent_creation() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "test_agent".to_string(),
            instructions: "You are a test assistant".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm);

        assert_eq!(agent.get_name(), "test_agent");
        assert_eq!(agent.get_instructions(), "You are a test assistant");
        assert_eq!(agent.get_status(), AgentStatus::Idle);
    }

    /// 测试 Agent 名称和指令设置
    #[test]
    fn test_agent_name_and_instructions() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "assistant".to_string(),
            instructions: "Initial instructions".to_string(),
            ..Default::default()
        };

        let mut agent = BasicAgent::new(config, llm);

        // 测试初始值
        assert_eq!(agent.get_name(), "assistant");
        assert_eq!(agent.get_instructions(), "Initial instructions");

        // 测试修改指令
        agent.set_instructions("Updated instructions".to_string());
        assert_eq!(agent.get_instructions(), "Updated instructions");
    }

    /// 测试 Base trait 实现
    #[test]
    fn test_agent_base_trait() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "base_test".to_string(),
            instructions: "Test".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm);

        // 测试 Base trait 方法
        assert_eq!(agent.name(), Some("base_test"));
        // Component 不实现 PartialEq，所以我们只测试它存在
        let _ = agent.component();
        // Logger 返回 Arc，我们只测试它存在
        let _ = agent.logger();
        assert!(agent.telemetry().is_none());
    }

    /// 测试 Agent 状态管理
    #[test]
    fn test_agent_status() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig::default();
        let mut agent = BasicAgent::new(config, llm);

        // 初始状态应该是 Idle
        assert_eq!(agent.get_status(), AgentStatus::Idle);

        // 测试状态切换
        agent.set_status(AgentStatus::Processing);
        assert_eq!(agent.get_status(), AgentStatus::Processing);

        agent.set_status(AgentStatus::Paused);
        assert_eq!(agent.get_status(), AgentStatus::Paused);

        agent.set_status(AgentStatus::Idle);
        assert_eq!(agent.get_status(), AgentStatus::Idle);
    }

    /// 测试 Agent 错误状态
    #[test]
    fn test_agent_error_status() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig::default();
        let mut agent = BasicAgent::new(config, llm);

        agent.set_status(AgentStatus::Error("Test error".to_string()));
        
        match agent.get_status() {
            AgentStatus::Error(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected Error status"),
        }
    }

    /// 测试 Tool 注册
    #[test]
    fn test_tool_registration() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig::default();
        let agent = BasicAgent::new(config, llm);

        // 初始状态应该没有 tools
        assert_eq!(agent.get_tools().len(), 0);
        assert!(!agent.has_tool("test_tool"));
    }

    /// 测试 Memory 集成
    #[test]
    fn test_memory_integration() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "memory_test".to_string(),
            instructions: "Test".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm);

        // 默认情况下应该没有 memory
        assert!(agent.get_memory().is_none());
        assert!(!agent.has_own_memory());
    }

    /// 测试 LLM provider 获取
    #[test]
    fn test_llm_provider() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig::default();
        let agent = BasicAgent::new(config, llm.clone());

        let agent_llm = agent.get_llm();
        assert!(Arc::ptr_eq(&llm, &agent_llm));
    }

    /// 测试 Agent 配置验证
    #[test]
    fn test_agent_config_validation() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "valid_agent".to_string(),
            instructions: "Valid instructions".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm);
        assert!(agent.validate_config().is_ok());
    }

    /// 测试 Agent 默认配置
    #[test]
    fn test_agent_default_config() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig::default();
        let agent = BasicAgent::new(config, llm);

        assert!(!agent.get_name().is_empty());
        assert!(!agent.get_instructions().is_empty());
        assert_eq!(agent.get_status(), AgentStatus::Idle);
    }

    /// 测试 Agent function calling 配置
    #[test]
    fn test_function_calling_config() {
        let llm = create_test_zhipu_provider_arc();
        
        // 测试启用 function calling
        let config_enabled = AgentConfig {
            enable_function_calling: Some(true),
            ..Default::default()
        };
        let agent_enabled = BasicAgent::new(config_enabled, llm.clone());
        assert!(agent_enabled.is_function_calling_enabled());

        // 测试禁用 function calling
        let config_disabled = AgentConfig {
            enable_function_calling: Some(false),
            ..Default::default()
        };
        let agent_disabled = BasicAgent::new(config_disabled, llm.clone());
        assert!(!agent_disabled.is_function_calling_enabled());

        // 测试默认值（应该启用）
        let config_default = AgentConfig::default();
        let agent_default = BasicAgent::new(config_default, llm);
        assert!(agent_default.is_function_calling_enabled());
    }

    /// 测试 Agent 克隆（如果实现了 Clone）
    #[test]
    fn test_agent_properties() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "prop_test".to_string(),
            instructions: "Test instructions".to_string(),
            enable_function_calling: Some(true),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm);

        // 验证所有属性都正确设置
        assert_eq!(agent.get_name(), "prop_test");
        assert_eq!(agent.get_instructions(), "Test instructions");
        assert!(agent.is_function_calling_enabled());
        assert_eq!(agent.get_status(), AgentStatus::Idle);
        assert!(agent.get_memory().is_none());
        assert_eq!(agent.get_tools().len(), 0);
    }

    /// 测试 Agent 多个实例独立性
    #[test]
    fn test_multiple_agents_independence() {
        let llm1 = create_test_zhipu_provider_arc();
        let llm2 = create_test_zhipu_provider_arc();

        let config1 = AgentConfig {
            name: "agent1".to_string(),
            instructions: "Instructions 1".to_string(),
            ..Default::default()
        };

        let config2 = AgentConfig {
            name: "agent2".to_string(),
            instructions: "Instructions 2".to_string(),
            ..Default::default()
        };

        let mut agent1 = BasicAgent::new(config1, llm1);
        let agent2 = BasicAgent::new(config2, llm2);

        // 验证两个 agent 是独立的
        assert_eq!(agent1.get_name(), "agent1");
        assert_eq!(agent2.get_name(), "agent2");

        // 修改 agent1 不应影响 agent2
        agent1.set_instructions("Modified instructions".to_string());
        assert_eq!(agent1.get_instructions(), "Modified instructions");
        assert_eq!(agent2.get_instructions(), "Instructions 2");
    }
}

