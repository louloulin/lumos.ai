# 📦 安装指南

> 在您的环境中安装和配置 LumosAI

本指南将帮助您在各种环境中安装 LumosAI，包括本地开发、Docker 部署和云端部署。

## 🚀 快速安装

### 方式一：Cargo 安装（推荐）

```bash
# 新建项目
cargo new my_lumosai_app
cd my_lumosai_app

# 添加 LumosAI 依赖
cargo add lumosai tokio serde

# 启动项目
cargo run
```

### 方式二：Docker 安装

```bash
# 拉取预构建镜像
docker pull lumosai/lumosai:latest

# 运行容器
docker run -it --rm lumosai/lumosai:latest bash
```

### 方式三：源码安装

```bash
# 克隆仓库
git clone https://github.com/louloulin/lumos.ai.git
cd lumosai

# 构建项目
cargo build --release

# 运行示例
cargo run --example basic_agent
```

---

## 🔧 环境要求

### 最低要求

- **Rust**: 1.70 或更高版本
- **Cargo**: 1.70 或更高版本
- **操作系统**: Linux, macOS, 或 Windows
- **内存**: 至少 512MB RAM
- **磁盘空间**: 至少 1GB 可用空间

### 推荐配置

- **Rust**: 最新稳定版本
- **内存**: 2GB 或更多 RAM
- **CPU**: 多核心处理器
- **网络**: 稳定的互联网连接

### 可选组件

- **Docker**: 20.10+ (用于容器化部署)
- **PostgreSQL**: 13+ (用于向量数据库)
- **Redis**: 6+ (用于缓存)
- **Nginx**: 1.20+ (用于反向代理)

---

## 📋 详细安装步骤

### 1. 安装 Rust

#### Windows

```powershell
# 使用 rustup 安装
winget install Rustlang.Rust.MSVC

# 或者从官网下载
# https://rustup.rs/
```

#### macOS

```bash
# 使用 Homebrew
brew install rust

# 或者使用 rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Linux

```bash
# 使用 rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 或者使用包管理器
# Ubuntu/Debian
sudo apt update
sudo apt install rustc cargo

# CentOS/RHEL
sudo yum install rust cargo
```

### 2. 验证 Rust 安装

```bash
# 检查版本
rustc --version
cargo --version

# 应该显示类似：
# rustc 1.73.0 (cc66ad468 2023-10-04)
# cargo 1.73.0 (399881a05 2024-01-30)
```

### 3. 创建新项目

```bash
# 创建二进制项目
cargo new my_lumosai_app --bin
cd my_lumosai_app

# 或者创建库项目
cargo new my_lumosai_lib --lib
cd my_lumosai_lib
```

### 4. 配置 Cargo.toml

编辑 `Cargo.toml` 文件：

```toml
[package]
name = "my_lumosai_app"
version = "0.1.0"
edition = "2021"

[dependencies]
# LumosAI 核心库
lumosai = "0.2.0"

# 异步运行时
tokio = { version = "1.0", features = ["full"] }

# 序列化支持
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 错误处理
thiserror = "1.0"
anyhow = "1.0"

# 日志和追踪
tracing = "0.1"
tracing-subscriber = "0.3"

# 可选依赖
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 5. 配置环境变量

创建 `.env` 文件（开发环境）：

```bash
# LLM 提供商 API 密钥
OPENAI_API_KEY=sk-your-openai-key-here
ANTHROPIC_API_KEY=your-anthropic-key-here
DEEPSEEK_API_KEY=sk-your-deepseek-key-here
ZHIPUAI_API_KEY=your-zhipuai-key-here

# LumosAI 配置
LUMOSAI_LOG_LEVEL=info
LUMOSAI_CACHE_DIR=/tmp/lumosai
LUMOSAI_DEFAULT_MODEL=gpt-4
```

或者在系统环境中设置：

```bash
# Linux/macOS
echo 'export OPENAI_API_KEY="your-key"' >> ~/.bashrc
source ~/.bashrc

# Windows (PowerShell)
[System.Environment]::SetEnvironmentVariable("OPENAI_API_KEY", "your-key")
```

---

## 🐳 Docker 安装

### 1. 使用预构建镜像

```bash
# 拉取镜像
docker pull lumosai/lumosai:latest

# 运行交互式容器
docker run -it --rm \
  -e OPENAI_API_KEY="$OPENAI_API_KEY" \
  lumosai/lumosai:latest bash
```

### 2. 使用 Docker Compose

创建 `docker-compose.yml` 文件：

```yaml
version: '3.8'

services:
  lumosai:
    image: lumosai/lumosai:latest
    container_name: lumosai-app
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - LUMOSAI_LOG_LEVEL=info
    volumes:
      - ./data:/app/data
      - ./config:/app/config
    networks:
      - lumosai-network

  postgres:
    image: postgres:15
    container_name: lumosai-db
    restart: unless-stopped
    environment:
      - POSTGRES_DB=lumosai
      - POSTGRES_USER=lumosai
      - POSTGRES_PASSWORD=securepassword
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./sql/init.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "5432:5432"
    networks:
      - lumosai-network

  redis:
    image: redis:7-alpine
    container_name: lumosai-cache
    restart: unless-stopped
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    networks:
      - lumosai-network

networks:
  lumosai-network:
    driver: bridge

volumes:
  postgres_data:
  redis_data:
```

启动服务：

```bash
# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f lumosai

# 停止服务
docker-compose down
```

### 3. 构建自定义镜像

创建 `Dockerfile`：

```dockerfile
FROM rust:1.73-slim as builder

WORKDIR /app
COPY . .

# 构建应用
RUN cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/lumosai /app/
COPY --from=builder /app/data /app/data

# 创建非 root用户
RUN useradd -m -u 1000 lumosai
USER lumosai

EXPOSE 8080

CMD ["./lumosai"]
```

构建和运行：

```bash
# 构建镜像
docker build -t my-lumosai .

# 运行容器
docker run -p 8080:8080 my-lumosai
```

---

## ☁️ 云平台部署

### 1. Vercel 部署

```bash
# 安装 Vercel CLI
npm i -g vercel

# 部署
vercel --prod
```

### 2. Railway 部署

```bash
# 安装 Railway CLI
npm install -g @railway/cli

# 登录
railway login

# 部署
railway up
```

### 3. AWS EC2 部署

```bash
# 启动 EC2 实例
aws ec2 run-instances \
  --image-id ami-0c55b159cbfafe1f0 \
  --instance-type t2.micro \
  --key-name my-key \
  --security-group-ids sg-12345678 \
  --subnet-id subnet-12345678

# SSH 连接并部署
ssh -i my-key.pem ec2-user@your-instance-ip

# 在实例上运行
git clone https://github.com/louloulin/lumos.ai.git
cd lumos.ai
docker-compose up -d
```

---

## 🛠️ 开发环境配置

### 1. IDE 配置

#### VS Code

安装推荐扩展：
- **Rust Analyzer** - Rust 语言支持
- **Cargo** - Cargo 命令支持
- **Better TOML** - TOML 文件支持
- **Error Lens** - 内联错误显示

创建 `.vscode/settings.json`：

```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.imports.granularity.group": "module",
    "rust-analyzer.completion.addCallParentheses": true,
    "rust-analyzer.inlayHints.enable": true
}
```

#### IntelliJ IDEA

1. 安装 Rust 插件
2. 配置项目 SDK
3. 启用 Cargo 集成

### 2. Git 配置

```bash
# 配置用户信息
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# 创建 .gitignore
cat > .gitignore << EOF
/target/
Cargo.lock
.env
.DS_Store
*.log
.vscode/settings.json
EOF
```

### 3. 开发工具

```bash
# 安装有用的 cargo 工具
cargo install cargo-watch    # 文件监控自动重新编译
cargo install cargo-expand    # 宏展开
cargo install cargo-audit     # 安全审计
cargo install cargo-deny     # 依赖检查
cargo install cargo-outdated  # 检查过期依赖
```

---

## ✅ 验证安装

### 1. 创建测试项目

创建 `src/main.rs`：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 测试 LumosAI 安装");
    
    // 测试基础 Agent 创建
    let agent = lumosai::agent::simple("gpt-4", "你是一个友好的助手").await?;
    println!("✅ Agent 创建成功");
    
    // 测试对话功能
    let response = agent.chat("你好！").await?;
    println!("✅ 对话成功: {}", response);
    
    // 测试向量存储
    let storage = lumosai::vector::memory().await?;
    println!("✅ 向量存储创建成功");
    
    // 测试 RAG 系统
    let rag = lumosai::rag::simple(storage, "openai").await?;
    println!("✅ RAG 系统创建成功");
    
    println!("🎉 LumosAI 安装验证成功！");
    
    Ok(())
}
```

### 2. 运行测试

```bash
# 安装依赖
cargo install

# 运行测试
cargo run

# 运行单元测试
cargo test

# 代码检查
cargo clippy

# 格式化代码
cargo fmt
```

### 3. 运行示例

```bash
# 运行基础示例
cargo run --example basic_agent

# 运行 RAG 示例
cargo run --example rag_system

# 运行多 Agent 示例
cargo run --example multi_agent_workflow
```

---

## 🔧 高级配置

### 1. 性能优化

```toml
# Cargo.toml - Release 优化
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.release.package.lumosai]
opt-level = 3
```

### 2. 特性标志

```toml
# 启用特定功能
[dependencies]
lumosai = { version = "0.2.0", features = ["integrations", "vector-memory"] }

# 完整功能
lumosai = { version = "0.2.0", features = [
    "integrations",
    "vector-memory",
    "enterprise",
    "monitoring",
    "tools"
] }
```

### 3. 自定义配置

```rust
// src/config.rs
use lumosai::config::Config;

pub fn load_config() -> Config {
    Config::builder()
        .model("gpt-4")
        .temperature(0.7)
        .max_tokens(2000)
        .cache_enabled(true)
        .debug_mode(false)
        .build()
}
```

---

## 🚨 故障排除

### 常见问题

#### 1. Rust 版本过低

```bash
Error: package `lumosai v0.2.0` cannot be built because it requires rustc 1.70
```

**解决方案**：
```bash
# 更新 Rust
rustup update
rustup install stable
```

#### 2. API 密钥错误

```bash
Error: API key not found or invalid
```

**解决方案**：
```bash
# 检查环境变量
echo $OPENAI_API_KEY

# 设置正确的环境变量
export OPENAI_API_KEY="sk-your-actual-key"
```

#### 3. 内存不足

```bash
Error: Out of memory
```

**解决方案**：
```bash
# 限制内存使用
export LUMOSAI_MEMORY_LIMIT=512000000  # 512MB

# 或者使用更小的模型
export LUMOSAI_DEFAULT_MODEL="gpt-3.5-turbo"
```

#### 4. 网络连接问题

```bash
Error: Network timeout
```

**解决方案**：
```bash
# 设置代理（如果需要）
export https_proxy=http://proxy.example.com:8080
export http_proxy=http://proxy.example.com:8080

# 或者使用镜像
export LUMOSAI_REGISTRY=https://mirror.example.com
```

### 调试技巧

```bash
# 启用详细日志
export RUST_LOG=debug
export LUST_BACKTRACE=1

# 检查依赖
cargo tree

# 清理缓存
cargo clean

# 重新构建
cargo build --verbose
```

---

## 📞 获取帮助

### 官方资源

- **📖 文档**: [完整文档中心](../README.md)
- **🐛 问题报告**: [GitHub Issues](https://github.com/louloulin/lumos.ai/issues)
- **💬 讨论**: [GitHub Discussions](https://github.com/louloulin/lumos.ai/discussions)

### 社区支持

- **Discord**: 实时讨论和帮助
- **Reddit**: r/LumosAI
- **Stack Overflow**: #lumosai 标签

### 商业支持

- **企业支持**: enterprise@lumosai.com
- **技术咨询**: consulting@lumosai.com

---

**🎉 恭喜！您已成功安装 LumosAI！**

接下来可以：
- 🚀 [快速开始](quick-start.md) - 5分钟体验
- 📚 [基础教程](../tutorials/basics/) - 系统学习
- 🔧 [API 参考](../api-reference/) - 深入开发