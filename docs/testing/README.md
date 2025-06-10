# 🧪 LumosAI 测试框架

LumosAI 项目的全面测试框架，包含单元测试、集成测试、性能测试和自动化测试。

## 📋 目录结构

```
tests/
├── test_config.rs          # 测试配置和工具
├── simple_test.rs          # 简单测试验证
├── lib.rs                  # 测试库入口
├── unit/                   # 单元测试
│   ├── mod.rs
│   ├── agent_tests.rs      # Agent 系统测试
│   ├── vector_tests.rs     # 向量存储测试
│   ├── rag_tests.rs        # RAG 系统测试
│   ├── memory_tests.rs     # 内存系统测试
│   ├── tool_tests.rs       # 工具系统测试
│   ├── session_tests.rs    # 会话系统测试
│   └── workflow_tests.rs   # 工作流测试
├── integration/            # 集成测试
│   ├── mod.rs
│   ├── agent_rag_integration.rs  # Agent + RAG 集成
│   ├── workflow_integration.rs   # 工作流集成
│   ├── memory_integration.rs     # 内存集成
│   ├── full_system_integration.rs # 全系统集成
│   └── api_integration.rs        # API 集成
├── performance/            # 性能测试
│   ├── mod.rs
│   ├── agent_performance.rs      # Agent 性能测试
│   ├── vector_performance.rs     # 向量存储性能
│   ├── rag_performance.rs        # RAG 性能测试
│   ├── memory_performance.rs     # 内存性能测试
│   ├── concurrent_performance.rs # 并发性能测试
│   └── load_testing.rs           # 负载测试
├── coverage/               # 测试覆盖率
│   └── mod.rs
└── automation/             # 自动化测试
    └── test_runner.rs      # 测试运行器

scripts/
├── run_tests.sh           # Linux/macOS 测试脚本
└── run_tests.bat          # Windows 测试脚本

.github/workflows/
└── tests.yml              # GitHub Actions CI/CD
```

## 🚀 快速开始

### 运行所有测试

```bash
# 使用脚本（推荐）
./scripts/run_tests.sh

# 或者直接使用 cargo
cargo test
```

### 运行特定测试类型

```bash
# 单元测试
cargo test --lib

# 集成测试
cargo test --tests integration

# 性能测试
cargo test --tests performance --release

# 简单验证测试
cargo test --test simple_test
```

### 运行特定测试

```bash
# 运行 Agent 相关测试
cargo test agent

# 运行向量存储测试
cargo test vector

# 运行 RAG 系统测试
cargo test rag
```

## 🛠️ 测试工具和配置

### TestConfig

测试配置类，提供统一的测试环境设置：

```rust
use tests::test_config::*;

#[tokio::test]
async fn my_test() {
    init_test_env();
    let config = TestConfig::default();
    // 你的测试代码
}
```

### TestUtils

测试工具类，提供常用的测试辅助函数：

```rust
// 创建测试 Agent
let agent = TestUtils::create_test_agent("test-agent").await?;

// 创建测试向量存储
let storage = TestUtils::create_test_vector_storage().await?;

// 生成测试文档
let docs = TestUtils::generate_test_documents(10);
```

### PerformanceTestUtils

性能测试工具：

```rust
// 测量执行时间
let (result, duration) = PerformanceTestUtils::measure_time(|| async {
    // 你的异步操作
}).await;

// 基准测试
let durations = PerformanceTestUtils::benchmark(
    "test_name",
    100, // 迭代次数
    || async { /* 测试代码 */ }
).await;
```

### TestAssertions

测试断言工具：

```rust
// 验证 Agent 响应
TestAssertions::assert_valid_agent_response(&response);

// 验证搜索结果
TestAssertions::assert_valid_search_results(&results, min_count);

// 验证会话状态
TestAssertions::assert_valid_session_state(&session);
```

## 📊 测试覆盖率

### 生成覆盖率报告

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir target/coverage
```

### 覆盖率配置

```rust
use tests::coverage::*;

let report = CoverageUtils::generate_report();
let config = CoverageConfig::default(); // 80% 目标覆盖率

if CoverageUtils::check_coverage_requirements(&report, &config) {
    println!("✅ 覆盖率达标");
} else {
    println!("❌ 覆盖率不足");
}
```

## ⚡ 性能测试

### 基准测试

```rust
#[tokio::test]
async fn benchmark_agent_creation() {
    let durations = PerformanceTestUtils::benchmark(
        "agent_creation",
        100,
        || async {
            let _agent = Agent::builder()
                .name("benchmark-agent")
                .build()
                .await
                .unwrap();
        }
    ).await;
    
    // 性能断言
    let avg_duration = durations.iter().sum::<Duration>() / 100;
    assert!(avg_duration < Duration::from_millis(100));
}
```

### 并发测试

```rust
#[tokio::test]
async fn test_concurrent_operations() {
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            // 并发操作
        });
        handles.push(handle);
    }
    
    // 等待所有操作完成
    for handle in handles {
        handle.await.unwrap();
    }
}
```

## 🤖 自动化测试

### 测试运行器

```rust
use tests::automation::test_runner::TestRunner;

let mut runner = TestRunner::new();
let success = runner.run_all().await;

// 生成报告
runner.save_report("test_report.md").unwrap();
```

### CI/CD 集成

项目包含 GitHub Actions 配置，自动运行：

- 代码质量检查（格式化、Clippy）
- 单元测试
- 集成测试
- 示例验证
- 性能测试
- 安全审计

## 📝 编写测试

### 单元测试模板

```rust
use crate::test_config::*;

#[tokio::test]
async fn test_my_feature() {
    init_test_env();
    
    // 准备测试数据
    let test_data = "test input";
    
    // 执行测试
    let result = my_function(test_data).await;
    
    // 验证结果
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response, "expected output");
}
```

### 集成测试模板

```rust
use crate::test_config::*;

#[tokio::test]
async fn test_system_integration() {
    init_test_env();
    
    // 设置集成环境
    let env = IntegrationTestUtils::setup_integration_env().await.unwrap();
    
    // 执行集成测试
    let result = integrated_operation(&env).await;
    
    // 验证集成结果
    assert!(result.is_ok());
}
```

### 性能测试模板

```rust
use crate::test_config::*;
use std::time::Duration;

#[tokio::test]
async fn test_performance() {
    init_test_env();
    
    let (result, duration) = PerformanceTestUtils::measure_time(|| async {
        // 性能测试代码
        expensive_operation().await
    }).await;
    
    // 性能断言
    assert!(result.is_ok());
    PerformanceTestUtils::assert_execution_time_within(
        duration,
        Duration::from_secs(5)
    );
}
```

## 🔧 测试配置

### 环境变量

```bash
# 设置测试超时
export TEST_TIMEOUT=300

# 启用详细日志
export RUST_LOG=debug

# 设置测试并发数
export RUST_TEST_THREADS=4
```

### Cargo.toml 配置

```toml
[dev-dependencies]
tokio-test = "0.4"
criterion = "0.5"
proptest = "1.0"

[[test]]
name = "integration"
path = "tests/integration/mod.rs"

[[test]]
name = "performance"
path = "tests/performance/mod.rs"
```

## 📈 测试指标

### 目标指标

- **代码覆盖率**: ≥ 80%
- **单元测试通过率**: 100%
- **集成测试通过率**: ≥ 95%
- **性能测试**: 符合基准要求
- **测试执行时间**: < 10 分钟

### 监控指标

- 测试执行时间趋势
- 覆盖率变化
- 失败率统计
- 性能回归检测

## 🚨 故障排除

### 常见问题

1. **测试超时**
   ```bash
   # 增加超时时间
   cargo test -- --test-threads=1 --nocapture
   ```

2. **内存不足**
   ```bash
   # 减少并发测试数量
   export RUST_TEST_THREADS=2
   ```

3. **依赖冲突**
   ```bash
   # 清理并重新构建
   cargo clean && cargo test
   ```

### 调试技巧

```rust
// 启用测试日志
#[tokio::test]
async fn debug_test() {
    env_logger::init();
    log::debug!("Debug information");
    // 测试代码
}
```

## 🤝 贡献指南

1. 为新功能编写对应的测试
2. 确保测试覆盖率不低于 80%
3. 性能测试应包含基准和回归检测
4. 集成测试应覆盖主要用户场景
5. 提交前运行完整测试套件

## 📚 相关文档

- [开发指南](../development/README.md)
- [API 文档](../api/README.md)
- [部署指南](../deployment/README.md)
- [贡献指南](../../CONTRIBUTING.md)
