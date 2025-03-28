# Lomusai - Rust语言的AI Agent框架

Lomusai是一个用Rust实现的AI Agent框架，专注于性能、安全性和可扩展性。它提供了创建、管理和部署智能代理的工具和抽象，使开发者能够轻松构建高效的AI应用。

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

- `lomusai_core`：核心库，包含基本抽象和接口
  - `agent`：Agent trait和实现
  - `tool`：Tool trait和实现
  - `memory`：内存和状态管理
  - `llm`：LLM适配器和抽象
  - `eval`：评估和测试框架
  - `rag`：检索增强生成支持
  - `mcp`：MCP（Mastra Compatible Protocol）支持
- `lumos_macro`：宏库，提供简化API使用的过程宏
  - 基础宏：`#[tool]`、`#[agent]`、`#[derive(LlmAdapter)]`等
  - DSL宏：`workflow!`、`rag_pipeline!`、`eval_suite!`、`mcp_client!`等
- `examples`：示例代码
- `docs`：文档

## 快速开始

添加依赖到你的`Cargo.toml`：

```toml
[dependencies]
lomusai_core = "0.1.0"
```

若要使用宏功能，启用`macros`特性：

```toml
[dependencies]
lomusai_core = { version = "0.1.0", features = ["macros"] }
```

### 基础使用示例

```rust
use lomusai_core::{Result, Error};
use lomusai_core::agent::{Agent, SimpleAgent};
use lomusai_core::tool::{Tool, FunctionTool};
use lomusai_core::llm::{LlmProvider, OpenAiAdapter};
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
use lomusai_core::{Result, Error};
use lomusai_core::llm::OpenAiAdapter;
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
use lomusai_core::{Result, Error};
use lomusai_core::agent::Agent;
use lomusai_core::llm::OpenAiAdapter;
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

## 文档

查看[完整文档](docs/index.md)了解更多详情。

## 贡献

欢迎贡献代码、报告问题或提出新功能建议。在提交PR前，请确保通过所有测试并遵循项目的代码风格。

## 许可证

MIT 