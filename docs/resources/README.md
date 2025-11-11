# 📦 资源中心

> 实用工具、示例代码和参考资料

本部分提供 LumosAI 开发过程中需要的各种资源，包括示例代码、项目模板、术语表和常见问题解答。

## 📂 资源导航

- **[示例代码](examples/)** - 丰富的可运行示例集合
- **[项目模板](templates/)** - 快速启动项目模板
- **[术语表](glossary.md)** - 专业术语解释
- **[常见问题](faq.md)** - FAQ 和问题解决

---

## 💡 快速查找

### 按需求查找
- **🎯 我想要** → [使用场景导航](#使用场景导航)
- **🆕 我是新手** → [新手路径](#新手路径)
- **🔧 我是开发者** → [开发者资源](#开发者资源)

### 按主题查找
- **Agent 相关** → [Agent 示例](examples/#agent-示例)
- **RAG 系统** → [RAG 示例](examples/#rag-示例)
- **工具集成** → [工具示例](examples/#工具示例)
- **部署配置** → [部署模板](templates/)

---

## 🎯 使用场景导航

### 我想要...

#### 🚀 快速体验
```bash
# 5分钟快速开始
cargo new my_ai_app
cd my_ai_app
cargo add lumosai tokio

# 创建基础 Agent
cargo run --example basic_agent
```

#### 🔧 构建知识问答系统
- 查看: [RAG 基础教程](../tutorials/basics/rag-basics.md)
- 示例: `examples/rag_system.rs`
- 模板: `templates/rag-chatbot/`

#### 🤖 创建多 Agent 应用
- 查看: [多 Agent 协作教程](../tutorials/intermediate/multi-agent.md)
- 示例: `examples/multi_agent_workflow.rs`
- 模板: `templates/agent-team/`

#### 🏢 企业级部署
- 查看: [企业部署指南](../tutorials/advanced/enterprise.md)
- 示例: `examples/enterprise_validation.rs`
- 模板: `templates/production/`

---

## 🆕 新手路径

### Day 1-7: 基础入门
1. **安装和环境设置** (30分钟)
2. **[快速开始指南](../getting-started/quick-start.md)** (1小时)
3. **[Agent 基础教程](../tutorials/basics/agent-basics.md)** (2小时)
4. **运行第一个示例** (30分钟)

### Week 2-3: 核心功能
1. **[RAG 基础教程](../tutorials/basics/rag-basics.md)** (3小时)
2. **[工具集成教程](../tutorials/basics/tool-integration.md)** (2小时)
3. **尝试修改示例代码** (2小时)

### Week 4: 实践项目
1. **选择项目模板** (30分钟)
2. **定制功能实现** (5小时)
3. **测试和优化** (2小时)

---

## 🔧 开发者资源

### 代码示例

```rust
// 基础 Agent 使用
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple("gpt-4", "You are helpful").await?;
    let response = agent.chat("Hello!").await?;
    println!("{}", response);
    Ok(())
}
```

### 配置示例

```toml
# Cargo.toml
[dependencies]
lumosai = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### 环境变量

```bash
# API 密钥配置
export OPENAI_API_KEY="your_openai_key"
export DEEPSEEK_API_KEY="your_deepseek_key"
export ANTHROPIC_API_KEY="your_anthropic_key"

# 可选配置
export LUMOSAI_LOG_LEVEL="info"
export LUMOSAI_CACHE_DIR="/tmp/lumosai"
```

---

## 🛠️ 开发工具

### IDE 配置

#### VS Code
推荐插件：
- **Rust Analyzer** - Rust 语言支持
- **Better TOML** - TOML 文件支持
- **Error Lens** - 内联错误显示

#### 配置文件 (.vscode/settings.json)
```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.imports.granularity.group": "module",
    "files.associations": {
        "*.rs": "rust"
    }
}
```

### 命令行工具

```bash
# 项目初始化
cargo new my_lumosai_app --bin
cd my_lumosai_app

# 依赖管理
cargo add lumosai tokio serde
cargo add serde_json --optional
cargo add uuid --features "v4"

# 开发命令
cargo run                    # 运行应用
cargo test                   # 运行测试
cargo clippy                 # 代码检查
cargo fmt                    # 代码格式化
cargo doc                    # 生成文档
cargo expand                 # 展开宏
```

---

## 📚 学习资源

### 官方文档
- **[完整文档](../README.md)** - 完整文档中心
- **[API 参考](../api-reference/)** - 完整 API 文档
- **[核心概念](../concepts/)** - 深入理解设计理念

### 外部资源
- **Rust 官方文档** - https://doc.rust-lang.org/
- **Tokio 教程** - https://tokio.rs/tokio/tutorial
- **OpenAI API 文档** - https://platform.openai.com/docs
- **Anthropic API 文档** - https://docs.anthropic.com/

### 社区资源
- **GitHub 仓库** - https://github.com/louloulin/lumos.ai
- **Discord 社区** - 实时讨论和帮助
- **论坛** - 深度技术讨论
- **Stack Overflow** - #lumosai 标签

---

## 🆘 故障排除

### 常见问题快速解决

#### 编译错误
```bash
# 清理缓存
cargo clean

# 重新生成
cargo build

# 更新依赖
cargo update
```

#### API 调用失败
```bash
# 检查 API 密钥
echo $OPENAI_API_KEY

# 测试连接
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
     https://api.openai.com/v1/models
```

#### 性能问题
- 检查网络连接
- 调整模型参数
- 使用缓存机制
- 考虑异步处理

### 获取帮助

1. **查看 FAQ** - [常见问题解答](faq.md)
2. **搜索文档** - 使用页面搜索功能
3. **社区提问** - GitHub Issues 或 Discord
4. **报告问题** - 包含详细的错误信息和复现步骤

---

## 🔄 版本信息

- **当前版本**: v0.2.0
- **Rust 要求**: 1.70+
- **最低内存**: 512MB
- **推荐内存**: 2GB+

### 更新日志

查看 [CHANGELOG.md](../../CHANGELOG.md) 了解版本更新详情。

---

## 🌟 贡献指南

欢迎为 LumosAI 贡献资源！

### 如何贡献

1. **示例代码**: 提交新的可运行示例
2. **项目模板**: 创建新的项目模板
3. **文档改进**: 修正错误和补充内容
4. **问题解答**: 帮助回答社区问题

### 贡献流程

1. Fork 项目仓库
2. 创建功能分支
3. 提交更改
4. 创建 Pull Request
5. 等待代码审查

详细指南请参考 [贡献指南](../contributing/documentation.md)。

---

**🎉 希望这些资源对您的 LumosAI 开发之旅有帮助！**