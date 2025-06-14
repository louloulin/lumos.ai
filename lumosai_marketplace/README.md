# Lumos.ai 工具市场建设模块

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Lumos.ai的工具市场建设模块，提供完整的工具生态系统，包括工具发布、发现、评估、安全扫描等功能。

## 🚀 主要特性

### 核心功能
- **🔧 工具注册和发布**: 完整的工具包生命周期管理
- **🔍 智能搜索和发现**: 基于Tantivy的全文搜索引擎
- **⭐ 个性化推荐**: 基于用户行为的智能推荐系统
- **📊 使用分析**: 详细的下载、使用和评分统计
- **🛡️ 安全扫描**: 自动化的安全漏洞检测和评估
- **✅ 质量验证**: 多层次的工具包质量验证机制

### 高级特性
- **🎯 分类管理**: 支持多种工具分类和标签系统
- **🔄 版本控制**: 完整的版本管理和依赖解析
- **📈 趋势分析**: 热门工具和趋势分析
- **🔗 相似推荐**: 基于内容的相似工具推荐
- **🌐 RESTful API**: 完整的HTTP API接口
- **📦 批量操作**: 支持批量工具管理操作

## 📦 架构设计

### 模块结构
```
lumosai_marketplace/
├── src/
│   ├── lib.rs              # 主模块入口
│   ├── error.rs            # 错误类型定义
│   ├── config.rs           # 配置管理
│   ├── models.rs           # 数据模型
│   ├── storage.rs          # 存储层实现
│   ├── search.rs           # 搜索引擎
│   ├── registry.rs         # 工具注册表
│   ├── validator.rs        # 工具验证器
│   ├── publisher.rs        # 工具发布器
│   ├── discovery.rs        # 发现引擎
│   ├── analytics.rs        # 分析引擎
│   ├── security.rs         # 安全扫描器
│   ├── marketplace.rs      # 主市场类
│   └── api.rs              # HTTP API
├── examples/
│   └── marketplace_demo.rs # 完整演示
└── tests/                  # 测试用例
```

### 核心组件

#### 1. 工具注册表 (ToolRegistry)
- 工具包的注册、更新、删除
- 版本管理和依赖解析
- 元数据存储和检索

#### 2. 搜索引擎 (SearchEngine)
- 基于Tantivy的全文搜索
- 模糊搜索和语义搜索
- 实时索引更新

#### 3. 发现引擎 (DiscoveryEngine)
- 个性化推荐算法
- 相似工具发现
- 热门趋势分析

#### 4. 安全扫描器 (SecurityScanner)
- 代码安全扫描
- 依赖漏洞检测
- 权限风险评估

#### 5. 分析引擎 (Analytics)
- 使用统计收集
- 性能指标分析
- 报告生成

## 🎯 快速开始

### 基础使用

```rust
use lumosai_marketplace::{ToolMarketplace, MarketplaceBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建工具市场
    let marketplace = MarketplaceBuilder::new()
        .database_url("sqlite://marketplace.db")
        .search_index_path("./search_index")
        .enable_security_scanning(true)
        .enable_analytics(true)
        .build()
        .await?;
    
    // 搜索工具
    let results = marketplace.search("web scraping").await?;
    println!("找到 {} 个工具", results.len());
    
    // 获取热门工具
    let trending = marketplace.get_trending(None, 10).await?;
    for tool in trending {
        println!("热门工具: {}", tool.package.name);
    }
    
    Ok(())
}
```

### 工具包发布

```rust
use lumosai_marketplace::{
    publisher::{PublishRequest, PublisherInfo},
    models::ToolPackage,
};

// 创建发布请求
let publish_request = PublishRequest {
    package: my_tool_package,
    package_path: "./my_tool.tar.gz".to_string(),
    skip_validation: false,
    skip_security_scan: false,
    publisher_info: PublisherInfo {
        id: "my_publisher".to_string(),
        name: "My Publisher".to_string(),
        email: "publisher@example.com".to_string(),
        api_key: Some("api_key_123".to_string()),
    },
};

// 发布工具包
let result = marketplace.publish_package(publish_request).await?;
if result.success {
    println!("工具包发布成功!");
} else {
    println!("发布失败: {}", result.error_message.unwrap_or_default());
}
```

### 高级搜索

```rust
use lumosai_marketplace::{
    search::SearchQuery,
    models::ToolCategory,
};

let query = SearchQuery {
    text: "data processing".to_string(),
    categories: vec![ToolCategory::Data, ToolCategory::AI],
    published_only: true,
    verified_only: true,
    min_rating: Some(4.0),
    limit: 20,
    ..Default::default()
};

let results = marketplace.advanced_search(&query).await?;
```

## 🔧 配置选项

### 数据库配置
- **SQLite**: `sqlite://path/to/database.db`
- **PostgreSQL**: `postgresql://user:pass@host/db`
- **MySQL**: `mysql://user:pass@host/db`

### 搜索配置
- **索引路径**: 本地文件系统路径
- **更新间隔**: 索引更新频率
- **模糊搜索**: 启用/禁用模糊匹配

### 安全配置
- **代码扫描**: 静态代码分析
- **依赖扫描**: 已知漏洞检测
- **权限检查**: 权限风险评估

## 📊 数据模型

### 工具包 (ToolPackage)
```rust
pub struct ToolPackage {
    pub id: Uuid,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub author: String,
    pub license: String,
    pub keywords: Vec<String>,
    pub categories: Vec<ToolCategory>,
    pub dependencies: HashMap<String, String>,
    pub manifest: ToolManifest,
    pub created_at: DateTime<Utc>,
    pub download_count: u64,
    pub rating: f64,
    pub published: bool,
    pub verified: bool,
    // ... 更多字段
}
```

### 工具分类
- 🌐 **Web**: 网络工具
- 📁 **File**: 文件操作
- 📊 **Data**: 数据处理
- 🤖 **AI**: AI相关
- ⚙️ **System**: 系统工具
- 🔢 **Math**: 数学计算
- 🔐 **Crypto**: 加密工具
- 🗄️ **Database**: 数据库
- 🔌 **API**: API工具
- 🛠️ **Utility**: 实用工具

## 🧪 测试

运行所有测试：
```bash
cargo test -p lumosai_marketplace
```

运行特定测试：
```bash
cargo test -p lumosai_marketplace test_marketplace_creation
```

运行示例：
```bash
cargo run --example marketplace_demo
```

## 📈 性能特性

### 搜索性能
- **全文索引**: 基于Tantivy的高性能搜索
- **增量更新**: 实时索引更新
- **缓存机制**: 查询结果缓存

### 存储优化
- **连接池**: 数据库连接复用
- **批量操作**: 减少数据库往返
- **索引优化**: 针对查询模式的索引设计

### 并发处理
- **异步架构**: 全异步I/O操作
- **并发限制**: 防止资源过载
- **负载均衡**: 请求分发优化

## 🔒 安全特性

### 代码安全
- **静态分析**: 危险代码模式检测
- **依赖扫描**: 已知漏洞数据库
- **权限审计**: 权限使用合理性检查

### 数据安全
- **输入验证**: 严格的输入验证
- **SQL注入防护**: 参数化查询
- **访问控制**: 基于角色的权限控制

## 🚀 部署指南

### 开发环境
```bash
# 克隆项目
git clone https://github.com/louloulin/lumos.ai.git
cd lumos.ai/lumosai_marketplace

# 安装依赖
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example marketplace_demo
```

### 生产环境
1. **数据库设置**: 配置PostgreSQL或MySQL
2. **搜索索引**: 设置专用的搜索索引目录
3. **缓存配置**: 配置Redis缓存
4. **监控设置**: 配置日志和监控

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](../LICENSE) 文件了解详细信息。

## 🔗 相关链接

- [Lumos.ai 主项目](https://github.com/louloulin/lumos.ai)
- [API 文档](https://docs.lumosai.com/marketplace)
- [开发者指南](https://docs.lumosai.com/developers)

---

**Lumos.ai 工具市场** - 构建强大的AI工具生态系统 🚀
