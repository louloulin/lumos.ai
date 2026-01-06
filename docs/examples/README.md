# LumosAI 示例项目

欢迎来到 LumosAI 示例项目集合！这里提供完整的示例项目，展示 LumosAI 的各种功能和使用场景。

## 📚 示例项目目录

### 🎯 基础示例

| 项目 | 描述 | 难度 | 时长 |
|------|------|------|------|
| [hello-world](./hello-world/) | 最简单的 Agent 示例 | 初级 | 5分钟 |
| [chatbot](./chatbot/) | 基础聊天机器人 | 初级 | 15分钟 |
| [tool-integration](./tool-integration/) | 工具集成示例 | 初级 | 20分钟 |

### 🚀 进阶示例

| 项目 | 描述 | 难度 | 时长 |
|------|------|------|------|
| [research-assistant](./research-assistant/) | 带工具的研究助手 | 中级 | 30分钟 |
| [rag-system](./rag-system/) | RAG 知识问答系统 | 中级 | 45分钟 |
| [multi-agent](./multi-agent/) | 多 Agent 协作系统 | 高级 | 60分钟 |

### 🏢 企业示例

| 项目 | 描述 | 难度 | 时长 |
|------|------|------|------|
| [workflow-automation](./workflow-automation/) | 工作流自动化 | 高级 | 90分钟 |
| [customer-service](./customer-service/) | 智能客服系统 | 高级 | 120分钟 |
| [document-analysis](./document-analysis/) | 文档分析系统 | 高级 | 90分钟 |

## 🎓 学习路径

### 新手路径（1-2 小时）
```
hello-world → chatbot → tool-integration
```

### 开发者路径（3-4 小时）
```
新手路径 → research-assistant → rag-system
```

### 架构师路径（6-8 小时）
```
开发者路径 → multi-agent → workflow-automation
```

## 📋 示例项目结构

每个示例项目都包含：

```
project-name/
├── README.md              # 项目说明和使用指南
├── Cargo.toml            # 项目依赖配置
├── src/
│   ├── main.rs           # 主程序入口
│   ├── lib.rs            # 库代码（如果有）
│   └── modules/          # 模块代码
├── examples/             # 额外示例
├── tests/                # 测试代码
├── docs/                 # 项目文档
└── assets/               # 资源文件
```

## 🚀 快速开始

### 1. 克隆示例项目

```bash
# 克隆整个仓库
git clone https://github.com/lumosai/lumosai.git
cd lumosai/docs/examples

# 或者只下载特定示例
curl -L https://github.com/lumosai/lumosai/archive/main.zip -o lumosai.zip
unzip lumosai.zip
cd lumosai-main/docs/examples
```

### 2. 选择示例项目

```bash
# 进入感兴趣的示例目录
cd hello-world

# 查看项目说明
cat README.md
```

### 3. 运行示例

```bash
# 设置环境变量
export OPENAI_API_KEY="your-api-key"

# 安装依赖并运行
cargo run
```

## 📖 示例项目详情

### 🌟 hello-world
**最简单的 Agent 示例**

展示如何创建和使用基本的 AI Agent：
- Agent 创建和配置
- 基本对话功能
- 错误处理

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("Hello Agent")
        .instructions("你是一个友好的助手")
        .model("gpt-3.5-turbo")
        .build()?;
    
    let response = agent.generate("你好！").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

### 💬 chatbot
**基础聊天机器人**

展示如何构建一个交互式聊天机器人：
- 持续对话循环
- 用户输入处理
- 对话历史管理
- 优雅退出机制

### 🔧 tool-integration
**工具集成示例**

展示如何为 Agent 添加工具能力：
- 使用 `#[tool]` 宏定义工具
- 工具注册和管理
- 工具执行和结果处理
- 错误处理和重试

### 🔍 research-assistant
**研究助手**

展示如何构建带有多种工具的研究助手：
- 网络搜索工具
- 文档处理工具
- 数据分析工具
- 结果整合和报告

### 📚 rag-system
**RAG 知识问答系统**

展示如何构建检索增强生成系统：
- 文档加载和处理
- 向量化和存储
- 语义搜索和检索
- 上下文增强生成

### 🤝 multi-agent
**多 Agent 协作系统**

展示如何设计和实现多 Agent 系统：
- Agent 间通信协议
- 任务分配和协调
- 结果聚合和整合
- 冲突解决机制

### ⚙️ workflow-automation
**工作流自动化**

展示如何构建复杂的自动化工作流：
- 工作流定义和执行
- 条件分支和循环
- 异常处理和恢复
- 监控和日志记录

## 🛠️ 开发工具

### 代码生成器

使用 LumosAI CLI 快速生成示例项目：

```bash
# 安装 CLI 工具
cargo install lumosai-cli

# 生成新项目
lumosai new my-project --template chatbot
cd my-project

# 运行项目
lumosai dev
```

### 调试工具

每个示例都包含调试配置：

```bash
# 启用详细日志
RUST_LOG=debug cargo run

# 使用调试器
rust-gdb target/debug/example-name
```

### 性能分析

```bash
# 性能分析
cargo build --release
perf record target/release/example-name
perf report
```

## 🧪 测试示例

### 运行所有测试

```bash
# 在示例根目录
cargo test --all

# 运行特定示例的测试
cd hello-world
cargo test
```

### 集成测试

```bash
# 运行集成测试
cargo test --test integration

# 运行端到端测试
cargo test --test e2e
```

## 📊 性能基准

每个示例都包含性能基准测试：

```bash
# 运行基准测试
cargo bench

# 查看基准报告
open target/criterion/report/index.html
```

## 🤝 贡献示例

我们欢迎社区贡献新的示例项目！

### 贡献指南

1. **选择主题**: 选择有实际价值的使用场景
2. **编写代码**: 遵循项目代码规范
3. **添加文档**: 提供详细的说明和注释
4. **测试验证**: 确保示例可以正常运行
5. **提交 PR**: 提交 Pull Request

### 示例模板

```rust
//! # 示例项目名称
//! 
//! 简短描述示例的功能和用途
//! 
//! ## 功能特性
//! - 特性1
//! - 特性2
//! 
//! ## 使用方法
//! ```bash
//! cargo run
//! ```

use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 示例代码
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_example() {
        // 测试代码
    }
}
```

### 文档要求

每个示例项目的 README.md 应包含：

1. **项目描述**: 清晰的功能说明
2. **前置要求**: 依赖和环境要求
3. **安装步骤**: 详细的安装指南
4. **使用说明**: 运行和使用方法
5. **代码解释**: 关键代码的解释
6. **扩展建议**: 进一步改进的建议

## 📞 获取帮助

### 示例相关问题

- 📖 查看示例项目的 README
- 🔍 搜索相关 Issue
- 💬 在社区提问

### 技术支持

- 🐛 [报告问题](https://github.com/lumosai/lumosai/issues)
- 💬 [社区讨论](https://github.com/lumosai/lumosai/discussions)
- 📧 [技术支持](mailto:support@lumosai.com)

---

*通过实际示例学习 LumosAI，快速掌握 AI 应用开发！* 🚀
