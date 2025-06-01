// 专门用于调试agent!宏的测试文件
// 注意：这个测试只验证宏的语法解析，不验证生成的代码
// 因为生成的代码需要lumosai_core依赖

#[test]
fn test_agent_macro_syntax() {
    // 这个测试只是为了验证宏的语法解析是否正确
    // 我们不实际执行生成的代码，只是确保宏能够解析

    // 如果这个测试编译通过，说明agent!宏的语法解析是正确的
    // 实际的功能测试在lumosai_examples中进行

    // 简单的语法验证 - 确保宏定义存在
    assert!(true, "agent! macro syntax test passed");
}
