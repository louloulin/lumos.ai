# RAG 系统示例

这是一个完整的 RAG（检索增强生成）系统示例，展示了如何使用 LumosAI 构建知识库和问答系统。

## 功能特性

- **文档管理**: 添加、搜索和管理文档
- **智能检索**: 基于语义相似度的文档检索
- **问答系统**: 结合检索结果的智能问答
- **交互模式**: 支持命令行交互式对话
- **统计信息**: 查看知识库统计数据

## 快速开始

### 1. 安装依赖

```bash
cd docs/examples/rag-system
cargo build
```

### 2. 设置环境变量

```bash
# 设置 OpenAI API 密钥（或其他支持的模型提供商）
export OPENAI_API_KEY="your-api-key-here"

# 可选：设置其他模型提供商
export ANTHROPIC_API_KEY="your-anthropic-key"
export GOOGLE_API_KEY="your-google-key"
```

### 3. 运行示例

```bash
# 添加文档到知识库
cargo run -- add "人工智能是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统。"

# 搜索相关文档
cargo run -- search "什么是人工智能"

# 基于知识库问答
cargo run -- ask "人工智能的定义是什么？"

# 启动交互模式
cargo run -- interactive

# 查看统计信息
cargo run -- stats
```

## 命令详解

### 添加文档 (add)

将文档添加到知识库中：

```bash
cargo run -- add "文档内容"
cargo run -- add "机器学习是人工智能的一个子领域，专注于开发能够从数据中学习的算法。"
```

### 搜索文档 (search)

在知识库中搜索相关文档：

```bash
cargo run -- search "查询内容"
cargo run -- search "机器学习"
```

### 智能问答 (ask)

基于知识库内容回答问题：

```bash
cargo run -- ask "你的问题"
cargo run -- ask "机器学习和人工智能有什么关系？"
```

### 交互模式 (interactive)

启动交互式对话模式：

```bash
cargo run -- interactive
```

在交互模式中，你可以：
- 输入问题进行问答
- 使用 `/add <内容>` 添加文档
- 使用 `/search <查询>` 搜索文档
- 使用 `/stats` 查看统计信息
- 输入 `quit` 或 `exit` 退出

### 统计信息 (stats)

查看知识库统计数据：

```bash
cargo run -- stats
```

## 示例用法

### 构建技术知识库

```bash
# 添加技术文档
cargo run -- add "Rust 是一种系统编程语言，专注于安全、速度和并发性。"
cargo run -- add "Python 是一种高级编程语言，以其简洁的语法和强大的库生态系统而闻名。"
cargo run -- add "JavaScript 是一种动态编程语言，主要用于 Web 开发。"

# 搜索相关内容
cargo run -- search "编程语言"

# 智能问答
cargo run -- ask "Rust 和 Python 有什么区别？"
```

### 构建产品知识库

```bash
# 添加产品信息
cargo run -- add "我们的产品支持多种支付方式，包括信用卡、支付宝和微信支付。"
cargo run -- add "产品提供 7x24 小时客户支持，响应时间通常在 2 小时内。"
cargo run -- add "我们提供 30 天无理由退款保证。"

# 客户咨询
cargo run -- ask "你们支持哪些支付方式？"
cargo run -- ask "如果不满意可以退款吗？"
```

## 技术实现

### 核心组件

1. **SimpleRAGSystem**: 主要的 RAG 系统实现
   - 文档存储和管理
   - 语义搜索功能
   - 智能问答集成

2. **Document**: 文档数据结构
   - 唯一标识符
   - 文档内容
   - 创建时间戳

3. **SearchResult**: 搜索结果结构
   - 文档引用
   - 相似度分数

### 工作流程

1. **文档添加**: 
   - 接收文档内容
   - 生成唯一 ID
   - 存储到内存中

2. **语义搜索**:
   - 计算查询与文档的相似度
   - 返回最相关的文档

3. **智能问答**:
   - 搜索相关文档
   - 构建包含上下文的提示
   - 调用 AI 模型生成回答

## 扩展功能

### 持久化存储

当前示例使用内存存储，可以扩展为：

```rust
// 文件存储
impl SimpleRAGSystem {
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        // 实现文件保存逻辑
    }
    
    pub fn load_from_file(path: &str) -> Result<Self> {
        // 实现文件加载逻辑
    }
}
```

### 向量数据库集成

```rust
// 集成专业向量数据库
use lumosai::vector::VectorStorage;

impl SimpleRAGSystem {
    pub fn with_vector_storage(storage: Arc<dyn VectorStorage>) -> Self {
        // 使用向量数据库进行语义搜索
    }
}
```

### 文档分块

```rust
// 长文档分块处理
impl SimpleRAGSystem {
    pub fn add_long_document(&mut self, content: &str, chunk_size: usize) -> Result<Vec<String>> {
        // 将长文档分割为多个块
    }
}
```

## 相关示例

- **hello-world**: 基础 Agent 使用
- **chatbot**: 对话系统实现
- **research-assistant**: 网络研究助手
- **multi-agent**: 多 Agent 协作（即将推出）
- **workflow-automation**: 工作流自动化（即将推出）

## 故障排除

### 常见问题

1. **API 密钥错误**
   ```
   Error: API key not found
   ```
   解决方案：确保设置了正确的环境变量

2. **网络连接问题**
   ```
   Error: Failed to connect to API
   ```
   解决方案：检查网络连接和 API 服务状态

3. **编译错误**
   ```
   Error: failed to compile
   ```
   解决方案：运行 `cargo clean` 然后重新构建

### 调试模式

启用详细日志：

```bash
RUST_LOG=debug cargo run -- ask "你的问题"
```

## 许可证

本示例遵循 MIT 许可证。
