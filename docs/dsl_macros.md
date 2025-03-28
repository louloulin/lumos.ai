# Lomusai DSL宏详解

Lomusai框架提供了一系列受Mastra API启发的声明式DSL宏，用于简化常见任务的定义。这些DSL宏使用Rust的过程宏系统实现，提供了优雅、直观的语法，同时保持了Rust的类型安全和编译时检查。

## 概述

DSL宏系统包含七个主要的宏：

1. `workflow!`：定义多步骤工作流
2. `rag_pipeline!`：定义RAG（检索增强生成）管道
3. `eval_suite!`：定义评估测试套件
4. `mcp_client!`：配置MCP（Mastra Compatible Protocol）客户端
5. `agent!`：定义和配置代理
6. `tools!`：一次性定义多个工具
7. `lumos!`：应用配置，类似Mastra的应用初始化方式

## 工作流DSL (workflow!)

工作流DSL提供了一种声明式方式来定义由多个步骤组成的工作流。每个步骤可以由不同的代理执行，并且可以具有条件依赖关系。

### 语法结构

```rust
let workflow = workflow! {
    name: "工作流名称",
    description: "工作流描述",
    steps: {
        {
            name: "步骤名称",
            agent: agent_instance,
            instructions: "步骤指令",
            input: { /* 初始输入 */ },
            when: { /* 条件 */ },
            timeout: 30000, // 毫秒
            retry: {
                count: 3,
                delay: 1000
            }
        },
        // 更多步骤...
    },
    options: {
        // 全局选项
    }
};
```

### 条件表达式

`when`字段支持以下条件表达式：

- `completed(step_name)`：指定步骤已完成
- `failed(step_name)`：指定步骤失败
- `output_contains(step_name, key, value)`：指定步骤的输出包含特定值
- `&& 和 ||`：逻辑AND和OR操作符

### 示例

```rust
use lomusai_core::agent::Agent;
use lumos_macro::workflow;

let content_workflow = workflow! {
    name: "content_creation",
    description: "创建高质量的内容",
    steps: {
        {
            name: "research",
            agent: researcher,
            instructions: "进行深入的主题研究",
            input: { "topic": "Rust中的并发" }
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
            when: { completed("writing") },
            retry: {
                count: 2,
                delay: 1000
            }
        },
        {
            name: "publish",
            agent: publisher,
            instructions: "发布最终文章",
            when: { 
                completed("review") && 
                output_contains("review", "approved", true) 
            }
        }
    }
};

// 执行工作流
let result = content_workflow.execute(input_data).await?;
```

## RAG管道DSL (rag_pipeline!)

RAG管道DSL提供了一种声明式方式来定义RAG（检索增强生成）处理管道，包括文档源、处理步骤和查询配置。

### 语法结构

```rust
let pipeline = rag_pipeline! {
    name: "管道名称",
    
    // 单一数据源
    source: DocumentSource::from_directory("路径"),
    
    // 或多个数据源
    sources: [
        DocumentSource::from_directory("路径1"),
        DocumentSource::from_url("URL"),
        // 更多数据源...
    ],
    
    pipeline: {
        // 预处理配置
        preprocess: {
            remove_headers: true,
            normalize_whitespace: true,
            // 更多预处理选项...
        },
        
        // 分块配置
        chunk: {
            chunk_size: 1000,
            chunk_overlap: 200,
            separator: "\n\n",
            strategy: "recursive" // 或 "sentence", "token" 等
        },
        
        // 嵌入配置
        embed: {
            model: "模型名称",
            dimensions: 1536,
            batch_size: 32,
            max_retries: 3
        },
        
        // 存储配置
        store: {
            db: "存储类型", // "memory", "pgvector", "pinecone" 等
            collection: "集合名称",
            connection_string: "连接字符串或环境变量",
            options: {
                // 存储特定选项
            }
        }
    },
    
    // 查询配置
    query_pipeline: {
        rerank: true, // 或详细配置
        top_k: 5,
        filter: "过滤条件",
        hybrid_search: {
            enabled: true,
            weight: {
                semantic: 0.7,
                keyword: 0.3
            }
        }
    }
};
```

### 示例

```rust
use lomusai_core::rag::DocumentSource;
use lumos_macro::rag_pipeline;

let kb = rag_pipeline! {
    name: "api_docs",
    
    source: DocumentSource::from_directory("./docs/api"),
    
    pipeline: {
        chunk: {
            chunk_size: 1000,
            chunk_overlap: 200,
            strategy: "recursive"
        },
        
        embed: {
            model: "text-embedding-3-small",
            dimensions: 1536
        },
        
        store: {
            db: "memory",
            collection: "api_embeddings"
        }
    },
    
    query_pipeline: {
        rerank: true,
        top_k: 5
    }
};

// 执行查询
let results = kb.query("如何使用Lomusai工具?").await?;
```

## 评估套件DSL (eval_suite!)

评估套件DSL提供了一种声明式方式来定义评估测试套件，用于评估代理或LLM的性能。

### 语法结构

```rust
let suite = eval_suite! {
    name: "套件名称",
    
    // 评估指标
    metrics: {
        metric_name: MetricImplementation,
        // 更多指标...
        custom: {
            name: "自定义指标名称",
            implementation: |response, expected| -> f64 {
                // 自定义实现
                1.0
            }
        }
    },
    
    // 测试用例
    test_cases: [
        {
            query: "查询",
            expected: "期望结果",
            weight: 1.0
        },
        // 更多测试用例...
    ],
    
    // 或从文件加载测试用例
    test_cases: {
        source: "文件路径",
        filter: "过滤条件"
    },
    
    // 阈值设置
    thresholds: {
        metric_name: 0.8,
        // 更多阈值...
    },
    
    // 报告配置
    reporting: {
        format: "格式", // "markdown", "html", "json" 等
        output: "输出路径",
        include_responses: true
    },
    
    // 其他选项
    options: {
        parallel: true,
        timeout: 30000,
        retry_count: 2
    }
};
```

### 示例

```rust
use lomusai_core::eval::{AccuracyMetric, RelevanceMetric};
use lumos_macro::eval_suite;

let suite = eval_suite! {
    name: "agent_performance",
    
    metrics: {
        accuracy: AccuracyMetric::new(0.8),
        relevance: RelevanceMetric::new(0.7),
        custom: {
            name: "brevity",
            implementation: |response, _| -> f64 {
                if response.len() < 200 { 1.0 } else { 0.5 }
            }
        }
    },
    
    test_cases: [
        {
            query: "Rust的所有权规则是什么?",
            expected: "所有权,借用,生命周期",
            weight: 1.0
        },
        {
            query: "如何在Rust中处理错误?",
            expected: "Result,Option,?运算符",
            weight: 0.8
        }
    ],
    
    thresholds: {
        accuracy: 0.8,
        relevance: 0.7,
        brevity: 0.9
    },
    
    reporting: {
        format: "markdown",
        output: "./reports/eval_results.md"
    }
};

// 运行评估
let results = suite.run(agent).await?;
```

## MCP客户端DSL (mcp_client!)

MCP客户端DSL提供了一种声明式方式来配置MCP（Mastra Compatible Protocol）客户端，用于与外部工具和API集成。

### 语法结构

```rust
let client = mcp_client! {
    // 发现配置
    discovery: {
        endpoints: ["端点URL"],
        auto_register: true,
        interval: 60, // 秒
        registry: {
            url: "注册中心URL",
            auth: {
                type: "认证类型",
                token_env: "TOKEN_ENV_VAR"
            }
        }
    },
    
    // 工具配置
    tools: {
        tool_name: {
            enabled: true,
            auth: {
                type: "认证类型",
                key_env: "API_KEY_ENV_VAR"
            },
            rate_limit: 100,
            version: "版本",
            options: {
                // 工具特定选项
            }
        },
        // 更多工具...
    },
    
    // 缓存配置
    cache: {
        enabled: true,
        ttl: 3600, // 秒
        max_size: 100 // MB
    },
    
    // 默认选项
    defaults: {
        timeout: 30000, // 毫秒
        retry: {
            count: 3,
            backoff: "exponential"
        }
    },
    
    // 日志配置
    logging: {
        level: "info",
        format: "json",
        destination: "file",
        path: "日志路径"
    },
    
    // 代理配置
    proxy: {
        url: "代理URL",
        auth: {
            username_env: "USERNAME_ENV_VAR",
            password_env: "PASSWORD_ENV_VAR"
        },
        bypass: ["localhost", "127.0.0.1"]
    }
};
```

### 示例

```rust
use lumos_macro::mcp_client;

let client = mcp_client! {
    discovery: {
        endpoints: ["https://api.mcp.example.com"],
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
        image_generation: {
            enabled: true,
            rate_limit: 100
        }
    },
    
    cache: {
        enabled: true,
        ttl: 3600
    },
    
    defaults: {
        timeout: 30000,
        retry: {
            count: 3,
            backoff: "exponential"
        }
    }
};

// 获取可用工具
let tools = client.get_available_tools().await?;

// 执行工具
let result = client.execute_tool("data_analysis", params).await?;
```

## 与JSON定义的比较

这些DSL宏与使用JSON进行配置相比具有以下优势：

1. **类型安全**：DSL宏在编译时进行类型检查，避免运行时错误
2. **IDE支持**：更好的代码补全和语法高亮
3. **编译时验证**：配置错误在编译时即可发现
4. **性能优化**：编译为本地代码，无需在运行时解析JSON
5. **Rust集成**：可以直接使用Rust变量、表达式和代码块

## 在项目中启用DSL宏

要在项目中使用DSL宏，需要在`Cargo.toml`中启用`macros`特性：

```toml
[dependencies]
lomusai_core = { version = "0.1.0", features = ["macros"] }
```

然后在代码中导入相应的宏：

```rust
use lumos_macro::{
    workflow, 
    rag_pipeline, 
    eval_suite, 
    mcp_client,
    agent,
    tools,
    lumos
};
```

## 代理DSL (agent!)

代理DSL提供了一种声明式方式来定义和配置代理，包括LLM提供者、内存系统和工具集成。

### 语法结构

```rust
let agent = agent! {
    name: "代理名称",
    instructions: "代理指令",
    
    // LLM配置
    llm: {
        provider: llm_provider_instance,
        model: "模型名称",
        options: {
            // LLM特定选项
        }
    },
    
    // 内存配置（可选）
    memory: {
        store_type: "内存类型", // "buffer", "vector", "db" 等
        capacity: 10,
        options: {
            // 内存特定选项
        }
    },
    
    // 工具配置
    tools: {
        tool_name,
        tool_with_options: { 
            // 工具特定选项
        },
        // 更多工具...
    }
};
```

### 示例

```rust
use lomusai_core::llm::OpenAiAdapter;
use lumos_macro::agent;

let agent = agent! {
    name: "research_assistant",
    instructions: "你是一个专业的研究助手，擅长收集和整理信息。",
    
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
        web_browser: { javascript: true, screenshots: true }
    }
};

// 使用代理
let response = agent.run("帮我查找关于量子计算的最新研究").await?;
```

## 工具集DSL (tools!)

工具集DSL提供了一种声明式方式来一次性定义多个工具，简化工具创建过程。

### 语法结构

```rust
tools! {
    {
        name: "工具名称",
        description: "工具描述",
        parameters: {
            {
                name: "参数名称",
                description: "参数描述",
                type: "参数类型", // "string", "number", "boolean", "object", "array" 等
                required: true,
                default: "默认值" // 可选
            },
            // 更多参数...
        },
        handler: |params| async move {
            // 工具实现代码
            Ok(json!({ "result": "结果" }))
        }
    },
    // 更多工具...
}
```

### 示例

```rust
use lomusai_core::Error;
use lumos_macro::tools;
use serde_json::json;

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
                "divide" => if b == 0.0 {
                    return Err(Error::InvalidInput("Cannot divide by zero".into()))
                } else {
                    a / b
                },
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
        handler: get_weather_data // 使用预定义函数
    }
}

// 使用生成的工具
let calc_result = calculator().execute(calc_params, &options).await?;
let weather_info = weather().execute(weather_params, &options).await?;
```

## Lumos应用配置 (lumos!)

Lumos应用DSL提供了一种声明式方式来一次性配置整个应用，类似于Mastra的应用初始化方式。

### 语法结构

```rust
let app = lumos! {
    agents: {
        agent_name: agent_instance,
        // 更多代理...
    },
    
    tools: {
        tool_name: tool_instance,
        // 更多工具...
    },
    
    workflows: workflow_collection,
    
    rag: rag_instance,
    
    mcp: mcp_client_instance
};
```

### 示例

```rust
use lomusai_core::llm::OpenAiAdapter;
use lumos_macro::{agent, tools, lumos};

// 定义工具
tools! {
    {
        name: "stock_checker",
        description: "查询股票信息",
        parameters: {
            {
                name: "symbol",
                description: "股票代码",
                type: "string",
                required: true
            }
        },
        handler: |params| async move {
            let symbol = params.get("symbol").unwrap().as_str().unwrap();
            
            // 获取股票数据的实现
            Ok(json!({ "price": 175.42, "change": "+2.3%" }))
        }
    },
    {
        name: "weather_checker",
        description: "查询天气信息",
        parameters: {
            {
                name: "city",
                description: "城市名称",
                type: "string",
                required: true
            }
        },
        handler: |params| async move {
            let city = params.get("city").unwrap().as_str().unwrap();
            
            // 获取天气数据的实现
            Ok(json!({ "temperature": 25, "condition": "晴朗" }))
        }
    }
}

// 定义代理
let stock_agent = agent! {
    name: "stock_assistant",
    instructions: "你是一个股票助手，可以提供股票市场信息。",
    
    llm: {
        provider: OpenAiAdapter::new("api-key"),
        model: "gpt-4"
    },
    
    tools: {
        stock_checker
    }
};

let weather_agent = agent! {
    name: "weather_assistant",
    instructions: "你是一个天气助手，可以提供天气预报信息。",
    
    llm: {
        provider: OpenAiAdapter::new("api-key"),
        model: "gpt-4"
    },
    
    tools: {
        weather_checker
    }
};

// 使用lumos!宏一次性配置整个应用
let app = lumos! {
    agents: {
        stockAgent: stock_agent,
        weatherAgent: weather_agent
    },
    tools: {
        stockChecker: stock_checker(),
        weatherChecker: weather_checker()
    },
    rag: kb,
    mcp: mcp_client
};

// 使用应用处理请求
let result = app.run("查询苹果公司的股票价格").await?;
```

这种方式的配置与JavaScript中的Mastra应用初始化非常相似:

```javascript
export const mastra = new Mastra({
  agents: { stockAgent, weatherAgent },
  tools: { stockChecker, weatherChecker },
  workflows: [contentWorkflow],
  rag: knowledgeBase,
  mcp: mcpClient
});
```