# Hello World - 最简单的 LumosAI Agent 示例

这是 LumosAI 的入门示例，展示了如何创建和使用最基本的 AI Agent。

## 🎯 学习目标

通过这个示例，您将学会：
- 创建基本的 AI Agent
- 配置 Agent 的基本参数
- 发送消息并获取回复
- 处理基本的错误情况
- 理解不同配置参数的影响

## 📋 前置要求

- Rust 1.75+
- 有效的 OpenAI API 密钥
- 基本的 Rust 语法知识

## 🚀 快速开始

### 1. 设置环境

```bash
# 设置 API 密钥
export OPENAI_API_KEY="your-openai-api-key"

# 或者创建 .env 文件
echo "OPENAI_API_KEY=your-openai-api-key" > .env
```

### 2. 运行基础示例

```bash
# 进入示例目录
cd docs/examples/hello-world

# 运行基础示例
cargo run
```

**预期输出：**
```
🤖 LumosAI Hello World 示例
========================================
📝 创建 Agent...
✅ Agent 创建成功: Hello Agent
💬 发送消息: 你好！
🤖 Agent 回复: 你好！我是 Hello Agent，很高兴为您服务...
```

### 3. 运行交互式示例

```bash
# 运行交互式对话
cargo run --example interactive
```

这将启动一个交互式对话界面，您可以与 Agent 进行实时对话。

### 4. 运行配置示例

```bash
# 运行配置演示
cargo run --example configured
```

这个示例展示了不同配置参数对 Agent 行为的影响。

## 📖 代码解析

### 基本 Agent 创建

```rust
use lumosai::prelude::*;

let agent = Agent::builder()
    .name("Hello Agent")                    // Agent 名称
    .instructions("你是一个友好的助手")      // 系统指令
    .model("gpt-3.5-turbo")                // LLM 模型
    .build()?;                             // 构建 Agent
```

### 发送消息

```rust
let response = agent.generate("你好！").await?;
println!("Agent 回复: {}", response);
```

### 配置参数

```rust
let agent = Agent::builder()
    .name("专业助手")
    .instructions("详细的系统指令...")
    .model("gpt-3.5-turbo")
    .temperature(0.7)      // 创造性控制 (0.0-2.0)
    .max_tokens(500)       // 最大回复长度
    .build()?;
```

## 🔧 配置参数详解

### 核心参数

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `name` | String | "Agent" | Agent 的名称标识 |
| `instructions` | String | "" | 系统指令，定义 Agent 行为 |
| `model` | String | "gpt-3.5-turbo" | 使用的 LLM 模型 |

### 生成参数

| 参数 | 类型 | 默认值 | 范围 | 描述 |
|------|------|--------|------|------|
| `temperature` | f32 | 0.7 | 0.0-2.0 | 控制回复的创造性和随机性 |
| `max_tokens` | u32 | 1000 | 1-4096 | 限制回复的最大长度 |
| `top_p` | f32 | 1.0 | 0.0-1.0 | 核采样参数 |
| `frequency_penalty` | f32 | 0.0 | -2.0-2.0 | 频率惩罚，减少重复 |
| `presence_penalty` | f32 | 0.0 | -2.0-2.0 | 存在惩罚，鼓励新话题 |

### 温度参数指南

- **0.0-0.3**: 非常保守，适合需要准确性的任务
- **0.4-0.7**: 平衡，适合大多数对话场景
- **0.8-1.2**: 创造性，适合创意写作和头脑风暴
- **1.3-2.0**: 非常创造性，可能产生意外结果

## 🧪 测试

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_agent_creation

# 显示测试输出
cargo test -- --nocapture
```

### 测试说明

- `test_agent_creation`: 测试 Agent 创建功能
- `test_agent_generate`: 测试消息生成功能（需要 API 密钥）

## 🔍 故障排除

### 常见问题

**Q: 运行时出现 "API key not found" 错误**
```
A: 请确保设置了正确的环境变量：
export OPENAI_API_KEY="your-api-key"
```

**Q: Agent 回复很慢**
```
A: 这是正常的，因为需要调用远程 API。可以：
1. 检查网络连接
2. 尝试使用更快的模型
3. 减少 max_tokens 参数
```

**Q: 回复质量不好**
```
A: 可以尝试：
1. 改进 instructions 指令
2. 调整 temperature 参数
3. 使用更强的模型如 gpt-4
```

**Q: 编译错误**
```
A: 请确保：
1. Rust 版本 >= 1.75
2. 网络连接正常（下载依赖）
3. 在正确的目录运行命令
```

## 📚 下一步

完成这个示例后，您可以继续学习：

1. **[chatbot 示例](../chatbot/)** - 学习构建交互式聊天机器人
2. **[tool-integration 示例](../tool-integration/)** - 为 Agent 添加工具能力
3. **[教程系列](../../tutorials/)** - 深入学习 LumosAI 功能

## 🤝 扩展练习

### 练习 1: 个性化 Agent
创建一个具有特定个性的 Agent（如幽默、严肃、专业等）。

### 练习 2: 多语言 Agent
创建一个能够检测并使用多种语言回复的 Agent。

### 练习 3: 专业领域 Agent
创建一个特定领域的专家 Agent（如医学、法律、技术等）。

### 练习 4: 情感感知 Agent
创建一个能够感知用户情感并适当回应的 Agent。

## 📞 获取帮助

- 📖 查看 [LumosAI 文档](../../README.md)
- 🐛 [报告问题](https://github.com/lumosai/lumosai/issues)
- 💬 [社区讨论](https://github.com/lumosai/lumosai/discussions)

---

*恭喜完成第一个 LumosAI 示例！继续探索更多功能吧！* 🎉
