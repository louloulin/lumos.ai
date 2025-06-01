// 专门用于调试agent!宏的测试文件
use lumos_macro::agent;

// 最简单的提供者函数
fn simple_provider() -> String {
    "test".to_string()
}

#[test]
fn test_minimal_agent() {
    // 最简单的agent!宏调用
    let _agent = agent! {
        name: "test_agent",
        instructions: "test instructions",
        
        llm: {
            provider: simple_provider(),
            model: "test-model"
        },
        
        tools: {}
    };
}
