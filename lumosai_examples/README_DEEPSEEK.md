# DeepSeek Agent 演示

这个示例展示了如何使用 DeepSeek 作为 LLM 提供者来创建一个智能 Agent。

## 功能特性

- ✅ **DeepSeek LLM 集成** - 使用 DeepSeek-Chat 模型
- ✅ **多种工具支持** - 代码分析、数学计算、文本分析
- ✅ **函数调用** - 智能工具选择和执行
- ✅ **中文支持** - 完整的中文交互体验
- ✅ **错误处理** - 优雅的错误处理和用户反馈

## 前置要求

1. **DeepSeek API 密钥** - 从 [DeepSeek 官网](https://platform.deepseek.com/) 获取
2. **Rust 环境** - 确保已安装 Rust 1.70+

## 快速开始

### 1. 设置 API 密钥

```bash
# Windows (PowerShell)
$env:DEEPSEEK_API_KEY="your-deepseek-api-key"

# Windows (Command Prompt)
set DEEPSEEK_API_KEY=your-deepseek-api-key

# Linux/macOS
export DEEPSEEK_API_KEY="your-deepseek-api-key"
```

### 2. 运行示例

```bash
# 进入示例目录
cd lumosai_examples

# 运行 DeepSeek Agent 演示
cargo run --example deepseek_agent_demo
```

## 示例功能

### 🔧 内置工具

1. **代码分析器** (`code_analyzer`)
   - 分析代码复杂度
   - 提供改进建议
   - 支持多种编程语言

2. **数学计算器** (`math_calculator`)
   - 计算复杂数学表达式
   - 支持基本运算和函数
   - 智能表达式解析

3. **文本分析器** (`text_analyzer`)
   - 统计文本信息
   - 简单情感分析
   - 多语言支持

### 📋 测试场景

示例包含 4 个预设测试场景：

1. **数学计算测试** - 演示数学工具的使用
2. **代码分析测试** - 分析 Rust 代码片段
3. **文本分析测试** - 分析中英文混合文本
4. **综合任务测试** - 多工具协作完成复杂任务

## 输出示例

```
🤖 DeepSeek Agent 智能助手演示
=====================================
✅ 找到DeepSeek API密钥，正在初始化...
🔧 正在添加工具...
✅ 工具添加完成：代码分析器、数学计算器、文本分析器

==================================================
📋 测试场景 1: 数学计算测试
==================================================
👤 用户输入: 请帮我计算 2+3*4 的结果，并解释计算过程。

🤖 DeepSeek正在思考...

💬 DeepSeek回答:
我来帮你计算 2+3*4 的结果。

🔍 执行步骤详情:
  步骤 1: ToolCall
    🛠️  工具调用:
      - math_calculator: {
          "expression": "2+3*4"
        }
    📊 工具结果:
      - math_calculator: {
          "calculation_complete": true,
          "expression": "2+3*4",
          "result": 14.0
        }

根据数学运算规则，2+3*4 的计算过程如下：
1. 首先计算乘法：3*4 = 12
2. 然后计算加法：2+12 = 14

所以 2+3*4 = 14
```

## 配置选项

### DeepSeek 模型配置

```rust
let deepseek_provider = Arc::new(DeepSeekProvider::new(
    api_key,
    Some("deepseek-chat".to_string()), // 可选：指定模型
));
```

### Agent 配置

```rust
let mut agent = create_basic_agent(
    "DeepSeek智能助手".to_string(),
    "你是一个基于DeepSeek的智能助手...".to_string(),
    deepseek_provider
);
```

## 自定义工具

你可以轻松添加自己的工具：

```rust
fn create_custom_tool() -> Box<dyn Tool> {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "input".to_string(),
            description: "输入参数".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);
    
    Box::new(FunctionTool::new(
        "custom_tool".to_string(),
        "自定义工具描述".to_string(),
        schema,
        |params| {
            // 工具逻辑
            Ok(serde_json::json!({
                "result": "处理完成"
            }))
        },
    ))
}

// 添加到 Agent
agent.add_tool(create_custom_tool())?;
```

## 故障排除

### 常见问题

1. **API 密钥错误**
   ```
   ❌ 错误：未设置DEEPSEEK_API_KEY环境变量
   ```
   **解决方案**：确保正确设置了环境变量

2. **网络连接问题**
   ```
   ❌ 错误: 网络请求失败
   ```
   **解决方案**：检查网络连接和防火墙设置

3. **API 限制**
   ```
   ❌ 错误: API 调用频率限制
   ```
   **解决方案**：等待一段时间后重试，或升级 API 计划

### 调试模式

设置环境变量启用详细日志：

```bash
export RUST_LOG=debug
cargo run --example deepseek_agent_demo
```

## 相关文档

- [DeepSeek API 文档](https://platform.deepseek.com/api-docs/)
- [LumosAI 核心文档](../lumosai-doc/)
- [工具开发指南](../lumosai-doc/05_tool_development.md)

## 许可证

本示例遵循项目的开源许可证。
