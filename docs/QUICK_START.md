# 🚀 LumosAI 快速开始指南

欢迎使用 LumosAI！这是一个功能强大的企业级 AI 框架，让您能够快速构建智能应用。

## 📋 系统要求

### 基础要求
- **Rust**: 1.70+ (推荐最新稳定版)
- **操作系统**: Windows 10+, macOS 10.15+, Linux (Ubuntu 20.04+)
- **内存**: 最少 4GB RAM (推荐 8GB+)
- **存储**: 最少 2GB 可用空间

### 可选要求 (用于完整功能)
- **Docker**: 用于容器化部署
- **PostgreSQL**: 用于数据持久化
- **Redis**: 用于缓存和会话管理

## ⚡ 5 分钟快速开始

### 1. 克隆项目

```bash
git clone https://github.com/your-org/lumosai.git
cd lumosai
```

### 2. 安装依赖

```bash
# 确保 Rust 工具链是最新的
rustup update

# 构建项目
cargo build --release
```

### 3. 运行基础测试

```bash
# 验证安装
cargo test --test simple_test

# 应该看到：test result: ok. 7 passed; 0 failed
```

### 4. 创建您的第一个 AI Agent

创建文件 `examples/my_first_agent.rs`:

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建一个简单的 AI Agent
    let agent = Agent::builder()
        .name("我的第一个Agent")
        .model("gpt-3.5-turbo")  // 或使用其他支持的模型
        .system_prompt("你是一个友好的AI助手")
        .build()
        .await?;

    // 与 Agent 对话
    let response = agent.chat("你好！请介绍一下自己。").await?;
    println!("Agent 回复: {}", response);

    Ok(())
}
```

### 5. 运行您的第一个 Agent

```bash
cargo run --example my_first_agent
```

## 🎯 核心功能示例

### 🤖 创建智能对话 Agent

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("智能助手")
        .model("gpt-4")
        .temperature(0.7)
        .max_tokens(1000)
        .system_prompt("你是一个专业的技术顾问，擅长解答编程和技术问题。")
        .build()
        .await?;

    // 持续对话
    loop {
        print!("您: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim() == "退出" {
            break;
        }

        let response = agent.chat(&input).await?;
        println!("助手: {}", response);
    }

    Ok(())
}
```

### 🧠 构建 RAG 知识问答系统

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建 RAG 系统
    let rag = RagSystem::builder()
        .embedding_provider("openai")
        .vector_store("memory")  // 或使用 "qdrant", "weaviate"
        .chunk_size(500)
        .chunk_overlap(50)
        .build()
        .await?;

    // 添加知识文档
    rag.add_document("LumosAI是一个强大的Rust AI框架，支持多种LLM模型。").await?;
    rag.add_document("LumosAI提供RAG、Agent、工作流等企业级功能。").await?;
    rag.add_document("LumosAI支持OpenAI、Claude、Qwen等多种模型提供商。").await?;

    // 创建带 RAG 的 Agent
    let agent = Agent::builder()
        .name("知识助手")
        .model("gpt-3.5-turbo")
        .rag_system(rag)
        .build()
        .await?;

    // 基于知识库回答问题
    let response = agent.chat("LumosAI支持哪些功能？").await?;
    println!("回答: {}", response);

    Ok(())
}
```

### 🔄 创建工作流

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建工作流
    let workflow = Workflow::builder()
        .name("文档处理工作流")
        .add_step("extract", "提取文档内容")
        .add_step("summarize", "生成摘要")
        .add_step("translate", "翻译内容")
        .build()
        .await?;

    // 执行工作流
    let result = workflow.execute(json!({
        "document": "这是一个需要处理的文档内容..."
    })).await?;

    println!("工作流结果: {}", result);
    Ok(())
}
```

## 🛠️ 配置和环境变量

### 环境变量设置

创建 `.env` 文件：

```env
# LLM 提供商 API 密钥
OPENAI_API_KEY=your_openai_api_key
ANTHROPIC_API_KEY=your_anthropic_api_key
QWEN_API_KEY=your_qwen_api_key

# 数据库配置
DATABASE_URL=postgresql://user:password@localhost/lumosai
REDIS_URL=redis://localhost:6379

# 向量数据库配置
QDRANT_URL=http://localhost:6333
WEAVIATE_URL=http://localhost:8080

# 日志级别
RUST_LOG=info
```

### 配置文件

创建 `lumosai.toml`:

```toml
[agent]
default_model = "gpt-3.5-turbo"
default_temperature = 0.7
max_tokens = 2000

[rag]
default_embedding_provider = "openai"
default_chunk_size = 500
default_chunk_overlap = 50

[storage]
default_vector_store = "memory"
database_pool_size = 10

[security]
enable_auth = true
jwt_secret = "your-jwt-secret"

[monitoring]
enable_telemetry = true
metrics_endpoint = "http://localhost:9090"
```

## 📚 更多示例

### 使用工具的 Agent

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建计算器工具
    let calculator = Tool::builder()
        .name("calculator")
        .description("执行数学计算")
        .function(|input: &str| {
            // 简单的计算逻辑
            Ok(format!("计算结果: {}", input))
        })
        .build();

    // 创建带工具的 Agent
    let agent = Agent::builder()
        .name("数学助手")
        .model("gpt-4")
        .add_tool(calculator)
        .build()
        .await?;

    let response = agent.chat("请计算 15 * 23").await?;
    println!("回答: {}", response);

    Ok(())
}
```

### 多模态 Agent

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("多模态助手")
        .model("gpt-4-vision")
        .enable_vision(true)
        .enable_voice(true)
        .build()
        .await?;

    // 处理图像
    let response = agent.chat_with_image(
        "这张图片里有什么？",
        "path/to/image.jpg"
    ).await?;
    
    println!("图像分析: {}", response);
    Ok(())
}
```

## 🚀 部署选项

### 1. 本地开发

```bash
# 开发模式运行
cargo run

# 监听文件变化自动重启
cargo install cargo-watch
cargo watch -x run
```

### 2. Docker 部署

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/lumosai /usr/local/bin/
CMD ["lumosai"]
```

```bash
# 构建镜像
docker build -t lumosai .

# 运行容器
docker run -p 8080:8080 lumosai
```

### 3. Kubernetes 部署

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: lumosai
spec:
  replicas: 3
  selector:
    matchLabels:
      app: lumosai
  template:
    metadata:
      labels:
        app: lumosai
    spec:
      containers:
      - name: lumosai
        image: lumosai:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
```

## 🔧 故障排除

### 常见问题

**Q: 编译失败，提示依赖版本冲突**
```bash
# 清理并重新构建
cargo clean
cargo update
cargo build
```

**Q: 运行时提示 API 密钥错误**
```bash
# 检查环境变量
echo $OPENAI_API_KEY

# 或在代码中设置
std::env::set_var("OPENAI_API_KEY", "your-key");
```

**Q: 向量数据库连接失败**
```bash
# 启动本地 Qdrant
docker run -p 6333:6333 qdrant/qdrant

# 或使用内存存储
let rag = RagSystem::builder()
    .vector_store("memory")
    .build().await?;
```

### 获取帮助

- 📖 **文档**: [docs/](docs/)
- 🐛 **问题报告**: [GitHub Issues](https://github.com/your-org/lumosai/issues)
- 💬 **社区讨论**: [GitHub Discussions](https://github.com/your-org/lumosai/discussions)
- 📧 **邮件支持**: support@lumosai.com

## 🎉 下一步

恭喜！您已经成功开始使用 LumosAI。接下来您可以：

1. 📚 **深入学习**: 查看 [完整文档](docs/README.md)
2. 🔧 **高级配置**: 了解 [企业级功能](docs/enterprise/)
3. 🚀 **生产部署**: 参考 [部署指南](docs/deployment/)
4. 🤝 **参与贡献**: 查看 [贡献指南](CONTRIBUTING.md)

**开始构建您的 AI 应用吧！** 🚀
