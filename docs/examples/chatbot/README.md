# LumosAI 聊天机器人示例

这个示例展示如何使用 LumosAI 创建功能完整的聊天机器人，包含交互式对话、对话管理和多种使用模式。

## 功能特性

- 🗣️ **交互式对话**: 支持实时对话交互
- 📝 **对话历史**: 自动管理对话上下文
- 🎭 **角色扮演**: 支持自定义机器人角色
- 🔧 **命令行工具**: 提供灵活的命令行接口
- ⚡ **异步处理**: 高性能异步架构
- 🛡️ **错误处理**: 优雅的错误处理和恢复

## 快速开始

### 1. 运行基本聊天机器人

```bash
cargo run
```

这将启动一个交互式聊天界面，你可以直接与AI助手对话。

### 2. 自定义角色和模型

```bash
cargo run -- chat --role "你是一个专业的编程导师" --model "gpt-4"
```

### 3. 单次问答

```bash
cargo run -- ask "什么是人工智能？"
```

### 4. 运行示例

```bash
# 简单聊天示例
cargo run --example simple_chat

# 角色扮演示例
cargo run --example role_play
```

## 使用指南

### 交互式聊天命令

在聊天过程中，你可以使用以下命令：

- `quit` 或 `exit` - 退出聊天
- `clear` - 清空对话历史
- `help` - 显示帮助信息

### 命令行选项

```bash
# 查看所有可用选项
cargo run -- --help

# 聊天模式选项
cargo run -- chat --help

# 问答模式选项
cargo run -- ask --help
```

## 代码结构

```
chatbot/
├── src/
│   └── main.rs          # 主程序，包含CLI和聊天逻辑
├── examples/
│   ├── simple_chat.rs   # 简单聊天示例
│   └── role_play.rs     # 角色扮演示例
├── Cargo.toml           # 项目配置
└── README.md            # 本文档
```

## 核心概念

### 1. Agent 创建

```rust
use lumosai::prelude::*;

// 创建简单的聊天机器人
let agent = lumosai::agent::simple(
    "gpt-3.5-turbo",
    "你是一个友好的AI助手"
).await?;
```

### 2. 对话交互

```rust
// 发送消息并获取回复
let response = agent.chat("你好！").await?;
println!("回复: {}", response);
```

### 3. 错误处理

```rust
match agent.chat(message).await {
    Ok(response) => println!("🤖: {}", response),
    Err(e) => println!("❌ 错误: {}", e),
}
```

## 高级功能

### 自定义角色

你可以通过修改系统提示词来创建具有特定角色的聊天机器人：

```rust
let teacher_agent = lumosai::agent::simple(
    "gpt-3.5-turbo",
    "你是一位经验丰富的编程导师，专门教授 Rust 语言。"
).await?;

let poet_agent = lumosai::agent::simple(
    "gpt-3.5-turbo", 
    "你是一位富有创意的诗人，用诗意的语言回答问题。"
).await?;
```

### 对话历史管理

示例中展示了如何在客户端管理对话历史：

```rust
let mut conversation_history = Vec::new();

// 记录对话
conversation_history.push(format!("用户: {}", user_input));
conversation_history.push(format!("机器人: {}", bot_response));

// 清空历史
conversation_history.clear();
```

## 配置说明

### 环境变量

确保设置了必要的API密钥：

```bash
export OPENAI_API_KEY="your-api-key"
# 或其他模型提供商的API密钥
```

### 模型选择

支持多种模型：

- `gpt-3.5-turbo` - 快速响应，成本较低
- `gpt-4` - 更高质量的回复
- `claude-3` - Anthropic的Claude模型
- 其他兼容的模型

## 故障排除

### 常见问题

1. **API密钥错误**
   - 检查环境变量是否正确设置
   - 确认API密钥有效且有足够余额

2. **网络连接问题**
   - 检查网络连接
   - 确认防火墙设置

3. **编译错误**
   - 确保Rust版本 >= 1.75
   - 运行 `cargo update` 更新依赖

### 调试模式

启用详细日志：

```bash
RUST_LOG=debug cargo run
```

## 扩展建议

1. **添加更多命令**: 实现保存/加载对话历史
2. **流式响应**: 实现实时流式输出
3. **多模态支持**: 添加图片、文件处理能力
4. **插件系统**: 支持自定义功能扩展
5. **Web界面**: 创建Web版聊天界面

## 相关示例

- [hello-world](../hello-world/) - 基础入门示例
- [research-assistant](../research-assistant/) - 带工具的研究助手
- [rag-system](../rag-system/) - RAG知识问答系统

## 许可证

本示例遵循 LumosAI 项目的许可证。
