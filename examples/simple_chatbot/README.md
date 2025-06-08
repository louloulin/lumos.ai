# 简单聊天机器人示例

这个示例展示了如何使用 LumosAI 创建一个简单的命令行聊天机器人。

## 功能特性

- 🤖 基于 LumosAI Agent 的聊天机器人
- 💬 支持多轮对话和历史记录
- ⚙️ 可配置的模型和参数
- 📊 内置统计信息显示
- 🎛️ 交互式命令支持

## 运行方式

```bash
# 基本运行
cargo run

# 指定模型和参数
cargo run -- --model gpt-4 --temperature 0.8

# 自定义系统提示
cargo run -- --system-prompt "你是一个专业的技术顾问"
```

## 命令行参数

- `--model, -m`: 指定 LLM 模型 (默认: gpt-3.5-turbo)
- `--system-prompt, -s`: 设置系统提示 (默认: 友善的AI助手)
- `--temperature, -t`: 设置温度参数 (默认: 0.7)

## 交互命令

在聊天过程中，您可以使用以下命令：

- `/help` - 显示帮助信息
- `/stats` - 显示统计信息
- `/clear` - 清除对话历史
- `/info` - 显示 Agent 信息
- `quit` 或 `exit` - 退出程序

## 示例对话

```
🤖 LumosAI 聊天机器人
模型: gpt-3.5-turbo
输入 'quit' 或 'exit' 退出

👤 你: 你好！
🤖 AI: 你好！我是你的AI助手，有什么可以帮助你的吗？

👤 你: /stats
📊 统计信息:
  总对话数: 42
  平均响应时间: 750ms
  成功率: 95.00%

👤 你: 什么是人工智能？
🤖 AI: 人工智能（AI）是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统...

👤 你: quit
👋 再见！
```

## 技术实现

### 核心组件

1. **Agent 创建**: 使用 LumosAI 的构建器模式创建 Agent
2. **对话管理**: 维护对话历史和上下文
3. **命令处理**: 支持特殊命令和功能
4. **错误处理**: 优雅的错误处理和用户反馈

### 代码结构

```rust
// 创建 Agent
let agent = lumosai::agent::builder()
    .model(&args.model)
    .system_prompt(&args.system_prompt)
    .temperature(args.temperature)
    .build()
    .await?;

// 聊天循环
loop {
    let input = get_user_input()?;
    let response = agent.chat_with_history(input, &history).await?;
    println!("🤖 AI: {}", response);
}
```

## 扩展功能

您可以基于这个示例添加更多功能：

### 1. 工具集成

```rust
let agent = lumosai::agent::builder()
    .model("gpt-4")
    .tools(vec![
        weather_tool,
        calculator_tool,
        web_search_tool,
    ])
    .build()
    .await?;
```

### 2. RAG 集成

```rust
let rag = lumosai::rag::simple(vector_storage, "openai").await?;
let agent = lumosai::agent::builder()
    .model("gpt-4")
    .rag_context(rag)
    .build()
    .await?;
```

### 3. 多模态支持

```rust
let agent = lumosai::agent::builder()
    .model("gpt-4-vision")
    .multimodal(true)
    .build()
    .await?;

// 支持图片输入
let response = agent.chat_with_image("描述这张图片", image_data).await?;
```

### 4. 流式响应

```rust
let mut stream = agent.chat_stream("长篇问题").await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk);
    io::stdout().flush()?;
}
```

## 配置文件

您可以创建配置文件来管理设置：

```toml
# chatbot.toml
[agent]
model = "gpt-4"
temperature = 0.7
system_prompt = "你是一个专业的AI助手"

[ui]
show_typing_indicator = true
save_history = true
history_file = "chat_history.json"
```

## 部署

### Docker 部署

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/simple_chatbot /usr/local/bin/
CMD ["simple_chatbot"]
```

### 云部署

```bash
# 构建并推送到云端
docker build -t lumosai-chatbot .
docker push your-registry/lumosai-chatbot

# 在云端运行
kubectl apply -f deployment.yaml
```

## 故障排除

### 常见问题

1. **API 密钥错误**
   ```bash
   export OPENAI_API_KEY="your-api-key"
   ```

2. **网络连接问题**
   ```bash
   # 检查网络连接
   curl -I https://api.openai.com
   ```

3. **依赖问题**
   ```bash
   # 清理并重新构建
   cargo clean
   cargo build
   ```

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License
