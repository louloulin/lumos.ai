# 教程 02：工具（Tool）与 Agent 集成

> 本教程讲解如何为 Agent 集成工具，包括创建自定义工具与使用内置工具集（Web/File 等）。示例将严格匹配 `lumosai_core::tool::Tool` 的签名，并在函数级添加注释。

## 🔧 Tool Trait 核心签名
- 位置：`lumosai_core/src/tool/tool.rs`
- 关键方法：
  - `id(&self) -> &str`
  - `description(&self) -> &str`
  - `schema(&self) -> ToolSchema`
  - `output_schema(&self) -> Option<Value>`（可选）
  - `category(&self) -> Option<String>`（可选）
  - `async fn execute(&self, params: Value, context: ToolExecutionContext, options: &ToolExecutionOptions) -> Result<Value>`

## 🧪 最小实现：使用 `GenericTool`
- 建议用 `GenericTool::new(id, description, schema, execute_fn)` 快速创建工具。

```rust
use lumosai::prelude::*;
use lumosai_core::tool::{GenericTool, ToolSchema, ParameterSchema};
use serde_json::Value;

/// 创建一个简单的词数统计工具
fn create_word_count_tool() -> GenericTool<impl Fn(Value, lumosai_core::tool::ToolExecutionContext) -> lumosai_core::Result<Value> + Clone + Send + Sync + 'static> {
    // 参数模式：输入文本
    let schema = ToolSchema {
        name: "word_count".into(),
        description: "统计文本中的词数量".into(),
        parameters: vec![ParameterSchema {
            name: "text".into(),
            description: "待统计的文本".into(),
            r#type: "string".into(),
            required: true,
            properties: None,
            default: None,
        }],
        examples: None,
        returns: Some("{ count: number }".into()),
        version: Some("1.0".into()),
        metadata: None,
    };

    // 执行闭包：统计词数
    let exec = |params: Value, _ctx: lumosai_core::tool::ToolExecutionContext| -> lumosai_core::Result<Value> {
        let text = params["text"].as_str().unwrap_or("");
        let count = text.split_whitespace().count();
        Ok(serde_json::json!({ "count": count }))
    };

    GenericTool::new("word_count", "统计文本词数", schema, exec)
}

#[tokio::main]
/// 将自定义工具集成到 Agent，并演示调用
async fn main() -> lumosai_core::Result<()> {
    let tool = create_word_count_tool();

    // 通过 AgentBuilder 添加工具
    let agent = lumosai_core::agent::AgentBuilder::new()
        .name("工具助手")
        .instructions("当需要统计词数时，调用 word_count 工具")
        .model_name("gpt-4o")
        .tool(Box::new(tool))
        .build()?;

    // 在真实调用中，LLM 将根据指令自动选择工具；此处直接展示工具调用
    let result = agent.invoke_tool("word_count", serde_json::json!({ "text": "Rust 是一个系统级语言" })).await?;
    println!("工具返回: {}", result);
    Ok(())
}
```

## 🌐 使用内置工具集
- Web 工具集：`AgentBuilder::with_web_tools()` 包含 HTTP 请求、网页抓取、JSON API 调用与 URL 校验。

```rust
use lumosai::prelude::*;

#[tokio::main]
/// 使用内置 Web 工具集构建 Agent
async fn main() -> Result<()> {
    let agent = lumosai_core::agent::AgentBuilder::new()
        .name("Web 能力助手")
        .instructions("当需要访问网络时，请使用内置工具")
        .model_name("gpt-4o")
        .with_web_tools()
        .build()?;

    let resp = agent.generate("抓取 https://example.com 的标题，并简要说明").await?;
    println!("回复: {}", resp);
    Ok(())
}
```

## 📁 文件工具集
- File 工具集：`AgentBuilder::with_file_tools()` 包含读写文件、列目录、获取元数据等。

```rust
use lumosai::prelude::*;

#[tokio::main]
/// 使用文件工具集构建 Agent 以进行基本文件操作
async fn main() -> Result<()> {
    let agent = lumosai_core::agent::AgentBuilder::new()
        .name("文件助手")
        .instructions("需要本地文件操作时调用工具")
        .model_name("gpt-4o")
        .with_file_tools()
        .build()?;

    let resp = agent.generate("读取当前目录下 README.md 的前100字符").await?;
    println!("回复: {}", resp);
    Ok(())
}
```

## 🧱 进阶：Arc 工具与批量添加
```rust
use lumosai::prelude::*;
use std::sync::Arc;

#[tokio::main]
/// 展示 add_tool(Arc<dyn Tool>) 与 tools(Vec<Box<dyn Tool>>) 用法
async fn main() -> Result<()> {
    let t1 = Box::new(create_word_count_tool());
    let t2 = Box::new(create_word_count_tool()); // 假设另一个变体

    let agent = lumosai_core::agent::AgentBuilder::new()
        .name("多工具助手")
        .instructions("根据任务选择合适工具")
        .model_name("gpt-4o")
        .tools(vec![t1, t2])
        .build()?;

    Ok(())
}
```

## ✅ 最佳实践
- 为工具提供精确的 `schema` 与 `description`，便于 LLM 调度。
- 在 `execute` 中进行参数校验与错误处理，返回结构化 JSON。
- 使用分类（`category()`）与输出模式（`output_schema()`）提升可维护性。
- 在生产中结合日志与监控（`Base`/`Telemetry`）以追踪工具调用。

---

若你需要某类工具（如数据库查询、第三方 API），告诉我具体需求，我会为你的场景提供完整实现与测试示例。