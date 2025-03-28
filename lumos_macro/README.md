# Lumos Macro

Lumos Macro是Lomusai框架的一部分，提供了一系列过程宏，用于简化Lomusai框架中工具、代理和LLM适配器的定义和使用。

## 特性

### 基础宏
- `#[tool]`：简化工具定义，自动生成Tool trait实现
- `#[agent]`：定义代理及其工具，自动处理工具注册
- `#[derive(LlmAdapter)]`：为自定义LLM适配器生成默认实现
- `lumos_execute_tool!`：快速执行工具的宏

### 受Mastra API启发的DSL宏
- `workflow!`：工作流定义DSL，支持步骤、条件和代理集成
- `rag_pipeline!`：RAG管道DSL，简化文档处理、嵌入和向量检索
- `eval_suite!`：评估框架DSL，用于定义指标、测试用例和报告格式
- `mcp_client!`：MCP客户端配置DSL，简化外部工具集成
- `agent!`：代理定义DSL，简化代理创建和配置
- `tools!`：工具集合DSL，一次性定义多个工具
- `lumos!`：应用配置DSL，类似Mastra的应用初始化方式

## 安装

将下面的依赖添加到项目的Cargo.toml文件中：

```toml
[dependencies]
lomusai_core = { version = "0.1.0", features = ["macros"] }
```

核心的`macros`特性会自动包含`lumos_macro`库。

## 使用示例

### 工具定义 (使用#[tool]宏)

```rust
use lomusai_core::{Error, Result};
use serde_json::{Value, json};
use lumos_macro::tool;

#[tool(
    name = "calculator",
    description = "执行基本的数学运算"
)]
fn calculator(
    #[parameter(
        name = "operation",
        description = "要执行的操作: add, subtract, multiply, divide",
        r#type = "string", 
        required = true
    )]
    operation: String,
    
    #[parameter(
        name = "a",
        description = "第一个数字",
        r#type = "number",
        required = true
    )]
    a: f64,
    
    #[parameter(
        name = "b",
        description = "第二个数字",
        r#type = "number",
        required = true
    )]
    b: f64,
) -> Result<Value> {
    let result = match operation.as_str() {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return Err(Error::InvalidInput("Cannot divide by zero".to_string()));
            }
            a / b
        },
        _ => return Err(Error::InvalidInput(format!("Unknown operation: {}", operation))),
    };
    
    Ok(json!({ "result": result }))
}
```

宏展开后，会生成一个返回`Box<dyn Tool>`的函数，可以直接用于工具注册。

### 代理定义 (使用agent!宏)

```rust
use std::sync::Arc;
use lomusai_core::llm::LlmProvider;
use lumos_macro::agent;

#[agent(
    name = "math_agent",
    instructions = "你是一个能够执行数学计算的助手。",
    model = "gpt-4"
)]
struct MathAgent {
    #[tool]
    calculator: calculator,
    
    #[tool]
    unit_converter: unit_converter,
}
```

宏展开后，会生成一个名为`create_mathagent`的函数，接受`Arc<dyn LlmProvider>`参数，返回配置好的代理实例。

### LLM适配器 (使用#[derive(LlmAdapter)]宏)

```rust
use async_trait::async_trait;
use lomusai_core::{Result, Message, Role, Error};
use lomusai_core::llm::{LlmProvider, LlmOptions};
use lumos_macro::LlmAdapter;

#[derive(LlmAdapter)]
struct CustomLlmAdapter {
    api_key: String,
    model: String,
}

impl CustomLlmAdapter {
    fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

#[async_trait]
impl LlmProvider for CustomLlmAdapter {
    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        // 实现自定义的消息生成逻辑
        Ok("这是一个模拟的LLM响应".to_string())
    }
}
```

宏展开后，会为`LlmProvider` trait的其他方法提供默认实现，你只需要实现`generate_with_messages`方法即可。

### 快速执行工具 (使用lumos_execute_tool!宏)

```rust
use lumos_macro::lumos_execute_tool;

async fn main() -> Result<()> {
    let result = lumos_execute_tool! {
        tool: calculator,
        params: {
            "operation": "add",
            "a": 10.5,
            "b": 20.3
        }
    };
    
    println!("结果: {}", result);
    Ok(())
}
```

## 新增DSL宏

### 工作流定义 (使用workflow!宏)

```rust
use lumos_macro::workflow;
use lomusai_core::agent::Agent;

let content_workflow = workflow! {
    name: "content_creation",
    description: "创建高质量的内容",
    steps: {
        {
            name: "research",
            agent: researcher,
            instructions: "进行深入的主题研究",
        },
        {
            name: "writing",
            agent: writer,
            instructions: "将研究结果整理成文章",
            when: { completed("research") },
        },
        {
            name: "review",
            agent: reviewer,
            instructions: "检查文章质量和准确性",
            when: { completed("writing") },
        }
    }
};

// 执行工作流
let result = content_workflow.execute(input_data).await?;
```

### RAG管道 (使用rag_pipeline!宏)

```rust
use lumos_macro::rag_pipeline;
use lomusai_core::rag::DocumentSource;

let kb = rag_pipeline! {
    name: "knowledge_base",
    
    source: DocumentSource::from_directory("./docs"),
    
    pipeline: {
        chunk: {
            chunk_size: 1000,
            chunk_overlap: 200,
            separator: "\n",
            strategy: "recursive"
        },
        
        embed: {
            model: "text-embedding-3-small",
            dimensions: 1536,
            max_retries: 3
        },
        
        store: {
            db: "pgvector",
            collection: "embeddings",
            connection_string: env!("DATABASE_URL")
        }
    },
    
    query_pipeline: {
        rerank: true,
        top_k: 5,
        filter: r#"{ "type": { "$in": ["article", "faq"] } }"#
    }
};

// 执行查询
let results = kb.query("如何使用RAG?").await?;
```

### 评估套件 (使用eval_suite!宏)

```rust
use lumos_macro::eval_suite;
use lomusai_core::eval::{AccuracyMetric, RelevanceMetric, CompletenessMetric};

let suite = eval_suite! {
    name: "agent_performance",
    
    metrics: {
        accuracy: AccuracyMetric::new(0.8),
        relevance: RelevanceMetric::new(0.7),
        completeness: CompletenessMetric::new(0.6)
    },
    
    test_cases: {
        basic_queries: "./tests/basic_queries.json",
        complex_queries: "./tests/complex_queries.json"
    },
    
    reporting: {
        format: "html",
        output: "./reports/eval_results.html"
    }
};

// 运行评估
let results = suite.run(agent).await?;
```

### MCP客户端 (使用mcp_client!宏)

```rust
use lumos_macro::mcp_client;

let client = mcp_client! {
    discovery: {
        endpoints: ["https://tools.example.com/mcp", "https://api.mcp.run"],
        auto_register: true
    },
    
    tools: {
        data_analysis: {
            enabled: true,
            auth: {
                type: "api_key",
                key_env: "DATA_ANALYSIS_API_KEY"
            }
        },
        image_processing: {
            enabled: true,
            rate_limit: 100
        }
    }
};

// 获取可用的MCP工具
let tools = client.get_available_tools().await?;
```

### 代理定义 (使用agent!宏)

```rust
use lumos_macro::agent;

let agent = agent! {
    name: "research_assistant",
    instructions: "你是一个专业的研究助手，擅长收集和整理信息。",
    
    llm: {
        provider: openai_adapter,
        model: "gpt-4"
    },
    
    memory: {
        store_type: "buffer",
        capacity: 10
    },
    
    tools: {
        search_tool,
        calculator_tool: { precision: 2 },
        web_browser: { javascript: true, screenshots: true }
    }
};

// 使用代理处理请求
let response = agent.run("帮我查找关于量子计算的最新研究").await?;
```

### 工具集合定义 (使用tools!宏)

```rust
use lumos_macro::tools;

tools! {
    {
        name: "calculator",
        description: "执行基本的数学运算",
        parameters: {
            {
                name: "operation",
                description: "要执行的操作: add, subtract, multiply, divide",
                type: "string",
                required: true
            },
            {
                name: "a",
                description: "第一个数字",
                type: "number",
                required: true
            },
            {
                name: "b",
                description: "第二个数字",
                type: "number",
                required: true
            }
        },
        handler: |params| async move {
            let operation = params.get("operation").unwrap().as_str().unwrap();
            let a = params.get("a").unwrap().as_f64().unwrap();
            let b = params.get("b").unwrap().as_f64().unwrap();
            
            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => a / b,
                _ => return Err(Error::InvalidInput("Unknown operation".into()))
            };
            
            Ok(json!({ "result": result }))
        }
    },
    {
        name: "weather",
        description: "获取指定城市的天气信息",
        parameters: {
            {
                name: "city",
                description: "城市名称",
                type: "string",
                required: true
            }
        },
        handler: get_weather_data
    }
}

// 使用工具
let result = calculator().execute(params, &options).await?;
let weather = weather().execute(params, &options).await?;
```

### Lumos应用配置 (使用lumos!宏)

```rust
use lumos_macro::lumos;

let app = lumos! {
    agents: {
        stockAgent: stock_agent,
        weatherAgent: weather_agent
    },
    tools: {
        stockChecker: stock_checker(),
        weatherChecker: weather_checker()
    },
    workflows: vec![content_workflow, research_workflow],
    rag: knowledge_base,
    mcp: mcp_client
};

// 使用应用处理请求
let result = app.run("查询苹果公司的股票").await?;
```

## 与Mastra API的比较

Lumos宏的设计受到了Mastra API的启发，提供了类似的声明式API，但专为Rust语言和Lomusai框架量身定制。相比于Mastra的JavaScript API，Lumos宏利用了Rust的强类型系统和编译时检查，以提供更安全和高效的代码。

新增的DSL宏直接受到Mastra的工作流、RAG、评估和MCP功能的启发，提供了相似的声明式语法，但保持了Rust语言的特性和安全性。

## 贡献

欢迎贡献代码、报告问题或提出新功能建议。在提交PR前，请确保通过所有测试并遵循项目的代码风格。

## 许可证

MIT 