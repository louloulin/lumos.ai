# LumosAI 研究助手示例

这个示例展示如何使用 LumosAI 创建一个功能强大的研究助手，集成网络搜索、内容分析和智能总结功能。

## 功能特性

- 🔍 **智能搜索**: 模拟网络搜索和信息收集
- 📄 **内容分析**: 深度分析网页内容和文档
- 📊 **研究报告**: 生成结构化的研究报告
- 🌐 **网页抓取**: 提取和分析网页内容
- 🤖 **交互模式**: 支持实时问答和研究指导
- 🛠️ **工具集成**: 展示如何集成外部工具和API

## 快速开始

### 1. 基本研究功能

```bash
# 研究特定主题
cargo run -- research "人工智能发展趋势"

# 指定搜索深度和模型
cargo run -- research "区块链技术" --depth 5 --model "gpt-4"
```

### 2. 网页内容分析

```bash
# 分析网页内容
cargo run -- analyze "https://example.com/article" "这篇文章的主要观点是什么？"

# 使用不同模型分析
cargo run -- analyze "https://example.com/research" "总结研究结论" --model "claude-3"
```

### 3. 交互式研究

```bash
# 启动交互模式
cargo run -- interactive

# 使用指定模型
cargo run -- interactive --model "gpt-4"
```

### 4. 运行示例

```bash
# 网络研究演示
cargo run --example web_research

# 内容分析演示
cargo run --example content_analysis
```

## 使用指南

### 交互式命令

在交互模式中，你可以使用以下命令：

- **直接提问**: 输入研究问题，获得基于搜索的回答
- **search: 关键词**: 搜索特定关键词
- **analyze: URL**: 分析指定网页
- **help**: 显示帮助信息
- **quit/exit**: 退出程序

### 示例对话

```
🔍 请输入研究问题: 人工智能在教育领域的应用

🤖 研究助手:
人工智能在教育领域有多种重要应用：

1. **个性化学习**
   - 根据学生的学习进度和能力调整教学内容
   - 提供定制化的学习路径和建议

2. **智能辅导系统**
   - 24/7在线答疑和指导
   - 自动批改作业和提供反馈

3. **学习分析**
   - 分析学习数据，识别学习模式
   - 预测学习困难，提前干预

📚 参考来源:
- 人工智能在教育领域的详细介绍
- AI教育应用的最新研究进展
- 智能教育系统实践指南
```

## 代码结构

```
research-assistant/
├── src/
│   └── main.rs              # 主程序，包含CLI和核心功能
├── examples/
│   ├── web_research.rs      # 网络研究演示
│   └── content_analysis.rs  # 内容分析演示
├── Cargo.toml               # 项目配置
└── README.md                # 本文档
```

## 核心组件

### 1. 研究助手Agent

```rust
async fn create_research_agent(model: &str) -> Result<SimpleAgent> {
    let system_prompt = r#"
你是一个专业的研究助手，具备以下能力：
1. 信息搜索和收集
2. 内容分析和总结
3. 批判性思维和评估
4. 结构化报告生成
"#;
    
    lumosai::agent::simple(model, system_prompt).await
}
```

### 2. 网络搜索工具

```rust
struct WebSearchTool {
    client: reqwest::Client,
}

impl WebSearchTool {
    async fn search(&self, query: &str, num_results: usize) -> Result<Vec<SearchResult>> {
        // 搜索实现
    }
}
```

### 3. 内容提取工具

```rust
struct WebScrapingTool {
    client: reqwest::Client,
}

impl WebScrapingTool {
    async fn extract_content(&self, url: &str) -> Result<WebContent> {
        // 内容提取实现
    }
}
```

## 高级功能

### 自定义搜索引擎

你可以集成真实的搜索API：

```rust
// 集成Google Custom Search API
async fn google_search(&self, query: &str) -> Result<Vec<SearchResult>> {
    let url = format!(
        "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}",
        api_key, search_engine_id, query
    );
    
    let response: GoogleSearchResponse = self.client
        .get(&url)
        .send()
        .await?
        .json()
        .await?;
    
    Ok(response.items.into_iter().map(|item| SearchResult {
        title: item.title,
        url: item.link,
        snippet: item.snippet,
    }).collect())
}
```

### 智能内容过滤

```rust
async fn filter_relevant_content(&self, content: &str, query: &str) -> Result<String> {
    let filter_prompt = format!(
        "从以下内容中提取与查询 '{}' 最相关的部分：\n\n{}",
        query, content
    );
    
    self.agent.chat(&filter_prompt).await
}
```

### 多源信息融合

```rust
async fn synthesize_information(&self, sources: Vec<WebContent>) -> Result<String> {
    let synthesis_prompt = format!(
        "请综合以下多个信息源，生成一份连贯的研究报告：\n\n{}",
        sources.iter()
            .map(|s| format!("来源：{}\n内容：{}", s.title, s.content))
            .collect::<Vec<_>>()
            .join("\n\n")
    );
    
    self.agent.chat(&synthesis_prompt).await
}
```

## 配置说明

### 环境变量

```bash
# API密钥配置
export OPENAI_API_KEY="your-openai-key"
export GOOGLE_SEARCH_API_KEY="your-google-key"
export GOOGLE_SEARCH_ENGINE_ID="your-search-engine-id"

# 日志级别
export RUST_LOG=debug
```

### 模型选择

支持多种AI模型：

- `gpt-3.5-turbo` - 快速响应，适合一般研究
- `gpt-4` - 高质量分析，适合深度研究
- `claude-3` - 长文本处理，适合文档分析

## 扩展建议

1. **数据库集成**: 存储研究历史和结果
2. **PDF处理**: 支持PDF文档分析
3. **图表生成**: 自动生成研究图表
4. **协作功能**: 支持团队研究协作
5. **API接口**: 提供RESTful API服务

## 故障排除

### 常见问题

1. **网络连接错误**
   - 检查网络连接
   - 验证API密钥配置

2. **内容提取失败**
   - 确认目标网站可访问
   - 检查反爬虫策略

3. **分析质量不佳**
   - 尝试使用更高级的模型
   - 优化提示词设计

### 调试模式

```bash
RUST_LOG=debug cargo run -- interactive
```

## 相关示例

- [hello-world](../hello-world/) - 基础入门示例
- [chatbot](../chatbot/) - 对话机器人示例
- [rag-system](../rag-system/) - RAG知识问答系统

## 许可证

本示例遵循 LumosAI 项目的许可证。
