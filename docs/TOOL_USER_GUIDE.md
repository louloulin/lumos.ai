# LumosAI 工具系统用户手册

欢迎使用 LumosAI 宏驱动工具系统！本手册将帮助您快速上手并掌握工具系统的使用。

## 📚 目录

- [快速入门](#快速入门)
- [核心概念](#核心概念)
- [工具分类](#工具分类)
- [使用示例](#使用示例)
- [API 参考](#api-参考)
- [常见问题](#常见问题)
- [最佳实践](#最佳实践)

---

## 🚀 快速入门

### 安装

在您的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
lumosai_core = "0.2.0"
lumos_macro = "0.2.0"
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 第一个工具

```rust
use lumosai_core::tool::builtin::macro_tools::*;
use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建工具实例
    let tool = read_file_tool();
    
    // 2. 准备参数
    let params = json!({
        "path": "/tmp/test.txt",
        "encoding": "utf-8",
        "max_size": 1048576
    });
    
    // 3. 创建执行上下文
    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();
    
    // 4. 执行工具
    let result = tool.execute(params, context, &options).await?;
    
    // 5. 处理结果
    println!("Result: {}", result);
    
    Ok(())
}
```

### 运行示例

```bash
# 运行基础示例
cargo run --example macro_tools_demo

# 运行错误处理示例
cargo run --example error_handling_demo
```

---

## 💡 核心概念

### 1. 工具 (Tool)

工具是 LumosAI 中执行特定任务的基本单元。每个工具都实现了 `Tool` trait：

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> &Value;
    async fn execute(
        &self,
        params: Value,
        context: ToolExecutionContext,
        options: &ToolExecutionOptions,
    ) -> Result<Value>;
}
```

### 2. 宏驱动工具

使用 `#[tool]` 宏可以自动生成 `Tool` trait 的实现：

```rust
use lumos_macro::tool;
use serde_json::{json, Value};
use lumosai_core::Result;

#[tool(
    name = "my_tool",
    description = "My custom tool"
)]
async fn my_tool(
    required_param: String,
    optional_param: Option<i64>,
) -> Result<Value> {
    Ok(json!({
        "result": format!("Processed: {}", required_param),
        "optional": optional_param
    }))
}
```

### 3. 参数类型

工具支持以下参数类型：

| Rust 类型 | JSON 类型 | 说明 |
|-----------|-----------|------|
| `String` | string | 字符串 |
| `i64`, `i32`, `u64`, `u32` | integer | 整数 |
| `f64`, `f32` | number | 浮点数 |
| `bool` | boolean | 布尔值 |
| `Value` | object/array | 任意 JSON 值 |
| `Option<T>` | - | 可选参数 |

### 4. 执行上下文

`ToolExecutionContext` 提供工具执行时的上下文信息：

```rust
pub struct ToolExecutionContext {
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub metadata: HashMap<String, Value>,
}
```

### 5. 执行选项

`ToolExecutionOptions` 控制工具的执行行为：

```rust
pub struct ToolExecutionOptions {
    pub timeout: Option<Duration>,
    pub retry_count: u32,
    pub cache_enabled: bool,
}
```

---

## 🗂️ 工具分类

### 文件操作工具 (4个)

| 工具名称 | 函数名 | 说明 |
|---------|--------|------|
| file_reader_v2 | `read_file_tool()` | 读取文件内容 |
| file_writer_v2 | `write_file_tool()` | 写入文件内容 |
| directory_lister_v2 | `list_directory_tool()` | 列出目录内容 |
| file_info_v2 | `get_file_info_tool()` | 获取文件信息 |

### 网络请求工具 (3个)

| 工具名称 | 函数名 | 说明 |
|---------|--------|------|
| http_get_v2 | `http_get_tool()` | HTTP GET 请求 |
| http_post_v2 | `http_post_tool()` | HTTP POST 请求 |
| api_call_v2 | `api_call_tool()` | 通用 API 调用 |

### 数据处理工具 (3个)

| 工具名称 | 函数名 | 说明 |
|---------|--------|------|
| json_parser_v2 | `parse_json_tool()` | JSON 解析和验证 |
| text_processor_v2 | `process_text_tool()` | 文本处理 |
| data_converter_v2 | `convert_data_tool()` | 数据格式转换 |

---

## 📖 使用示例

### 示例 1: 文件读取

```rust
use lumosai_core::tool::builtin::macro_tools::read_file_tool;
use lumosai_core::tool::Tool;
use serde_json::json;

async fn read_file_example() -> Result<(), Box<dyn std::error::Error>> {
    let tool = read_file_tool();
    
    let params = json!({
        "path": "/tmp/test.txt",
        "encoding": "utf-8",
        "max_size": 1048576  // 1MB
    });
    
    let context = Default::default();
    let options = Default::default();
    
    let result = tool.execute(params, context, &options).await?;
    
    if result["success"].as_bool().unwrap_or(false) {
        println!("Content: {}", result["content"]);
    } else {
        println!("Error: {}", result["error"]);
    }
    
    Ok(())
}
```

### 示例 2: HTTP 请求

```rust
use lumosai_core::tool::builtin::macro_tools::http_get_tool;
use lumosai_core::tool::Tool;
use serde_json::json;

async fn http_get_example() -> Result<(), Box<dyn std::error::Error>> {
    let tool = http_get_tool();
    
    let params = json!({
        "url": "https://api.github.com/users/octocat",
        "timeout_seconds": 30
    });
    
    let context = Default::default();
    let options = Default::default();
    
    let result = tool.execute(params, context, &options).await?;
    
    println!("Response: {}", result);
    
    Ok(())
}
```

### 示例 3: JSON 解析

```rust
use lumosai_core::tool::builtin::macro_tools::parse_json_tool;
use lumosai_core::tool::Tool;
use serde_json::json;

async fn json_parse_example() -> Result<(), Box<dyn std::error::Error>> {
    let tool = parse_json_tool();
    
    let params = json!({
        "input": r#"{"name": "Alice", "age": 30}"#,
        "validate_schema": true
    });
    
    let context = Default::default();
    let options = Default::default();
    
    let result = tool.execute(params, context, &options).await?;
    
    if result["valid"].as_bool().unwrap_or(false) {
        println!("Parsed data: {}", result["data"]);
    } else {
        println!("Validation errors: {}", result["errors"]);
    }
    
    Ok(())
}
```

### 示例 4: 批量执行

```rust
use lumosai_core::tool::builtin::macro_tools::process_text_tool;
use lumosai_core::tool::Tool;
use serde_json::json;
use futures::future::join_all;

async fn batch_execution_example() -> Result<(), Box<dyn std::error::Error>> {
    let tool = process_text_tool();
    let context = Default::default();
    let options = Default::default();
    
    let texts = vec!["hello", "world", "rust"];
    
    let tasks: Vec<_> = texts.iter().map(|text| {
        let tool = tool.clone();
        let context = context.clone();
        let options = options.clone();
        async move {
            let params = json!({
                "text": text,
                "operation": "uppercase"
            });
            tool.execute(params, context, &options).await
        }
    }).collect();
    
    let results = join_all(tasks).await;
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(value) => println!("Result {}: {}", i, value),
            Err(e) => println!("Error {}: {}", i, e),
        }
    }
    
    Ok(())
}
```

### 示例 5: 自定义工具

```rust
use lumos_macro::tool;
use serde_json::{json, Value};
use lumosai_core::Result;

#[tool(
    name = "calculator",
    description = "Simple calculator tool"
)]
async fn calculator(
    operation: String,
    a: f64,
    b: f64,
) -> Result<Value> {
    let result = match operation.as_str() {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return Err(lumosai_core::Error::Tool("Division by zero".to_string()));
            }
            a / b
        }
        _ => return Err(lumosai_core::Error::Tool("Unknown operation".to_string())),
    };
    
    Ok(json!({
        "operation": operation,
        "a": a,
        "b": b,
        "result": result
    }))
}

// 使用自定义工具
async fn use_calculator() -> Result<(), Box<dyn std::error::Error>> {
    let tool = calculator_tool();
    
    let params = json!({
        "operation": "add",
        "a": 10.0,
        "b": 20.0
    });
    
    let context = Default::default();
    let options = Default::default();
    
    let result = tool.execute(params, context, &options).await?;
    println!("Result: {}", result["result"]); // 30.0
    
    Ok(())
}
```

---

## 📚 API 参考

### Tool Trait

```rust
pub trait Tool: Send + Sync {
    /// 获取工具名称
    fn name(&self) -> &str;
    
    /// 获取工具描述
    fn description(&self) -> &str;
    
    /// 获取参数定义 (JSON Schema)
    fn parameters(&self) -> &Value;
    
    /// 执行工具
    async fn execute(
        &self,
        params: Value,
        context: ToolExecutionContext,
        options: &ToolExecutionOptions,
    ) -> Result<Value>;
}
```

### #[tool] 宏

```rust
#[tool(
    name = "tool_name",           // 必需: 工具名称
    description = "description",  // 必需: 工具描述
    examples = ["example1"],      // 可选: 使用示例
    tags = ["tag1", "tag2"],      // 可选: 标签
    version = "1.0.0"             // 可选: 版本号
)]
async fn tool_function(
    param1: Type1,                // 必需参数
    param2: Option<Type2>,        // 可选参数
) -> Result<Value> {
    // 工具实现
}
```

---

## ❓ 常见问题

### Q1: 如何处理可选参数？

使用 `Option<T>` 类型：

```rust
#[tool(name = "my_tool", description = "My tool")]
async fn my_tool(
    required: String,
    optional: Option<i64>,  // 可选参数
) -> Result<Value> {
    let value = optional.unwrap_or(42);  // 提供默认值
    Ok(json!({"value": value}))
}
```

### Q2: 如何返回错误？

使用 `Error::Tool`：

```rust
if invalid_condition {
    return Err(Error::Tool("Invalid input".to_string()));
}
```

### Q3: 如何接受任意 JSON 值？

使用 `Value` 类型：

```rust
#[tool(name = "my_tool", description = "My tool")]
async fn my_tool(
    data: Value,  // 接受任意 JSON 值
) -> Result<Value> {
    Ok(json!({"received": data}))
}
```

### Q4: 如何实现超时控制？

使用 `ToolExecutionOptions`：

```rust
let options = ToolExecutionOptions {
    timeout: Some(Duration::from_secs(30)),
    ..Default::default()
};
```

### Q5: 如何获取所有可用工具？

```rust
use lumosai_core::tool::builtin::macro_tools::get_all_macro_tools;

let tools = get_all_macro_tools();
for tool in tools {
    println!("Tool: {} - {}", tool.name(), tool.description());
}
```

---

## 🎯 最佳实践

### 1. 参数验证

```rust
// ✅ 推荐: 使用强类型参数
#[tool(name = "my_tool", description = "My tool")]
async fn my_tool(path: String, size: u64) -> Result<Value> {
    // 参数已经验证
}

// ❌ 避免: 手动验证参数
async fn my_tool(params: Value) -> Result<Value> {
    let path = params["path"].as_str().ok_or(...)?;
    let size = params["size"].as_u64().ok_or(...)?;
}
```

### 2. 错误处理

```rust
// ✅ 推荐: 提供详细的错误信息
if !path.exists() {
    return Err(Error::Tool(format!(
        "File not found: {}",
        path.display()
    )));
}

// ❌ 避免: 模糊的错误信息
if !path.exists() {
    return Err(Error::Tool("Error".to_string()));
}
```

### 3. 返回值结构

```rust
// ✅ 推荐: 结构化的返回值
Ok(json!({
    "success": true,
    "data": result,
    "metadata": {
        "timestamp": Utc::now().to_rfc3339(),
        "version": "1.0.0"
    }
}))

// ❌ 避免: 不一致的返回值
Ok(json!(result))  // 有时返回对象，有时返回字符串
```

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-18  
**维护者**: LumosAI 开发团队

