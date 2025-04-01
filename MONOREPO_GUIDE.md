# LumosAI Monorepo 使用指南

## 项目结构

这个项目使用monorepo结构组织代码，包含以下主要部分：

```
lumosai/
├── packages/                # JS包 (client-js, ui-components等)
│   ├── client-js/           # LumosAI的JavaScript客户端
│   ├── ui-components/       # UI组件库
│   └── ...                  # 其他JS包
├── lumosai_ui/              # UI应用程序
├── lumosai_core/            # Rust核心库
├── lumosai_cli/             # CLI工具
├── scripts/                 # 工具脚本
│   ├── verify-workspace.js  # 验证工作区配置
│   └── create-package.js    # 创建新包
└── ...                      # 其他Rust包和项目文件
```

## 前端开发指南 (JavaScript)

### 安装依赖

```bash
# 安装所有工作区依赖
pnpm install
```

### 开发命令

```bash
# 运行客户端库开发模式（监视文件变化）
pnpm dev:client

# 运行UI开发服务器
pnpm dev:ui

# 运行UI组件库开发模式
pnpm dev:ui-components

# 构建所有包
pnpm build:all

# 构建特定包
pnpm build:client
pnpm build:ui
pnpm build:ui-components
```

### 验证工作区配置

```bash
pnpm verify-workspace
```

### 创建新的JS包

```bash
# 基本用法
pnpm create-package <包名>

# 示例
pnpm create-package server
# 将创建 @lumosai/server 包
```

## 包依赖关系

### 工作区引用

在monorepo中，包可以通过工作区引用相互依赖：

```json
{
  "dependencies": {
    "@lumosai/client-js": "workspace:*"
  }
}
```

这允许在开发过程中，对一个包的更改立即反映在依赖它的其他包中。

## 模块导入问题解决

如果在使用工作区包时遇到模块导入问题，请查看以下常见解决方案：

### 解决`Failed to resolve entry for package`错误

这个错误通常是由于包的`exports`字段配置不正确导致的。确保package.json文件正确配置：

```json
{
  "name": "@lumosai/client-js",
  "main": "dist/index.js", 
  "module": "dist/index.mjs",
  "types": "dist/index.d.ts",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.mjs",
      "require": "./dist/index.js",
      "default": "./dist/index.js"
    }
  }
}
```

关键点：
1. `types`字段应该在`import`和`require`前面
2. 应使用合适的构建工具生成多种格式输出
3. 对于Vite项目，确保生成.mjs文件用于ESM导入

### 使用tsup进行构建

建议使用tsup进行包的构建，它可以生成适合不同环境的输出格式：

```bash
# 安装tsup
pnpm add -D tsup --filter @lumosai/<package-name>

# 配置构建脚本
# package.json
{
  "scripts": {
    "build": "tsup src/index.ts --format esm,cjs --dts",
    "dev": "tsup src/index.ts --format esm,cjs --dts --watch"
  }
}
```

创建`tsup.config.ts`文件进行更详细的配置：

```typescript
import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['cjs', 'esm'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  outDir: 'dist',
  outExtension({ format }) {
    return {
      js: format === 'cjs' ? '.js' : '.mjs',
    };
  },
});
```

### 更新tsconfig.json

确保tsconfig.json正确配置：

```json
{
  "compilerOptions": {
    "moduleResolution": "node",
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true
  }
}
```

## 关键包说明

### @lumosai/client-js

JavaScript客户端库，提供与LumosAI API的交互能力：

```javascript
import { LumosAIClient } from '@lumosai/client-js';

// 初始化客户端
const client = new LumosAIClient({
  apiKey: 'your-api-key',
  baseUrl: 'https://api.lumosai.com', // 可选，默认为官方API
});

// 使用代理
const agent = client.getAgent('agent-id');
const response = await agent.generate('请介绍一下你自己');
```

### @lumosai/playground-ui (lumosai_ui)

LumosAI的UI应用程序，提供可视化界面来使用和测试代理。

### @lumosai/ui-components

可重用的UI组件库，用于构建与LumosAI集成的应用程序。

## 前后端集成

JavaScript客户端(@lumosai/client-js)用于连接Rust后端API，主要功能包括：

- 代理调用
- 流式响应处理
- 工具调用
- 身份验证管理

通过完整的TypeScript类型定义，确保API调用的类型安全。

## 常见问题

### 如何在UI项目中使用Rust后端API？

UI通过client-js库与Rust后端交互：

1. 确保在package.json中引用了client-js：`"@lumosai/client-js": "workspace:*"`
2. 初始化客户端：`const client = new LumosAIClient({ apiKey, baseUrl })`
3. 调用API：`const response = await client.getAgent(agentId).generate(prompt)`

### 如何添加新的API端点支持？

1. 在client-js库中添加新的方法
2. 在UI中通过client-js调用该方法
3. 确保Rust后端实现了相应的API端点

### 工作区脚本停止工作怎么办？

运行`pnpm install`重新链接工作区依赖，然后运行`pnpm verify-workspace`确认配置正确。

### 修改client-js后UI无法正确导入怎么办？

1. 确保在client-js目录运行`pnpm build`
2. 检查package.json的exports字段配置是否正确
3. 在UI项目中确认导入路径：`import { LumosAIClient } from '@lumosai/client-js';`
4. 如果仍有问题，尝试在UI项目中运行`pnpm install`重新链接依赖 