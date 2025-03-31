# Lumosai - Rust语言的AI Agent框架

Lumosai是一个用Rust实现的AI Agent框架，专注于性能、安全性和可扩展性。它提供了创建、管理和部署智能代理的工具和抽象，使开发者能够轻松构建高效的AI应用。

## 主要特性

- **高性能**：使用Rust语言实现，提供优秀的性能和内存安全
- **模块化设计**：核心框架、工具库和适配器的清晰分离
- **类型安全**：利用Rust的类型系统确保API使用的正确性
- **灵活扩展**：支持自定义工具、代理和LLM适配器
- **异步优先**：从设计之初就支持异步操作
- **内存管理**：提供多种内存存储选项
- **宏支持**：通过过程宏简化API使用
- **DSL语法**：提供受Mastra启发的声明式DSL，简化工作流、RAG、评估和MCP集成

## 项目结构

- `lumosai_core`：核心库，包含基本抽象和接口
  - `agent`：Agent trait和实现
  - `tool`：Tool trait和实现
  - `memory`：内存和状态管理
  - `llm`：LLM适配器和抽象
  - `eval`：评估和测试框架
  - `rag`：检索增强生成支持
  - `mcp`：MCP（Mastra Compatible Protocol）支持
- `lumosai_rag`：检索增强生成库，提供扩展的RAG功能
  - `embedding`：向量嵌入支持
  - `chunking`：文档分块策略
  - `retrieval`：检索算法和优化
  - `reranking`：结果重排序
- `lumosai_evals`：评估和测试框架，提供全面的评估工具
  - `metrics`：预定义和自定义评估指标
  - `evaluators`：评估器实现
  - `reporting`：结果报告和可视化
- `lumosai_examples`：示例代码，展示框架使用方法
  - 基础示例：代理创建、工具使用等
  - 工作流示例：多代理协作和流程编排
  - DSL示例：各种DSL的使用方法
- `lumos_macro`：宏库，提供简化API使用的过程宏
  - 基础宏：`#[tool]`、`#[agent]`、`#[derive(LlmAdapter)]`等
  - DSL宏：`workflow!`、`rag_pipeline!`、`eval_suite!`、`mcp_client!`等
- `docs`：文档

## 安装

添加依赖到你的`Cargo.toml`：

```toml
[dependencies]
lumosai_core = "0.1.0"
```

若要使用宏功能，启用`macros`特性：

```toml
[dependencies]
lumosai_core = { version = "0.1.0", features = ["macros"] }
lumos_macro = "0.1.0"
```

若要使用RAG或评估功能：

```toml
[dependencies]
lumosai_core = "0.1.0"
lumosai_rag = "0.1.0"
lumosai_evals = "0.1.0"
```

## 快速开始

### 基础使用示例

```rust
use lumosai_core::{Result, Error};
use lumosai_core::agent::{Agent, SimpleAgent};
use lumosai_core::tool::{Tool, FunctionTool};
use lumosai_core::llm::{LlmProvider, OpenAiAdapter};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建LLM适配器
    let llm = Arc::new(OpenAiAdapter::new(
        "your-api-key",
        "gpt-4",
    ));
    
    // 创建工具
    let calculator = FunctionTool::new(
        "calculator",
        "执行基础数学计算",
        |params| async move {
            // 工具实现...
            Ok(serde_json::json!({"result": 42}))
        },
    );
    
    // 创建代理
    let mut agent = SimpleAgent::new(
        "math_helper",
        "你是一个擅长数学的助手。",
        llm,
    );
    
    // 注册工具
    agent.add_tool(calculator);
    
    // 运行代理
    let response = agent.run("计算 (15 + 27) * 2").await?;
    println!("代理回答: {}", response);
    
    Ok(())
}
```

### 使用宏的简化示例

```rust
use lumosai_core::{Result, Error};
use lumosai_core::llm::OpenAiAdapter;
use lumos_macro::{tool, agent};
use std::sync::Arc;

// 使用宏定义工具
#[tool(
    name = "calculator",
    description = "执行基础数学计算"
)]
fn calculator(
    #[parameter(name = "a", description = "第一个数字", r#type = "number")]
    a: f64,
    #[parameter(name = "b", description = "第二个数字", r#type = "number")]
    b: f64,
    #[parameter(name = "operation", description = "运算符", r#type = "string")]
    operation: String,
) -> Result<serde_json::Value> {
    // 工具实现...
    Ok(serde_json::json!({"result": 42}))
}

// 使用宏定义代理
#[agent(
    name = "math_helper",
    instructions = "你是一个擅长数学的助手。",
    model = "gpt-4"
)]
struct MathHelper {
    #[tool]
    calculator: calculator,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 创建LLM适配器
    let llm = Arc::new(OpenAiAdapter::new(
        "your-api-key",
        "gpt-4",
    ));
    
    // 创建代理
    let agent = create_mathhelper(llm);
    
    // 运行代理
    let response = agent.run("计算 (15 + 27) * 2").await?;
    println!("代理回答: {}", response);
    
    Ok(())
}
```

### 使用DSL宏示例

```rust
use lumosai_core::{Result, Error};
use lumosai_core::agent::Agent;
use lumosai_core::llm::OpenAiAdapter;
use lumos_macro::{workflow, rag_pipeline};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建LLM适配器
    let llm = Arc::new(OpenAiAdapter::new(
        "your-api-key",
        "gpt-4",
    ));
    
    // 创建代理
    let researcher = create_researcher(llm.clone());
    let writer = create_writer(llm.clone());
    let reviewer = create_reviewer(llm.clone());
    
    // 定义知识库
    let kb = rag_pipeline! {
        name: "documentation_kb",
        source: DocumentSource::from_directory("./docs"),
        pipeline: {
            chunk: { chunk_size: 1000, chunk_overlap: 200 },
            embed: { model: "text-embedding-3-small" },
            store: { db: "memory" }
        }
    };
    
    // 定义工作流
    let content_workflow = workflow! {
        name: "content_creation",
        description: "创建高质量的内容",
        steps: {
            {
                name: "research",
                agent: researcher,
                instructions: "使用知识库进行深入的主题研究",
                context: { knowledge_base: kb }
            },
            {
                name: "writing",
                agent: writer,
                instructions: "将研究结果整理成文章",
                when: { completed("research") }
            },
            {
                name: "review",
                agent: reviewer,
                instructions: "检查文章质量和准确性",
                when: { completed("writing") }
            }
        }
    };
    
    // 执行工作流
    let result = content_workflow.execute(serde_json::json!({
        "topic": "Rust中的智能指针"
    })).await?;
    
    println!("工作流执行结果: {}", result);
    
    Ok(())
}
```

## 示例

请参阅 `lumosai_examples` 目录中的示例程序，了解更多使用方法。可以通过以下命令运行示例：

```bash
cargo run --example basic_usage
cargo run --example agent_usage
cargo run --example workflow_example
```

示例包括：

- `basic_usage` - 基础框架使用
- `agent_usage` - 代理创建和使用
- `agent_tools` - 代理工具实现
- `workflow_example` - 工作流示例
- `workflow_dsl` - 工作流DSL使用
- `rag_dsl` - RAG功能示例
- `eval_dsl` - 评估框架示例
- `mcp_dsl` - MCP集成示例
- `lumos_app` - 应用程序框架
- `lumos_macro_usage` - 宏使用示例
- `macro_tool_example` - 工具宏示例

## 核心功能

### Agent

Agent是框架的核心概念，代表一个能够执行任务的智能体：

```rust
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn instructions(&self) -> &str;
    fn add_tool(&mut self, tool: Box<dyn Tool>);
    async fn run(&self, input: &str) -> Result<String>;
    async fn run_with_memory(&self, input: &str, memory: Box<dyn Memory>) -> Result<String>;
}
```

### Tool

Tool代表代理可以使用的工具或功能：

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> &[Parameter];
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value>;
}
```

### Memory

Memory提供状态管理和持久化能力：

```rust
pub trait Memory: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn set(&self, key: &str, value: &str) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn append(&self, key: &str, value: &str) -> Result<()>;
}
```

### LlmProvider

LlmProvider抽象了与大语言模型的交互：

```rust
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, messages: &[Message], options: &GenerateOptions) -> Result<String>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}
```

## 扩展功能

### RAG (检索增强生成)

Lumosai提供了完整的RAG支持，包括：

- 文档加载和处理
- 向量嵌入生成
- 向量存储和检索
- 结果重排序和优化

```rust
let rag_pipeline = rag_pipeline! {
    name: "knowledge_base",
    source: {
        type: "directory",
        path: "./docs",
        pattern: "**/*.md"
    },
    pipeline: {
        chunk: {
            size: 1000,
            overlap: 200
        },
        embed: {
            model: "text-embedding-3-small"
        },
        store: {
            type: "memory"
        }
    }
};

let results = rag_pipeline.query("如何使用Rust的所有权系统？", 5).await?;
```

### 评估框架

Lumosai提供了评估代理性能的工具：

```rust
let eval_suite = eval_suite! {
    name: "agent_performance",
    metrics: {
        accuracy: AccuracyMetric,
        relevance: RelevanceMetric,
        completeness: CompletenessMetric
    },
    test_cases: [
        {
            query: "Rust的特点是什么？",
            expected: "内存安全,并发,性能",
            weight: 1.0
        }
    ],
    thresholds: {
        accuracy: 0.8,
        relevance: 0.7,
        completeness: 0.6
    }
};

let results = eval_suite.run(agent).await?;
```

### 工作流

Lumosai支持定义复杂的多代理工作流：

```rust
let workflow = workflow! {
    name: "content_creation",
    description: "创建高质量的内容",
    steps: {
        {
            name: "research",
            agent: researcher,
            instructions: "进行主题研究"
        },
        {
            name: "writing",
            agent: writer,
            instructions: "撰写内容",
            when: { completed("research") }
        }
    }
};

let result = workflow.execute(input_data).await?;
```

## 贡献指南

我们欢迎各种形式的贡献，包括但不限于：

- 代码贡献
- 文档改进
- 错误报告
- 功能建议

### 贡献流程

1. Fork 项目仓库
2. 创建您的特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交您的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启一个Pull Request

### 代码规范

- 遵循Rust标准编码风格
- 所有代码必须通过 `cargo clippy` 和 `cargo fmt` 检查
- 添加适当的测试覆盖率
- 保持代码文档的完整性

## 许可证

本项目采用MIT许可证 - 详见 [LICENSE](LICENSE) 文件 