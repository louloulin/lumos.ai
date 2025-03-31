# Lumosai Agent 结构化输出功能实现

## 功能概述

结构化输出功能使Agent能够生成符合特定schema的JSON数据，而不仅仅是纯文本。这对于需要结构化数据的应用场景（如表单填充、数据提取等）至关重要。

## 已实现功能

1. **结构化输出接口**
   - 在`Agent` trait中添加了`generate_structured<T>`泛型方法
   - 支持任意符合`DeserializeOwned + Send + 'static`约束的类型
   - 基于serde提供类型安全的反序列化

2. **基本实现**
   - 在`BasicAgent`中实现了结构化输出解析和验证
   - 使用`serde_json`支持JSON解析
   - 提供了详细的错误处理

3. **测试验证**
   - 添加了结构化输出的单元测试
   - 验证了从LLM响应到结构化数据的解析流程

## 实现细节

### Agent接口

```rust
#[async_trait]
pub trait Agent: Base + Send + Sync {
    // ...其他方法...
    
    async fn generate_structured<T: DeserializeOwned + Send + 'static>(
        &self, 
        messages: &[Message], 
        options: &AgentGenerateOptions
    ) -> Result<T>;
}
```

### BasicAgent实现

```rust
async fn generate_structured<T: DeserializeOwned + Send + 'static>(
    &self, 
    messages: &[Message], 
    options: &AgentGenerateOptions
) -> Result<T> {
    // 获取生成结果
    let result = self.generate(messages, options).await?;
    
    // 尝试解析为结构化输出
    match serde_json::from_str::<T>(&result.response) {
        Ok(structured) => Ok(structured),
        Err(e) => Err(Error::Parsing(format!("Failed to parse structured output: {}", e)))
    }
}
```

### 测试实现

```rust
// 用于结构化输出的测试结构
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestOutput {
    message: String,
    value: i32,
}

#[tokio::test]
async fn test_agent_structured_output() {
    // 创建一个结构化输出的LLM提供者
    let mock_llm = Arc::new(StructuredMockLlm);
    
    // 创建Agent
    let config = AgentConfig { /* ... */ };
    let mut agent = BasicAgent::new(config, mock_llm);
    
    // 测试结构化输出
    let user_message = Message { /* ... */ };
    let result: TestOutput = agent
        .generate_structured(&[user_message], &AgentGenerateOptions::default())
        .await
        .unwrap();
    
    assert_eq!(result.message, "Hello, world!");
    assert_eq!(result.value, 42);
}
```

## 优点

1. **类型安全** - 生成的结构化输出在编译时就有类型检查
2. **集成简单** - 与现有Agent API无缝集成
3. **错误处理** - 提供详细的解析错误信息
4. **灵活性** - 支持任意符合约束的Rust类型

## 使用示例

### 基本用法

```rust
// 定义结构化输出类型
#[derive(Debug, Deserialize)]
struct Person {
    name: String,
    age: u32,
    skills: Vec<String>,
}

// 使用Agent生成结构化输出
let messages = vec![
    user_message("Generate a JSON object describing a person with name, age and skills")
];

let person: Person = agent
    .generate_structured(&messages, &AgentGenerateOptions::default())
    .await?;

println!("Name: {}", person.name);
println!("Age: {}", person.age);
println!("Skills: {:?}", person.skills);
```

### 带提示的用法

```rust
// 创建带结构化输出提示的选项
let mut options = AgentGenerateOptions::default();
options.output_schema = Some(serde_json::json!({
    "type": "object",
    "properties": {
        "name": { "type": "string" },
        "age": { "type": "number" },
        "skills": { 
            "type": "array",
            "items": { "type": "string" }
        }
    },
    "required": ["name", "age", "skills"]
}));

// 使用提示生成结构化输出
let person: Person = agent.generate_structured(&messages, &options).await?;
```

## 下一步计划

1. **输出验证增强** - 添加更详细的schema验证
2. **格式转换** - 支持多种结构化输出格式（如YAML、XML等）
3. **集成提示工程** - 自动生成有助于LLM生成结构化输出的系统提示
4. **工具支持** - 与工具系统集成，支持工具返回结构化数据
5. **部分结构化** - 支持只解析响应的一部分为结构化数据

## 相关代码文件

- `src/agent/trait_def.rs` - 定义Agent接口
- `src/agent/executor.rs` - 实现BasicAgent的结构化输出功能
- `src/agent/types.rs` - 定义相关类型