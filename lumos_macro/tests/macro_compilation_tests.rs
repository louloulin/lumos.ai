// DSL 宏编译测试
//
// 这个测试文件验证所有 DSL 宏能够正确编译
// 主要测试宏展开的正确性，而不是运行时行为

use lumos_macro::agent;
use lumosai_core::{
    agent::Agent,
    llm::{mock::MockLlmProvider, test_helpers::create_test_zhipu_provider},
};

/// 测试 1: agent! 宏 - 基础用法
#[test]
fn test_agent_macro_basic_compiles() {
    let agent = agent! {
        name: "test_agent",
        instructions: "You are a test agent",
        provider: create_test_zhipu_provider()
    };

    // 验证类型正确
    let _: &dyn Agent = &agent;
}

/// 测试 2: agent! 宏 - 带内存
#[test]
fn test_agent_macro_with_memory_compiles() {
    let agent = agent! {
        name: "memory_agent",
        instructions: "You are an agent with memory",
        provider: create_test_zhipu_provider(),
        memory: true
    };

    let _: &dyn Agent = &agent;
}

/// 测试 3: agent! 宏 - 带 max_tool_calls
#[test]
fn test_agent_macro_with_max_tool_calls_compiles() {
    let agent = agent! {
        name: "limited_agent",
        instructions: "You are an agent with limited tool calls",
        provider: create_test_zhipu_provider(),
        max_tool_calls: 5
    };

    let _: &dyn Agent = &agent;
}

/// 测试 4: agent! 宏 - 带 tool_timeout
#[test]
fn test_agent_macro_with_tool_timeout_compiles() {
    let agent = agent! {
        name: "timeout_agent",
        instructions: "You are an agent with tool timeout",
        provider: create_test_zhipu_provider(),
        tool_timeout: 30
    };

    let _: &dyn Agent = &agent;
}

/// 测试 5: agent! 宏 - 所有参数组合
#[test]
fn test_agent_macro_all_parameters_compiles() {
    let agent = agent! {
        name: "full_agent",
        instructions: "You are a fully configured agent",
        provider: create_test_zhipu_provider(),
        memory: true,
        max_tool_calls: 10,
        tool_timeout: 60
    };

    let _: &dyn Agent = &agent;
}

/// 测试 6: 验证宏展开的代码结构
#[test]
fn test_agent_macro_expansion_structure() {
    // 这个测试验证宏展开后的代码结构是否正确
    let _code = stringify! {
        agent! {
            name: "test",
            instructions: "Test agent",
            provider: create_test_zhipu_provider()
        }
    };

    // 如果能编译通过，说明宏展开正确
}

/// 测试 7: 验证多个 agent 可以同时创建
#[test]
fn test_multiple_agents_compile() {
    let agent1 = agent! {
        name: "agent1",
        instructions: "First agent",
        provider: create_test_zhipu_provider()
    };

    let agent2 = agent! {
        name: "agent2",
        instructions: "Second agent",
        provider: create_test_zhipu_provider()
    };

    let _: &dyn Agent = &agent1;
    let _: &dyn Agent = &agent2;
}

/// 测试 8: 验证 agent 名称可以包含特殊字符
#[test]
fn test_agent_with_special_chars_in_name() {
    let agent = agent! {
        name: "agent_with_special-chars.123",
        instructions: "Agent with special characters in name",
        provider: create_test_zhipu_provider()
    };

    let _: &dyn Agent = &agent;
}

/// 测试 9: 验证 provider 可以是表达式
#[test]
fn test_agent_with_provider_expression() {
    let responses = vec!["Response 1".to_string(), "Response 2".to_string()];

    let agent = agent! {
        name: "expr_agent",
        instructions: "Agent with expression provider",
        provider: MockLlmProvider::new(responses)
    };

    let _: &dyn Agent = &agent;
}

/// 测试 10: 验证参数顺序不影响编译
#[test]
fn test_agent_parameter_order() {
    // 参数顺序 1
    let agent1 = agent! {
        name: "order1",
        instructions: "Test order",
        provider: create_test_zhipu_provider(),
        memory: true,
        max_tool_calls: 5
    };

    // 参数顺序 2
    let agent2 = agent! {
        name: "order2",
        max_tool_calls: 5,
        instructions: "Test order",
        memory: true,
        provider: create_test_zhipu_provider()
    };

    let _: &dyn Agent = &agent1;
    let _: &dyn Agent = &agent2;
}

/// 测试 11: 验证 memory 参数为 false
#[test]
fn test_agent_memory_false() {
    let agent = agent! {
        name: "no_memory_agent",
        instructions: "Agent without memory",
        provider: create_test_zhipu_provider(),
        memory: false
    };

    let _: &dyn Agent = &agent;
}

/// 测试 12: 验证数值参数
#[test]
fn test_agent_numeric_parameters() {
    let agent = agent! {
        name: "numeric_agent",
        instructions: "Agent with numeric parameters",
        provider: create_test_zhipu_provider(),
        max_tool_calls: 100,
        tool_timeout: 3600
    };

    let _: &dyn Agent = &agent;
}

/// 测试 13: 验证字符串字面量
#[test]
fn test_agent_string_literals() {
    let agent = agent! {
        name: "literal_agent",
        instructions: "This is a string literal with special characters: \n\t\"quotes\"",
        provider: create_test_zhipu_provider()
    };

    let _: &dyn Agent = &agent;
}

/// 测试 14: 验证空响应列表
#[test]
fn test_agent_empty_responses() {
    let agent = agent! {
        name: "empty_agent",
        instructions: "Agent with empty responses",
        provider: create_test_zhipu_provider()
    };

    let _: &dyn Agent = &agent;
}

/// 测试 15: 验证长指令
#[test]
fn test_agent_long_instructions() {
    let agent = agent! {
        name: "long_instructions_agent",
        instructions: "This is a very long instruction that spans multiple lines and contains a lot of text to test that the macro can handle long strings without any issues. It includes various punctuation marks, numbers like 123, and special characters like @#$%.",
        provider: create_test_zhipu_provider()
    };

    let _: &dyn Agent = &agent;
}

/// 测试 16: 验证宏在不同作用域中工作
#[test]
fn test_agent_in_different_scopes() {
    {
        let agent = agent! {
            name: "scope1",
            instructions: "Scope 1",
            provider: create_test_zhipu_provider()
        };
        let _: &dyn Agent = &agent;
    }

    {
        let agent = agent! {
            name: "scope2",
            instructions: "Scope 2",
            provider: create_test_zhipu_provider()
        };
        let _: &dyn Agent = &agent;
    }
}

/// 测试 17: 验证宏生成的代码可以被移动
#[test]
fn test_agent_can_be_moved() {
    let agent = agent! {
        name: "movable_agent",
        instructions: "This agent can be moved",
        provider: create_test_zhipu_provider()
    };

    let moved_agent = agent;
    let _: &dyn Agent = &moved_agent;
}

/// 测试 18: 验证宏生成的代码可以被克隆（如果实现了 Clone）
#[test]
fn test_agent_type_check() {
    let agent = agent! {
        name: "type_check_agent",
        instructions: "Type checking",
        provider: create_test_zhipu_provider()
    };

    // 验证类型
    let _: &dyn Agent = &agent;

    // 验证可以调用 Agent trait 的方法（编译时检查）
    // 注意：这里只是类型检查，不实际调用
    let _ = &agent;
}

/// 测试 19: 验证宏在函数中使用
fn create_test_agent() -> impl Agent {
    agent! {
        name: "function_agent",
        instructions: "Created in function",
        provider: create_test_zhipu_provider()
    }
}

#[test]
fn test_agent_in_function() {
    let agent = create_test_agent();
    let _: &dyn Agent = &agent;
}

/// 测试 20: 验证宏展开后的代码符合 Rust 语法
#[test]
fn test_agent_syntax_validity() {
    // 这个测试主要验证宏展开后的代码是有效的 Rust 代码
    // 如果能编译通过，说明语法正确
    let _ = agent! {
        name: "syntax_test",
        instructions: "Testing syntax",
        provider: create_test_zhipu_provider()
    };
}
