# LumosAI API选择指南

## 🎯 概述

LumosAI提供了5种不同层次的API，每种都有其适用场景。本指南将帮助您选择最适合您需求的API。

## 📊 API选择决策树

```
开始
├── 我是新手，想快速上手
│   └── 使用 Agent::quick() + prelude ✅
├── 我需要灵活配置但不想太复杂
│   └── 使用 AgentBuilder 构建器模式 ✅
├── 我喜欢声明式编程
│   └── 使用 agent! 宏 ✅
├── 我需要复杂的工作流
│   └── 使用 workflow! 宏 ✅
└── 我要配置整个应用
    └── 使用 lumos! 宏 ✅
```

## 🚀 场景导向的API选择

### 1. 快速原型开发
**推荐**: `Agent::quick()` + `prelude`

```rust
use lumosai_core::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 最简单的Agent创建
    let agent = quick_agent("assistant", "你是一个AI助手")
        .model(openai("gpt-4")?)
        .tools(vec![web_search(), calculator()])
        .build()?;
    
    let response = agent.generate("帮我计算 2+2").await?;
    println!("{}", response.content);
    
    Ok(())
}
```

**优势**:
- 一行代码创建Agent
- 内置常用工具
- 智能默认配置
- 快速验证想法

**适用场景**:
- 概念验证
- 快速原型
- 学习和实验
- 简单应用

### 2. 生产应用开发
**推荐**: `AgentBuilder` + 完整配置

```rust
use lumosai_core::agent::AgentBuilder;
use lumosai_core::llm::OpenAiProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let llm = Arc::new(OpenAiProvider::new("your-api-key"));
    
    let agent = AgentBuilder::new()
        .name("production_assistant")
        .instructions("你是一个专业的生产级AI助手")
        .model(llm)
        .tool(custom_tool)
        .max_tool_calls(10)
        .tool_timeout(30)
        .enable_function_calling(true)
        .add_metadata("version", "1.0")
        .add_metadata("environment", "production")
        .build()?;
    
    Ok(())
}
```

**优势**:
- 完全控制配置
- 类型安全
- 性能优化
- 错误处理完善

**适用场景**:
- 生产环境
- 企业应用
- 高性能要求
- 复杂业务逻辑

### 3. 声明式配置
**推荐**: `agent!` 宏

```rust
use lumos_macro::agent;
use lumosai_core::llm::OpenAiAdapter;

let agent = agent! {
    name: "research_assistant",
    instructions: "你是一个专业的研究助手",
    
    llm: {
        provider: OpenAiAdapter::new("api-key"),
        model: "gpt-4"
    },
    
    memory: {
        store_type: "buffer",
        capacity: 10
    },
    
    tools: {
        search_tool,
        calculator_tool: { precision: 2 },
        web_browser: { javascript: true }
    }
};
```

**优势**:
- 声明式配置
- 结构清晰
- 易于维护
- 配置复用

**适用场景**:
- 配置驱动应用
- 团队协作
- 配置管理
- 模板化开发

### 4. 复杂工作流
**推荐**: `workflow!` 宏

```rust
use lumos_macro::workflow;

let workflow = workflow! {
    name: "research_workflow",
    description: "智能研究工作流",
    
    steps: {
        {
            name: "research",
            agent: research_agent,
            instructions: "深度研究指定主题",
            timeout: 30000,
            retry: { count: 3, delay: 1000 }
        },
        {
            name: "analyze",
            agent: analyze_agent,
            instructions: "分析研究结果",
            when: { previous_step_success: true }
        },
        {
            name: "report",
            agent: report_agent,
            instructions: "生成研究报告",
            when: { all_previous_success: true }
        }
    },
    
    options: {
        max_retries: 3,
        timeout: 300000
    }
};
```

**优势**:
- 图形化工作流
- 条件执行
- 错误处理
- 状态管理

**适用场景**:
- 多步骤任务
- 条件分支
- 并行处理
- 复杂业务流程

### 5. 应用级配置
**推荐**: `lumos!` 宏

```rust
use lumos_macro::lumos;

let app = lumos! {
    name: "ai_assistant_app",
    description: "智能助手应用",
    
    agents: {
        research_agent,
        analysis_agent,
        report_agent
    },
    
    tools: {
        web_search,
        calculator,
        file_reader,
        database_query
    },
    
    workflows: {
        research_workflow,
        analysis_workflow
    },
    
    rags: {
        knowledge_base,
        document_store
    },
    
    mcp_endpoints: vec![
        "https://api.example.com/mcp"
    ]
};
```

**优势**:
- 一站式配置
- 组件整合
- 依赖管理
- 资源优化

**适用场景**:
- 完整应用
- 微服务架构
- 企业级系统
- 复杂集成

## 🔄 API迁移路径

### 从简单到复杂的迁移

1. **学习阶段**: `Agent::quick()` → 理解基本概念
2. **开发阶段**: `AgentBuilder` → 添加更多配置
3. **优化阶段**: `agent!` 宏 → 声明式配置
4. **扩展阶段**: `workflow!` 宏 → 复杂工作流
5. **生产阶段**: `lumos!` 宏 → 完整应用

### 混合使用策略

```rust
// 可以在同一个项目中混合使用不同API
use lumosai_core::prelude::*;
use lumos_macro::{agent, workflow, lumos};

// 快速原型Agent
let prototype_agent = quick_agent("prototype", "快速测试")
    .model(openai("gpt-4")?)
    .build()?;

// 生产级Agent
let production_agent = agent! {
    name: "production",
    instructions: "生产级助手",
    llm: { provider: OpenAiAdapter::new("key"), model: "gpt-4" }
};

// 复杂工作流
let complex_workflow = workflow! {
    name: "complex_flow",
    steps: { /* 复杂步骤 */ }
};

// 整合到应用
let app = lumos! {
    name: "mixed_app",
    agents: { prototype_agent, production_agent },
    workflows: { complex_workflow }
};
```

## 📚 进一步学习

- [快速开始教程](./getting-started/hello-world.md)
- [构建器模式详解](./api-reference/builder-api.md)
- [DSL宏使用指南](./api-reference/dsl-macros.md)
- [最佳实践指南](./best-practices/README.md)
- [示例代码库](../examples/README.md)

## ❓ 常见问题

**Q: 我应该从哪个API开始？**
A: 建议从`Agent::quick()`开始，理解基本概念后再选择适合的API。

**Q: 可以在同一个项目中使用多种API吗？**
A: 可以！不同API可以混合使用，选择最适合每个场景的API。

**Q: 哪个API性能最好？**
A: 所有API最终都编译为相同的底层代码，性能差异微乎其微。

**Q: 如何从一种API迁移到另一种？**
A: 参考上面的迁移路径，通常是渐进式迁移，保持向后兼容。
