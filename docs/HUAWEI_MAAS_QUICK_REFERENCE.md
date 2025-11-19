# 华为 MaaS 快速参考卡片

## 🚀 快速开始

### 1. 环境配置

```bash
export HUAWEI_MAAS_API_KEY="your-api-key"
```

### 2. 基本使用（非流式）

```rust
use lumosai_core::llm::{HuaweiMaasProvider, LlmOptions, LlmProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = HuaweiMaasProvider::from_env()?;
    let options = LlmOptions::default().with_temperature(0.7);
    let response = provider.generate("你好", &options).await?;
    println!("{}", response);
    Ok(())
}
```

### 3. 流式响应（新功能）

```rust
use futures::StreamExt;
use lumosai_core::llm::{HuaweiMaasProvider, LlmOptions, LlmProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = HuaweiMaasProvider::from_env()?;
    let options = LlmOptions::default().with_temperature(0.7);
    
    let mut stream = provider.generate_stream("你好", &options).await?;
    
    while let Some(chunk) = stream.next().await {
        if let Ok(text) = chunk {
            print!("{}", text);
        }
    }
    
    Ok(())
}
```

## 📋 API 速查表

### 创建 Provider

| 方法 | 说明 | 示例 |
|------|------|------|
| `from_env()` | 从环境变量创建 | `HuaweiMaasProvider::from_env()?` |
| `new(api_key, model)` | 手动创建 | `HuaweiMaasProvider::new("key".into(), Some("model".into()))` |
| `with_base_url(...)` | 自定义 URL | `HuaweiMaasProvider::with_base_url("key".into(), "url".into(), None)` |

### LlmOptions 配置

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `temperature` | `f32` | 0.7 | 温度参数 (0.0-2.0) |
| `max_tokens` | `u32` | 2048 | 最大 token 数 |
| `model` | `String` | "deepseek-v3.2-exp" | 模型名称 |

```rust
let options = LlmOptions::default()
    .with_temperature(0.7)
    .with_max_tokens(1000)
    .with_model("deepseek-v3.2-exp");
```

### 核心方法

| 方法 | 返回类型 | 说明 |
|------|---------|------|
| `generate(prompt, options)` | `Result<String>` | 非流式生成 |
| `generate_stream(prompt, options)` | `Result<BoxStream<Result<String>>>` | 流式生成 ⭐ |
| `generate_with_messages(messages, options)` | `Result<String>` | 多轮对话 |
| `generate_with_functions(...)` | `Result<FunctionCallingResponse>` | 函数调用 |

## 🎨 常用模式

### 模式 1: 简单问答

```rust
let response = provider.generate("什么是 Rust?", &options).await?;
```

### 模式 2: 流式显示

```rust
let mut stream = provider.generate_stream(prompt, &options).await?;
while let Some(Ok(chunk)) = stream.next().await {
    print!("{}", chunk);
    io::stdout().flush()?;
}
```

### 模式 3: 累积完整响应

```rust
let mut full = String::new();
let mut stream = provider.generate_stream(prompt, &options).await?;
while let Some(Ok(chunk)) = stream.next().await {
    print!("{}", chunk);
    full.push_str(&chunk);
}
println!("\n完整: {}", full);
```

### 模式 4: 错误处理

```rust
let mut stream = provider.generate_stream(prompt, &options).await?;
while let Some(result) = stream.next().await {
    match result {
        Ok(chunk) => print!("{}", chunk),
        Err(e) => {
            eprintln!("错误: {}", e);
            break;
        }
    }
}
```

### 模式 5: 多轮对话

```rust
let messages = vec![
    Message {
        role: Role::System,
        content: "你是一个助手".to_string(),
        metadata: None,
        name: None,
    },
    Message {
        role: Role::User,
        content: "你好".to_string(),
        metadata: None,
        name: None,
    },
];
let response = provider.generate_with_messages(&messages, &options).await?;
```

## 🔧 配置示例

### 代码生成配置

```rust
let code_options = LlmOptions::default()
    .with_temperature(0.3)  // 降低随机性
    .with_max_tokens(2000);  // 增加长度
```

### 创意写作配置

```rust
let creative_options = LlmOptions::default()
    .with_temperature(0.9)  // 增加创造性
    .with_max_tokens(500);
```

### 精确回答配置

```rust
let precise_options = LlmOptions::default()
    .with_temperature(0.1)  // 最小随机性
    .with_max_tokens(200);
```

## 📊 性能调优

### 优化首字节时间

```rust
// 使用流式 + 较小的 max_tokens
let options = LlmOptions::default()
    .with_max_tokens(500)
    .with_temperature(0.5);
let stream = provider.generate_stream(prompt, &options).await?;
```

### 优化总响应时间

```rust
// 降低温度 + 合理的 max_tokens
let options = LlmOptions::default()
    .with_temperature(0.3)
    .with_max_tokens(1000);
```

## ⚠️ 常见错误

### 错误 1: API Key 未设置

```
Error: 环境变量 HUAWEI_MAAS_API_KEY 未设置
```

**解决**: `export HUAWEI_MAAS_API_KEY="your-key"`

### 错误 2: 网络连接失败

```
Error: 华为 MaaS API 请求失败: connection error
```

**解决**: 检查网络连接和 API 端点

### 错误 3: 流式解析错误

```
解析华为 MaaS 流式响应失败: ...
```

**解决**: 通常可以忽略，继续处理后续数据

## 🎯 最佳实践

### ✅ 推荐做法

1. **使用环境变量**: 存储 API Key
2. **错误处理**: 总是处理 Result
3. **Flush 输出**: 流式时及时刷新
4. **累积响应**: 保存完整内容
5. **合理配置**: 根据场景调整参数

### ❌ 避免做法

1. **硬编码 API Key**: 不安全
2. **忽略错误**: 可能导致崩溃
3. **过大 max_tokens**: 浪费资源
4. **过高 temperature**: 结果不稳定
5. **阻塞主线程**: 使用 async/await

## 📦 依赖版本

```toml
[dependencies]
lumosai_core = "0.2.0"
tokio = { version = "1.48", features = ["full"] }
futures = "0.3"
```

## 🔗 相关链接

- [完整文档](./huawei_maas_streaming.md)
- [示例程序](../examples/huawei_maas_streaming_demo.rs)
- [更新日志](../CHANGELOG_HUAWEI_MAAS_STREAMING.md)
- [实现总结](./STREAMING_IMPLEMENTATION_SUMMARY.md)

## 🆘 获取帮助

### 查看示例

```bash
cargo run --example huawei_maas_demo          # 基本示例
cargo run --example huawei_maas_streaming_demo # 流式示例
```

### 查看文档

```bash
cargo doc --open
```

### 运行测试

```bash
cargo test --package lumosai_core --lib llm::huawei_maas
```

## 📝 快速检查清单

开始使用前，确保：

- [ ] 已设置 `HUAWEI_MAAS_API_KEY` 环境变量
- [ ] 已添加 `lumosai_core` 依赖
- [ ] 已添加 `tokio` 和 `futures` 依赖
- [ ] 使用 `#[tokio::main]` 标记 main 函数
- [ ] 导入必要的 traits (`LlmProvider`, `StreamExt`)
- [ ] 正确处理 `Result` 类型

## 🎓 学习路径

1. **入门**: 运行 `huawei_maas_demo.rs`
2. **流式**: 运行 `huawei_maas_streaming_demo.rs`
3. **深入**: 阅读 `huawei_maas_streaming.md`
4. **实践**: 集成到自己的项目
5. **优化**: 根据场景调整参数

---

**版本**: v0.2.0  
**更新**: 2025-11-19  
**维护**: LumosAI Team

