# 华为 MaaS 流式响应指南

## 概述

华为 ModelArts MaaS (Model as a Service) 提供商现已支持流式响应功能，允许实时接收 AI 生成的内容，而不是等待完整响应。

## 功能特性

- ✅ **真实 SSE 流式**: 使用 Server-Sent Events 协议进行实时流式传输
- ✅ **兼容 OpenAI 格式**: 遵循 OpenAI 的流式响应格式
- ✅ **错误处理**: 优雅处理网络错误和解析错误
- ✅ **UTF-8 支持**: 正确处理中文和其他 Unicode 字符
- ✅ **自动过滤**: 自动过滤空内容和无效数据

## 快速开始

### 1. 环境配置

```bash
export HUAWEI_MAAS_API_KEY="your-api-key"
```

### 2. 基本使用

```rust
use futures::StreamExt;
use lumosai_core::llm::{HuaweiMaasProvider, LlmOptions, LlmProvider};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 provider
    let provider = HuaweiMaasProvider::from_env()?;
    
    // 配置选项
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(500);
    
    // 发起流式请求
    let mut stream = provider
        .generate_stream("请介绍一下 Rust 编程语言", &options)
        .await?;
    
    // 处理流式响应
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                print!("{}", chunk);
                io::stdout().flush()?;
            }
            Err(e) => {
                eprintln!("错误: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}
```

## API 详解

### 流式响应方法

```rust
async fn generate_stream(
    &self,
    prompt: &str,
    options: &LlmOptions,
) -> Result<BoxStream<'static, Result<String>>>
```

**参数:**
- `prompt`: 用户输入的提示文本
- `options`: LLM 配置选项

**返回:**
- `BoxStream<'static, Result<String>>`: 异步流，每个元素是一个文本块

### 配置选项

```rust
let options = LlmOptions::default()
    .with_temperature(0.7)      // 温度参数 (0.0-2.0)
    .with_max_tokens(1000)      // 最大 token 数
    .with_model("deepseek-v3.2-exp"); // 可选：指定模型
```

## 实现细节

### SSE 数据格式

华为 MaaS 使用标准的 SSE 格式：

```
data: {"choices":[{"delta":{"content":"你好"},"index":0}],"id":"...","created":1234567890}

data: {"choices":[{"delta":{"content":"，"},"index":0}],"id":"...","created":1234567890}

data: [DONE]
```

### 流式响应结构

```rust
#[derive(Debug, Deserialize)]
struct HuaweiMaasStreamResponse {
    choices: Vec<HuaweiMaasStreamChoice>,
    id: Option<String>,
    created: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasStreamChoice {
    delta: HuaweiMaasStreamDelta,
    finish_reason: Option<String>,
    index: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasStreamDelta {
    content: Option<String>,
    role: Option<String>,
}
```

### 处理流程

1. **发送请求**: 设置 `"stream": true` 参数
2. **接收字节流**: 使用 `response.bytes_stream()`
3. **解析 SSE**: 按行解析 `data:` 前缀的数据
4. **提取内容**: 从 `delta.content` 中提取文本
5. **过滤空值**: 自动过滤空内容和无效数据
6. **错误处理**: 记录解析错误但继续处理

## 高级用法

### 1. 累积完整响应

```rust
let mut stream = provider.generate_stream(prompt, &options).await?;
let mut full_response = String::new();

while let Some(chunk_result) = stream.next().await {
    if let Ok(chunk) = chunk_result {
        full_response.push_str(&chunk);
        print!("{}", chunk);
        io::stdout().flush()?;
    }
}

println!("\n完整响应: {}", full_response);
```

### 2. 添加延迟效果

```rust
while let Some(chunk_result) = stream.next().await {
    if let Ok(chunk) = chunk_result {
        print!("{}", chunk);
        io::stdout().flush()?;
        
        // 添加延迟以模拟打字效果
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
}
```

### 3. 统计流式性能

```rust
let start = std::time::Instant::now();
let mut first_chunk_time = None;
let mut chunk_count = 0;

let mut stream = provider.generate_stream(prompt, &options).await?;

while let Some(chunk_result) = stream.next().await {
    if let Ok(chunk) = chunk_result {
        if first_chunk_time.is_none() {
            first_chunk_time = Some(start.elapsed());
        }
        chunk_count += 1;
        print!("{}", chunk);
        io::stdout().flush()?;
    }
}

let total_time = start.elapsed();
println!("\n首个响应块耗时: {:?}", first_chunk_time.unwrap());
println!("总耗时: {:?}", total_time);
println!("响应块数量: {}", chunk_count);
```

### 4. 错误恢复

```rust
let mut stream = provider.generate_stream(prompt, &options).await?;
let mut retry_count = 0;
const MAX_RETRIES: u32 = 3;

while let Some(chunk_result) = stream.next().await {
    match chunk_result {
        Ok(chunk) => {
            print!("{}", chunk);
            io::stdout().flush()?;
            retry_count = 0; // 重置重试计数
        }
        Err(e) => {
            eprintln!("\n错误: {}", e);
            retry_count += 1;
            
            if retry_count >= MAX_RETRIES {
                eprintln!("达到最大重试次数，停止处理");
                break;
            }
            
            // 可以选择重新发起请求
            eprintln!("尝试恢复... ({}/{})", retry_count, MAX_RETRIES);
        }
    }
}
```

## 性能优化

### 1. 缓冲区大小

流式响应会自动处理缓冲，但可以通过调整 `max_tokens` 来控制响应长度：

```rust
let options = LlmOptions::default()
    .with_max_tokens(500);  // 较小的值会更快完成
```

### 2. 温度参数

较低的温度值通常会产生更快的响应：

```rust
let options = LlmOptions::default()
    .with_temperature(0.3);  // 更确定的输出
```

## 错误处理

### 常见错误

1. **网络错误**: HTTP 连接失败
   ```
   Error::Llm("HTTP 流错误: ...")
   ```

2. **解析错误**: JSON 格式不正确
   ```
   解析华为 MaaS 流式响应失败: ...
   ```

3. **UTF-8 错误**: 字符编码问题
   ```
   Error::Llm("UTF-8 解码错误: ...")
   ```

### 错误处理策略

```rust
while let Some(chunk_result) = stream.next().await {
    match chunk_result {
        Ok(chunk) => {
            // 处理正常数据
            process_chunk(&chunk);
        }
        Err(e) if e.to_string().contains("UTF-8") => {
            // 字符编码错误，可能是部分数据
            eprintln!("编码错误，跳过此块: {}", e);
            continue;
        }
        Err(e) if e.to_string().contains("HTTP") => {
            // 网络错误，可能需要重试
            eprintln!("网络错误: {}", e);
            break;
        }
        Err(e) => {
            // 其他错误
            eprintln!("未知错误: {}", e);
            break;
        }
    }
}
```

## 示例程序

运行完整的示例程序：

```bash
cargo run --example huawei_maas_streaming_demo
```

示例包含：
- ✅ 简单流式文本生成
- ✅ 多轮对话流式生成
- ✅ 代码生成流式响应
- ✅ 创意写作流式响应
- ✅ 流式与非流式性能对比

## 最佳实践

1. **始终处理错误**: 流式响应可能在任何时候失败
2. **使用 flush**: 确保实时显示输出
3. **累积完整响应**: 保存完整内容以备后用
4. **监控性能**: 记录首个响应块时间和总时间
5. **合理设置超时**: 避免长时间等待
6. **优雅降级**: 如果流式失败，可以回退到非流式模式

## 与其他提供商对比

| 特性 | 华为 MaaS | OpenAI | DeepSeek | 智谱 AI |
|------|-----------|--------|----------|---------|
| SSE 流式 | ✅ | ✅ | ❌ (模拟) | ✅ |
| 中文支持 | ✅ | ✅ | ✅ | ✅ |
| 函数调用 | ✅ | ✅ | ✅ | ✅ |
| 错误恢复 | ✅ | ✅ | ❌ | ✅ |

## 故障排查

### 问题: 流式响应卡住

**解决方案:**
1. 检查网络连接
2. 增加超时时间
3. 检查 API Key 是否有效

### 问题: 响应内容乱码

**解决方案:**
1. 确保终端支持 UTF-8
2. 检查响应数据编码
3. 使用 `flush()` 确保输出完整

### 问题: 响应速度慢

**解决方案:**
1. 减少 `max_tokens`
2. 降低 `temperature`
3. 使用更快的模型

## 相关资源

- [华为 MaaS 官方文档](https://support.huaweicloud.com/modelarts/index.html)
- [OpenAI 流式 API 文档](https://platform.openai.com/docs/api-reference/streaming)
- [Rust Futures 文档](https://rust-lang.github.io/async-book/)

## 更新日志

### v0.1.0 (2025-11-19)
- ✅ 初始实现流式响应功能
- ✅ 支持 SSE 协议
- ✅ 添加错误处理和过滤
- ✅ 完整的示例和文档

