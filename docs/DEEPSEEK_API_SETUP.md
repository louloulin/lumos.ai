# DeepSeek API 设置指南

本文档介绍如何设置 DeepSeek API Key 以运行真实的 API 验证示例。

## 📋 前置要求

1. **DeepSeek 账户**：访问 [DeepSeek 平台](https://platform.deepseek.com/) 注册账户
2. **API Key**：在平台中生成您的 API Key
3. **网络连接**：确保能够访问 DeepSeek API 服务

## 🔑 获取 DeepSeek API Key

### 步骤 1：注册 DeepSeek 账户
1. 访问 https://platform.deepseek.com/
2. 点击"注册"或"Sign Up"
3. 完成账户注册流程

### 步骤 2：获取 API Key
1. 登录 DeepSeek 平台
2. 进入 API 管理页面
3. 点击"创建新的 API Key"
4. 复制生成的 API Key（格式类似：`sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`）

⚠️ **重要提醒**：
- API Key 是敏感信息，请妥善保管
- 不要将 API Key 提交到版本控制系统
- 定期轮换 API Key 以确保安全

## ⚙️ 设置环境变量

### Windows (PowerShell)
```powershell
# 临时设置（当前会话有效）
$env:DEEPSEEK_API_KEY = "your-api-key-here"

# 永久设置（需要管理员权限）
[Environment]::SetEnvironmentVariable("DEEPSEEK_API_KEY", "your-api-key-here", "User")
```

### Windows (命令提示符)
```cmd
# 临时设置（当前会话有效）
set DEEPSEEK_API_KEY=your-api-key-here

# 永久设置（需要管理员权限）
setx DEEPSEEK_API_KEY "your-api-key-here"
```

### Linux/macOS (Bash)
```bash
# 临时设置（当前会话有效）
export DEEPSEEK_API_KEY="your-api-key-here"

# 永久设置（添加到 ~/.bashrc 或 ~/.zshrc）
echo 'export DEEPSEEK_API_KEY="your-api-key-here"' >> ~/.bashrc
source ~/.bashrc
```

### 使用 .env 文件（推荐）
1. 在项目根目录创建 `.env` 文件：
```env
DEEPSEEK_API_KEY=your-api-key-here
```

2. 确保 `.env` 文件已添加到 `.gitignore`：
```gitignore
.env
*.env
```

## 🧪 验证设置

### 方法 1：使用环境变量检查
```bash
# Windows (PowerShell)
echo $env:DEEPSEEK_API_KEY

# Linux/macOS
echo $DEEPSEEK_API_KEY
```

### 方法 2：运行验证示例
```bash
# 运行真实 API 验证示例
cargo run --example real_deepseek_api_validation
```

如果设置正确，您应该看到：
```
✅ 找到 DeepSeek API Key: sk-xxxxx...xxxxx
```

## 🚀 运行示例

### 基础验证示例
```bash
# 运行真实 DeepSeek API 验证
cargo run --example real_deepseek_api_validation
```

### 可用的验证示例
1. **`real_deepseek_api_validation.rs`** - 完整的真实 API 验证
2. **`simple_api_validation.rs`** - 使用模拟 API 的基础验证
3. **`api_validation_examples.rs`** - 完整功能验证示例

## 📊 API 使用费用

DeepSeek API 按使用量计费，具体费用请参考：
- [DeepSeek 定价页面](https://platform.deepseek.com/pricing)

### 费用估算
运行完整的验证示例大约会产生：
- **基础对话测试**：~0.001-0.01 元
- **工具调用测试**：~0.002-0.02 元  
- **复杂对话测试**：~0.005-0.05 元
- **性能测试**：~0.01-0.1 元

**总计**：约 0.02-0.2 元人民币

## 🔧 配置选项

### 模型选择
```rust
// 使用不同的 DeepSeek 模型
let llm = Arc::new(DeepSeekProvider::new(
    api_key,
    Some("deepseek-chat".to_string())  // 默认模型
));

// 可选模型：
// - "deepseek-chat"     - 通用对话模型
// - "deepseek-coder"    - 代码专用模型
```

### 请求参数
```rust
let options = AgentGenerateOptions {
    temperature: Some(0.7),      // 创造性 (0.0-1.0)
    max_tokens: Some(2048),      // 最大输出长度
    top_p: Some(0.9),           // 核采样参数
    ..Default::default()
};
```

## 🛠️ 故障排除

### 常见问题

#### 1. API Key 无效
```
❌ Configuration error: DEEPSEEK_API_KEY 环境变量未设置
```
**解决方案**：检查 API Key 是否正确设置，格式是否正确

#### 2. 网络连接问题
```
❌ 网络请求失败: Connection timeout
```
**解决方案**：
- 检查网络连接
- 确认防火墙设置
- 尝试使用代理

#### 3. API 配额超限
```
❌ API 请求失败: Rate limit exceeded
```
**解决方案**：
- 等待配额重置
- 升级 API 计划
- 减少请求频率

#### 4. 模型不可用
```
❌ 模型错误: Model not found
```
**解决方案**：
- 检查模型名称是否正确
- 确认账户是否有权限使用该模型

### 调试技巧

1. **启用详细日志**：
```bash
RUST_LOG=debug cargo run --example real_deepseek_api_validation
```

2. **检查网络连接**：
```bash
curl -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
     https://api.deepseek.com/v1/models
```

3. **验证 API Key 格式**：
   - 应该以 `sk-` 开头
   - 长度通常为 32-64 个字符
   - 只包含字母、数字和连字符

## 📚 相关文档

- [DeepSeek API 官方文档](https://platform.deepseek.com/api-docs/)
- [LumosAI Agent 使用指南](../README.md)
- [API 设计文档](../plan10.md)
- [更多示例](../examples/)

## 🤝 获取帮助

如果遇到问题，可以：
1. 查看 [DeepSeek 官方文档](https://platform.deepseek.com/docs/)
2. 在 GitHub 上提交 Issue
3. 联系 DeepSeek 技术支持

---

**注意**：请确保遵守 DeepSeek 的使用条款和 API 使用政策。
