# LumosAI框架

LumosAI是一个用Rust语言构建的现代化AI Agent框架。此外，我们还提供了完整的JavaScript客户端和UI组件库。

## 项目结构（Monorepo）

这个仓库采用monorepo结构，包含多个相互关联的包：

```
lumosai/
├── packages/         # JavaScript包
│   ├── client-js/    # JavaScript客户端
│   └── ...           # 未来的其他包
├── lumosai_ui/       # UI组件库和演示界面
├── lumosai_core/     # 核心Rust库
├── lumosai_cli/      # 命令行工具
└── ...               # 其他相关项目
```

> 📝 **更多详情:** 查看 [Monorepo使用指南](./MONOREPO_GUIDE.md) 了解完整的目录结构和更详细的使用说明。

## JavaScript开发指南

这个仓库使用pnpm作为包管理工具，并通过workspace功能管理多个JS包。

### 安装依赖

```bash
pnpm install
```

### 构建所有JS包

```bash
pnpm build:all
```

### 单独开发UI

```bash
pnpm dev:ui
```

### 单独开发客户端库

```bash
pnpm dev:client
```

## 使用LumosAI JavaScript客户端

```typescript
import { LumosAIClient } from '@lumosai/client-js';

// 初始化客户端
const client = new LumosAIClient({
  apiKey: 'your-api-key',
  baseUrl: 'https://api.lumosai.com', // 可选，默认为官方API
});

// 使用代理
const agent = client.getAgent('agent-id');
const response = await agent.generate('你好，请介绍一下你自己');

console.log(response.message.content);
```

# Lumosai - Rust语言的AI Agent框架

Lumosai是一个用Rust实现的AI Agent框架，专注于性能、安全性和可扩展性。它提供了创建、管理和部署智能代理的工具和抽象，使开发者能够轻松构建高效的AI应用。

## 主要特性

- **高性能**：使用Rust语言实现，提供优秀的性能和内存安全
- **模块化设计**：核心框架、工具库和适配器的清晰分离
- **类型安全**：利用Rust的类型系统确保API使用的正确性
- **灵活扩展**：支持自定义工具、代理和LLM适配器
- **异步优先**：从设计之初就支持异步操作
- **内存管理**：提供多种内存存储选项
- **宏支持**：通过过程宏简化API使用
- **DSL语法**：提供受Mastra启发的声明式DSL，简化工作流、RAG、评估和MCP集成

## 项目结构

- `lumosai_core`：核心库，包含基本抽象和接口
  - `agent`：Agent trait和实现
  - `tool`：Tool trait和实现
  - `memory`：内存和状态管理
  - `llm`：LLM适配器和抽象
  - `eval`：评估和测试框架
  - `rag`：检索增强生成支持
  - `mcp`：MCP（Mastra Compatible Protocol）支持
- `lumosai_rag`：检索增强生成库，提供扩展的RAG功能
- `lumosai_evals`：评估和测试框架，提供全面的评估工具
- `lumosai_examples`：示例代码，展示框架使用方法
- `lumos_macro`：宏库，提供简化API使用的过程宏
- `docs`：文档

## 安装

添加依赖到你的`Cargo.toml`：

```toml
[dependencies]
lumosai_core = "0.1.0"
```

若要使用宏功能，启用`macros`特性：

```toml
[dependencies]
lumosai_core = { version = "0.1.0", features = ["macros"] }
lumos_macro = "0.1.0"
```

若要使用RAG或评估功能：

```toml
[dependencies]
lumosai_core = "0.1.0"
lumosai_rag = "0.1.0"
lumosai_evals = "0.1.0"
```

## 快速开始

### 基础使用示例

```rust
use lumosai_core::{Result, Error};
use lumosai_core::agent::{Agent, SimpleAgent};
use lumosai_core::tool::{Tool, FunctionTool};
use lumosai_core::llm::{LlmProvider, OpenAiAdapter};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建LLM适配器
    let llm = Arc::new(OpenAiAdapter::new(
        "your-api-key",
        "gpt-4",
    ));
    
    // 创建工具
    let calculator = FunctionTool::new(
        "calculator",
        "执行基础数学计算",
        |params| async move {
            // 工具实现...
            Ok(serde_json::json!({"result": 42}))
        },
    );
    
    // 创建代理
    let mut agent = SimpleAgent::new(
        "math_helper",
        "你是一个擅长数学的助手。",
        llm,
    );
    
    // 注册工具
    agent.add_tool(calculator);
    
    // 运行代理
    let response = agent.run("计算 (15 + 27) * 2").await?;
    println!("代理回答: {}", response);
    
    Ok(())
}
```

## 许可证

本项目采用MIT许可证 - 详见 [LICENSE](LICENSE) 文件
