# LumosAI 技术栈和依赖管理策略

## 🎯 技术栈选择原则

1. **性能优先**: 选择高性能的 Rust 生态系统
2. **类型安全**: 充分利用 Rust 的类型系统
3. **生产就绪**: 选择稳定、成熟的技术栈
4. **可维护性**: 代码清晰，易于维护
5. **可扩展性**: 支持水平扩展和功能扩展

## 🏗️ 核心技术栈

### 运行时和异步
```toml
# 异步运行时 - 高性能异步 I/O
tokio = { version = "1.40", features = ["full"] }
tokio-util = "0.7"
tokio-stream = "0.1"

# 异步 trait
async-trait = "0.1"

# 通道和并发
async-channel = "2.3"
flume = "0.11"

# 任务调度
tokio-cron-scheduler = "0.10"
```

### 序列化和数据格式
```toml
# JSON 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_with = "3.9"

# 其他序列化格式
serde_yaml = "0.9"
toml = "0.8"
bincode = "1.3"

# UUID 生成
uuid = { version = "1.11", features = ["v4", "v7", "serde"] }

# 时间处理
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.9"
```

### 错误处理和日志
```toml
# 错误处理
thiserror = "1.0"
anyhow = "1.0"
eyre = "0.6"

# 日志系统
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"
tracing-log = "0.2"

# 日志格式化
tracing-bunyan-formatter = "0.3"
tracing-actix-web = "0.7"
```

### 网络和 HTTP
```toml
# HTTP 客户端
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"] }
reqwest-middleware = "0.3"
reqwest-retry = "0.5"

# HTTP 服务器
axum = { version = "0.7", features = ["macros", "multipart"] }
axum-extra = { version = "0.9", features = ["typed-header", "cookie"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["full"] }
hyper = { version = "1.4", features = ["full"] }

# WebSocket
tokio-tungstenite = "0.24"
tungstenite = "0.24"
futures-util = "0.3"

# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"
```

### 数据库和存储
```toml
# SQL 数据库
sqlx = { version = "0.8", features = [
    "runtime-tokio-rustls",
    "postgres",
    "sqlite",
    "mysql",
    "chrono",
    "uuid",
    "json",
    "migrate"
] }

# Redis
redis = { version = "0.26", features = ["tokio-comp", "connection-manager"] }
bb8 = "0.8"
bb8-redis = "0.15"

# 连接池
deadpool-postgres = "0.14"
deadpool-redis = "0.16"
```

### 向量数据库
```toml
# Qdrant
qdrant-client = "1.10"

# Weaviate
weaviate-client = "0.4"

# 本地向量存储
arrow = "54.0"
arrow-array = "54.0"
arrow-schema = "54.0"
datafusion = "44.0"

# 向量操作
ndarray = "0.16"
nalgebra = "0.33"
```

### AI 和机器学习
```toml
# OpenAI API
async-openai = "0.25"

# 本地模型 (可选)
candle-core = "0.7"
candle-transformers = "0.7"
candle-nn = "0.7"

# 嵌入计算
tokenizers = "0.20"
rust-bert = "0.21"

# 文本处理
textwrap = "0.16"
unicode-segmentation = "1.12"
regex = "1.11"
```

### 认证和安全
```toml
# JWT
jsonwebtoken = "9.3"
jwtk = "0.4"

# 密码学
ring = "0.17"
rsa = "0.9"
ed25519-dalek = "2.0"

# 加密
aes-gcm = "0.10"
chacha20poly1305 = "0.10"
argon2 = "0.5"

# 安全扫描
secrecy = "0.8"
zeroize = { version = "1.8", features = ["zeroize_derive"] }
```

### 配置和环境
```toml
# 配置管理
config = { version = "0.14", features = ["yaml", "toml", "json"] }
dotenvy = "0.15"

# 环境变量管理
clap = { version = "4.5", features = ["derive", "env"] }
env_logger = "0.11"

# 特性标志
feature-flag = "0.2"
```

### 工具和实用程序
```toml
# 生成器
derive_builder = "0.20"
derive_more = { version = "1.0", features = ["full"] }
async-recursion = "1.1"

# 互斥体和并发
parking_lot = "0.12"
dashmap = "6.1"

# 缓存
lru = "0.13"
moka = { version = "0.12", features = ["sync", "future"] }

# 指标和监控
prometheus = "0.13"
metrics = "0.24"
metrics-exporter-prometheus = "0.15"

# 时间测量
quanta = "0.12"
hdrhistogram = "7.5"
```

### 测试框架
```toml
[dev-dependencies]
# 测试框架
tokio-test = "0.4"
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.5"

# Mock 框架
mockall = "0.13"
mockall_double = "0.3"

# 测试工具
assert_cmd = "2.0"
predicates = "3.1"
assert_fs = "1.1"

# HTTP 测试
httpmock = "0.7"
wiremock = "0.6"

# 随机测试
fake = { version = "3.0", features = ["derive", "uuid", "chrono"] }
rand = "0.8"
```

## 📦 依赖管理策略

### 1. 版本管理策略

#### 语义化版本控制
```toml
# 使用宽松版本约束，允许补丁和次版本更新
serde = "~1.0"     # 允许 1.x 版本
tokio = "1.40"    # 固定主版本，允许次版本和补丁更新

# 严格版本约束 (对于关键依赖)
sqlx = "=0.8.0"   # 精确版本 (仅在必要时使用)
```

#### 依赖版本同步
```toml
# workspace Cargo.toml
[workspace.dependencies]
# 核心依赖版本统一管理
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
```

### 2. 依赖分类策略

#### 核心依赖 (Core Dependencies)
- 运行时、序列化、错误处理
- 版本控制严格，变更需要充分测试
- 定期更新，但不频繁

#### 应用依赖 (Application Dependencies)
- HTTP 服务器、数据库、AI 模型
- 使用兼容版本，避免破坏性变更
- 跟踪上游版本发布

#### 开发依赖 (Development Dependencies)
- 测试框架、代码检查工具
- 频繁更新，使用最新版本
- 不影响生产环境

### 3. 依赖更新策略

#### 自动化更新检查
```bash
# 检查过期依赖
cargo outdated

# 检查依赖更新
cargo update

# 交互式依赖更新
cargo install cargo-edit
cargo upgrade --interactive
```

#### 更新流程
1. **每周检查**: 自动检查依赖更新
2. **每月更新**: 批量更新非核心依赖
3. **季度大更新**: 核心依赖更新，充分测试
4. **安全更新**: 立即更新安全漏洞

#### 依赖锁定
```bash
# 生成锁定文件
cargo generate-lockfile

# 提交锁定文件到版本控制
git add Cargo.lock
git commit -m "Update dependency lock file"
```

### 4. 依赖安全策略

#### 安全扫描
```bash
# 安装 cargo-audit
cargo install cargo-audit

# 安全漏洞扫描
cargo audit

# 忽略特定漏洞
echo '{"advisories": {"RUSTSEC-2023-1234": "reason"}}' > .cargo/audit.toml
```

#### 依赖审计
```bash
# 使用 cargo-deny 进行许可证和依赖审计
cargo install cargo-deny
cargo deny check

# 生成依赖图
cargo install cargo-tree
cargo tree --duplicates
```

### 5. 依赖优化策略

#### 编译时间优化
```toml
# 使用 feature-gates 减少编译时间
[features]
default = ["full"]
full = ["database", "ai", "monitoring"]
database = ["sqlx", "redis"]
ai = ["async-openai", "candle-core"]
monitoring = ["prometheus", "metrics"]
```

#### 依赖去重
```bash
# 检查重复依赖
cargo tree --duplicates

# 移除未使用依赖
cargo install cargo-udep
cargo udep
```

#### 构建优化
```toml
# 优化构建配置
[profile.release]
lto = true
codegen-units = 1
panic = "abort"

[profile.dev]
opt-level = 1
debug = true
```

## 🏭 构建系统策略

### 1. Workspace 管理

#### Workspace 结构
```toml
# 根目录 Cargo.toml
[workspace]
members = [
    "lumosai_core",
    "lumosai_server",
    "lumosai_client",
    "lumosai_cli",
    "lumosai_macros",
]

resolver = "2"

[workspace.dependencies]
# 统一依赖版本
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
# ... 其他依赖
```

#### 依赖继承
```toml
# 子包 Cargo.toml
[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
```

### 2. 构建配置

#### 开发环境
```toml
[profile.dev]
opt-level = 1
debug = true
debug-assertions = true
overflow-checks = true
lto = false
incremental = true
codegen-units = 256
```

#### 生产环境
```toml
[profile.release]
opt-level = 3
debug = false
debug-assertions = false
overflow-checks = false
lto = true
incremental = false
codegen-units = 1
panic = "abort"
```

#### 测试环境
```toml
[profile.test]
opt-level = 1
debug = true
debug-assertions = true
overflow-checks = true
lto = false
incremental = true
codegen-units = 256
```

### 3. 特性管理

#### 条件编译
```rust
#[cfg(feature = "database")]
pub mod database;

#[cfg(feature = "ai")]
pub mod ai;

#[cfg(test)]
mod tests;
```

#### 默认特性
```toml
[features]
default = ["default-features"]
default-features = ["tokio/full", "serde/std"]

# 可选特性
database = ["sqlx", "redis"]
ai = ["async-openai", "candle-core"]
monitoring = ["prometheus", "metrics"]
```

## 🔧 开发工具链

### 1. 代码格式化和质量检查
```bash
# 安装工具
rustup component add clippy rustfmt

# 格式化代码
cargo fmt --all

# 代码质量检查
cargo clippy --all-targets --all-features -- -D warnings

# 修复代码问题
cargo clippy --fix
```

### 2. 测试工具
```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 并行测试
cargo install cargo-nextest
cargo nextest run

# 性能测试
cargo bench
```

### 3. 文档生成
```bash
# 生成文档
cargo doc --no-deps --all-features

# 在本地查看文档
cargo doc --open
```

### 4. 发布工具
```bash
# 安装发布工具
cargo install cargo-release

# 发布新版本
cargo release patch   # 补丁版本
cargo release minor   # 次版本
cargo release major   # 主版本

# 干运行（不实际发布）
cargo release --dry-run
```

## 📊 监控和维护

### 1. 依赖监控
```bash
# 设置定期检查
cargo outdated --exit-code 1
cargo audit
```

### 2. 性能监控
```bash
# 编译时间监控
cargo install cargo-timing
cargo build --timings

# 二进制大小监控
cargo install cargo-bloat
cargo bloat --release
```

### 3. 文档检查
```bash
# 检查文档覆盖率
cargo install cargo-deadlinks
cargo doc --no-deps --all-features && cargo deadlinks
```

## 🎯 最佳实践

### 1. 依赖管理最佳实践
- 使用 workspace 统一管理依赖版本
- 定期更新依赖，但保持谨慎
- 使用 feature-gates 减少编译时间
- 提交 Cargo.lock 到版本控制

### 2. 版本控制最佳实践
- 遵循语义化版本控制
- 在更新前测试兼容性
- 维护 CHANGELOG.md
- 使用 git tag 标记版本

### 3. 安全最佳实践
- 定期进行安全扫描
- 及时更新有漏洞的依赖
- 使用安全的默认配置
- 最小化权限原则

### 4. 性能最佳实践
- 优化编译配置
- 使用异步 I/O
- 合理使用连接池
- 监控内存使用

这个技术栈和依赖管理策略为 LumosAI 提供了一个稳定、高性能、可维护的基础设施。通过合理的依赖管理和构建策略，确保项目的长期成功。