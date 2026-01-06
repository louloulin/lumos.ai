# 💡 LumosAI 示例代码集合

**丰富的可运行示例，从基础入门到企业级应用**

## 🎯 示例概览

本示例库展示了LumosAI的各种实际应用场景，每个示例都包含：
- ✅ 完整的源代码
- 📝 详细的说明文档
- 🛠️ 运行环境配置
- 📊 性能基准测试

---

## 🌟 快速开始示例

### 1. 基础Agent对话 ⭐

**场景**: 创建第一个AI助手进行简单对话

```rust
// examples/basic_agent.rs
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建简单的AI助手
    let agent = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是一个友好的AI助手"
    ).await?;
    
    // 开始对话
    let response = agent.chat("你好！请介绍一下LumosAI框架。").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

**运行方式**:
```bash
export OPENAI_API_KEY="your-key"
cargo run --example basic_agent
```

**难度**: ⭐ | **时间**: 5分钟 | **相关教程**: [Agent基础概念](./tutorials/basics/agent-basics.md)

---

### 2. 智能问答系统 ⭐⭐

**场景**: 基于知识库的智能问答

```rust
// examples/rag_qa_system.rs
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建向量存储
    let storage = lumosai::vector::memory().await?;
    
    // 创建RAG系统
    let rag = lumosai::rag::builder()
        .storage(storage)
        .embedding_provider("openai")
        .build()
        .await?;
    
    // 添加知识文档
    let documents = vec![
        "LumosAI是基于Rust的企业级AI框架",
        "支持Agent、RAG、工具集成等功能",
        "具有高性能和高安全性特点"
    ];
    
    for doc in documents {
        rag.add_document(doc).await?;
    }
    
    // 创建问答Agent
    let qa_agent = Agent::builder()
        .name("知识问答助手")
        .model("gpt-3.5-turbo")
        .system_prompt("基于提供的文档信息回答问题")
        .rag(rag)
        .build()
        .await?;
    
    // 问答测试
    let response = qa_agent.chat("LumosAI有什么特点？").await?;
    println!("回答: {}", response);
    
    Ok(())
}
```

**难度**: ⭐⭐ | **时间**: 15分钟 | **相关教程**: [RAG系统入门](./tutorials/basics/rag-basics.md)

---

## 🛠️ 工具集成示例

### 3. 计算器工具Agent ⭐⭐

**场景**: Agent使用计算器工具进行数学计算

```rust
// examples/calculator_agent.rs
use lumosai::prelude::*;
use lumos_macro::tool;

// 定义计算工具
#[tool]
async fn calculate(operation: String, a: f64, b: f64) -> Result<f64> {
    match operation.as_str() {
        "add" => Ok(a + b),
        "subtract" => Ok(a - b),
        "multiply" => Ok(a * b),
        "divide" => {
            if b != 0.0 {
                Ok(a / b)
            } else {
                Err("除零错误".into())
            }
        }
        _ => Err("不支持的操作".into())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 创建带工具的Agent
    let agent = Agent::builder()
        .name("计算助手")
        .model("gpt-3.5-turbo")
        .system_prompt("你是一个数学计算助手，可以执行基本数学运算")
        .tool(CalculateTool::new())
        .build()
        .await?;
    
    // 测试计算功能
    let math_questions = vec![
        "计算 123 + 456",
        "计算 100 - 25", 
        "计算 15 * 8",
        "计算 100 / 4"
    ];
    
    for question in math_questions {
        let response = agent.chat(question).await?;
        println!("问: {} 答: {}", question, response);
    }
    
    Ok(())
}
```

**难度**: ⭐⭐ | **时间**: 20分钟 | **相关教程**: [工具集成详解](./tutorials/basics/tool-integration.md)

---

### 4. 网络搜索Agent ⭐⭐⭐

**场景**: Agent使用网络搜索工具获取实时信息

```rust
// examples/web_search_agent.rs
use lumosai::prelude::*;
use lumos_macro::tool;

// 模拟网络搜索工具
#[tool]
async fn web_search(query: String) -> Result<String> {
    // 这里应该是实际的搜索API调用
    // 为了示例简化，我们返回模拟结果
    let mock_results = match query.to_lowercase().as_str() {
        q if q.contains("rust") => {
            "Rust是系统编程语言，以安全、并发著称，常用于高性能应用开发。"
        }
        q if q.contains("ai") => {
            "AI人工智能正在快速发展，包括机器学习、深度学习、大语言模型等技术。"
        }
        _ => "搜索结果未找到相关信息，请尝试其他关键词。"
    };
    
    Ok(mock_results.to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("搜索助手")
        .model("gpt-4")
        .system_prompt("你是一个信息查询助手，可以搜索网络获取最新信息")
        .tool(WebSearchTool::new())
        .build()
        .await?;
    
    let search_queries = vec![
        "Rust编程语言的优势",
        "人工智能的最新发展",
        "企业级AI框架对比"
    ];
    
    for query in search_queries {
        println!("🔍 搜索: {}", query);
        let response = agent.chat(query).await?;
        println!("📊 结果: {}\n", response);
    }
    
    Ok(())
}
```

**难度**: ⭐⭐⭐ | **时间**: 25分钟 | **相关教程**: [自定义工具开发](./topics/custom-tools.md)

---

## 🏢 企业级应用示例

### 5. 多Agent客服系统 ⭐⭐⭐⭐

**场景**: 多个Agent协作处理客户服务请求

```rust
// examples/customer_service_system.rs
use lumosai::prelude::*;
use std::collections::HashMap;

struct CustomerServiceSystem {
    agents: HashMap<String, Agent>,
}

impl CustomerServiceSystem {
    async fn new() -> Result<Self> {
        let mut agents = HashMap::new();
        
        // 分类Agent
        agents.insert("classifier".to_string(), 
            Agent::builder()
                .name("问题分类器")
                .model("gpt-3.5-turbo")
                .system_prompt("你是客服问题分类器，将用户问题分类为：技术支持、账单查询、产品咨询、投诉建议")
                .build()
                .await?
        );
        
        // 技术支持Agent
        agents.insert("tech_support".to_string(),
            Agent::builder()
                .name("技术支持")
                .model("gpt-4")
                .system_prompt("你是技术支持专家，专门解决产品技术问题")
                .build()
                .await?
        );
        
        // 账单查询Agent
        agents.insert("billing".to_string(),
            Agent::builder()
                .name("账单查询")
                .model("gpt-3.5-turbo")
                .system_prompt("你是账单查询助手，帮助用户了解费用和账单信息")
                .build()
                .await?
        );
        
        Ok(Self { agents })
    }
    
    async fn process_request(&self, user_message: &str) -> Result<String> {
        // 1. 分类问题
        let classifier = &self.agents["classifier"];
        let classification = classifier.chat(
            &format!("请分类以下问题：{}", user_message)
        ).await?;
        
        // 2. 路由到相应Agent
        let agent_name = if classification.contains("技术支持") {
            "tech_support"
        } else if classification.contains("账单") {
            "billing"
        } else {
            "tech_support"  // 默认
        };
        
        let specialist = &self.agents[agent_name];
        let response = specialist.chat(user_message).await?;
        
        Ok(format!("{}回答：\n{}", agent_name, response))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let system = CustomerServiceSystem::new().await?;
    
    let customer_requests = vec![
        "我的产品无法启动，该怎么办？",
        "我想查询上个月的账单",
        "产品有什么新功能？",
        "如何升级到最新版本？"
    ];
    
    for request in customer_requests {
        println!("👤 客户: {}", request);
        let response = system.process_request(request).await?;
        println!("🤖 系统: {}\n", response);
    }
    
    Ok(())
}
```

**难度**: ⭐⭐⭐⭐ | **时间**: 45分钟 | **相关教程**: [多Agent协作](./tutorials/advanced/multi-agent.md)

---

### 6. 企业知识库系统 ⭐⭐⭐⭐⭐

**场景**: 大型企业文档管理和智能检索系统

```rust
// examples/enterprise_knowledge.rs
use lumosai::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    id: String,
    title: String,
    content: String,
    department: String,
    tags: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

struct EnterpriseKnowledgeBase {
    rag: RAGEngine,
    documents: HashMap<String, Document>,
}

impl EnterpriseKnowledgeBase {
    async fn new() -> Result<Self> {
        let storage = lumosai::vector::postgres("postgresql://user:pass@localhost/enterprise_kb").await?;
        
        let rag = lumosai::rag::builder()
            .storage(storage)
            .embedding_provider("openai")
            .chunking_strategy("recursive")
            .build()
            .await?;
        
        Ok(Self {
            rag,
            documents: HashMap::new(),
        })
    }
    
    async fn add_document(&mut self, doc: Document) -> Result<String> {
        let doc_content = format!(
            "部门: {}\n标题: {}\n标签: {}\n创建时间: {}\n\n{}",
            doc.department,
            doc.title,
            doc.tags.join(", "),
            doc.created_at.format("%Y-%m-%d %H:%M:%S"),
            doc.content
        );
        
        self.rag.add_document(doc_content).await?;
        self.documents.insert(doc.id.clone(), doc);
        
        Ok(doc.id)
    }
    
    async fn search(&self, query: &str, department: Option<&str>) -> Result<Vec<Document>> {
        let search_query = if let Some(dept) = department {
            format!("部门: {} 查询: {}", dept, query)
        } else {
            query.to_string()
        };
        
        let results = self.rag.search(&search_query, 10).await?;
        
        // 从搜索结果中提取原始文档
        let mut found_docs = Vec::new();
        for result in results {
            // 这里需要实现从搜索结果到文档ID的映射
            // 为了简化示例，我们假设返回所有匹配的文档
        }
        
        Ok(found_docs)
    }
    
    async fn ask_question(&self, question: &str) -> Result<String> {
        let qa_agent = Agent::builder()
            .name("企业知识问答")
            .model("gpt-4")
            .system_prompt(
                "你是企业知识库的问答助手。基于提供的文档信息回答问题，\
                并提供相关的文档引用和部门信息。"
            )
            .rag(self.rag.clone())
            .build()
            .await?;
        
        qa_agent.chat(question).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut kb = EnterpriseKnowledgeBase::new().await?;
    
    // 添加示例文档
    let tech_doc = Document {
        id: "doc_001".to_string(),
        title: "LumosAI部署指南".to_string(),
        content: "本指南介绍如何在生产环境中部署LumosAI...".to_string(),
        department: "技术部".to_string(),
        tags: vec!["部署".to_string(), "运维".to_string(), "生产环境".to_string()],
        created_at: chrono::Utc::now(),
    };
    
    let hr_doc = Document {
        id: "doc_002".to_string(),
        title: "员工手册".to_string(),
        content: "本手册包含公司的规章制度和工作流程...".to_string(),
        department: "人事部".to_string(),
        tags: vec!["规章制度".to_string(), "工作流程".to_string()],
        created_at: chrono::Utc::now(),
    };
    
    kb.add_document(tech_doc).await?;
    kb.add_document(hr_doc).await?;
    
    // 问答测试
    let questions = vec![
        "如何在生产环境部署LumosAI？",
        "公司的请假制度是什么？",
        "技术部有什么文档？"
    ];
    
    for question in questions {
        println!("❓ 问题: {}", question);
        let answer = kb.ask_question(question).await?;
        println!("💡 回答: {}\n", answer);
    }
    
    Ok(())
}
```

**难度**: ⭐⭐⭐⭐⭐ | **时间**: 60分钟 | **相关教程**: [企业级部署](./tutorials/advanced/deployment.md)

---

## 🔧 高级功能示例

### 7. 流式响应Agent ⭐⭐⭐

**场景**: 实时流式响应，提升用户体验

```rust
// examples/streaming_agent.rs
use lumosai::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple(
        "gpt-4",
        "你是一个善于讲解复杂概念的老师"
    ).await?;
    
    let questions = vec![
        "请解释什么是区块链技术",
        "如何理解微服务架构",
        "什么是Rust语言的所有权系统"
    ];
    
    for (i, question) in questions.iter().enumerate() {
        println!("\n🎓 第{}个问题: {}", i + 1, question);
        print!("🤖 老师: ");
        
        // 流式响应
        let mut stream = agent.stream_chat(question).await?;
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(text) => {
                    print!("{}", text);
                    std::io::stdout().flush().unwrap();
                    // 模拟打字效果
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
                Err(e) => {
                    eprintln!("错误: {}", e);
                    break;
                }
            }
        }
        
        println!("\n");
    }
    
    Ok(())
}
```

**难度**: ⭐⭐⭐ | **时间**: 20分钟 | **相关API**: [Agent流式API](../reference/api/agent.md#stream_chat)

---

### 8. 性能基准测试 ⭐⭐⭐

**场景**: 测试LumosAI系统性能和并发能力

```rust
// examples/performance_benchmark.rs
use lumosai::prelude::*;
use std::sync::Arc;
use std::time::Instant;
use tokio::task::JoinSet;

async fn benchmark_single_requests(agent: &Agent, questions: &[&str]) -> Result<Vec<Duration>> {
    let mut durations = Vec::new();
    
    for question in questions {
        let start = Instant::now();
        let _response = agent.chat(question).await?;
        let duration = start.elapsed();
        durations.push(duration);
        
        println!("单次请求: {:?} - {}", duration, question);
    }
    
    Ok(durations)
}

async fn benchmark_concurrent_requests(agent: &Agent, question: &str, count: usize) -> Result<Duration> {
    let agent = Arc::new(agent);
    let mut tasks = JoinSet::new();
    
    let start = Instant::now();
    
    // 并发执行请求
    for i in 0..count {
        let agent_clone = Arc::clone(&agent);
        let question = format!("{} #{}", question, i);
        
        tasks.spawn(async move {
            let start = Instant::now();
            let _response = agent_clone.chat(&question).await.unwrap();
            start.elapsed()
        });
    }
    
    // 收集所有结果
    let mut durations = Vec::new();
    while let Some(duration) = tasks.join_next().await {
        durations.push(duration.unwrap());
    }
    
    let total_time = start.elapsed();
    
    // 统计结果
    let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
    let min_duration = durations.iter().min().unwrap();
    let max_duration = durations.iter().max().unwrap();
    
    println!("并发测试结果:");
    println!("  并发数: {}", count);
    println!("  总时间: {:?}", total_time);
    println!("  平均响应时间: {:?}", avg_duration);
    println!("  最快响应时间: {:?}", min_duration);
    println!("  最慢响应时间: {:?}", max_duration);
    println!("  QPS: {:.2}", count as f64 / total_time.as_secs_f64());
    
    Ok(total_time)
}

#[tokio::main]
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是一个简单的问答助手"
    ).await?;
    
    let test_questions = vec![
        "你好",
        "什么是AI？",
        "Rust语言有什么特点？",
        "如何优化代码性能？",
        "解释一下微服务架构"
    ];
    
    println!("🏃‍♂️ LumosAI 性能基准测试");
    println!("========================================");
    
    // 单次请求测试
    println!("\n📊 单次请求测试:");
    let single_durations = benchmark_single_requests(&agent, &test_questions).await?;
    
    // 并发测试
    println!("\n🚀 并发性能测试:");
    
    for concurrency in [10, 50, 100] {
        println!("\n--- 并发数: {} ---", concurrency);
        let _total_time = benchmark_concurrent_requests(&agent, "测试问题", concurrency).await?;
    }
    
    // 生成报告
    let avg_single_time = single_durations.iter().sum::<Duration>() / single_durations.len() as u32;
    println!("\n📋 测试报告:");
    println!("  平均单次响应时间: {:?}", avg_single_time);
    println!("  建议生产环境并发数: 50-100");
    println!("  建议响应超时时间: 30秒");
    
    Ok(())
}
```

**难度**: ⭐⭐⭐ | **时间**: 30分钟 | **相关指南**: [性能优化](./guides/performance.md)

---

## 📚 示例导航

### 按难度分类

| 难度 | 示例 | 描述 | 学习时间 |
|------|------|------|----------|
| ⭐ | [基础Agent对话](#1-基础agent对话-⭐) | 创建第一个AI助手 | 5分钟 |
| ⭐⭐ | [智能问答系统](#2-智能问答系统-⭐⭐) | RAG知识库问答 | 15分钟 |
| ⭐⭐ | [计算器工具Agent](#3-计算器工具agent-⭐⭐) | 工具集成示例 | 20分钟 |
| ⭐⭐⭐ | [网络搜索Agent](#4-网络搜索agent-⭐⭐⭐) | 自定义工具开发 | 25分钟 |
| ⭐⭐⭐⭐ | [多Agent客服系统](#5-多agent客服系统-⭐⭐⭐⭐) | Agent协作模式 | 45分钟 |
| ⭐⭐⭐⭐⭐ | [企业知识库系统](#6-企业知识库系统-⭐⭐⭐⭐⭐) | 生产级应用 | 60分钟 |

### 按功能分类

**基础功能**:
- Agent创建和配置
- 基础对话交互
- 简单工具集成

**进阶功能**:
- RAG知识检索
- 自定义工具开发
- 流式响应处理
- 内存管理

**企业级功能**:
- 多Agent协作
- 性能优化
- 并发处理
- 生产部署

---

## 🛠️ 运行环境配置

### 基础环境

```bash
# 1. 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. 克隆项目
git clone https://github.com/louloulin/lumos.ai.git
cd lumos.ai

# 3. 安装依赖
cargo build

# 4. 设置API密钥
export OPENAI_API_KEY="your-key"
```

### 运行示例

```bash
# 运行基础示例
cargo run --example basic_agent

# 运行RAG示例
cargo run --example rag_qa_system

# 运行多Agent示例
cargo run --example customer_service_system

# 运行性能测试
cargo run --example performance_benchmark
```

### 开发环境增强

```bash
# 安装有用的工具
cargo install cargo-watch    # 自动重载
cargo install cargo-edit     # cargo add命令
cargo install cargo-audit     # 安全审计

# 开发模式运行
cargo watch -x run --example basic_agent
```

---

## 🎯 学习路径建议

### 新手路径 (1-2天)
1. 运行 `basic_agent` - 理解基础概念
2. 运行 `rag_qa_system` - 学习RAG原理
3. 修改示例代码 - 尝试自定义参数
4. 查看相关教程 - 深入理解原理

### 进阶路径 (3-5天)
1. 运行 `calculator_agent` - 学习工具集成
2. 运行 `web_search_agent` - 开发自定义工具
3. 运行 `customer_service_system` - 理解多Agent协作
4. 运行 `enterprise_knowledge` - 学习企业级架构

### 专家路径 (1周+)
1. 分析所有示例代码结构
2. 实现自己的企业级应用
3. 运行性能基准测试
4. 优化和重构代码

---

## 🔗 相关资源

- [API参考文档](../reference/api/README.md)
- [教程系列](../tutorials/README.md)
- [最佳实践指南](../guides/best-practices.md)
- [故障排除](../getting-started/troubleshooting.md)

---

**开始探索LumosAI的强大功能吧！** 🚀

*[← 返回学习资源](../learn/README.md)*