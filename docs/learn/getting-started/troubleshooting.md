# 🔧 故障排除

**常见问题解决方案和调试指南**

## ⏱️ 快速诊断

### 1分钟检查清单

在深入排查前，先确认以下基本项目：

```bash
# 检查Rust版本
rustc --version  # 需要1.70+

# 检查网络连接
curl -I https://api.openai.com/v1/models

# 检查环境变量
echo $OPENAI_API_KEY

# 检查项目编译
cargo check
```

---

## 🚨 常见错误及解决方案

### 安装相关问题

#### Rust版本过低
**错误**: `error: package compiler v1.70.0 is required but your version is 1.65.0`

**解决方案**:
```bash
rustup update stable
rustup default stable
rustc --version  # 确认版本
```

#### SSL/TLS连接失败
**错误**: `failed to resolve: SSL error` 或 `certificate verify failed`

**解决方案**:
```bash
# Ubuntu/Debian
sudo apt update && sudo apt install libssl-dev pkg-config

# macOS
brew install openssl
export PKG_CONFIG_PATH="/usr/local/opt/openssl/lib/pkgconfig"

# Rust SSL证书更新
curl -o /etc/ssl/cert.pem https://curl.se/ca/cacert.pem
export SSL_CERT_FILE=/etc/ssl/cert.pem
```

#### 权限问题
**错误**: `Permission denied` 或 `Access denied`

**解决方案**:
```bash
# 修复cargo权限
chmod +x ~/.cargo/bin/*

# 重新安装rustup（如果问题持续）
rustup self uninstall
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### API相关问题

#### API密钥无效
**错误**: `API key not found or invalid` 或 `401 Unauthorized`

**解决方案**:
```bash
# 1. 检查环境变量
echo $OPENAI_API_KEY

# 2. 重新设置密钥
export OPENAI_API_KEY="sk-your-actual-key-here"

# 3. 验证密钥格式（OpenAI密钥以sk-开头）
echo $OPENAI_API_KEY | grep "^sk-"

# 4. 测试API连接
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
     https://api.openai.com/v1/models
```

#### 网络连接问题
**错误**: `Network error` 或 `Connection timeout`

**解决方案**:
```bash
# 1. 检查基础网络连接
ping api.openai.com

# 2. 检查代理设置
echo $HTTP_PROXY
echo $HTTPS_PROXY

# 3. 如果在中国大陆，可能需要代理
export HTTP_PROXY=http://127.0.0.1:7890
export HTTPS_PROXY=http://127.0.0.1:7890

# 4. 或使用国内镜像
export LUMOSAI_API_BASE="https://api.openai-proxy.com/v1"
```

#### 模型不存在
**错误**: `Model not found` 或 `Invalid model`

**解决方案**:
```rust
// 检查模型名称拼写
let agent = Agent::builder()
    .model("gpt-4")  // 正确
    // .model("gpt4") // 错误
    .build()
    .await?;

// 查看可用模型
let models = vec![
    "gpt-4", "gpt-4-turbo", "gpt-3.5-turbo",
    "claude-3-opus", "claude-3-sonnet", "claude-3-haiku"
];
```

### 编译相关问题

#### 依赖编译失败
**错误**: `Could not find crate xxx` 或 `Failed to compile`

**解决方案**:
```bash
# 1. 更新依赖索引
cargo update

# 2. 清理编译缓存
cargo clean

# 3. 重新编译
cargo build --verbose

# 4. 如果特定crate有问题，尝试重新解析
cargo update -p crate_name
```

#### 特性冲突
**错误**: `duplicate definition` 或 `conflicting crate versions`

**解决方案**:
```bash
# 检查依赖树
cargo tree | grep "crate_name"

# 更新Cargo.toml，明确指定版本
[dependencies]
lumosai = "0.2.0"
tokio = { version = "1.0", features = ["full"] }

# 清理并重新编译
cargo clean && cargo build
```

### 运行时问题

#### Agent创建失败
**错误**: `Failed to create agent` 或 `Invalid configuration`

**调试代码**:
```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() {
    // 逐步调试Agent创建
    match Agent::builder()
        .name("test")
        .model("gpt-3.5-turbo")  // 使用稳定模型
        .system_prompt("Test")
        .build()
        .await 
    {
        Ok(agent) => println!("✅ Agent创建成功"),
        Err(e) => {
            eprintln!("❌ Agent创建失败: {}", e);
            eprintln!("🔍 详细错误: {:?}", e);
        }
    }
}
```

#### 内存不足
**错误**: `Out of memory` 或程序响应缓慢

**解决方案**:
```rust
// 限制对话历史长度
let agent = Agent::builder()
    .memory(ConversationMemory::with_max_history(10))
    .build()
    .await?;

// 定期清理内存
if agent.get_conversation_history().len() > 50 {
    agent.clear_memory().await?;
}

// 监控内存使用
use std::process::Command;
fn check_memory() {
    let output = Command::new("ps")
        .arg("aux")
        .output()
        .expect("Failed to execute command");
    println!("Memory usage: {}", String::from_utf8_lossy(&output.stdout));
}
```

---

## 🔍 调试技巧

### 启用详细日志

```rust
// 在main.rs中设置日志
use log::debug;

fn main() {
    env_logger::init();
    
    debug!("开始调试程序...");
    
    // 你的代码
}
```

```bash
# 运行时查看详细日志
RUST_LOG=debug cargo run
```

### 分步调试

```rust
async fn debug_agent_creation() -> Result<()> {
    println!("🔍 步骤1: 检查环境变量");
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY未设置")?;
    println!("✅ API密钥已设置: {}...", &api_key[..10]);

    println!("🔍 步骤2: 创建基础配置");
    let config = AgentConfig {
        name: "debug-agent".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        // ... 其他配置
    };
    println!("✅ 配置创建成功");

    println!("🔍 步骤3: 创建Agent");
    let agent = Agent::from_config(config).await?;
    println!("✅ Agent创建成功");

    Ok(())
}
```

### 网络调试

```bash
# 测试API连接
curl -v -H "Authorization: Bearer $OPENAI_API_KEY" \
     https://api.openai.com/v1/chat/completions \
     -d '{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"Hello"}],"max_tokens":5}'

# 检查DNS解析
nslookup api.openai.com

# 检查端口连通性
telnet api.openai.com 443
```

---

## 🛠️ 环境特定问题

### Windows

```powershell
# PowerShell环境变量设置
[Environment]::SetEnvironmentVariable("OPENAI_API_KEY", "your-key", "User")

# 检查Windows版本兼容性
systeminfo | findstr /B /C:"OS Name"

# 安装Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools
```

### macOS

```bash
# Xcode命令行工具
xcode-select --install

# Homebrew依赖
brew install openssl pkg-config rust

# 设置环境变量
echo 'export OPENAI_API_KEY="your-key"' >> ~/.zshrc
source ~/.zshrc
```

### Linux

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum install gcc gcc-c++ make openssl-devel

# 设置环境变量
echo 'export OPENAI_API_KEY="your-key"' >> ~/.bashrc
source ~/.bashrc
```

---

## 📞 获取更多帮助

### 自助资源

1. **查看完整日志**: `RUST_LOG=debug cargo run 2>&1 | tee debug.log`
2. **检查已知问题**: [GitHub Issues](https://github.com/louloulin/lumos.ai/issues)
3. **查看文档**: [在线文档](https://docs.rs/lumosai)

### 社区支持

1. **GitHub Discussions**: [技术讨论](https://github.com/louloulin/lumos.ai/discussions)
2. **提交Issue**: [问题报告](https://github.com/louloulin/lumos.ai/issues/new)
3. **FAQ页面**: [常见问题](../community/faq.md)

### 报告问题时请包含

- **操作系统**: `uname -a` 或系统信息
- **Rust版本**: `rustc --version`
- **完整错误信息**: 包括堆栈跟踪
- **最小复现代码**: 能够重现问题的最简代码
- **配置信息**: 相关的环境变量和配置

---

## ✅ 问题解决检查清单

当问题解决后，确认以下项目：

- [ ] 程序能够正常编译
- [ ] Agent能够成功创建
- [ ] API调用正常工作
- [ ] 没有明显的性能问题
- [ ] 错误日志已清理

---

**🎉 希望这个故障排除指南能帮助你解决问题！**

*[← 返回 Getting Started](README.md)*