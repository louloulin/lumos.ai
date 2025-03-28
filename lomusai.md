# Mastra 到 Rust 的迁移计划

## 1. 项目概述

Mastra是一个用TypeScript编写的AI应用框架，提供以下核心功能：

- **Agent系统**：允许语言模型选择执行一系列动作的系统 ✅
- **工作流引擎**：可持久化的基于图的状态机 ✅
- **RAG (检索增强生成)**：构建知识库的ETL管道 ✅
- **集成系统**：自动生成的类型安全的第三方服务API客户端
- **评估框架**：使用模型评分、基于规则和统计方法评估LLM输出的工具
- **工具系统**：可由Agent或工作流执行的类型化函数 ✅
- **记忆系统**：帮助存储和检索上下文信息的组件 ✅

## 2. 技术栈对比

| 功能 | Mastra (TypeScript) | Lomusai (Rust) |
|------|-------------------|----------------|
| 语言 | TypeScript/JavaScript | Rust ✅ |
| 包管理 | pnpm | Cargo ✅ |
| 依赖解析 | node_modules | Cargo.toml ✅ |
| 构建工具 | Turbo, tsup | Cargo/rustc ✅ |
| 类型系统 | TypeScript静态类型 | Rust静态类型+所有权系统 ✅ |
| 异步模型 | Promise/async-await | Futures/async-await ✅ |
| 内存管理 | 垃圾回收 | RAII, 所有权系统 ✅ |
| 错误处理 | try/catch, Result | Result<T, E> ✅ |

## 3. 迁移策略

### 3.1 阶段划分

1. **阶段一：核心库迁移** ✅
   - 设计Rust核心库架构 ✅
   - 实现基础类型和接口 ✅
   - 迁移核心功能：Agents, Workflows, Tools ✅

2. **阶段二：功能模块迁移**
   - 实现RAG模块 ✅
   - 实现Memory模块 ✅
   - 实现Evals模块

3. **阶段三：集成和部署**
   - 实现第三方服务集成
   - 开发部署器
   - 迁移CLI工具

4. **阶段四：性能优化和测试**
   - 性能基准测试
   - 内存优化
   - 多线程/并行优化

### 3.2 模块映射关系

| Mastra模块 | Lomusai模块 | 状态 |
|-----------|-----------|------|
| @mastra/core | lomusai_core | ✅ 已实现基础功能 |
| @mastra/rag | lomusai_rag | ✅ 已实现基础功能 |
| @mastra/memory | lomusai_memory | ✅ 已实现基础功能 |
| @mastra/evals | lomusai_evals | 🔄 待实现 |
| @mastra/cli | lomusai_cli | 🔄 待实现 |
| @mastra/deployer | lomusai_deployer | 🔄 待实现 |
| integrations/* | lomusai_integrations | 🔄 待实现 |

## 4. 详细实现计划

### 4.1 优化的模块化架构设计 ✅

```
lomusai/
├── crates/                      // 所有crate集中管理
│   ├── lomusai/                 // 主包/入口
│   │   └── src/
│   │       ├── lib.rs           // 重导出所有模块
│   │       └── bin/             // CLI入口
│   │
│   ├── lomusai_core/            // 核心功能 ✅
│   │   └── src/
│   │       ├── lib.rs           // 公共API导出
│   │       ├── error.rs         // 错误处理 ✅ 
│   │       ├── types/           // 通用核心类型
│   │       │   └── mod.rs
│   │       ├── agent/           // Agent实现 ✅
│   │       │   ├── mod.rs
│   │       │   ├── config.rs    // Agent配置
│   │       │   └── executor.rs  // Agent执行器
│   │       ├── workflow/        // 工作流实现 ✅ 
│   │       │   ├── mod.rs
│   │       │   ├── step.rs      // 工作流步骤
│   │       │   ├── state.rs     // 工作流状态
│   │       │   └── executor.rs  // 工作流执行器
│   │       ├── tool/            // 工具系统 ✅
│   │       │   ├── mod.rs
│   │       │   ├── schema.rs    // 工具模式定义
│   │       │   └── function.rs  // 函数工具
│   │       ├── llm/             // LLM接口 ✅
│   │       │   ├── mod.rs
│   │       │   ├── provider.rs  // LLM提供者特性
│   │       │   ├── openai.rs    // OpenAI实现
│   │       │   └── anthropic.rs // Anthropic实现
│   │       └── telemetry/       // 遥测和跟踪 ✅
│   │           ├── mod.rs
│   │           ├── event.rs     // 事件定义
│   │           ├── sink.rs      // 遥测接收器特性
│   │           └── span.rs      // 跟踪区间
│   │
│   ├── lomusai_memory/          // 记忆系统 ✅
│   │   └── src/
│   │       ├── lib.rs           // 公共API导出
│   │       ├── types.rs         // 记忆类型定义
│   │       ├── storage/         // 存储实现
│   │       │   ├── mod.rs
│   │       │   ├── in_memory.rs // 内存存储
│   │       │   └── persistent.rs // 持久化存储
│   │       └── retrieval/       // 检索策略
│   │           ├── mod.rs
│   │           ├── recent.rs    // 最近记忆
│   │           └── semantic.rs  // 语义搜索
│   │
│   ├── lomusai_rag/             // RAG功能 ✅
│   │   └── src/
│   │       ├── lib.rs           // 公共API导出
│   │       ├── error.rs         // 错误处理
│   │       ├── types.rs         // RAG类型定义
│   │       ├── document/        // 文档处理
│   │       │   ├── mod.rs
│   │       │   ├── loader.rs    // 文档加载器
│   │       │   ├── parser.rs    // 文档解析器
│   │       │   └── chunker.rs   // 文档分块
│   │       ├── embedding/       // 向量嵌入
│   │       │   ├── mod.rs
│   │       │   ├── provider.rs  // 嵌入提供者特性
│   │       │   └── openai.rs    // OpenAI嵌入实现
│   │       └── retriever/       // 检索器
│   │           ├── mod.rs
│   │           ├── vector_store.rs // 向量存储接口
│   │           └── in_memory.rs // 内存向量存储
│   │
│   ├── lomusai_evals/           // 评估框架 🔄
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── metrics/         // 评估指标
│   │       │   ├── mod.rs
│   │       │   └── accuracy.rs  // 准确性指标
│   │       └── evaluator/       // 评估器
│   │           ├── mod.rs
│   │           ├── llm_eval.rs  // LLM评估
│   │           └── rule_eval.rs // 规则评估
│   │
│   ├── lomusai_integrations/    // 集成库 🔄
│   │   └── src/
│   │       ├── lib.rs
│   │       └── providers/       // 第三方服务
│   │           ├── mod.rs
│   │           └── ...
│   │
│   └── lomusai_deployer/        // 部署工具 🔄
│       └── src/
│           ├── lib.rs
│           ├── platforms/       // 部署平台
│           │   ├── mod.rs
│   │           └── ...
│   │
│   └── packaging/       // 打包工具
│       ├── mod.rs
│       └── ...
│
├── examples/                    // 示例应用
│   ├── simple_agent/            // 简单Agent示例
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   ├── workflow_demo/           // 工作流示例
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── rag_app/                 // RAG应用示例
│       ├── Cargo.toml
│       └── src/main.rs
│
├── benches/                     // 性能基准测试
│   ├── agent_bench.rs           // Agent性能测试
│   └── workflow_bench.rs        // 工作流性能测试
│
├── tests/                       // 集成测试
│   ├── agent_tests.rs           // Agent集成测试
│   └── workflow_tests.rs        // 工作流集成测试
│
├── docs/                        // 文档
│   ├── architecture.md          // 架构说明
│   ├── usage/                   // 使用指南
│   │   ├── agent.md
│   │   └── workflow.md
│   └── api/                     // API文档
│       ├── agent.md
│       └── workflow.md
│
├── Cargo.toml                   // 工作区配置
└── rust-toolchain.toml          // Rust工具链配置
```

### 4.2 核心类型定义 ✅

已在Rust中定义与TypeScript等效的类型，并利用Rust的类型系统优势：

```rust
// Agent类型 ✅
pub struct Agent {
    id: String,
    config: AgentConfig,
    tools: Vec<Box<dyn Tool>>,
    memory: Option<Box<dyn Memory>>,
    llm: Box<dyn LlmProvider>,
}

// 工作流类型 ✅
pub struct Workflow {
    id: String,
    definition: WorkflowDefinition,
    state: WorkflowState,
}

// 工具类型 ✅
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> &ToolSchema;
    fn execute(&self, params: HashMap<String, Value>) -> Result<Value, ToolError>;
}
```

### 4.3 LLM集成 ✅

```rust
pub trait LlmProvider: Send + Sync {
    fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String, LlmError>;
    fn generate_stream(&self, prompt: &str, options: &LlmOptions) -> BoxStream<'static, Result<String, LlmError>>;
    fn get_embedding(&self, text: &str) -> Result<Vec<f32>, LlmError>;
}

// 具体实现
pub struct OpenAiProvider {
    api_key: String,
    model: String,
}

pub struct AnthropicProvider {
    api_key: String,
    model: String,
}
```

### 4.4 依赖管理优化

每个模块将明确声明其依赖关系，使用特性标志(features)支持可选功能：

```toml
# lomusai_core/Cargo.toml 示例
[package]
name = "lomusai_core"
version = "0.1.0"
# ...

[features]
default = ["openai"]
openai = ["reqwest"]
anthropic = ["reqwest"]
full = ["openai", "anthropic"]

[dependencies]
# 必要依赖
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
async-trait = "0.1"
futures = "0.3"
tokio = { version = "1.0", features = ["rt", "sync"], optional = true }

# 可选依赖
reqwest = { version = "0.11", features = ["json"], optional = true }

# 内部依赖
lomusai_types = { path = "../lomusai_types", version = "0.1.0" }
```

## 5. 迁移计划时间线

| 阶段 | 估计时间 | 主要任务 | 状态 |
|------|---------|---------|------|
| 初始设计与架构 | 2周 | 设计架构，定义核心接口和类型 | ✅ 完成 |
| 核心库实现 | 8周 | 实现Agent, Workflow, Tool核心功能 | ✅ 完成 |
| RAG实现 | 4周 | 实现文档处理、向量化和检索功能 | ✅ 完成 |
| Memory实现 | 3周 | 实现记忆存储和检索系统 | ✅ 完成 |
| 集成开发 | 6周 | 开发主要第三方服务集成 | 🔄 待开始 |
| CLI与部署 | 3周 | 实现命令行工具和部署功能 | 🔄 待开始 |
| 测试与优化 | 4周 | 全面测试和性能优化 | 🔄 部分完成 |
| 文档编写 | 2周 | 编写API文档和使用指南 | 🔄 待开始 |

## 6. 性能优化目标

利用Rust的性能优势，我们设定以下优化目标：

1. **延迟降低**：关键操作延迟比TypeScript版本降低至少50%
2. **内存使用减少**：内存占用减少至少40%
3. **并发性能提升**：在多核环境下实现线性扩展
4. **启动时间优化**：冷启动时间减少至少60%

## 7. 迁移挑战与解决方案

### 7.1 挑战

1. **异步模型差异**：TypeScript和Rust的异步模型有显著差异 - ✅ 已通过使用async_trait和futures解决
2. **类型系统映射**：将TypeScript的结构类型映射到Rust的标称类型 - ✅ 已解决
3. **生态系统差异**：找到TypeScript库的Rust等效库 - ✅ 部分解决
4. **动态vs静态**：处理某些动态行为到静态类型的转换 - ✅ 部分解决，使用了trait对象
5. **模块化设计**：确保各模块之间的清晰边界和最小依赖 - ✅ 采用更好的模块化结构解决

### 7.2 解决方案

1. **异步处理**：利用Rust的async/await和tokio生态系统 ✅
2. **类型映射**：设计适合Rust的类型结构，不仅仅是1:1转换 ✅
3. **生态系统**：使用serde处理JSON，reqwest处理HTTP等 ✅
4. **动态行为**：使用枚举、trait对象和泛型提供灵活性 ✅
5. **模块化**：采用更清晰的模块边界，使用特性标志控制依赖 ✅

## 8. 测试策略

1. **单元测试**：每个模块的核心功能 ✅
2. **集成测试**：模块间交互 🔄
3. **基准测试**：性能对比与优化 🔄
4. **兼容性测试**：确保API兼容性 🔄
5. **模拟测试**：针对第三方服务的模拟测试 ✅
6. **示例应用**：完整的示例应用验证框架功能 🔄

## 9. 后续工作

1. **增强功能**：利用Rust生态系统增强现有功能
2. **跨平台支持**：编译到多种目标平台
3. **WebAssembly支持**：编译到WASM以支持浏览器环境
4. **FFI绑定**：为其他语言提供绑定
5. **性能优化**：持续优化关键路径的性能
6. **生态系统**：构建周边工具和库生态系统

## 10. 结论

通过将Mastra从TypeScript迁移到Rust，我们可以获得显著的性能提升、可靠性提高和更良好的类型安全。同时，Rust的多种编译目标也使得更广泛的部署场景成为可能。优化的模块化设计将使代码更易于维护、扩展和理解。

## 11. 已完成功能

已实现的核心组件:

1. **核心错误处理系统** ✅ - 使用thiserror提供类型安全的错误处理
2. **LLM提供者接口** ✅ - 支持OpenAI和Anthropic模型的抽象接口
   - 已修复HTTP响应所有权问题 ✅ - 确保正确处理HTTP响应的消费
   - 改进错误处理逻辑 ✅ - 使用更清晰的if-else结构区分成功和失败响应
   - 消除了未使用的导入和变量 ✅ - 提高了代码整洁度
3. **工具系统** ✅ - 可扩展的工具接口，包括参数校验和执行
4. **内存系统** ✅ - 用于存储和检索上下文信息
5. **Agent实现** ✅ - 支持工具调用和上下文管理的Agent
   - 修复了临时变量和未使用导入问题 ✅ - 提高了代码质量
   - 优化了测试实现 ✅ - 不再依赖外部HTTP模拟库，简化了测试过程
6. **工作流引擎** ✅ - 基于图的状态机实现
7. **遥测系统** ✅ - 用于记录性能和调试信息
8. **RAG系统** ✅ - 文档处理、向量嵌入和相似度搜索
   - 文档加载和解析 ✅ - 支持不同格式文档的加载和处理
   - 文档分块 ✅ - 智能分块策略将文档切分为合适的片段
   - 向量嵌入 ✅ - 生成文本的向量表示
   - 向量存储和检索 ✅ - 基于相似度的文档检索
9. **项目结构优化** ✅
   - 更新了工作区配置 ✅ - 使用Rust 2021版本推荐的resolver = "2"设置
   - 优化了模块结构 ✅ - 提高了代码可维护性和可扩展性
   - 实现了更加清晰的模块边界 ✅ - 减少了不必要的依赖

所有已实现的组件均包含单元测试，并已通过测试。下一步将继续实现Evals模块。 