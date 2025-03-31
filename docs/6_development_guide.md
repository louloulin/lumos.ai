# 6. 开发指南

本章节提供Lumos-X项目的开发指南，帮助开发者理解如何扩展和定制平台功能，以及如何参与项目贡献。

## 6.1 环境搭建

开始开发Lumos-X前，需要先搭建开发环境。

### 6.1.1 前置要求

- **Rust** (1.60或更高版本)
- **Node.js** (v16或更高版本)
- **pnpm** (推荐使用)
- **Docker** (用于容器化部署)
- **Git**
- 支持WebAssembly的浏览器

### 6.1.2 代码获取

从GitHub克隆项目仓库：

```bash
# 克隆主仓库
git clone https://github.com/lumosai/lumos-x.git
cd lumos-x

# 初始化子模块
git submodule update --init --recursive
```

### 6.1.3 开发环境设置

#### Rust开发环境

```bash
# 安装rustup (Rust版本管理工具)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 添加WebAssembly目标
rustup target add wasm32-unknown-unknown

# 安装wasm-pack (用于构建WebAssembly)
cargo install wasm-pack

# 安装Cargo工具
cargo install cargo-watch cargo-expand
```

#### JavaScript开发环境

```bash
# 安装pnpm
npm install -g pnpm

# 安装依赖
pnpm install

# 设置开发环境变量
cp .env.example .env.local
```

### 6.1.4 项目结构

```
lumos-x/
├── lumos_core/               # Rust核心库
├── lumos_server/             # Rust服务器
├── packages/
│   ├── client-js/            # JavaScript客户端
│   └── ui/                   # React UI组件
├── examples/                 # 示例项目
├── scripts/                  # 构建和开发脚本
├── docs/                     # 文档
└── tests/                    # 测试
```

## 6.2 核心库开发

### 6.2.1 构建和测试

```bash
# 进入核心库目录
cd lumos_core

# 构建库
cargo build

# 运行测试
cargo test

# 构建WebAssembly绑定
wasm-pack build --target web
```

### 6.2.2 创建新Agent能力

Agent能力是Lumos-X的核心扩展点，以下是创建新能力的基本步骤：

1. 在`lumos_core/src/agent/capabilities/`中创建新的模块文件

```rust
// lumos_core/src/agent/capabilities/my_capability.rs

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::agent::{Capability, CapabilityResult, ExecutionContext};

// 能力配置
#[derive(Debug, Serialize, Deserialize)]
pub struct MyCapabilityConfig {
    pub param1: String,
    pub param2: Option<u32>,
}

// 能力实现
pub struct MyCapability {
    config: MyCapabilityConfig,
}

impl MyCapability {
    pub fn new(config: MyCapabilityConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Capability for MyCapability {
    async fn execute(&self, input: serde_json::Value, context: &mut ExecutionContext) -> CapabilityResult {
        // 实现能力逻辑
        let result = // ...处理输入...
        
        Ok(serde_json::json!({
            "status": "success",
            "result": result
        }))
    }
    
    fn get_name(&self) -> &str {
        "my_capability"
    }
}
```

2. 在`mod.rs`中注册能力

```rust
// lumos_core/src/agent/capabilities/mod.rs

// ... 现有代码 ...
mod my_capability;
pub use my_capability::{MyCapability, MyCapabilityConfig};

// 在capability_factory函数中添加:
pub fn capability_factory(cap_type: &str, config: &serde_json::Value) -> Result<Box<dyn Capability>, CapabilityError> {
    match cap_type {
        // ... 其他能力 ...
        "my_capability" => {
            let config: MyCapabilityConfig = serde_json::from_value(config.clone())?;
            Ok(Box::new(MyCapability::new(config)))
        },
        _ => Err(CapabilityError::UnsupportedCapability(cap_type.to_string())),
    }
}
```

### 6.2.3 扩展内存系统

添加新的内存后端:

```rust
// lumos_core/src/memory/providers/my_storage.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::memory::{MemoryManager, MemoryItem, MemoryQuery, MemoryUpdate, MemoryError};

#[derive(Debug, Serialize, Deserialize)]
pub struct MyStorageConfig {
    pub connection_string: String,
    pub table_name: String,
}

pub struct MyStorageMemory {
    config: MyStorageConfig,
    // 实现特定的存储连接
}

impl MyStorageMemory {
    pub async fn new(config: MyStorageConfig) -> Result<Self, MemoryError> {
        // 初始化存储连接
        Ok(Self {
            config,
            // 初始化其他字段
        })
    }
}

#[async_trait]
impl MemoryManager for MyStorageMemory {
    async fn store(&self, item: MemoryItem) -> Result<String, MemoryError> {
        // 实现存储逻辑
    }
    
    async fn retrieve(&self, id: &str) -> Result<Option<MemoryItem>, MemoryError> {
        // 实现检索逻辑
    }
    
    async fn query(&self, query: MemoryQuery) -> Result<Vec<MemoryItem>, MemoryError> {
        // 实现查询逻辑
    }
    
    async fn update(&self, id: &str, updates: MemoryUpdate) -> Result<MemoryItem, MemoryError> {
        // 实现更新逻辑
    }
    
    async fn delete(&self, id: &str) -> Result<bool, MemoryError> {
        // 实现删除逻辑
    }
}
```

## 6.3 客户端库开发

### 6.3.1 TypeScript客户端库结构

```bash
cd packages/client-js

# 安装依赖
pnpm install

# 开发构建
pnpm dev

# 生产构建
pnpm build
```

### 6.3.2 添加新功能到客户端

以下是向客户端库添加新功能的示例:

```typescript
// packages/client-js/src/myFeature.ts

export interface MyFeatureConfig {
  option1: string;
  option2?: number;
}

export class MyFeature {
  private config: MyFeatureConfig;
  private client: LumosClient;
  
  constructor(client: LumosClient, config: MyFeatureConfig) {
    this.client = client;
    this.config = config;
  }
  
  async doSomething(input: any): Promise<any> {
    // 实现功能逻辑
    // 可以调用客户端的其他方法
    
    // 例如调用Rust核心库
    if (this.client.mode === 'local') {
      const result = await this.client.rustBridge.call_my_feature(
        JSON.stringify(input)
      );
      return JSON.parse(result);
    } else {
      // 调用远程API
      const response = await fetch(`${this.client.endpoint}/api/my-feature`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${this.client.apiKey}`
        },
        body: JSON.stringify(input)
      });
      
      return await response.json();
    }
  }
}

// 在client.ts中注册功能
// packages/client-js/src/client.ts

import { MyFeature, MyFeatureConfig } from './myFeature';

export class LumosClient {
  // ...现有代码...
  
  async myFeature(config: MyFeatureConfig): Promise<MyFeature> {
    await this.ensureInitialized();
    return new MyFeature(this, config);
  }
}
```

### 6.3.3 绑定Rust和JavaScript

通过WebAssembly连接Rust和JavaScript:

```typescript
// packages/client-js/src/rustBindings/bridge.ts

export class RustBridge {
  private wasm: any;
  private instance: any;
  
  constructor() {
    this.wasm = null;
    this.instance = null;
  }
  
  async initialize(wasmPath: string): Promise<void> {
    try {
      this.wasm = await import(wasmPath);
      this.instance = new this.wasm.LumosFFI();
    } catch (error) {
      throw new Error(`Failed to initialize Rust WASM: ${error.message}`);
    }
  }
  
  isInitialized(): boolean {
    return !!this.instance;
  }
  
  // 调用Rust核心库方法的通用包装器
  async call(method: string, ...args: any[]): Promise<string> {
    if (!this.isInitialized()) {
      throw new Error('Rust bridge not initialized');
    }
    
    try {
      const result = await (this.instance as any)[method](...args);
      return result;
    } catch (error) {
      throw new Error(`Error calling Rust method ${method}: ${error.message}`);
    }
  }
  
  // 为新功能添加特定方法
  async call_my_feature(input: string): Promise<string> {
    return this.call('my_feature', input);
  }
}
```

## 6.4 UI组件开发

### 6.4.1 React组件库

```bash
cd packages/ui

# 安装依赖
pnpm install

# 启动Storybook预览
pnpm storybook

# 构建组件库
pnpm build
```

### 6.4.2 创建新UI组件

```tsx
// packages/ui/src/components/MyComponent/MyComponent.tsx

import React, { useState } from 'react';
import './MyComponent.css';

export interface MyComponentProps {
  title: string;
  data?: Record<string, any>;
  onAction?: (actionType: string, payload: any) => void;
}

export const MyComponent: React.FC<MyComponentProps> = ({ 
  title, 
  data = {}, 
  onAction 
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  
  const handleToggle = () => {
    setIsExpanded(!isExpanded);
  };
  
  const handleActionClick = (actionType: string) => {
    if (onAction) {
      onAction(actionType, { ...data, timestamp: Date.now() });
    }
  };
  
  return (
    <div className="my-component">
      <div className="my-component-header" onClick={handleToggle}>
        <h3>{title}</h3>
        <span className={`toggle-icon ${isExpanded ? 'expanded' : ''}`}>
          ▼
        </span>
      </div>
      
      {isExpanded && (
        <div className="my-component-content">
          <pre>{JSON.stringify(data, null, 2)}</pre>
          
          <div className="action-buttons">
            <button onClick={() => handleActionClick('process')}>
              处理
            </button>
            <button onClick={() => handleActionClick('save')}>
              保存
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

// packages/ui/src/components/MyComponent/MyComponent.css
// 添加组件样式...

// packages/ui/src/components/MyComponent/index.ts
export * from './MyComponent';

// packages/ui/src/index.ts 中添加导出
export * from './components/MyComponent';
```

### 6.4.3 创建Storybook示例

```tsx
// packages/ui/src/components/MyComponent/MyComponent.stories.tsx

import React from 'react';
import { Meta, Story } from '@storybook/react';
import { MyComponent, MyComponentProps } from './MyComponent';

export default {
  title: 'Components/MyComponent',
  component: MyComponent,
  argTypes: {
    onAction: { action: 'action triggered' }
  },
} as Meta;

const Template: Story<MyComponentProps> = (args) => <MyComponent {...args} />;

export const Default = Template.bind({});
Default.args = {
  title: '默认组件',
  data: {
    id: '1234',
    status: 'active',
    items: [
      { name: 'Item 1', value: 100 },
      { name: 'Item 2', value: 200 },
    ]
  }
};

export const Empty = Template.bind({});
Empty.args = {
  title: '空数据组件',
  data: {}
};

export const LongTitle = Template.bind({});
LongTitle.args = {
  title: '这是一个非常长的标题，用于测试组件在处理长文本时的表现',
  data: {
    id: '5678',
    status: 'pending'
  }
};
```

## 6.5 服务器开发

### 6.5.1 Rust服务器结构

```bash
cd lumos_server

# 构建服务器
cargo build

# 运行服务器
cargo run

# 开发模式(自动重载)
cargo watch -x run
```

### 6.5.2 添加新API端点

```rust
// lumos_server/src/api/my_feature.rs

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::services::MyFeatureService;

#[derive(Deserialize)]
pub struct MyFeatureRequest {
    pub param1: String,
    pub param2: Option<u32>,
}

#[derive(Serialize)]
pub struct MyFeatureResponse {
    pub result: String,
    pub status: String,
}

pub async fn handle_my_feature(
    req: web::Json<MyFeatureRequest>,
    service: web::Data<MyFeatureService>,
) -> impl Responder {
    match service.process_feature(req.0).await {
        Ok(result) => HttpResponse::Ok().json(MyFeatureResponse {
            result,
            status: "success".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string(),
            "status": "error"
        })),
    }
}

// 在lumos_server/src/api/mod.rs中配置路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // ... 现有路由 ...
        .service(
            web::resource("/api/my-feature")
                .route(web::post().to(handle_my_feature))
        );
}

// 创建对应的服务
// lumos_server/src/services/my_feature.rs

pub struct MyFeatureService {
    // 依赖项
}

impl MyFeatureService {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn process_feature(&self, request: MyFeatureRequest) -> Result<String, anyhow::Error> {
        // 实现服务逻辑
        Ok("Feature processed successfully".to_string())
    }
}

// 在main.rs中注册服务
let my_feature_service = MyFeatureService::new();
let app_data = web::Data::new(my_feature_service);

HttpServer::new(move || {
    App::new()
        .app_data(app_data.clone())
        // ... 其他配置 ...
        .configure(api::configure_routes)
})
```

### 6.5.3 添加gRPC服务

使用tonic构建gRPC服务:

```rust
// 定义proto文件
// protos/my_feature.proto
syntax = "proto3";

package lumosai;

service MyFeatureService {
  rpc ProcessFeature(MyFeatureRequest) returns (MyFeatureResponse);
}

message MyFeatureRequest {
  string param1 = 1;
  optional uint32 param2 = 2;
}

message MyFeatureResponse {
  string result = 1;
  string status = 2;
}

// 实现服务
// lumos_server/src/grpc/my_feature.rs
use tonic::{Request, Response, Status};
use crate::proto::my_feature::{
    my_feature_service_server::MyFeatureService,
    MyFeatureRequest, MyFeatureResponse
};
use crate::services::MyFeatureService as CoreService;

pub struct MyFeatureGrpcService {
    core_service: CoreService,
}

impl MyFeatureGrpcService {
    pub fn new(core_service: CoreService) -> Self {
        Self { core_service }
    }
}

#[tonic::async_trait]
impl MyFeatureService for MyFeatureGrpcService {
    async fn process_feature(
        &self,
        request: Request<MyFeatureRequest>,
    ) -> Result<Response<MyFeatureResponse>, Status> {
        let req = request.into_inner();
        
        match self.core_service.process_feature(req.into()).await {
            Ok(result) => {
                let response = MyFeatureResponse {
                    result,
                    status: "success".to_string(),
                };
                Ok(Response::new(response))
            },
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

// 在main.rs中添加gRPC服务器
// 在同一个进程中运行HTTP和gRPC服务器
```

## 6.6 测试开发

### 6.6.1 核心库测试

```rust
// lumos_core/src/agent/capabilities/my_capability_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::ExecutionContext;
    
    #[tokio::test]
    async fn test_my_capability_basic() {
        // 准备测试数据
        let config = MyCapabilityConfig {
            param1: "test".to_string(),
            param2: Some(42),
        };
        
        let capability = MyCapability::new(config);
        let input = serde_json::json!({
            "test_input": "value"
        });
        
        let mut context = ExecutionContext::default();
        
        // 执行能力
        let result = capability.execute(input, &mut context).await;
        
        // 验证结果
        assert!(result.is_ok());
        let result_value = result.unwrap();
        assert_eq!(result_value["status"], "success");
        // 其他断言...
    }
    
    #[tokio::test]
    async fn test_my_capability_error_handling() {
        // 测试错误处理...
    }
}
```

### 6.6.2 JavaScript客户端测试

```typescript
// packages/client-js/tests/myFeature.test.ts

import { jest } from '@jest/globals';
import { LumosClient } from '../src/client';
import { MyFeature } from '../src/myFeature';

// 模拟Rust桥接
jest.mock('../src/rustBindings/bridge', () => {
  return {
    RustBridge: jest.fn().mockImplementation(() => {
      return {
        initialize: jest.fn().mockResolvedValue(undefined),
        isInitialized: jest.fn().mockReturnValue(true),
        call_my_feature: jest.fn().mockImplementation((input) => {
          const parsed = JSON.parse(input);
          return Promise.resolve(JSON.stringify({
            result: `Processed: ${parsed.value || 'unknown'}`,
            success: true
          }));
        })
      };
    })
  };
});

describe('MyFeature', () => {
  let client: LumosClient;
  
  beforeEach(() => {
    client = new LumosClient({ mode: 'local' });
    // 初始化客户端
    return client.initialize();
  });
  
  test('doSomething should process input correctly', async () => {
    const myFeature = await client.myFeature({ option1: 'test' });
    
    const result = await myFeature.doSomething({ value: 'test-input' });
    
    expect(result).toEqual({
      result: 'Processed: test-input',
      success: true
    });
  });
  
  // 更多测试...
});
```

### 6.6.3 集成测试

```typescript
// tests/integration/my_feature_integration.test.ts

import { LumosClient } from '@lomusai/client-js';

describe('MyFeature Integration Tests', () => {
  let client: LumosClient;
  
  beforeAll(async () => {
    // 启动测试服务器
    // ...
    
    // 创建客户端
    client = new LumosClient({
      mode: 'local',
      // 测试配置
    });
    
    await client.initialize();
  });
  
  afterAll(async () => {
    // 清理资源
    // ...
  });
  
  test('should process feature through entire stack', async () => {
    // 准备测试数据
    const myFeature = await client.myFeature({
      option1: 'integration-test'
    });
    
    // 执行操作
    const result = await myFeature.doSomething({
      value: 'test-through-stack'
    });
    
    // 验证结果
    expect(result).toHaveProperty('success', true);
    expect(result).toHaveProperty('result');
    // 更多断言...
  });
  
  // 更多集成测试...
});
```

## 6.7 贡献指南

### 6.7.1 提交规范

遵循约定式提交规范(Conventional Commits):

```
<类型>[可选 作用域]: <描述>

[可选 正文]

[可选 脚注]
```

类型包括:
- **feat**: 新功能
- **fix**: Bug修复
- **docs**: 文档变更
- **style**: 代码风格变更(格式化等)
- **refactor**: 重构代码
- **perf**: 性能优化
- **test**: 添加测试
- **chore**: 构建/工具变更

例如:
```
feat(agent): 添加新的Agent能力系统

- 实现了能力注册机制
- 添加了能力测试框架
- 更新了文档

Closes #123
```

### 6.7.2 代码风格

#### Rust代码风格

- 使用`rustfmt`格式化代码
- 遵循Rust官方API指南
- 使用`clippy`检查代码质量

```bash
# 运行rustfmt
cargo fmt

# 运行clippy
cargo clippy
```

#### TypeScript代码风格

- 使用ESLint和Prettier
- 遵循项目中定义的规则

```bash
# 检查代码风格
pnpm lint

# 自动修复
pnpm lint:fix
```

### 6.7.3 Pull Request流程

1. Fork项目仓库
2. 创建功能分支 (`git checkout -b feat/my-feature`)
3. 提交更改 (`git commit -am 'feat: add feature X'`)
4. 推送到远程分支 (`git push origin feat/my-feature`)
5. 创建Pull Request

Pull Request应包含:
- 清晰的标题和描述
- 关联的Issue编号
- 实现的功能或修复的问题
- 任何破坏性变更的说明
- 测试结果

### 6.7.4 文档贡献

项目文档主要包括:

1. **API文档**: 使用Rust文档注释和TypeScript JSDoc
2. **开发指南**: 本文档中的内容
3. **用户手册**: 面向终端用户的使用指南

提交文档变更时，请遵循以下原则:
- 保持简洁清晰
- 提供实用示例
- 确保与当前代码保持同步
- 包含足够的上下文信息

## 6.8 发布流程

### 6.8.1 版本管理

遵循语义化版本(SemVer):

- **主版本号(x.0.0)**: 不兼容的API变更
- **次版本号(0.x.0)**: 向后兼容的功能性新增
- **修订号(0.0.x)**: 向后兼容的问题修正

### 6.8.2 发布检查清单

在发布之前确保完成以下步骤:

1. 所有测试通过
2. 文档已更新
3. CHANGELOG已更新
4. 版本号已更新
5. 性能测试通过
6. 安全审计通过

### 6.8.3 发布流程

```bash
# 准备发布
pnpm release:prepare

# 发布核心库
cd lumos_core
cargo publish

# 发布客户端库
cd packages/client-js
pnpm publish

# 发布UI组件库
cd packages/ui
pnpm publish

# 创建发布标签
git tag v1.2.3
git push origin v1.2.3
```

### 6.8.4 持续集成

项目使用GitHub Actions进行CI/CD:

- 代码提交时自动运行测试
- Pull Request合并前验证构建和测试
- 发布流程自动化

配置文件位于`.github/workflows/`目录下。 