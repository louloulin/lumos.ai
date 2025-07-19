# LumosAI 错误解决指南

## 🎯 概述

本指南提供了LumosAI开发中常见错误的解决方案，帮助您快速定位和解决问题。

## 🔍 错误分类

### 配置错误 (E001)

#### 错误: "Agent model is required"
**原因**: 创建Agent时未指定LLM模型

**解决方案**:
```rust
// ❌ 错误的做法
let agent = AgentBuilder::new()
    .name("assistant")
    .build(); // 缺少model

// ✅ 正确的做法
let llm = Arc::new(MockLlmProvider::new(vec!["Hello".to_string()]));
let agent = AgentBuilder::new()
    .name("assistant")
    .model(llm)
    .build()?;
```

#### 错误: "Invalid configuration parameter"
**原因**: 配置参数格式错误或值无效

**解决方案**:
1. 检查参数类型是否正确
2. 验证参数值是否在有效范围内
3. 参考[API文档](../api-reference/)获取正确格式

### LLM提供者错误 (E002)

#### 错误: "API key is missing or invalid"
**原因**: API密钥未设置或格式错误

**解决方案**:
```rust
// 方法1: 环境变量
std::env::set_var("OPENAI_API_KEY", "your-api-key");
let llm = Arc::new(OpenAiProvider::from_env()?);

// 方法2: 直接设置
let llm = Arc::new(OpenAiProvider::new("your-api-key"));

// 方法3: 使用配置文件
let config = LlmConfig::from_file("config.toml")?;
let llm = Arc::new(OpenAiProvider::from_config(config)?);
```

#### 错误: "Rate limit exceeded"
**原因**: API调用频率超过限制

**解决方案**:
```rust
// 添加重试机制
let agent = AgentBuilder::new()
    .name("assistant")
    .model(llm)
    .max_retries(3)
    .retry_delay(Duration::from_secs(1))
    .build()?;

// 或使用指数退避
let agent = AgentBuilder::new()
    .name("assistant")
    .model(llm)
    .retry_strategy(RetryStrategy::ExponentialBackoff {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        multiplier: 2.0,
    })
    .build()?;
```

#### 错误: "Model not found"
**原因**: 指定的模型名称不存在

**解决方案**:
```rust
// 检查可用模型
let available_models = llm_provider.list_models().await?;
println!("可用模型: {:?}", available_models);

// 使用正确的模型名称
let agent = quick_agent("assistant", "You are helpful")
    .model(openai("gpt-4")?)  // 确保模型名称正确
    .build()?;
```

### 工具执行错误 (E003)

#### 错误: "Tool not found"
**原因**: 尝试调用不存在的工具

**解决方案**:
```rust
// 检查工具是否已注册
let agent = quick_agent("assistant", "You are helpful")
    .model(llm)
    .tools(vec![
        calculator(),  // 确保工具已添加
        weather_tool(),
    ])
    .build()?;

// 列出所有可用工具
for tool in agent.get_tools() {
    println!("工具: {} - {}", tool.name(), tool.description());
}
```

#### 错误: "Tool execution failed"
**原因**: 工具执行过程中出现错误

**解决方案**:
```rust
// 添加工具执行超时
let agent = AgentBuilder::new()
    .name("assistant")
    .model(llm)
    .tool_timeout(30)  // 30秒超时
    .max_tool_calls(5) // 最大调用次数
    .build()?;

// 检查工具参数
let tool_result = tool.execute(ToolExecutionContext::new(
    json!({"param": "value"}),
    ToolExecutionOptions::default()
)).await;

match tool_result {
    Ok(result) => println!("工具执行成功: {:?}", result),
    Err(e) => println!("工具执行失败: {}", e),
}
```

### 工作流执行错误 (E004)

#### 错误: "Workflow step failed"
**原因**: 工作流中某个步骤执行失败

**解决方案**:
```rust
// 添加错误处理和重试
let workflow = workflow! {
    name: "robust_workflow",
    steps: {
        {
            name: "step1",
            agent: agent1,
            instructions: "Execute step 1",
            retry: { count: 3, delay: 1000 },
            on_error: "continue"  // 或 "stop", "retry"
        }
    }
};

// 检查工作流状态
let status = workflow.get_status().await?;
println!("工作流状态: {:?}", status);

// 获取失败步骤的详细信息
if let Some(failed_step) = status.failed_steps.first() {
    println!("失败步骤: {}", failed_step.name);
    println!("错误信息: {}", failed_step.error);
}
```

#### 错误: "Circular dependency detected"
**原因**: 工作流步骤间存在循环依赖

**解决方案**:
```rust
// 检查依赖关系
let workflow = WorkflowBuilder::new()
    .id("linear_workflow")
    .add_step(step1)  // 无依赖
    .add_step(step2.depends_on("step1"))  // 依赖step1
    .add_step(step3.depends_on("step2"))  // 依赖step2
    .build()?;

// 验证工作流图
workflow.validate_dependencies()?;
```

### 内存管理错误 (E005)

#### 错误: "Memory capacity exceeded"
**原因**: 内存使用超过配置的容量限制

**解决方案**:
```rust
// 增加内存容量
let memory_config = MemoryConfig {
    capacity: 1000,  // 增加到1000条记录
    cleanup_strategy: CleanupStrategy::LRU,
    ..Default::default()
};

let agent = AgentBuilder::new()
    .name("assistant")
    .model(llm)
    .memory_config(memory_config)
    .build()?;

// 手动清理内存
agent.clear_memory().await?;

// 设置自动清理
let agent = AgentBuilder::new()
    .name("assistant")
    .model(llm)
    .auto_cleanup_memory(true)
    .memory_cleanup_interval(Duration::from_secs(300))
    .build()?;
```

### 网络错误 (E008)

#### 错误: "Connection timeout"
**原因**: 网络连接超时

**解决方案**:
```rust
// 增加超时时间
let llm = OpenAiProvider::builder()
    .api_key("your-key")
    .timeout(Duration::from_secs(60))  // 60秒超时
    .build()?;

// 配置重试
let llm = OpenAiProvider::builder()
    .api_key("your-key")
    .max_retries(3)
    .retry_delay(Duration::from_secs(2))
    .build()?;

// 使用代理
let llm = OpenAiProvider::builder()
    .api_key("your-key")
    .proxy("http://proxy.example.com:8080")
    .build()?;
```

## 🛠️ 调试技巧

### 1. 启用详细日志

```rust
use tracing::{info, debug, error};
use tracing_subscriber;

// 初始化日志
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();

// 在代码中添加日志
let agent = AgentBuilder::new()
    .name("debug_agent")
    .model(llm)
    .build()?;

info!("Agent创建成功: {}", agent.get_name());

let response = agent.generate("Hello").await?;
debug!("Agent响应: {}", response.content);
```

### 2. 使用错误上下文

```rust
use anyhow::{Context, Result};

fn create_agent() -> Result<Agent> {
    let llm = create_llm()
        .context("创建LLM提供者失败")?;
    
    let agent = AgentBuilder::new()
        .name("assistant")
        .model(llm)
        .build()
        .context("构建Agent失败")?;
    
    Ok(agent)
}
```

### 3. 性能监控

```rust
use std::time::Instant;

let start = Instant::now();
let response = agent.generate("Hello").await?;
let duration = start.elapsed();

if duration > Duration::from_secs(5) {
    warn!("Agent响应时间过长: {:?}", duration);
}
```

## 📊 常见问题检查清单

### Agent创建问题
- [ ] 是否设置了LLM模型？
- [ ] API密钥是否正确？
- [ ] 网络连接是否正常？
- [ ] 依赖库是否正确安装？

### 工具使用问题
- [ ] 工具是否已添加到Agent？
- [ ] 工具参数格式是否正确？
- [ ] 工具权限是否足够？
- [ ] 工具依赖是否满足？

### 工作流问题
- [ ] 步骤依赖关系是否正确？
- [ ] Agent配置是否完整？
- [ ] 错误处理是否设置？
- [ ] 超时时间是否合理？

### 性能问题
- [ ] 内存使用是否过高？
- [ ] 网络延迟是否过大？
- [ ] 并发数量是否合理？
- [ ] 缓存是否有效？

## 🆘 获取帮助

如果以上解决方案都无法解决您的问题：

1. **查看日志**: 启用详细日志记录，查看具体错误信息
2. **搜索文档**: 在[文档](../README.md)中搜索相关关键词
3. **查看示例**: 参考[示例代码](../../examples/)中的类似实现
4. **社区求助**: 在[GitHub Issues](https://github.com/lumosai/lumos.ai/issues)中提问
5. **联系支持**: 发送邮件到 support@lumosai.com

## 📚 相关资源

- [API选择指南](../api-choice-guide.md)
- [最佳实践](../best-practices/)
- [示例代码](../../examples/)
- [常见问题](../faq.md)
- [性能优化指南](../best-practices/performance.md)
