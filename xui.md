# XUI: LumosAI UI Framework

## 概述

XUI 是一个基于 Next.js 和 Shadcn UI 的现代化 UI 框架，为 LumosAI 提供一致、高效且美观的用户界面组件。这个框架旨在解决 lumosai_ui 项目中遇到的问题，特别是在构建、启动和 Tailwind CSS 配置方面的问题，同时借鉴 playground-ui 的优秀设计理念。

## 技术栈

- **核心框架**: [Next.js 14+](https://nextjs.org/) (使用 App Router)
- **UI 组件**: [Shadcn UI](https://ui.shadcn.com/) (基于 Radix UI 的组件集合)
- **样式**: [Tailwind CSS](https://tailwindcss.com/)
- **状态管理**: [Zustand](https://github.com/pmndrs/zustand)
- **类型系统**: [TypeScript](https://www.typescriptlang.org/)
- **包管理器**: [pnpm](https://pnpm.io/) (推荐使用，但也支持 npm 和 yarn)
- **构建工具**: 使用 Next.js 内置的构建系统

## 目录结构

```
xui/
├── app/                      # Next.js 应用路由
│   ├── layout.tsx            # 根布局
│   ├── page.tsx              # 首页
│   └── [...]/                # 其他页面路由
├── components/               # UI 组件
│   ├── ui/                   # Shadcn UI 基础组件
│   └── domains/              # 领域特定组件
├── lib/                      # 工具函数和辅助模块
├── styles/                   # 全局样式
├── types/                    # TypeScript 类型定义
├── public/                   # 静态资源
├── next.config.js            # Next.js 配置
├── tailwind.config.js        # Tailwind CSS 配置
├── postcss.config.js         # PostCSS 配置
├── tsconfig.json             # TypeScript 配置
└── package.json              # 项目依赖
```

## 特点与优势

1. **可靠的构建系统**：使用 Next.js 的构建系统，避免 Vite 和 Tailwind CSS 配置问题
2. **服务端渲染 (SSR) 支持**：提高首屏加载性能和 SEO
3. **TypeScript 集成**：提供类型安全的开发体验
4. **组件驱动开发**：使用 Shadcn UI 提供一致的设计系统
5. **主题支持**：轻松实现浅色/深色主题和品牌定制
6. **响应式设计**：适配各种屏幕尺寸的设备
7. **无障碍支持**：通过 Radix UI 原语确保 UI 的可访问性
8. **模块化架构**：便于扩展和维护

## 主要组件

XUI 将提供以下关键组件：

### 核心 UI 组件
- 按钮 (Button)
- 文本输入 (Input)
- 选择菜单 (Select)
- 对话框 (Dialog)
- 表格 (Table)
- 卡片 (Card)
- 标签页 (Tabs)
- 导航 (Navigation)
- 通知 (Notifications)

### 专业组件
- 代码编辑器 (使用 Monaco Editor)
- Markdown 查看器 (使用 react-markdown)
- 语法高亮 (使用 Prism 或 Shiki)
- 流式响应展示 (Stream Response Display)
- 聊天界面 (Chat Interface)
- 工作流可视化 (Workflow Visualization)
- 数据图表 (Data Charts)

## 实现路线图

### 阶段 1: 基础架构 (1周)
- 初始化 Next.js 项目
- 配置 Tailwind CSS 和 Shadcn UI
- 设置基本目录结构和开发环境
- 实现基础布局和主题系统

### 阶段 2: 核心组件开发 (2周)
- 实现所有基础 UI 组件
- 开发布局系统
- 创建主题切换功能
- 设计响应式网格系统

### 阶段 3: 专业组件开发 (2周)
- 集成代码编辑器
- 构建聊天界面组件
- 开发 Markdown 渲染系统
- 实现数据可视化组件

### 阶段 4: 示例与文档 (1周)
- 创建组件展示页面
- 编写组件使用文档
- 开发示例应用程序
- 提供最佳实践指南

## 性能优化策略

- 使用组件懒加载
- 实现图像优化
- 采用增量静态生成 (ISR)
- 利用 Next.js 内置的性能优化功能
- 最小化 JavaScript 包大小
- 使用 Edge Runtime 加速数据获取

## 如何使用

```jsx
// 示例: 创建一个简单的聊天界面
import { ChatContainer, ChatMessage, ChatInput } from '@lumosai/xui';

export default function ChatPage() {
  return (
    <ChatContainer>
      <ChatMessage 
        role="user"
        content="你好，我需要一些帮助！" 
      />
      <ChatMessage 
        role="assistant"
        content="您好！我是 LumosAI 助手，很高兴为您提供帮助。请告诉我您需要什么协助？" 
      />
      <ChatInput 
        placeholder="输入您的消息..."
        onSend={(message) => console.log('发送消息:', message)} 
      />
    </ChatContainer>
  );
}
```

## 发布策略

XUI 将作为 npm 包发布，支持以下使用方式：

1. **完整引入**: 导入所有组件
2. **按需加载**: 只导入需要的组件
3. **主题定制**: 通过 CSS 变量覆盖默认主题

## 与现有系统的集成

XUI 将与 LumosAI 的其他组件无缝集成，特别是：

- 与 LumosAI API 集成
- 支持现有的数据结构和模型
- 提供向后兼容的 API

## 为什么选择 Next.js 而不是 Vite

在 lumosai_ui 项目中，我们遇到了 Vite 与 Tailwind CSS 配置和构建相关的问题。使用 Next.js 的主要优势包括：

1. **稳定的构建系统**：Next.js 提供了一个成熟且经过良好测试的构建系统
2. **简化的配置**：内置的样式支持和优化功能减少了配置工作
3. **TypeScript 支持**：开箱即用的 TypeScript 支持
4. **服务端渲染**：提高性能和 SEO
5. **静态导出选项**：可以导出为静态网站
6. **全栈能力**：API 路由和数据获取功能

## 结论

XUI 框架将为 LumosAI 提供一个稳定、高效、美观的 UI 解决方案，解决当前 lumosai_ui 项目中的构建和配置问题。通过采用 Next.js 和 Shadcn UI，我们可以利用现代 Web 开发的最佳实践，提供出色的开发体验和用户体验。 