# LumosAI 真实 API 验证示例

本目录包含使用真实 DeepSeek API 的验证示例，展示 LumosAI 在实际生产环境中的功能。

## 📋 示例列表

### 🧪 验证示例

1. **`simple_api_validation.rs`** - 基础 API 功能验证（使用模拟 API）
2. **`real_deepseek_api_validation.rs`** - 真实 DeepSeek API 验证 ⭐
3. **`api_validation_examples.rs`** - 完整 API 功能验证
4. **`advanced_api_validation.rs`** - 高级功能和性能验证
5. **`python_api_validation.py`** - Python 绑定验证

### 🔧 配置脚本

1. **`../scripts/setup_deepseek_api.ps1`** - Windows PowerShell 设置脚本
2. **`../scripts/setup_deepseek_api.sh`** - Linux/macOS Bash 设置脚本

## 🚀 快速开始

### 步骤 1：获取 DeepSeek API Key

1. 访问 [DeepSeek 平台](https://platform.deepseek.com/)
2. 注册并登录账户
3. 在 API 管理页面创建新的 API Key
4. 复制 API Key（格式：`sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`）

### 步骤 2：设置环境变量

#### 方法 1：使用自动化脚本（推荐）

**Windows (PowerShell):**
```powershell
# 临时设置
.\scripts\setup_deepseek_api.ps1 -ApiKey "your-api-key-here"

# 永久设置
.\scripts\setup_deepseek_api.ps1 -ApiKey "your-api-key-here" -Permanent

# 验证设置
.\scripts\setup_deepseek_api.ps1 -Verify
```

**Linux/macOS (Bash):**
```bash
# 临时设置
./scripts/setup_deepseek_api.sh -k "your-api-key-here"

# 永久设置
./scripts/setup_deepseek_api.sh -k "your-api-key-here" -p

# 验证设置
./scripts/setup_deepseek_api.sh -v
```

#### 方法 2：手动设置

**Windows (PowerShell):**
```powershell
$env:DEEPSEEK_API_KEY = "your-api-key-here"
```

**Linux/macOS (Bash):**
```bash
export DEEPSEEK_API_KEY="your-api-key-here"
```

### 步骤 3：运行验证示例

```bash
# 运行真实 API 验证（推荐）
cargo run --example real_deepseek_api_validation

# 运行基础验证（模拟 API）
cargo run --example simple_api_validation

# 运行完整验证
cargo run --example api_validation_examples
```

## 🧪 真实 API 验证示例详解

### `real_deepseek_api_validation.rs`

这是最重要的验证示例，包含以下测试：

#### 测试 1：基础对话
```rust
// 验证基本的 Agent 创建和对话功能
let agent = quick("deepseek_assistant", "你是一个友好的AI助手")
    .model(deepseek_llm)
    .build()?;

let response = agent.generate(&messages, &options).await?;
```

#### 测试 2：工具调用
```rust
// 验证工具集成和函数调用
let agent = AgentBuilder::new()
    .name("math_assistant")
    .instructions("你是一个数学助手")
    .model(deepseek_llm)
    .tool(Box::new(CalculatorTool::default()))
    .enable_function_calling(true)
    .build()?;
```

#### 测试 3：复杂对话
```rust
// 验证多轮对话和上下文管理
let conversations = vec![
    "请解释一下什么是人工智能？",
    "那么机器学习和深度学习有什么区别？",
    "你能举个具体的例子说明深度学习在实际中的应用吗？",
];
```

#### 测试 4：性能测试
```rust
// 验证 API 响应速度和并发处理
let test_questions = vec![
    "1+1等于多少？",
    "今天天气怎么样？",
    "请说一个笑话",
    // ...
];
```

## 📊 预期结果

### 成功运行的输出示例

```
🎯 LumosAI 真实 DeepSeek API 验证
=================================
✅ 找到 DeepSeek API Key: sk-xxxxx...xxxxx

🚀 示例 1: 真实 API 基础对话测试
================================
✅ Agent 创建成功:
   名称: deepseek_assistant
   指令: 你是一个友好的AI助手，请用中文回答问题

📤 发送消息: 你好！请简单介绍一下你自己。
📥 DeepSeek 响应: 你好！我是 DeepSeek 开发的人工智能助手...

🔧 示例 2: 真实 API 工具调用测试
===============================
✅ 数学助手创建成功:
   名称: math_assistant
   工具数量: 1

📤 发送数学问题: 请帮我计算 (15 + 25) * 3 - 8 的结果
📥 DeepSeek 响应: 我来帮你计算这个数学表达式...

🔍 执行步骤:
   步骤 1: 调用计算器工具
   结果: 112

🎉 真实 API 验证完成！
========================
✅ 通过: 4/4
📊 成功率: 100.0%

🏆 所有真实 API 测试通过！
✅ 基础对话 - DeepSeek API 正常工作
✅ 工具调用 - 函数调用功能正常
✅ 复杂对话 - 多轮对话支持良好
✅ 性能测试 - API 响应速度正常
```

## 💰 费用说明

运行真实 API 验证示例会产生少量费用：

- **基础对话测试**：~0.001-0.01 元
- **工具调用测试**：~0.002-0.02 元  
- **复杂对话测试**：~0.005-0.05 元
- **性能测试**：~0.01-0.1 元

**总计**：约 0.02-0.2 元人民币

## 🛠️ 故障排除

### 常见问题

#### 1. API Key 未设置
```
❌ Configuration error: DEEPSEEK_API_KEY 环境变量未设置
```
**解决方案**：使用设置脚本或手动设置环境变量

#### 2. API Key 格式错误
```
❌ API Key 格式无效！
```
**解决方案**：确认 API Key 以 `sk-` 开头，长度为 32-64 字符

#### 3. 网络连接问题
```
❌ 网络请求失败: Connection timeout
```
**解决方案**：检查网络连接和防火墙设置

#### 4. API 配额超限
```
❌ API 请求失败: Rate limit exceeded
```
**解决方案**：等待配额重置或升级 API 计划

### 调试技巧

1. **启用详细日志**：
```bash
RUST_LOG=debug cargo run --example real_deepseek_api_validation
```

2. **验证 API Key**：
```bash
# Windows
echo $env:DEEPSEEK_API_KEY

# Linux/macOS
echo $DEEPSEEK_API_KEY
```

3. **测试网络连接**：
```bash
curl -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
     https://api.deepseek.com/v1/models
```

## 📚 相关文档

- [DeepSeek API 设置指南](../docs/DEEPSEEK_API_SETUP.md)
- [LumosAI API 设计文档](../plan10.md)
- [DeepSeek 官方文档](https://platform.deepseek.com/api-docs/)

## 🤝 获取帮助

如果遇到问题：

1. 查看 [故障排除](#故障排除) 部分
2. 阅读 [DeepSeek API 设置指南](../docs/DEEPSEEK_API_SETUP.md)
3. 在 GitHub 上提交 Issue
4. 联系 DeepSeek 技术支持

---

**注意**：请确保遵守 DeepSeek 的使用条款和 API 使用政策。
