
# 第一阶段实现计划：CLI工具和本地模型支持

## CLI工具实现

### 1. CLI框架设计
- 使用`clap`创建命令行解析
- 实现类似`cargo`的子命令结构
- 创建`lomusai_cli` crate作为主入口点

### 2. 核心命令实现
- `init` - 创建新项目
  - 提供多种项目模板（agent、workflow、rag）
  - 生成`Cargo.toml`和基础文件结构
- `dev` - 本地开发服务器
  - 自动重载代码变更
  - 提供开发日志和调试信息
- `run` - 运行应用
- `build` - 构建优化版本
- `deploy` - 部署应用（基础版）

### 3. 工具模块
- 项目模板管理模块
- 依赖分析与管理模块
- 配置文件处理模块

## 本地模型支持

### 1. 本地模型接口
- 扩展`lomusai_core::llm::provider`trait
- 实现统一的模型加载和预测接口

### 2. 集成推理引擎
- **llm-rs** - 支持多种量化模型
- **candle** - Rust原生推理
- **llama-rs** - 轻量级推理

### 3. 支持的模型类型
- Mistral模型
- Phi-2等轻量模型

## 技术挑战与解决方案

### CLI挑战
- **跨平台兼容性** - 使用`dirs`和`home`处理路径
- **项目状态管理** - 实现.lomusai配置目录

## 依赖需求

```toml
# lomusai_cli/Cargo.toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
dialoguer = "0.10" # 交互式提示
indicatif = "0.17" # 进度条
console = "0.15" # 终端彩色输出
dirs = "4.0" # 目录管理
serde = { version = "1.0", features = ["derive"] }
toml = "0.5"
lomusai_core = { path = "../lomusai_core" }

# lomusai_core/Cargo.toml (本地模型部分)
[features]
default = ["openai", "anthropic"]
local-models = ["llm-rs"]

[dependencies]
llm-rs = { version = "0.1", optional = true }
candle-core = { version = "0.1", optional = true }
```

## 实现时间估计

- CLI基础框架: 1周
- 项目模板与命令: 2周
- 本地模型接口: 1周
- 模型集成与优化: 2-3周

总计: 约6-7周可完成第一阶段核心功能实现与基本测试
