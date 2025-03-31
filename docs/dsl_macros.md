# Lumosai DSL宏详解

Lumosai框架提供了一系列受Mastra API启发的声明式DSL宏，用于简化常见任务的定义。这些DSL宏使用Rust的过程宏系统实现，提供了优雅、直观的语法，同时保持了Rust的类型安全和编译时检查。

## 概述

DSL宏系统包含七个主要的宏：

1. `workflow!`：定义多步骤工作流
2. `rag_pipeline!`：定义RAG（检索增强生成）管道
3. `eval_suite!`：定义评估测试套件
4. `mcp_client!`：配置MCP（Mastra Compatible Protocol）客户端
5. `agent!`：定义和配置代理
6. `tools!`：一次性定义多个工具
7. `lumos!`：应用级配置，整合所有组件

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
use lumosai_core::agent::Agent;
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
use lumosai_core::rag::DocumentSource;
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
let results = kb.query("如何使用Lumosai工具?").await?;
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
use lumosai_core::eval::{AccuracyMetric, RelevanceMetric};
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
lumosai_core = { version = "0.1.0", features = ["macros"] }
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
use lumosai_core::llm::OpenAiAdapter;
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
use lumosai_core::Error;
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

`lumos!`宏是Lumosai框架中最高层次的DSL，提供了一种声明式方式来配置整个应用程序。它允许开发者在一个地方整合所有组件，包括代理、工具、RAG系统、工作流和MCP端点，从而简化应用开发和配置过程。

### 语法结构

```rust
let app = lumos! {
    name: "应用名称",
    description: "应用描述",
    
    // 定义或引用应用中使用的代理
    agents: {
        agent_name1,
        agent_name2: agent_expression2,
        // 或使用agent!宏内联定义
        agent_name3: agent! {
            name: "内联代理",
            instructions: "代理指令",
            // ...代理配置
        },
        // 更多代理...
    },
    
    // 定义或引用应用中使用的工具
    tools: {
        tool_name1,
        tool_name2: tool_expression2,
        // 更多工具...
    },
    
    // 定义或引用应用中使用的RAG系统
    rags: {
        rag_name1,
        rag_name2: rag_expression2,
        // 或使用rag_pipeline!宏内联定义
        rag_name3: rag_pipeline! {
            name: "内联RAG",
            // ...RAG配置
        },
        // 更多RAG...
    },
    
    // 定义或引用应用中使用的工作流
    workflows: {
        workflow_name1,
        workflow_name2: workflow_expression2,
        // 更多工作流...
    },
    
    // 定义MCP端点
    mcp_endpoints: ["端点URL1", "端点URL2"],
    
    // 全局选项和配置
    options: {
        // 日志配置
        logging: {
            level: "info", // "debug", "info", "warn", "error"
            format: "json", // "text", "json"
            destination: "stdout" // "stdout", "file", "both"
        },
        
        // 安全配置
        security: {
            auth_required: true,
            auth_provider: auth_provider_instance,
            rate_limiting: {
                requests_per_minute: 100,
                burst: 20
            }
        },
        
        // 性能配置
        performance: {
            concurrent_requests: 10,
            timeout: 30000, // 毫秒
            cache: {
                enabled: true,
                ttl: 3600 // 秒
            }
        },
        
        // 观测性配置
        observability: {
            metrics: true,
            tracing: true,
            exporters: ["prometheus", "jaeger"]
        }
    }
};

// 启动应用
app.start().await?;

// 或处理单个请求
let response = app.run("用户请求").await?;
```

### 关键功能

1. **组件整合**：在一个地方管理所有应用组件
2. **依赖管理**：自动处理组件之间的依赖关系
3. **资源优化**：共享LLM提供者和内存系统等资源
4. **配置验证**：在编译时验证配置的完整性和一致性
5. **启动管理**：提供统一的接口来启动和停止应用
6. **请求路由**：自动将请求路由到适当的代理或工作流
7. **健康检查**：内置应用组件的健康监控

### 完整示例

以下是一个完整的股票助手应用示例，展示如何使用`lumos!`宏整合多个组件：

```rust
use lumosai_core::{
    llm::OpenAiAdapter,
    rag::DocumentSource,
    memory::BufferMemory
};
use lumos_macro::{agent, tools, rag_pipeline, workflow, lumos};
use serde_json::json;

// 定义工具
tools! {
    {
        name: "stock_price",
        description: "获取股票价格信息",
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
            // 实际应用中，这里会调用真实的股票API
            let price = match symbol {
                "AAPL" => 175.25,
                "MSFT" => 320.15,
                "GOOG" => 140.50,
                _ => 100.00
            };
            Ok(json!({ "price": price, "currency": "USD", "symbol": symbol }))
        }
    },
    {
        name: "stock_news",
        description: "获取股票相关新闻",
        parameters: {
            {
                name: "symbol",
                description: "股票代码",
                type: "string",
                required: true
            },
            {
                name: "limit",
                description: "返回的新闻数量",
                type: "number",
                required: false,
                default: 3
            }
        },
        handler: |params| async move {
            let symbol = params.get("symbol").unwrap().as_str().unwrap();
            // 实际应用中，这里会调用真实的新闻API
            Ok(json!([
                { "title": format!("{} 季度收益超预期", symbol), "date": "2023-07-20" },
                { "title": format!("{} 宣布新产品线", symbol), "date": "2023-07-15" },
                { "title": format!("分析师上调 {} 目标价", symbol), "date": "2023-07-10" }
            ]))
        }
    },
    {
        name: "financial_analysis",
        description: "分析公司财务状况",
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
            // 实际应用中，这里会分析真实的财务数据
            Ok(json!({
                "pe_ratio": 25.4,
                "market_cap": "2.1T",
                "dividend_yield": 0.5,
                "revenue_growth": 15.3,
                "recommendation": "买入"
            }))
        }
    }
}

// 定义RAG知识库
let stock_knowledge = rag_pipeline! {
    name: "stock_knowledge",
    
    source: DocumentSource::from_directory("./docs/stocks"),
    
    pipeline: {
        chunk: {
            chunk_size: 1000,
            chunk_overlap: 200
        },
        
        embed: {
            model: "text-embedding-3-small"
        },
        
        store: {
            db: "memory",
            collection: "stock_data"
        }
    },
    
    query_pipeline: {
        top_k: 3
    }
};

// 定义分析师代理
let analyst_agent = agent! {
    name: "financial_analyst",
    instructions: "你是一个专业的金融分析师，擅长分析股票数据和公司财务状况。请根据工具提供的数据和知识库中的信息，给出专业、深入的分析。",
    
    llm: {
        provider: OpenAiAdapter::default(),
        model: "gpt-4"
    },
    
    memory: {
        store_type: "buffer",
        capacity: 10
    },
    
    tools: {
        stock_price,
        financial_analysis
    }
};

// 定义新闻代理
let news_agent = agent! {
    name: "news_reporter",
    instructions: "你是一个财经新闻记者，擅长总结和报道股票相关新闻。请根据工具提供的新闻数据，给出简明、客观的新闻摘要。",
    
    llm: {
        provider: OpenAiAdapter::default(),
        model: "gpt-3.5-turbo"
    },
    
    tools: {
        stock_news
    }
};

// 定义股票分析工作流
let stock_analysis_workflow = workflow! {
    name: "stock_analysis",
    description: "分析股票价格、新闻和财务状况",
    
    steps: {
        {
            name: "get_price",
            agent: analyst_agent,
            instructions: "获取股票价格信息",
            input: { "request": "{user_input}" }
        },
        {
            name: "get_news",
            agent: news_agent,
            instructions: "获取并总结相关新闻",
            when: { completed("get_price") }
        },
        {
            name: "analyze",
            agent: analyst_agent,
            instructions: "根据价格和新闻，结合财务数据进行全面分析",
            when: { completed("get_news") }
        }
    }
};

// 使用lumos!宏整合所有组件，配置完整应用
let stock_app = lumos! {
    name: "stock_assistant",
    description: "一个专业的股票分析助手，能够提供股票价格、新闻和分析",
    
    agents: {
        analyst_agent,
        news_agent
    },
    
    tools: {
        stock_price,
        stock_news,
        financial_analysis
    },
    
    rags: {
        stock_knowledge
    },
    
    workflows: {
        stock_analysis_workflow
    },
    
    options: {
        logging: {
            level: "info",
            format: "json"
        },
        
        performance: {
            concurrent_requests: 5,
            timeout: 60000,
            cache: {
                enabled: true,
                ttl: 1800
            }
        }
    }
};

// 启动应用
stock_app.start().await?;

// 处理用户请求
let response = stock_app.run("分析苹果公司(AAPL)的股票").await?;
println!("应用回答: {}", response);
```

### 与其他宏的集成

`lumos!`宏可以与其他DSL宏无缝集成，允许开发者在应用定义中内联使用其他宏：

```rust
let app = lumos! {
    name: "my_app",
    
    agents: {
        // 内联使用agent!宏
        main_agent: agent! {
            name: "内联代理",
            // ...配置
        }
    },
    
    tools: {
        // 内联使用tools!宏
        custom_tools: tools! {
            // ...工具定义
        }
    },
    
    rags: {
        // 内联使用rag_pipeline!宏
        knowledge_base: rag_pipeline! {
            // ...RAG配置
        }
    },
    
    workflows: {
        // 内联使用workflow!宏
        main_workflow: workflow! {
            // ...工作流配置
        }
    }
};
```

### 最佳实践

1. **模块化设计**：将大型应用拆分为多个组件，使用`lumos!`宏整合
2. **环境变量**：使用环境变量存储敏感信息，如API密钥
3. **错误处理**：为每个组件实现适当的错误处理逻辑
4. **测试**：使用`eval_suite!`宏为应用创建测试套件
5. **可观测性**：启用日志和指标收集，监控应用性能
6. **缓存策略**：为频繁使用的组件配置适当的缓存
7. **资源限制**：设置合理的超时和并发限制，避免资源耗尽