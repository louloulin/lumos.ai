# 🔧 安装指南

详细的环境搭建和配置说明，确保LumosAI在你的系统上正常运行。

## ⏱️ 预计时间: 10-15分钟

## 📋 系统要求

### 最低要求
- **操作系统**: Linux, macOS, 或 Windows 10+
- **Rust版本**: 1.70 或更高版本
- **内存**: 4GB RAM
- **存储空间**: 1GB 可用空间
- **网络**: 需要访问AI模型API

### 推荐配置
- **操作系统**: Linux (Ubuntu 20.04+) 或 macOS 12+
- **Rust版本**: 最新稳定版
- **内存**: 8GB+ RAM
- **存储空间**: 5GB+ 可用空间
- **网络**: 稳定的互联网连接

---

## 🦀 第一步：安装Rust

### 方法1：使用rustup（推荐）

```bash
# 安装rustup和Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 按照提示完成安装，选择默认配置

# 重新加载环境变量
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 方法2：使用包管理器

**macOS (Homebrew):**
```bash
brew install rust
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install rustc cargo
```

**Windows (Chocolatey):**
```bash
choco install rust
```

### 验证Rust安装

```bash
# 检查版本（需要1.70+）
rustc --version
# 应该显示类似：rustc 1.75.0 (82e1608df 2023-12-21)

# 检查Cargo
cargo --version
# 应该显示类似：cargo 1.75.0 (1d85b19f4 2023-11-27)
```

---

## 🔧 第二步：安装系统依赖

### Linux (Ubuntu/Debian)

```bash
# 更新包管理器
sudo apt update

# 安装构建工具
sudo apt install build-essential pkg-config

# 安装SSL库（用于HTTPS请求）
sudo apt install libssl-dev

# 安装其他常用依赖
sudo apt install curl git
```

### macOS

```bash
# 安装Xcode命令行工具
xcode-select --install

# 或使用Homebrew
brew install openssl pkg-config
```

### Windows

```bash
# 安装Microsoft Visual Studio C++ Build Tools
# 下载地址：https://visualstudio.microsoft.com/visual-cpp-build-tools/

# 或通过rustup安装MSVC工具链
rustup toolchain install stable-x86_64-pc-windows-msvc
rustup default stable-x86_64-pc-windows-msvc
```

---

## 🤖 第三步：配置AI模型API

### OpenAI (推荐)

1. **获取API密钥**
   - 访问 [OpenAI API](https://platform.openai.com/api-keys)
   - 创建账户并生成API密钥

2. **设置环境变量**
   ```bash
   # Linux/macOS
   echo 'export OPENAI_API_KEY="your-api-key-here"' >> ~/.bashrc
   source ~/.bashrc

   # 或在当前会话中设置
   export OPENAI_API_KEY="your-api-key-here"

   # Windows (PowerShell)
   [System.Environment]::SetEnvironmentVariable("OPENAI_API_KEY", "your-api-key-here", "User")
   ```

### Anthropic Claude

```bash
export ANTHROPIC_API_KEY="your-api-key-here"
```

### 其他支持的模型

LumosAI还支持其他模型提供商，请参考[模型配置文档](../../reference/configuration/models.md)。

---

## 🚀 第四步：验证安装

### 1. 创建测试项目

```bash
# 创建新项目
cargo new lumosai-test
cd lumosai-test

# 添加LumosAI依赖
cargo add lumosai
```

### 2. 创建测试代码

在 `src/main.rs` 中写入：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI安装测试");
    
    // 测试Agent创建
    let agent = Agent::builder()
        .name("test")
        .system_prompt("你是一个测试助手")
        .model("gpt-3.5-turbo")
        .build()
        .await?;
    
    println!("✅ Agent创建成功");
    
    // 测试对话（如果设置了API密钥）
    if std::env::var("OPENAI_API_KEY").is_ok() {
        let response = agent.chat("测试消息").await?;
        println!("✅ 对话测试成功: {}", response);
    } else {
        println!("⚠️  未设置API密钥，跳过对话测试");
    }
    
    println!("🎉 LumosAI安装完成！");
    Ok(())
}
```

### 3. 运行测试

```bash
cargo run
```

**预期输出：**
```
🚀 LumosAI安装测试
✅ Agent创建成功
✅ 对话测试成功: ...
🎉 LumosAI安装完成！
```

---

## 🔧 可选配置

### Docker支持（可选）

如果你想使用Docker运行LumosAI：

```bash
# 安装Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# 验证安装
docker --version
```

### 开发环境增强

**VS Code + rust-analyzer（推荐）:**

1. 安装 [VS Code](https://code.visualstudio.com/)
2. 安装 rust-analyzer 扩展
3. 安装 Copilot（可选）

**其他有用的Cargo扩展：**

```bash
# 安装有用的cargo工具
cargo install cargo-watch      # 自动重新编译
cargo install cargo-edit       # cargo add命令
cargo install cargo-audit      # 安全审计
cargo install cargo-outdated   # 检查过期依赖
```

---

## 🔧 故障排除

### 常见问题及解决方案

#### Rust版本问题

**问题**: `error: package compiler v1.70.0 is required but your version is 1.65.0`

**解决方案**:
```bash
rustup update stable
rustup default stable
```

#### SSL/TLS问题

**问题**: `failed to resolve: SSL error`

**解决方案**:
```bash
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# macOS
brew install openssl
export PKG_CONFIG_PATH="/usr/local/opt/openssl/lib/pkgconfig"
```

#### 权限问题

**问题**: `Permission denied`

**解决方案**:
```bash
# 确保有正确的文件权限
chmod +x ~/.cargo/bin/*

# 或重新安装rustup
rustup self uninstall
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 网络连接问题

**问题**: 无法下载crates

**解决方案**:
```bash
# 使用国内镜像源
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << EOF
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
EOF
```

#### API密钥问题

**问题**: `API key not found or invalid`

**解决方案**:
```bash
# 检查环境变量
echo $OPENAI_API_KEY

# 重新设置
export OPENAI_API_KEY="your-correct-api-key"
```

---

## 🔗 相关资源

- [Rust官方文档](https://doc.rust-lang.org/book/)
- [Cargo使用指南](https://doc.rust-lang.org/cargo/)
- [环境变量配置](../../reference/configuration/environment.md)
- [模型API配置](../../reference/configuration/models.md)

---

## ✅ 安装完成检查清单

完成安装后，请确认以下项目：

- [ ] Rust 1.70+ 已安装
- [ ] 系统依赖已安装
- [ ] AI模型API密钥已配置
- [ ] 测试项目运行成功
- [ ] 编辑器环境已配置（可选）

---

## 🎯 下一步

安装完成后，你可以：

1. **[快速开始](quick-start.md)** - 5分钟创建第一个Agent
2. **[基础教程](../tutorials/basics/README.md)** - 系统学习核心概念
3. **[示例集合](../examples/README.md)** - 查看实际应用案例

---

**🎉 恭喜完成LumosAI环境搭建！**

*[← 返回 Getting Started](README.md)*