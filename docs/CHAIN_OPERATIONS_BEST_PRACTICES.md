# LumosAI 链式操作最佳实践指南

## 📖 概述

LumosAI 的链式操作系统是 Plan 10 API 改造的重要成果之一，它提供了流畅、直观的对话流程管理能力。本指南将介绍如何有效使用链式操作来构建复杂的 AI 应用。

## 🔗 核心概念

### 链式操作 (Chain Operations)
链式操作允许你以流畅的方法链式调用方式管理多轮对话：

```rust
let response = agent
    .chain()
    .system("你是一个专业顾问")
    .ask("第一个问题")
    .await?
    .then_ask("第二个问题")
    .await?
    .then_ask("第三个问题")
    .await?;
```

### 核心组件

1. **AgentChain** - 链式操作的主要接口
2. **ChainContext** - 上下文状态管理
3. **ChainResponse** - 链式响应对象
4. **ChainStep** - 操作步骤记录

## 🚀 基础用法

### 1. 开始链式操作

```rust
use lumosai_core::agent::chain::AgentChainExt;

// 方法 1: 从 Agent 开始
let chain = agent.chain();

// 方法 2: 使用现有上下文
let chain = agent.chain_with_context(existing_context);
```

### 2. 基本对话流程

```rust
// 简单的问答链
let response = agent
    .chain()
    .ask("你好，请介绍一下自己")
    .await?;

// 继续对话
let response2 = response
    .then_ask("你能帮我做什么？")
    .await?;
```

### 3. 添加系统消息和上下文

```rust
let response = agent
    .chain()
    .system("你是一个专业的技术顾问")
    .set_variable("user_level", json!("expert"))
    .ask("请解释微服务架构的优缺点")
    .await?;
```

## 🎯 高级用法

### 1. 上下文变量管理

```rust
let response = agent
    .chain()
    .set_variable("project_type", json!("web_app"))
    .set_variable("team_size", json!(5))
    .set_variable("budget", json!(100000))
    .ask("基于这些信息，请制定项目计划")
    .await?;

// 后续访问变量
let chain = response.chain();
if let Some(budget) = chain.get_variable("budget") {
    println!("项目预算: {}", budget);
}
```

### 2. 条件分支处理

```rust
let initial_response = agent
    .chain()
    .ask("用户的问题类型是什么？")
    .await?;

// 根据响应内容进行分支处理
let follow_up = if initial_response.content().contains("技术") {
    initial_response
        .then_ask("请提供技术解决方案")
        .await?
} else if initial_response.content().contains("商务") {
    initial_response
        .then_ask("请提供商务建议")
        .await?
} else {
    initial_response
        .then_ask("请提供通用建议")
        .await?
};
```

### 3. 持久化和恢复

```rust
// 保存对话上下文
let context_file = "conversation_state.json";
response.chain().save_context(context_file)?;

// 稍后恢复对话
let restored_chain = agent
    .chain()
    .load_context(context_file)?;

let continued_response = restored_chain
    .ask("继续我们之前的对话")
    .await?;
```

### 4. 与工具系统集成

```rust
let math_agent = AgentBuilder::new()
    .name("calculator")
    .instructions("你是一个数学助手")
    .tool(Box::new(CalculatorTool::default()))
    .enable_function_calling(true)
    .build()?;

let calculation_chain = math_agent
    .chain()
    .ask("请计算 (25 + 15) * 3")
    .await?
    .then_ask("然后除以 4")
    .await?;
```

## 📋 实际应用场景

### 1. 客户服务工作流

```rust
async fn customer_service_workflow(agent: &Agent, customer_issue: &str) -> Result<String> {
    let resolution = agent
        .chain()
        .system("你是专业的客服代表")
        .set_variable("customer_id", json!("C12345"))
        .set_variable("issue_type", json!("technical"))
        .ask(format!("客户问题：{}", customer_issue))
        .await?
        .then_ask("请提供详细的解决步骤")
        .await?
        .then_ask("还需要其他帮助吗？")
        .await?;
    
    Ok(resolution.content().to_string())
}
```

### 2. 多阶段决策支持

```rust
async fn decision_support_process(agent: &Agent) -> Result<()> {
    let decision = agent
        .chain()
        .system("你是决策分析专家")
        .ask("请分析当前的市场情况")
        .await?
        .then_ask("基于分析，有哪些可行的策略？")
        .await?
        .then_ask("请评估每个策略的风险和收益")
        .await?
        .then_ask("给出最终的推荐方案")
        .await?;
    
    // 保存决策过程
    decision.chain().save_context("decision_process.json")?;
    
    Ok(())
}
```

### 3. 教育和培训场景

```rust
async fn interactive_learning_session(agent: &Agent, topic: &str) -> Result<()> {
    let session = agent
        .chain()
        .system("你是一个耐心的老师")
        .set_variable("learning_topic", json!(topic))
        .set_variable("difficulty_level", json!("beginner"))
        .ask(format!("请介绍 {} 的基础概念", topic))
        .await?
        .then_ask("请举一个实际的例子")
        .await?
        .then_ask("我应该如何开始学习？")
        .await?
        .then_ask("有什么练习建议吗？")
        .await?;
    
    println!("学习会话完成，共 {} 个步骤", session.chain().get_steps().len());
    
    Ok(())
}
```

## ⚡ 性能优化建议

### 1. 合理控制链长度

```rust
// ✅ 好的做法：适中的链长度
let response = agent
    .chain()
    .ask("问题1").await?
    .then_ask("问题2").await?
    .then_ask("问题3").await?;

// ❌ 避免：过长的链
// 超过 10 轮的链式对话可能影响性能
```

### 2. 使用上下文变量减少重复

```rust
// ✅ 好的做法：使用变量存储状态
let chain = agent
    .chain()
    .set_variable("user_preferences", json!(preferences))
    .ask("基于我的偏好推荐产品").await?;

// ❌ 避免：在每次对话中重复信息
```

### 3. 适当的错误处理

```rust
let result = agent
    .chain()
    .ask("问题1")
    .await
    .and_then(|r| r.then_ask("问题2"))
    .await
    .and_then(|r| r.then_ask("问题3"))
    .await;

match result {
    Ok(response) => println!("成功: {}", response.content()),
    Err(e) => println!("链式操作失败: {}", e),
}
```

## 🛡️ 最佳实践

### 1. 上下文管理

- **及时清理**：长时间运行的应用应定期清理上下文
- **变量命名**：使用清晰、一致的变量命名
- **状态验证**：在关键步骤验证上下文状态

### 2. 错误处理

- **优雅降级**：链式操作失败时提供备选方案
- **状态恢复**：保存关键状态以便错误恢复
- **超时处理**：设置合理的超时时间

### 3. 性能考虑

- **批量操作**：合并相关的操作以减少 API 调用
- **缓存策略**：缓存常用的响应和上下文
- **异步处理**：利用 Rust 的异步特性提高并发性

### 4. 安全性

- **输入验证**：验证用户输入和上下文变量
- **权限控制**：确保链式操作的权限边界
- **数据保护**：敏感信息的安全存储和传输

## 🔧 调试和监控

### 1. 链式操作调试

```rust
// 启用详细日志
let response = agent
    .chain()
    .ask("调试问题")
    .await?;

// 检查执行步骤
for (i, step) in response.chain().get_steps().iter().enumerate() {
    println!("步骤 {}: {} - {}", i + 1, step.step_type, step.content);
}
```

### 2. 性能监控

```rust
use std::time::Instant;

let start = Instant::now();
let response = agent.chain().ask("性能测试").await?;
let duration = start.elapsed();

println!("链式操作耗时: {:?}", duration);
```

## 📚 相关资源

- [LumosAI Agent 构建指南](./AGENT_BUILDER_GUIDE.md)
- [工具系统集成文档](./TOOL_INTEGRATION.md)
- [API 参考文档](./API_REFERENCE.md)
- [示例代码库](../examples/)

## 🤝 社区和支持

- GitHub Issues: 报告问题和功能请求
- 讨论区: 技术讨论和最佳实践分享
- 文档贡献: 帮助改进文档和示例

---

通过遵循这些最佳实践，你可以充分利用 LumosAI 链式操作的强大功能，构建出高效、可靠的 AI 应用。
