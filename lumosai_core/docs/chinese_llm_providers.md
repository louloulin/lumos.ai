# 中国LLM提供商集成指南

本文档介绍如何在LumosAI中使用中国的大语言模型提供商，包括智谱AI (GLM) 和百度ERNIE。

## 支持的提供商

### 1. 智谱AI (GLM)
- **模型**: GLM-4, GLM-4-Plus, GLM-3-Turbo等
- **功能**: 文本生成、对话、函数调用、Embedding
- **官网**: https://open.bigmodel.cn/

### 2. 百度ERNIE
- **模型**: ERNIE-Bot, ERNIE-Bot-Turbo, ERNIE-Bot-4等  
- **功能**: 文本生成、对话、函数调用、Embedding
- **官网**: https://cloud.baidu.com/product/wenxinworkshop

## 快速开始

### 环境变量配置

#### 智谱AI
```bash
export ZHIPU_API_KEY="your_zhipu_api_key_here"
```

#### 百度ERNIE
```bash
export BAIDU_API_KEY="your_baidu_api_key_here"
export BAIDU_SECRET_KEY="your_baidu_secret_key_here"
```

### 基本使用

#### 1. 使用便利函数创建Provider

```rust
use lumosai_core::llm::providers;

// 智谱AI
let zhipu = providers::zhipu_from_env()?;

// 百度ERNIE  
let baidu = providers::baidu_from_env()?;

// 自动选择可用的provider
let auto_provider = providers::auto_provider()?;
```

#### 2. 手动创建Provider

```rust
use lumosai_core::llm::{ZhipuProvider, BaiduProvider};

// 智谱AI
let zhipu = ZhipuProvider::new(
    "your_api_key".to_string(),
    Some("glm-4".to_string())
);

// 百度ERNIE
let baidu = BaiduProvider::new(
    "your_api_key".to_string(),
    "your_secret_key".to_string(),
    Some("ernie-bot".to_string())
);
```

### 文本生成

```rust
use lumosai_core::llm::types::LlmOptions;

let options = LlmOptions::default()
    .with_temperature(0.7)
    .with_max_tokens(100);

// 简单文本生成
let response = zhipu.generate("你好，请介绍一下人工智能", &options).await?;
println!("智谱AI响应: {}", response);

let response = baidu.generate("你好，请介绍一下人工智能", &options).await?;
println!("百度ERNIE响应: {}", response);
```

### 对话功能

```rust
use lumosai_core::llm::types::{Message, Role};

let messages = vec![
    Message {
        role: Role::System,
        content: "你是一个友好的AI助手".to_string(),
        name: None,
        metadata: None,
    },
    Message {
        role: Role::User,
        content: "什么是机器学习？".to_string(),
        name: None,
        metadata: None,
    },
];

let response = zhipu.generate_with_messages(&messages, &options).await?;
println!("对话响应: {}", response);
```

### 函数调用

```rust
use lumosai_core::llm::function_calling::{FunctionDefinition, ToolChoice};
use serde_json::json;

// 定义函数
let functions = vec![
    FunctionDefinition {
        name: "get_weather".to_string(),
        description: "获取指定城市的天气信息".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "城市名称"
                }
            },
            "required": ["city"]
        }),
    }
];

let messages = vec![
    Message {
        role: Role::User,
        content: "北京今天的天气怎么样？".to_string(),
        name: None,
        metadata: None,
    },
];

// 调用函数
let response = zhipu.generate_with_functions(
    &messages,
    &functions,
    &ToolChoice::Auto,
    &options
).await?;

if !response.function_calls.is_empty() {
    println!("函数调用: {:?}", response.function_calls);
} else {
    println!("普通响应: {:?}", response.content);
}
```

### Embedding

```rust
let text = "人工智能是计算机科学的一个分支";

// 智谱AI Embedding
let embedding = zhipu.get_embedding(text).await?;
println!("Embedding维度: {}", embedding.len());

// 百度ERNIE Embedding  
let embedding = baidu.get_embedding(text).await?;
println!("Embedding维度: {}", embedding.len());
```

### 流式生成

```rust
use futures::StreamExt;

let mut stream = zhipu.generate_stream("请讲一个关于AI的故事", &options).await?;

print!("流式响应: ");
while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(text) => print!("{}", text),
        Err(e) => {
            println!("\n错误: {}", e);
            break;
        }
    }
}
println!();
```

## 高级配置

### 智谱AI配置

```rust
// 使用自定义模型和base URL
let zhipu = ZhipuProvider::with_base_url(
    "your_api_key".to_string(),
    "https://custom.api.url".to_string(),
    Some("glm-4-plus".to_string())
);
```

### 百度ERNIE配置

```rust
// 使用自定义base URL
let baidu = BaiduProvider::with_base_url(
    "your_api_key".to_string(),
    "your_secret_key".to_string(),
    "https://custom.api.url".to_string(),
    Some("ernie-bot-4".to_string())
);
```

### 模型选择

#### 智谱AI支持的模型
- `glm-4`: 最新的GLM-4模型
- `glm-4-plus`: GLM-4增强版
- `glm-3-turbo`: GLM-3快速版本

#### 百度ERNIE支持的模型
- `ernie-bot`: 标准ERNIE模型
- `ernie-bot-turbo`: ERNIE快速版本
- `ernie-bot-4`: ERNIE-4模型
- `ernie-3.5`: ERNIE-3.5模型

## 错误处理

```rust
use lumosai_core::Error;

match zhipu.generate("测试", &options).await {
    Ok(response) => println!("成功: {}", response),
    Err(Error::Llm(msg)) => println!("LLM错误: {}", msg),
    Err(e) => println!("其他错误: {}", e),
}
```

## 性能优化

### 1. 连接复用
Provider内部使用`reqwest::Client`，自动复用HTTP连接。

### 2. 并发请求
```rust
use tokio::try_join;

let (zhipu_response, baidu_response) = try_join!(
    zhipu.generate("问题1", &options),
    baidu.generate("问题2", &options)
)?;
```

### 3. 批量处理
```rust
let tasks: Vec<_> = questions.iter().map(|q| {
    zhipu.generate(q, &options)
}).collect();

let responses = futures::future::try_join_all(tasks).await?;
```

## 示例程序

运行完整的示例程序：

```bash
cd lumosai_core
cargo run --example chinese_llm_providers
```

## 注意事项

1. **API密钥安全**: 不要在代码中硬编码API密钥，使用环境变量
2. **速率限制**: 注意各提供商的API调用频率限制
3. **错误重试**: 建议实现指数退避的重试机制
4. **成本控制**: 监控API调用次数和token使用量

## 故障排除

### 常见问题

1. **认证失败**
   - 检查API密钥是否正确
   - 确认环境变量已正确设置

2. **网络连接问题**
   - 检查网络连接
   - 确认防火墙设置

3. **模型不支持**
   - 确认使用的模型名称正确
   - 检查账户是否有权限访问该模型

### 调试技巧

启用详细日志：
```rust
env_logger::init();
```

## 更多资源

- [智谱AI API文档](https://open.bigmodel.cn/dev/api)
- [百度ERNIE API文档](https://cloud.baidu.com/doc/WENXINWORKSHOP/index.html)
- [LumosAI完整文档](../README.md)
