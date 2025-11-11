# 🤖 Agent 系统

> LumosAI 的核心智能实体系统

## 什么是 Agent？

Agent 是 LumosAI 的核心抽象，代表一个具备推理、学习和执行能力的智能实体。每个 Agent 都有自己的身份、记忆、目标和能力。

### 核心特征

- **自主性** - 能够独立决策和执行任务
- **感知性** - 能够理解和响应环境变化
- **协作性** - 能够与其他 Agent 交互和协作
- **适应性** - 能够从经验中学习和改进
- **目标导向** - 以完成特定目标为导向

## Agent 生命周期

```
创建 → 配置 → 训练 → 执行 → 学习 → 升级 → 销毁
  ↓      ↓      ↓      ↓      ↓      ↓      ↓
 初始化 参数加载 模型训练 任务执行 记忆更新 能力扩展 资源清理
```

### 1. 创建阶段

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 基础创建
    let agent = lumosai::agent::simple("gpt-4", "You are helpful").await?;
    
    // 高级配置
    let agent = Agent::builder()
        .name("research_assistant")
        .model("gpt-4")
        .system_prompt("You are a professional research assistant")
        .max_tokens(2000)
        .temperature(0.7)
        .memory_limit(10000)
        .build()
        .await?;
    
    Ok(())
}
```

### 2. 配置阶段

#### 基础配置
```rust
let agent = Agent::builder()
    .name("assistant")
    .model("claude-3-sonnet")
    .system_prompt("Professional AI assistant")
    .max_tokens(4096)
    .temperature(0.5)
    .top_p(0.9)
    .presence_penalty(0.1)
    .frequency_penalty(0.1)
    .build()
    .await?;
```

#### 高级配置
```rust
let agent = Agent::builder()
    .name("enterprise_agent")
    .model("gpt-4")
    .system_prompt("Enterprise-grade assistant")
    .capabilities(vec![
        Capability::TextGeneration,
        Capability::CodeGeneration,
        Capability::DataAnalysis,
        Capability::WebSearch,
    ])
    .tools(vec![
        ToolConfig::builtin("file_reader"),
        ToolConfig::builtin("web_search"),
        ToolConfig::custom("weather_api"),
    ])
    .memory_config(MemoryConfig {
        short_term_limit: 10000,
        long_term_limit: 100000,
        compression_enabled: true,
    })
    .build()
    .await?;
```

### 3. 执行阶段

#### 单轮对话
```rust
let response = agent.chat("Explain quantum computing").await?;
println!("{}", response);
```

#### 多轮对话
```rust
let mut conversation = agent.start_conversation();

let response1 = conversation.send("What is machine learning?").await?;
println!("{}", response1);

let response2 = conversation.send("Can you give me a simple example?").await?;
println!("{}", response2);
```

#### 流式响应
```rust
let mut stream = agent.chat_stream("Write a story about AI").await?;

while let Some(chunk) = stream.next().await {
    print!("{}", chunk);
    std::io::Write::flush(&mut std::io::stdout())?;
}
```

## Agent 能力系统

### 内置能力

```rust
// 能力配置
let agent = Agent::builder()
    .capabilities(vec![
        Capability::TextGeneration,     // 文本生成
        Capability::CodeGeneration,      // 代码生成
        Capability::DataAnalysis,        // 数据分析
        Capability::ImageGeneration,     // 图像生成
        Capability::AudioProcessing,      // 音频处理
        Capability::WebSearch,           // 网络搜索
        Capability::FileOperations,      // 文件操作
        Capability::DatabaseQuery,       // 数据库查询
    ])
    .build()
    .await?;
```

### 能力组合

```rust
// 能力组合示例
let research_agent = Agent::builder()
    .name("researcher")
    .capabilities(vec![
        Capability::TextGeneration,
        Capability::WebSearch,
        Capability::DataAnalysis,
        Capability::AcademicWriting,
    ])
    .system_prompt("You are a research assistant specializing in academic writing")
    .build()
    .await?;

let developer_agent = Agent::builder()
    .name("developer")
    .capabilities(vec![
        Capability::CodeGeneration,
        Capability::CodeAnalysis,
        Capability::DocumentationWriting,
        Capability::Testing,
    ])
    .system_prompt("You are a senior software developer")
    .build()
    .await?;
```

## Agent 记忆系统

### 记忆类型

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .memory_config(MemoryConfig {
            // 短期记忆 - 当前对话上下文
            short_term_limit: 10000,
            
            // 长期记忆 - 持久化知识
            long_term_limit: 100000,
            
            // 工作记忆 - 临时任务状态
            working_memory_size: 1000,
            
            // 语义记忆 - 结构化知识
            semantic_memory_enabled: true,
            
            // 压缩选项
            compression_enabled: true,
            compression_threshold: 0.8,
        })
        .build()
        .await?;
    
    // 手动管理记忆
    agent.add_memory("fact", "Rust is a systems programming language").await?;
    agent.add_memory("preference", "Prefer functional programming style").await?;
    
    // 检索记忆
    let memories = agent.search_memory("Rust programming").await?;
    for memory in memories {
        println!("Memory: {}", memory.content);
    }
    
    Ok(())
}
```

### 记忆检索

```rust
// 按类型检索
let facts = agent.get_memories_by_type("fact").await?;
let preferences = agent.get_memories_by_type("preference").await?;

// 语义搜索
let relevant_memories = agent.semantic_search("programming languages").await?;

// 时间范围检索
let recent_memories = agent.get_memories_since(
    chrono::Utc::now() - chrono::Duration::days(7)
).await?;
```

## Agent 工具系统

### 工具注册

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("assistant")
        .model("gpt-4")
        .build()
        .await?;
    
    // 注册内置工具
    agent.register_builtin_tool("file_reader").await?;
    agent.register_builtin_tool("web_search").await?;
    agent.register_builtin_tool("calculator").await?;
    
    // 注册自定义工具
    agent.register_custom_tool(Box::new(WeatherTool)).await?;
    agent.register_custom_tool(Box::new(CodeExecutorTool)).await?;
    
    // 使用工具
    let response = agent.chat("What's the weather in Tokyo and calculate 2+2?").await?;
    println!("{}", response);
    
    Ok(())
}
```

### 自定义工具

```rust
use lumosai_core::tool::*;
use async_trait::async_trait;
use serde_json::{json, Value};

#[derive(Tool)]
struct WeatherTool {
    #[tool(description = "Get current weather for a location")]
    location: String,
    
    #[tool(description = "Temperature unit", default = "celsius")]
    unit: String,
}

#[async_trait]
impl ToolExecutor for WeatherTool {
    async fn execute(&self) -> Result<Value> {
        // 调用天气API
        let weather_data = fetch_weather(&self.location).await?;
        
        // 根据单位转换温度
        let temperature = match self.unit.as_str() {
            "fahrenheit" => celsius_to_fahrenheit(weather_data.temp_c),
            _ => weather_data.temp_c,
        };
        
        Ok(json!({
            "location": self.location,
            "temperature": temperature,
            "unit": self.unit,
            "condition": weather_data.condition
        }))
    }
}

async fn fetch_weather(location: &str) -> Result<WeatherData> {
    // 实现天气API调用逻辑
    todo!()
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}
```

## Agent 协作

### Agent 通信

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 创建协作 Agent
    let researcher = Agent::builder()
        .name("researcher")
        .capabilities(vec![Capability::WebSearch, Capability::DataAnalysis])
        .build()
        .await?;
    
    let writer = Agent::builder()
        .name("writer")
        .capabilities(vec![Capability::TextGeneration, Capability::Editing])
        .build()
        .await?;
    
    // 建立通信连接
    let message = Message::builder()
        .from("user")
        .to("researcher")
        .content("Research latest AI developments")
        .build();
    
    // 发送消息
    let research_result = researcher.send_message(message).await?;
    
    // 转发消息给写手
    let writer_message = Message::builder()
        .from("researcher")
        .to("writer")
        .content("Write an article based on this research")
        .data(json!({"research": research_result.content}))
        .build();
    
    let article = writer.send_message(writer_message).await?;
    
    println!("Final article: {}", article.content);
    
    Ok(())
}
```

### 工作流协作

```rust
use lumosai::workflow::*;

#[tokio::main]
async fn main() -> Result<()> {
    let workflow = Workflow::builder()
        .name("content_creation")
        .description("Research and write content")
        .agents(vec![
            AgentConfig::new("researcher", "gpt-4"),
            AgentConfig::new("writer", "gpt-4"),
            AgentConfig::new("editor", "claude-3"),
        ])
        .steps(vec![
            WorkflowStep {
                name: "research",
                agent: "researcher",
                input: "topic",
                output: "research_data",
            },
            WorkflowStep {
                name: "drafting",
                agent: "writer", 
                input: "research_data",
                output: "draft",
            },
            WorkflowStep {
                name: "editing",
                agent: "editor",
                input: "draft",
                output: "final_content",
            },
        ])
        .build()
        .await?;
    
    // 执行工作流
    let result = workflow.execute(json!({
        "topic": "Future of AI in healthcare"
    })).await?;
    
    println!("Final content: {}", result["final_content"]);
    
    Ok(())
}
```

## Agent 学习与适应

### 在线学习

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .learning_config(LearningConfig {
            enabled: true,
            learning_rate: 0.001,
            feedback_weight: 0.1,
            adaptation_threshold: 0.05,
        })
        .build()
        .await?;
    
    // 处理任务并收集反馈
    let response = agent.chat("Explain machine learning").await?;
    
    // 用户反馈
    let feedback = UserFeedback {
        rating: 4.5,
        comment: "Good explanation, but could use more examples",
        helpful: true,
        timestamp: chrono::Utc::now(),
    };
    
    // 基于反馈学习
    agent.learn_from_feedback(response, feedback).await?;
    
    Ok(())
}
```

### 能力适应

```rust
// Agent 可以基于使用模式自动调整能力
let agent = Agent::builder()
    .adaptive_capabilities(true)
    .capability_learning_rate(0.01)
    .build()
    .await?;

// 使用 Agent
for _ in 0..100 {
    agent.chat(random_question()).await?;
}

// Agent 自动适应最常用的能力
let stats = agent.get_usage_statistics().await?;
println!("Most used capabilities: {:?}", stats.top_capabilities);
```

## Agent 监控与调试

### 性能监控

```rust
use lumosai::telemetry::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .telemetry_enabled(true)
        .metrics_collector(Box::new(PrometheusCollector::new()))
        .build()
        .await?;
    
    // 执行任务
    let response = agent.chat("Complex question").await?;
    
    // 获取性能指标
    let metrics = agent.get_metrics().await?;
    println!("Response time: {}ms", metrics.response_time);
    println!("Token usage: {}", metrics.token_count);
    println!("Tool calls: {}", metrics.tool_call_count);
    
    Ok(())
}
```

### 调试工具

```rust
// 启用调试模式
let agent = Agent::builder()
    .debug_mode(true)
    .trace_level(TraceLevel::Verbose)
    .build()
    .await?;

// 执行并获取详细执行轨迹
let trace = agent.chat_with_trace("Debug this issue").await?;

// 分析执行轨迹
for step in trace.steps {
    println!("Step {}: {} ({})", 
        step.sequence, 
        step.description, 
        step.duration
    );
    
    if let Some(error) = step.error {
        println!("  Error: {}", error);
    }
}
```

---

## 🔗 相关文档

- [架构概览](architecture.md) - 系统架构
- [RAG 引擎](rag.md) - 知识检索
- [内存系统](memory.md) - 状态管理  
- [工具系统](tools.md) - 工具集成
- [API 参考](../api-reference/agents.md) - Agent API