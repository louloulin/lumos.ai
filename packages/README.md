# LumosAI JavaScript 包

这个目录包含LumosAI框架的JavaScript相关包。

## 当前包

- **client-js**: LumosAI的JavaScript客户端
  - 提供与LumosAI API交互的客户端库
  - 支持Agent、工具和流式响应
  - 可在Node.js和浏览器环境中使用

## 开发指南

所有包都被配置为PNPM工作区的一部分，可以使用以下命令进行开发：

```bash
# 开发client-js包
pnpm dev:client

# 构建client-js包
pnpm build:client

# 构建所有包
pnpm build:all
```

## 添加新包

添加新包时，请遵循以下命名规范：

1. 包名使用 `@lumosai/*` 命名空间
2. 在package.json中包含适当的描述、关键词和许可证
3. 确保包导出类型定义
4. 遵循monorepo的依赖约定

## 与UI集成

包被配置为可从UI项目引用：

```json
{
  "dependencies": {
    "@lumosai/client-js": "workspace:*"
  }
}
```

这使得在开发过程中对包的更改可以立即反映在UI中。 