# LumosAI 工具系统 API 参考

本文档提供了 LumosAI 宏驱动工具系统的完整 API 参考。

## 📚 目录

- [核心 Trait](#核心-trait)
- [宏驱动工具](#宏驱动工具)
- [文件操作工具](#文件操作工具)
- [网络请求工具](#网络请求工具)
- [数据处理工具](#数据处理工具)
- [工具配置](#工具配置)
- [错误处理](#错误处理)

---

## 核心 Trait

### Tool

所有工具都实现了 `Tool` trait。

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

#### 方法

##### `name() -> &str`

返回工具的唯一名称。

**返回值**: 工具名称字符串

**示例**:
```rust
let tool = read_file_tool();
assert_eq!(tool.name(), "file_reader_v2");
```

##### `description() -> &str`

返回工具的描述信息。

**返回值**: 工具描述字符串

**示例**:
```rust
let tool = read_file_tool();
println!("Description: {}", tool.description());
```

##### `parameters() -> &Value`

返回工具的参数定义（JSON Schema 格式）。

**返回值**: JSON Schema 对象

**示例**:
```rust
let tool = read_file_tool();
let params_schema = tool.parameters();
println!("Parameters: {}", params_schema);
```

##### `execute(params, context, options) -> Result<Value>`

执行工具。

**参数**:
- `params: Value` - 工具参数（JSON 对象）
- `context: ToolExecutionContext` - 执行上下文
- `options: &ToolExecutionOptions` - 执行选项

**返回值**: `Result<Value>` - 执行结果或错误

**示例**:
```rust
let tool = read_file_tool();
let params = json!({"path": "/tmp/test.txt"});
let context = ToolExecutionContext::default();
let options = ToolExecutionOptions::default();
let result = tool.execute(params, context, &options).await?;
```

---

## 宏驱动工具

### #[tool] 宏

用于将异步函数转换为工具的过程宏。

```rust
#[tool(
    name = "tool_name",
    description = "Tool description",
    examples = ["example1", "example2"],  // 可选
    tags = ["tag1", "tag2"],              // 可选
    version = "1.0.0"                     // 可选
)]
async fn tool_function(
    param1: Type1,
    param2: Option<Type2>,
) -> Result<Value> {
    // 实现
}
```

#### 属性

- `name` (必需): 工具名称
- `description` (必需): 工具描述
- `examples` (可选): 使用示例数组
- `tags` (可选): 标签数组
- `version` (可选): 版本号

#### 支持的参数类型

| Rust 类型 | JSON 类型 | 说明 |
|-----------|-----------|------|
| `String` | string | 字符串 |
| `i8`, `i16`, `i32`, `i64`, `i128`, `isize` | integer | 有符号整数 |
| `u8`, `u16`, `u32`, `u64`, `u128`, `usize` | integer | 无符号整数 |
| `f32`, `f64` | number | 浮点数 |
| `bool` | boolean | 布尔值 |
| `Value` | object/array | 任意 JSON 值 |
| `Option<T>` | - | 可选参数 |

---

## 文件操作工具

### read_file_tool()

读取文件内容。

**函数签名**:
```rust
pub fn read_file_tool() -> Box<dyn Tool>
```

**参数**:

| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| path | String | ✅ | - | 文件路径 |
| encoding | String | ❌ | "utf-8" | 文件编码 |
| max_size | u64 | ❌ | 1048576 | 最大文件大小（字节） |

**返回值**:

```json
{
  "success": true,
  "path": "/tmp/test.txt",
  "content": "file content",
  "size": 12,
  "encoding": "utf-8",
  "timestamp": "2025-10-18T12:00:00Z"
}
```

**错误情况**:

```json
{
  "success": false,
  "error": "File not found",
  "path": "/tmp/test.txt"
}
```

**示例**:
```rust
let tool = read_file_tool();
let params = json!({
    "path": "/tmp/test.txt",
    "encoding": "utf-8",
    "max_size": 1048576
});
let result = tool.execute(params, context, &options).await?;
```

### write_file_tool()

写入内容到文件。

**函数签名**:
```rust
pub fn write_file_tool() -> Box<dyn Tool>
```

**参数**:

| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| path | String | ✅ | - | 文件路径 |
| content | String | ✅ | - | 文件内容 |
| encoding | String | ❌ | "utf-8" | 文件编码 |
| create_backup | bool | ❌ | false | 是否创建备份 |
| create_dirs | bool | ❌ | true | 是否创建父目录 |

**返回值**:

```json
{
  "success": true,
  "path": "/tmp/output.txt",
  "size": 13,
  "encoding": "utf-8",
  "backup_created": false,
  "timestamp": "2025-10-18T12:00:00Z"
}
```

**示例**:
```rust
let tool = write_file_tool();
let params = json!({
    "path": "/tmp/output.txt",
    "content": "Hello, World!",
    "create_backup": true
});
let result = tool.execute(params, context, &options).await?;
```

### list_directory_tool()

列出目录内容。

**函数签名**:
```rust
pub fn list_directory_tool() -> Box<dyn Tool>
```

**参数**:

| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| path | String | ✅ | - | 目录路径 |
| recursive | bool | ❌ | false | 是否递归列出 |
| include_hidden | bool | ❌ | false | 是否包含隐藏文件 |

**返回值**:

```json
{
  "success": true,
  "path": "/tmp",
  "entries": [
    {
      "name": "test.txt",
      "path": "/tmp/test.txt",
      "is_file": true,
      "is_dir": false,
      "size": 12
    }
  ],
  "count": 1
}
```

### get_file_info_tool()

获取文件信息。

**函数签名**:
```rust
pub fn get_file_info_tool() -> Box<dyn Tool>
```

**参数**:

| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| path | String | ✅ | - | 文件路径 |

**返回值**:

```json
{
  "success": true,
  "path": "/tmp/test.txt",
  "size": 12,
  "is_file": true,
  "is_dir": false,
  "is_symlink": false,
  "readonly": false,
  "created": "2025-10-18T12:00:00Z",
  "modified": "2025-10-18T12:00:00Z",
  "accessed": "2025-10-18T12:00:00Z"
}
```

---

## 网络请求工具

### http_get_tool()

发送 HTTP GET 请求。

**函数签名**:
```rust
pub fn http_get_tool() -> Box<dyn Tool>
```

**参数**:

| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| url | String | ✅ | - | 请求 URL |
| timeout_seconds | u64 | ❌ | 30 | 超时时间（秒） |

**返回值**:

```json
{
  "success": true,
  "url": "https://api.example.com/data",
  "status": 200,
  "body": "response body",
  "timestamp": "2025-10-18T12:00:00Z"
}
```

**示例**:
```rust
let tool = http_get_tool();
let params = json!({
    "url": "https://api.github.com/users/octocat",
    "timeout_seconds": 30
});
let result = tool.execute(params, context, &options).await?;
```

### http_post_tool()

发送 HTTP POST 请求。

**函数签名**:
```rust
pub fn http_post_tool() -> Box<dyn Tool>
```

**参数**:

| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| url | String | ✅ | - | 请求 URL |
| body | Value | ✅ | - | 请求体（任意 JSON 值） |
| timeout_seconds | u64 | ❌ | 30 | 超时时间（秒） |

**返回值**:

```json
{
  "success": true,
  "url": "https://api.example.com/data",
  "status": 200,
  "body": "response body",
  "timestamp": "2025-10-18T12:00:00Z"
}
```

**示例**:
```rust
let tool = http_post_tool();
let params = json!({
    "url": "https://api.example.com/data",
    "body": {"key": "value"},
    "timeout_seconds": 30
});
let result = tool.execute(params, context, &options).await?;
```

### api_call_tool()

通用 API 调用工具。

**函数签名**:
```rust
pub fn api_call_tool() -> Box<dyn Tool>
```

**参数**:

| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| url | String | ✅ | - | 请求 URL |
| method | String | ✅ | - | HTTP 方法 (GET/POST/PUT/DELETE) |
| body | Value | ❌ | null | 请求体 |
| headers | Value | ❌ | {} | 请求头 |

**返回值**:

```json
{
  "success": true,
  "url": "https://api.example.com/data",
  "method": "POST",
  "status": 200,
  "body": "response body",
  "timestamp": "2025-10-18T12:00:00Z"
}
```

---

## 数据处理工具

### parse_json_tool()

解析和验证 JSON 数据。

**函数签名**:
```rust
pub fn parse_json_tool() -> Box<dyn Tool>
```

**参数**:

| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| input | String | ✅ | - | JSON 字符串 |
| validate_schema | bool | ❌ | false | 是否验证 schema |

**返回值**:

```json
{
  "valid": true,
  "data": {"parsed": "data"},
  "field_count": 1
}
```

### process_text_tool()

处理文本数据。

**函数签名**:
```rust
pub fn process_text_tool() -> Box<dyn Tool>
```

**参数**:

| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| text | String | ✅ | - | 输入文本 |
| operation | String | ✅ | - | 操作类型 (uppercase/lowercase/trim/reverse/length) |

**返回值**:

```json
{
  "success": true,
  "operation": "uppercase",
  "input": "hello",
  "output": "HELLO"
}
```

### convert_data_tool()

转换数据格式。

**函数签名**:
```rust
pub fn convert_data_tool() -> Box<dyn Tool>
```

**参数**:

| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| input | String | ✅ | - | 输入数据 |
| from_format | String | ✅ | - | 源格式 (json/yaml/toml) |
| to_format | String | ✅ | - | 目标格式 (json/yaml/toml) |

**返回值**:

```json
{
  "success": true,
  "from_format": "json",
  "to_format": "yaml",
  "output": "converted data"
}
```

---

## 工具配置

### ToolExecutionContext

工具执行上下文。

```rust
pub struct ToolExecutionContext {
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub metadata: HashMap<String, Value>,
}
```

### ToolExecutionOptions

工具执行选项。

```rust
pub struct ToolExecutionOptions {
    pub timeout: Option<Duration>,
    pub retry_count: u32,
    pub cache_enabled: bool,
}
```

---

## 错误处理

### Error 类型

```rust
pub enum Error {
    Tool(String),
    // ... 其他错误类型
}
```

### 错误处理示例

```rust
match tool.execute(params, context, &options).await {
    Ok(result) => println!("Success: {}", result),
    Err(Error::Tool(msg)) => eprintln!("Tool error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

---

**文档版本**: 1.0.0  
**最后更新**: 2025-10-18  
**维护者**: LumosAI 开发团队

