# 6. 开发指南

本章节提供 Lumosai 项目的开发环境配置、开发流程和最佳实践指南。

## 6.1 开发环境配置

### 6.1.1 前置条件

在开始开发 Lumosai 项目前，请确保您的系统满足以下要求：

| 依赖项 | 最低版本 | 推荐版本 | 说明 |
|-------|---------|---------|------|
| Rust  | 1.70.0  | 1.75.0  | Rust 编程语言 |
| Cargo | 1.70.0  | 1.75.0  | Rust 包管理工具（随 Rust 安装） |
| Node.js | 18.0.0 | 20.0.0 | JavaScript 运行时 |
| pnpm  | 7.0.0   | 8.0.0   | 高性能 JavaScript 包管理器 |
| Protoc | 3.15.0 | 3.21.0  | Protocol Buffers 编译器 |
| Git   | 2.30.0  | 2.40.0  | 版本控制系统 |
| Docker (可选) | 20.10.0 | 24.0.0 | 容器化平台 |
| Visual Studio Code (推荐) | - | 最新版 | 推荐的 IDE |

### 6.1.2 安装 Rust 环境

1. **安装 Rust 和 Cargo**:

```bash
# 使用 rustup 安装 Rust (推荐)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 选择 1) 默认安装

# 安装完成后，更新 PATH
source "$HOME/.cargo/env"

# 验证安装
rustc --version
cargo --version
```

2. **安装推荐的 Rust 组件**:

```bash
# 安装 Rust 格式化工具
rustup component add rustfmt

# 安装 Rust 代码检查工具
rustup component add clippy

# 安装文档生成工具
rustup component add rust-docs

# 安装交叉编译支持（可选）
rustup target add wasm32-unknown-unknown  # WebAssembly 支持
rustup target add x86_64-unknown-linux-gnu  # Linux 支持
```

### 6.1.3 安装 Node.js 环境

```bash
# 使用 nvm 安装 Node.js (推荐)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash

# 加载 nvm
export NVM_DIR="$([ -z "${XDG_CONFIG_HOME-}" ] && printf %s "${HOME}/.nvm" || printf %s "${XDG_CONFIG_HOME}/nvm")"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

# 安装 Node.js
nvm install 20

# 安装 pnpm
npm install -g pnpm

# 验证安装
node --version
pnpm --version
```

### 6.1.4 克隆项目

```bash
# 克隆 Lumosai 仓库
git clone https://github.com/your-org/lumosai.git
cd lumosai

# 初始化子模块（如果有）
git submodule update --init --recursive
```

### 6.1.5 IDE 配置

我们推荐使用 Visual Studio Code 进行开发。以下是推荐的 VS Code 扩展：

1. **Rust 相关**:
   - rust-analyzer: Rust 语言支持
   - crates: Rust 依赖管理辅助
   - Better TOML: TOML 文件支持
   - CodeLLDB: Rust 调试支持

2. **TypeScript/JavaScript 相关**:
   - ESLint: JavaScript 代码检查
   - Prettier: 代码格式化
   - TypeScript-TSlint-Plugin: TypeScript 支持

3. **其他工具**:
   - Docker: Docker 支持
   - GitLens: Git 增强
   - markdownlint: Markdown 检查

推荐的 VS Code 设置 (`.vscode/settings.json`):

```json
{
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.allFeatures": true,
  "rust-analyzer.procMacro.enable": true
}
```

## 6.2 构建和运行

### 6.2.1 Rust 核心库

```bash
# 进入核心库目录
cd lumosai_core

# 构建库（开发模式）
cargo build

# 构建库（发布模式）
cargo build --release

# 运行测试
cargo test

# 构建文档
cargo doc --open
```

### 6.2.2 JavaScript 客户端库

```bash
# 进入客户端库目录
cd client-js

# 安装依赖
pnpm install

# 构建库（开发模式）
pnpm build:dev

# 构建库（发布模式）
pnpm build

# 运行测试
pnpm test

# 生成文档
pnpm docs
```

### 6.2.3 用户界面

```bash
# 进入 UI 目录
cd lumosai_ui

# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev

# 构建生产版本
pnpm build

# 运行测试
pnpm test
```

### 6.2.4 一键构建全部组件

```bash
# 在项目根目录
# 安装所有依赖
pnpm install

# 构建所有组件
pnpm build

# 运行所有测试
pnpm test
```

## 6.3 开发流程

### 6.3.1 Git 工作流

Lumosai 项目使用 Git Flow 工作流程：

1. **分支策略**:
   - `main`: 稳定版本分支
   - `develop`: 开发分支，包含下一个版本的最新代码
   - `feature/*`: 功能分支，用于开发新功能
   - `bugfix/*`: 修复分支，用于修复 bug
   - `release/*`: 发布分支，用于准备新版本
   - `hotfix/*`: 热修复分支，用于修复生产环境的紧急问题

2. **开发新功能**:

```bash
# 从 develop 分支创建功能分支
git checkout develop
git pull
git checkout -b feature/your-feature-name

# 开发功能...

# 提交更改
git add .
git commit -m "feat: implement your feature"

# 定期从 develop 分支同步
git fetch origin
git merge origin/develop

# 功能完成后，推送到远程
git push -u origin feature/your-feature-name

# 然后创建 Pull Request 到 develop 分支
```

3. **代码审查**:
   - 所有代码更改必须通过 Pull Request 合并
   - 至少需要一个审核者批准
   - CI 检查必须通过
   - 遵循 [Angular Commit Message 规范](https://github.com/angular/angular/blob/main/CONTRIBUTING.md#commit)

### 6.3.2 测试策略

Lumosai 项目采用多层测试策略：

1. **单元测试**:
   - 测试单个组件和函数
   - 应该快速且独立运行
   - 使用 mock 和 stub 隔离依赖

2. **集成测试**:
   - 测试多个组件的交互
   - 可能需要外部依赖（如数据库）
   - 更全面但运行较慢

3. **端到端测试**:
   - 测试整个系统的流程
   - 模拟真实用户行为
   - 通常在 CI 中运行

测试命名约定：

- 单元测试文件：`{module_name}_test.rs` 或 `{module_name}.spec.ts`
- 集成测试位于 `tests/` 目录
- 端到端测试位于 `e2e/` 目录

### 6.3.3 文档生成

文档是开发过程的重要部分：

1. **API 文档**:
   - Rust: 使用文档注释 (`///`) 并运行 `cargo doc`
   - TypeScript: 使用 JSDoc 注释并运行 `pnpm docs`

2. **用户文档**:
   - 位于 `docs/` 目录
   - 使用 Markdown 格式
   - 可使用 `mdbook` 构建

3. **示例代码**:
   - 位于 `lumosai_examples/` 目录
   - 应包含详细注释
   - 每个示例都应该可以独立运行

## 6.4 代码风格和最佳实践

### 6.4.1 Rust 编码规范

Lumosai 项目遵循以下 Rust 编码规范：

1. **格式化**:
   - 使用 `rustfmt` 格式化代码
   - 配置文件为 `rustfmt.toml`

2. **命名约定**:
   - 模块: `snake_case`
   - 类型 (结构体、枚举): `PascalCase`
   - 特质 (trait): `PascalCase`
   - 函数、方法和变量: `snake_case`
   - 常量: `SCREAMING_SNAKE_CASE`
   - 宏: `snake_case!`

3. **文档注释**:
   - 所有公共 API 必须有文档注释
   - 使用 Markdown 格式的注释
   - 包含示例代码（如适用）

4. **错误处理**:
   - 使用 `Result` 类型返回错误
   - 避免 `unwrap()` 和 `expect()` 在生产代码中
   - 使用 `thiserror` 定义错误类型
   - 使用 `anyhow` 处理复杂错误情况

5. **性能考虑**:
   - 避免不必要的内存分配
   - 使用引用而不是拷贝（当合适时）
   - 优先使用标准库中的高效实现
   - 关键路径上避免锁竞争

### 6.4.2 TypeScript 编码规范

1. **格式化**:
   - 使用 Prettier 格式化代码
   - 配置文件为 `.prettierrc.js`

2. **命名约定**:
   - 文件名: `kebab-case.ts` 或 `CamelCase.tsx` (组件)
   - 类、接口、类型和枚举: `PascalCase`
   - 函数、方法和变量: `camelCase`
   - 常量: `SCREAMING_SNAKE_CASE`

3. **TypeScript 特性**:
   - 尽可能使用类型注解
   - 避免使用 `any` 类型
   - 使用接口定义复杂结构
   - 利用泛型增强代码复用性

4. **组件结构** (UI):
   - 使用函数组件和 Hooks
   - 将大型组件拆分为小型可复用组件
   - 使用 TypeScript Props 接口
   - 使用 CSS Modules 或 styled-components

### 6.4.3 通用最佳实践

1. **代码质量**:
   - 保持函数简短且职责单一
   - 使用有意义的变量和函数名
   - 避免重复代码 (DRY 原则)
   - 及时重构复杂代码

2. **注释**:
   - 注释解释"为什么"而不是"怎么做"
   - 更新代码时更新相关注释
   - 对于复杂算法，提供概述和参考
   - 使用 TODO/FIXME 标记需要改进的地方

3. **安全性**:
   - 验证所有用户输入
   - 避免直接使用不可信数据
   - 遵循最小权限原则
   - 定期更新依赖以修复安全漏洞

4. **可测试性**:
   - 设计便于测试的代码
   - 使用依赖注入而非硬编码依赖
   - 分离业务逻辑和外部交互
   - 编写可靠且独立的测试

## 6.5 贡献指南

### 6.5.1 提交错误报告

1. 使用 GitHub Issues 提交错误报告
2. 检查是否已存在相同问题
3. 提供详细的重现步骤
4. 包含环境信息（系统、版本等）
5. 如可能，提供最小复现代码

### 6.5.2 提出功能建议

1. 使用 GitHub Issues 提交功能建议
2. 清晰描述建议的功能
3. 说明为什么这个功能对项目有价值
4. 如可能，提供实现思路

### 6.5.3 提交 Pull Request

1. 在开始工作前创建 Issue 进行讨论
2. Fork 仓库并在功能分支上开发
3. 遵循项目的代码风格和测试策略
4. 提交前运行所有测试确保通过
5. 创建清晰的 PR 描述，引用相关 Issue

### 6.5.4 参与讨论

1. 加入项目 Discord 社区
2. 参与 GitHub Discussions
3. 帮助回答其他用户的问题
4. 分享你的使用经验和案例

## 6.6 调试技巧

### 6.6.1 Rust 代码调试

1. **使用 println! 调试**:
   ```rust
   println!("Debug value: {:?}", value);
   ```

2. **使用 tracing 进行结构化日志**:
   ```rust
   use tracing::{info, debug, warn, error};
   
   debug!("Processing item: {:?}", item);
   ```

3. **使用 LLDB/GDB 调试器**:
   ```bash
   # 构建调试版本
   cargo build
   
   # 使用 LLDB 调试 (VS Code 中使用 CodeLLDB 扩展)
   lldb target/debug/your_binary
   
   # 常用命令
   breakpoint set --name function_name  # 设置断点
   run                                 # 运行程序
   next                                # 下一步
   step                                # 步入
   finish                              # 步出
   ```

4. **使用 cargo-flamegraph 进行性能分析**:
   ```bash
   cargo install cargo-flamegraph
   cargo flamegraph --bin your_binary
   ```

### 6.6.2 TypeScript/JavaScript 调试

1. **浏览器开发者工具**:
   - 使用 `console.log`, `console.debug`, `console.error`
   - 设置断点并检查变量
   - 使用性能和网络面板分析问题

2. **在 VS Code 中调试**:
   ```json
   // launch.json 配置
   {
     "type": "chrome",
     "request": "launch",
     "name": "Debug UI",
     "url": "http://localhost:3000",
     "webRoot": "${workspaceFolder}/lumosai_ui"
   }
   ```

3. **使用 React DevTools**:
   - 检查组件层次结构
   - 查看 props 和 state
   - 分析渲染性能

4. **使用错误边界捕获组件错误**:
   ```tsx
   class ErrorBoundary extends React.Component {
     state = { hasError: false, error: null };
     
     static getDerivedStateFromError(error) {
       return { hasError: true, error };
     }
     
     componentDidCatch(error, info) {
       console.error("Component error:", error, info);
     }
     
     render() {
       if (this.state.hasError) {
         return <div>Something went wrong: {this.state.error.message}</div>;
       }
       return this.props.children;
     }
   }
   ```

### 6.6.3 日志级别和配置

Lumosai 使用 `tracing` 库进行日志管理。日志级别配置：

```bash
# 设置日志级别环境变量
export RUST_LOG=lumosai_core=debug,lumosai_rag=info,warn

# 或在运行时指定
RUST_LOG=debug cargo run
```

在代码中配置：

```rust
use tracing_subscriber::{fmt, EnvFilter};

fn init_logging() {
    let filter = EnvFilter::from_default_env()
        .add_directive("lumosai_core=debug".parse().unwrap());
        
    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();
}
``` 