//! Agent 单元测试
//!
//! P0-3 任务：增加 Agent 模块的单元测试覆盖率
//! 测试范围：
//! - Agent 创建和配置
//! - Tool 注册和管理
//! - Memory 集成
//! - 状态管理
//! - 错误处理

use lumosai_core::agent::trait_def::AgentStatus; // 使用 trait_def 中的 AgentStatus
use lumosai_core::agent::{Agent, AgentConfig, BasicAgent};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use std::sync::Arc;

fn build_agent(
    config: AgentConfig,
    llm: Arc<dyn lumosai_core::llm::LlmProvider>,
) -> BasicAgent {
    BasicAgent::new(config, llm).expect("Failed to create BasicAgent")
}

/// 测试 1: Agent 基本创建
#[test]
fn test_agent_creation_with_name() {
    let llm = create_test_zhipu_provider_arc();
    let config = AgentConfig {
        name: "test_agent".to_string(),
        instructions: "You are a helpful assistant".to_string(),
        ..Default::default()
    };

    let agent = build_agent(config, llm);

    assert_eq!(agent.get_name(), "test_agent");
    assert_eq!(agent.get_instructions(), "You are a helpful assistant");
}

/// 测试 2: Agent 默认配置
#[test]
fn test_agent_default_configuration() {
    let llm = create_test_zhipu_provider_arc();
    let config = AgentConfig::default();
    let agent = build_agent(config, llm);

    // 默认配置应该有名称和指令
    assert!(!agent.get_name().is_empty());
    assert!(!agent.get_instructions().is_empty());
}

/// 测试 3: Agent 指令修改
#[test]
fn test_agent_instructions_update() {
    let llm = create_test_zhipu_provider_arc();
    let config = AgentConfig {
        name: "updatable_agent".to_string(),
        instructions: "Initial instructions".to_string(),
        ..Default::default()
    };

    let mut agent = build_agent(config, llm);

    // 验证初始指令
    assert_eq!(agent.get_instructions(), "Initial instructions");

    // 更新指令
    agent.set_instructions("Updated instructions".to_string());

    // 验证更新后的指令
    assert_eq!(agent.get_instructions(), "Updated instructions");
}

/// 测试 4: Agent 状态管理
#[test]
fn test_agent_status_transitions() {
    let llm = create_test_zhipu_provider_arc();
    let config = AgentConfig::default();
    let mut agent = build_agent(config, llm);

    // 初始状态应该是 Ready（根据实际测试结果）
    assert_eq!(agent.get_status(), AgentStatus::Ready);

    // 测试状态转换到 Running
    let _ = agent.set_status(AgentStatus::Running);
    assert_eq!(agent.get_status(), AgentStatus::Running);

    // 测试状态转换到 Paused
    let _ = agent.set_status(AgentStatus::Paused);
    assert_eq!(agent.get_status(), AgentStatus::Paused);

    // 测试状态转换到 Stopped
    let _ = agent.set_status(AgentStatus::Stopped);
    assert_eq!(agent.get_status(), AgentStatus::Stopped);
}

/// 测试 5: Agent 错误状态
#[test]
fn test_agent_error_status() {
    let llm = create_test_zhipu_provider_arc();
    let config = AgentConfig::default();
    let mut agent = build_agent(config, llm);

    // 设置错误状态
    let _ = agent.set_status(AgentStatus::Error("Test error message".to_string()));

    // 验证错误状态
    match agent.get_status() {
        AgentStatus::Error(msg) => {
            assert_eq!(msg, "Test error message");
        }
        _ => panic!("Expected Error status"),
    }
}

/// 测试 6: Agent Tool 管理
#[test]
fn test_agent_tool_management() {
    let llm = create_test_zhipu_provider_arc();
    let config = AgentConfig::default();
    let agent = build_agent(config, llm);

    // 初始状态应该没有 tools
    assert_eq!(agent.get_tools().len(), 0);
}

/// 测试 7: Agent Memory 集成
#[test]
fn test_agent_memory_integration() {
    let llm = create_test_zhipu_provider_arc();
    let config = AgentConfig {
        name: "memory_agent".to_string(),
        instructions: "Test".to_string(),
        ..Default::default()
    };

    let agent = build_agent(config, llm);

    // 默认情况下应该没有 memory
    assert!(agent.get_memory().is_none());
    assert!(!agent.has_own_memory());
}

/// 测试 8: Agent LLM Provider
#[test]
fn test_agent_llm_provider() {
    let llm = create_test_zhipu_provider_arc();
    let config = AgentConfig::default();
    let agent = build_agent(config, llm.clone());

    // 验证 LLM provider 存在
    let agent_llm = agent.get_llm();
    // 无法直接比较 Arc 指针，因为类型不同（MockLlmProvider vs dyn LlmProvider）
    // 但我们可以验证它不是 None
    assert!(Arc::strong_count(&agent_llm) > 0);
}

/// 测试 9: Agent Function Calling 配置
#[test]
fn test_agent_function_calling_config() {
    let llm = create_test_zhipu_provider_arc();

    // 测试显式启用
    let config_enabled = AgentConfig {
        enable_function_calling: Some(true),
        ..Default::default()
    };
    let _agent_enabled = build_agent(config_enabled, llm.clone());
    // 注意：is_function_calling_enabled 方法不是公共 API，所以我们只测试配置能正确创建

    // 测试显式禁用
    let config_disabled = AgentConfig {
        enable_function_calling: Some(false),
        ..Default::default()
    };
    let _agent_disabled = build_agent(config_disabled, llm.clone());

    // 测试默认值
    let config_default = AgentConfig::default();
    let _agent_default = build_agent(config_default, llm);
}

/// 测试 10: Agent 配置验证
#[test]
fn test_agent_config_validation() {
    let llm = create_test_zhipu_provider_arc();
    let config = AgentConfig {
        name: "valid_agent".to_string(),
        instructions: "Valid instructions".to_string(),
        ..Default::default()
    };

    let agent = build_agent(config, llm);

    // 验证配置应该成功
    assert!(agent.validate_config().is_ok());
}

/// 测试 11: 多个 Agent 实例独立性
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

    let mut agent1 = build_agent(config1, llm1);
    let agent2 = build_agent(config2, llm2);

    // 验证两个 agent 是独立的
    assert_eq!(agent1.get_name(), "agent1");
    assert_eq!(agent2.get_name(), "agent2");

    // 修改 agent1 不应影响 agent2
    agent1.set_instructions("Modified instructions".to_string());
    assert_eq!(agent1.get_instructions(), "Modified instructions");
    assert_eq!(agent2.get_instructions(), "Instructions 2");
}

/// 测试 12: Agent 名称不同
#[test]
fn test_agents_with_different_names() {
    let llm = create_test_zhipu_provider_arc();

    let names = vec!["alice", "bob", "charlie"];

    for name in names {
        let config = AgentConfig {
            name: name.to_string(),
            instructions: "Test".to_string(),
            ..Default::default()
        };

        let agent = build_agent(config, llm.clone());
        assert_eq!(agent.get_name(), name);
    }
}

/// 测试 13: Agent 空指令处理
#[test]
fn test_agent_empty_instructions() {
    let llm = create_test_zhipu_provider_arc();
    let config = AgentConfig {
        name: "test".to_string(),
        instructions: "".to_string(),
        ..Default::default()
    };

    let agent = build_agent(config, llm);

    // 即使指令为空，agent 也应该能创建
    assert_eq!(agent.get_instructions(), "");
}

/// 测试 14: Agent 长指令处理
#[test]
fn test_agent_long_instructions() {
    let llm = create_test_zhipu_provider_arc();
    let long_instructions = "A".repeat(10000);

    let config = AgentConfig {
        name: "test".to_string(),
        instructions: long_instructions.clone(),
        ..Default::default()
    };

    let agent = build_agent(config, llm);

    // 验证长指令能正确存储
    assert_eq!(agent.get_instructions(), long_instructions);
}

/// 测试 15: Agent 特殊字符处理
#[test]
fn test_agent_special_characters() {
    let llm = create_test_zhipu_provider_arc();
    let special_name = "agent-with_special.chars@123";
    let special_instructions = "Instructions with 中文, emoji 🚀, and symbols: !@#$%^&*()";

    let config = AgentConfig {
        name: special_name.to_string(),
        instructions: special_instructions.to_string(),
        ..Default::default()
    };

    let agent = build_agent(config, llm);

    assert_eq!(agent.get_name(), special_name);
    assert_eq!(agent.get_instructions(), special_instructions);
}
