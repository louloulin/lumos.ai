# {{project_name}}

{{description}}

## 🚀 快速开始

### 1. 安装依赖

```bash
cargo build
```

### 2. 配置环境

{{#if use_openai}}
设置OpenAI API密钥：

```bash
export OPENAI_API_KEY="your-api-key-here"
```
{{else}}
此项目使用Mock LLM提供者，无需配置API密钥。
{{/if}}

### 3. 运行项目

```bash
cargo run
```

## 📋 项目结构

```
{{project_name}}/
├── Cargo.toml          # 项目配置和依赖
├── src/
│   └── main.rs         # 主程序入口
├── README.md           # 项目说明
{{#if include_config}}
├── config/
│   └── agent.toml      # Agent配置文件
{{/if}}
{{#if include_tests}}
└── tests/
    └── integration_tests.rs  # 集成测试
{{/if}}
```

## 🤖 Agent配置

当前Agent配置：
- **名称**: {{agent_name}}
- **角色**: {{agent_instructions}}
{{#if include_tools}}
- **工具**: 
  - 计算器 (calculator)
  - 时间工具 (time_tool)
  {{#if include_web_tools}}
  - 网络搜索 (web_search)
  {{/if}}
  {{#if include_file_tools}}
  - 文件读取 (file_reader)
  {{/if}}
{{/if}}

## 🔧 自定义配置

### 修改Agent行为

编辑 `src/main.rs` 中的Agent创建部分：

```rust
let agent = quick_agent("{{agent_name}}", "你的自定义指令")
    .model(llm)
    {{#if include_tools}}
    .tools(vec![
        // 添加或移除工具
        calculator(),
        time_tool(),
    ])
    {{/if}}
    .build()?;
```

### 添加更多工具

```rust
use lumosai_core::prelude::*;

let agent = quick_agent("{{agent_name}}", "{{agent_instructions}}")
    .model(llm)
    .tools(vec![
        // 数学工具
        calculator(),
        statistics(),
        
        // 系统工具
        time_tool(),
        uuid_generator(),
        
        // 文件工具
        file_reader(),
        file_writer(),
        
        // 网络工具
        web_search(),
        http_request(),
    ])
    .build()?;
```

### 使用构建器模式

如果需要更多配置选项：

```rust
use lumosai_core::agent::AgentBuilder;

let agent = AgentBuilder::new()
    .name("{{agent_name}}")
    .instructions("{{agent_instructions}}")
    .model(llm)
    .max_tool_calls(10)
    .tool_timeout(30)
    .enable_function_calling(true)
    .build()?;
```

## 🧪 测试

运行测试：

```bash
cargo test
```

运行特定测试：

```bash
cargo test test_agent_creation
```

## 📚 学习资源

- [LumosAI文档](../../docs/README.md)
- [API选择指南](../../docs/api-choice-guide.md)
- [最佳实践](../../docs/best-practices/README.md)
- [示例代码](../../examples/README.md)

## 🛠️ 开发建议

1. **从简单开始**: 先使用`quick_agent()`，熟悉后再使用`AgentBuilder`
2. **逐步添加工具**: 根据需要逐步添加工具，避免一次性添加太多
3. **测试驱动**: 为每个功能编写测试，确保代码质量
4. **日志记录**: 使用`tracing`记录重要事件，便于调试

## 🆘 获取帮助

如果遇到问题：

1. 查看[错误解决指南](../../docs/troubleshooting/error-guide.md)
2. 搜索[GitHub Issues](https://github.com/lumosai/lumos.ai/issues)
3. 参考[示例代码](../../examples/)
4. 在社区中提问

## 📄 许可证

此项目使用 MIT 许可证。详见 [LICENSE](../../LICENSE) 文件。
