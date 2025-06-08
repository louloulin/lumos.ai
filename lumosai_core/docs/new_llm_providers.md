# 新LLM提供商集成

本文档介绍了LumosAI核心库中新添加的四个LLM提供商：Cohere、Gemini、Ollama和Together AI。

## 概述

我们成功集成了以下新的LLM提供商：

1. **Cohere** - 企业级对话AI平台
2. **Gemini** - Google的多模态AI模型
3. **Ollama** - 本地运行的开源LLM
4. **Together AI** - 分布式AI推理平台

## 提供商详情

### 1. Cohere Provider

**特性：**
- 支持Cohere的Command系列模型
- 企业级API集成
- 高质量的文本生成和理解

**使用示例：**
```rust
use lumosai_core::llm::cohere::CohereProvider;

// 创建提供商
let provider = CohereProvider::new(
    "your-api-key".to_string(),
    "command-r-plus".to_string(),
);

// 从环境变量创建
let provider = CohereProvider::from_env()?;
```

**环境变量：**
- `COHERE_API_KEY` - Cohere API密钥

### 2. Gemini Provider

**特性：**
- 支持Google Gemini模型系列
- 支持函数调用功能
- 多模态能力（文本、图像等）

**使用示例：**
```rust
use lumosai_core::llm::gemini::GeminiProvider;

// 创建提供商
let provider = GeminiProvider::new(
    "your-api-key".to_string(),
    "gemini-1.5-pro".to_string(),
);

// 从环境变量创建
let provider = GeminiProvider::from_env()?;
```

**环境变量：**
- `GEMINI_API_KEY` - Google AI Studio API密钥

### 3. Ollama Provider

**特性：**
- 本地运行，无需API密钥
- 支持多种开源模型
- 完全私有化部署

**使用示例：**
```rust
use lumosai_core::llm::ollama::OllamaProvider;

// 创建本地提供商
let provider = OllamaProvider::localhost("llama2".to_string());

// 创建自定义端点提供商
let provider = OllamaProvider::new(
    "http://your-ollama-server:11434".to_string(),
    "llama2".to_string(),
);

// 从环境变量创建（使用默认localhost）
let provider = OllamaProvider::from_env();
```

**环境变量：**
- `OLLAMA_BASE_URL` - Ollama服务器地址（可选，默认localhost:11434）

### 4. Together AI Provider

**特性：**
- 支持多种开源模型
- 高性能分布式推理
- 成本效益高

**使用示例：**
```rust
use lumosai_core::llm::together::TogetherProvider;

// 创建提供商
let provider = TogetherProvider::new(
    "your-api-key".to_string(),
    "meta-llama/Llama-2-7b-chat-hf".to_string(),
);

// 从环境变量创建
let provider = TogetherProvider::from_env()?;
```

**环境变量：**
- `TOGETHER_API_KEY` - Together AI API密钥

## 统一接口

所有提供商都实现了`LlmProvider` trait，提供统一的接口：

```rust
use lumosai_core::llm::{LlmProvider, LlmOptions, Message, Role};

// 创建消息
let messages = vec![
    Message {
        role: Role::System,
        content: "你是一个有用的AI助手。".to_string(),
        metadata: None,
        name: None,
    },
    Message {
        role: Role::User,
        content: "你好！".to_string(),
        metadata: None,
        name: None,
    },
];

// 创建选项
let options = LlmOptions::default()
    .with_temperature(0.7)
    .with_max_tokens(100);

// 使用任何提供商
async fn use_provider(provider: &dyn LlmProvider) -> Result<String, Box<dyn std::error::Error>> {
    let response = provider.chat(&messages, &options).await?;
    Ok(response)
}
```

## 功能对比

| 提供商 | 函数调用 | 流式输出 | 本地部署 | 多模态 |
|--------|----------|----------|----------|--------|
| Cohere | ❌ | ✅ | ❌ | ❌ |
| Gemini | ✅ | ✅ | ❌ | ✅ |
| Ollama | ❌ | ✅ | ✅ | ❌ |
| Together | ❌ | ✅ | ❌ | ❌ |

## 配置选项

所有提供商都支持以下通用配置选项：

- `temperature` - 控制输出的随机性（0.0-1.0）
- `max_tokens` - 最大输出令牌数
- `stop` - 停止序列
- `extra` - 提供商特定的额外参数

### 提供商特定选项

通过`extra`字段可以传递提供商特定的参数：

```rust
let options = LlmOptions::default()
    .with_temperature(0.7)
    .with_extra("top_p", 0.9)
    .with_extra("frequency_penalty", 0.1);
```

## 错误处理

所有提供商都使用统一的错误处理机制：

```rust
use lumosai_core::error::LumosError;

match provider.chat(&messages, &options).await {
    Ok(response) => println!("成功: {}", response),
    Err(LumosError::ApiError { message, .. }) => {
        eprintln!("API错误: {}", message);
    }
    Err(LumosError::NetworkError { .. }) => {
        eprintln!("网络错误");
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

## 测试

运行新提供商的示例：

```bash
cd lumosai_core
cargo run --example new_llm_providers
```

运行单元测试：

```bash
cd lumosai_core
cargo test new_providers_test --lib
```

## 注意事项

1. **API密钥安全**：请妥善保管API密钥，不要在代码中硬编码
2. **速率限制**：各提供商都有不同的速率限制，请注意控制请求频率
3. **模型选择**：不同模型有不同的能力和成本，请根据需求选择
4. **本地部署**：Ollama需要本地安装和运行服务

## 未来计划

- [ ] 添加更多提供商（Claude、PaLM等）
- [ ] 增强函数调用支持
- [ ] 添加模型性能基准测试
- [ ] 实现智能提供商选择
- [ ] 添加成本跟踪功能

## 贡献

欢迎贡献新的LLM提供商集成！请参考现有提供商的实现模式，并确保：

1. 实现`LlmProvider` trait
2. 添加完整的错误处理
3. 编写单元测试
4. 更新文档

## 许可证

本项目采用MIT许可证。详见LICENSE文件。
