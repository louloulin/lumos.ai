# 安装指南

本指南将帮助您在不同环境中安装和配置 LumosAI。

## 📋 系统要求

### 最低要求
- **Rust**: 1.75 或更高版本
- **内存**: 至少 4GB RAM
- **存储**: 至少 1GB 可用空间
- **网络**: 访问 LLM API 的网络连接

### 推荐配置
- **Rust**: 最新稳定版
- **内存**: 8GB+ RAM
- **存储**: 5GB+ 可用空间（用于向量数据库）
- **CPU**: 多核处理器（用于并行处理）

## 🦀 安装 Rust

如果您还没有安装 Rust，请按照以下步骤：

### Windows
```powershell
# 下载并运行 rustup-init.exe
# 或使用 Chocolatey
choco install rust
```

### macOS
```bash
# 使用 rustup（推荐）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 或使用 Homebrew
brew install rust
```

### Linux
```bash
# 使用 rustup（推荐）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ubuntu/Debian
sudo apt update && sudo apt install rustc cargo

# CentOS/RHEL/Fedora
sudo dnf install rust cargo
```

### 验证安装
```bash
rustc --version
cargo --version
```

## 📦 添加 LumosAI 依赖

### 方法 1: 使用 Cargo（推荐）

在您的 `Cargo.toml` 文件中添加：

```toml
[dependencies]
lumosai = "0.1.4"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

### 方法 2: 使用 Cargo 命令

```bash
cargo add lumosai@0.1.4
cargo add tokio --features full
cargo add serde_json
```

### 方法 3: 从源码构建

```bash
git clone https://github.com/lumosai/lumosai.git
cd lumosai
cargo build --release
```

## 🔧 功能特性配置

LumosAI 提供多个可选功能特性，您可以根据需要启用：

```toml
[dependencies]
lumosai = { version = "0.1.4", features = [
    "default",          # 默认功能
    "rag",             # RAG 系统支持
    "vector-qdrant",   # Qdrant 向量数据库
    "vector-weaviate", # Weaviate 向量数据库
    "enterprise",      # 企业级功能
    "ui",              # Web UI 支持
    "mcp",             # Model Context Protocol
] }
```

### 功能特性说明

| 特性 | 描述 | 依赖 |
|------|------|------|
| `default` | 基础功能（Agent、工具、内存） | 无 |
| `rag` | RAG 系统和文档处理 | `vector-*` |
| `vector-qdrant` | Qdrant 向量数据库支持 | Docker |
| `vector-weaviate` | Weaviate 向量数据库支持 | Docker |
| `vector-memory` | 内存向量存储 | 无 |
| `enterprise` | 企业级功能（认证、监控） | 无 |
| `ui` | Web UI 界面 | 无 |
| `mcp` | Model Context Protocol | 无 |

## 🔑 环境变量配置

### LLM API 密钥

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic Claude
export ANTHROPIC_API_KEY="sk-ant-..."

# DeepSeek
export DEEPSEEK_API_KEY="sk-..."

# 本地模型（Ollama）
export OLLAMA_BASE_URL="http://localhost:11434"
```

### 向量数据库配置

```bash
# Qdrant
export QDRANT_URL="http://localhost:6333"
export QDRANT_API_KEY="your-api-key"

# Weaviate
export WEAVIATE_URL="http://localhost:8080"
export WEAVIATE_API_KEY="your-api-key"
```

### 其他配置

```bash
# 日志级别
export RUST_LOG="info"

# 并发限制
export LUMOSAI_MAX_CONCURRENT_REQUESTS="10"

# 缓存目录
export LUMOSAI_CACHE_DIR="./cache"
```

## 🐳 Docker 安装

### 使用预构建镜像

```bash
# 拉取镜像
docker pull lumosai/lumosai:latest

# 运行容器
docker run -d \
  --name lumosai \
  -p 8080:8080 \
  -e OPENAI_API_KEY="your-key" \
  lumosai/lumosai:latest
```

### 使用 Docker Compose

创建 `docker-compose.yml`：

```yaml
version: '3.8'
services:
  lumosai:
    image: lumosai/lumosai:latest
    ports:
      - "8080:8080"
    environment:
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - RUST_LOG=info
    volumes:
      - ./data:/app/data
    depends_on:
      - qdrant

  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"
    volumes:
      - ./qdrant_data:/qdrant/storage
```

运行：

```bash
docker-compose up -d
```

## ✅ 验证安装

创建一个简单的测试文件 `test.rs`：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("LumosAI 安装成功！");
    
    // 测试基本功能
    let agent = Agent::builder()
        .name("测试助手")
        .instructions("你是一个测试助手")
        .model("gpt-3.5-turbo")
        .build()?;
    
    println!("Agent 创建成功: {}", agent.name());
    Ok(())
}
```

运行测试：

```bash
cargo run --bin test
```

如果看到 "LumosAI 安装成功！" 和 "Agent 创建成功"，说明安装完成！

## 🚨 常见问题

### 编译错误

**问题**: `error: failed to compile lumosai`
**解决**: 确保 Rust 版本 >= 1.75

```bash
rustup update stable
```

### 网络连接问题

**问题**: 无法下载依赖
**解决**: 配置 Cargo 镜像源

```bash
# 创建 ~/.cargo/config.toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
```

### API 密钥问题

**问题**: `Authentication failed`
**解决**: 检查环境变量设置

```bash
echo $OPENAI_API_KEY
```

## 📖 下一步

安装完成后，您可以：

1. 阅读 [快速开始指南](./README.md)
2. 查看 [教程系列](../tutorials/)
3. 运行 [示例项目](../examples/)

---

*需要帮助？查看我们的 [故障排除指南](../guides/troubleshooting.md) 或提交 [Issue](https://github.com/lumosai/lumosai/issues)*
