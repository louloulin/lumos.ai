# LumosAI UI 迁移计划 (lumosai-nui-new)

## 概述

本文档概述了将现有的 `lumosai_ui` 和 `playground-ui` 组件库迁移到新的基于 Tauri 的 `lumosai-nui-new` 项目的计划。新框架将提供更现代化的用户界面，同时解决现有项目中的构建、启动和样式配置问题。

## 当前情况分析

### lumosai_ui

- 基于 Vite 构建
- 使用 Tailwind CSS 进行样式管理
- 使用 React 19
- 存在构建、启动和 Tailwind CSS 配置方面的问题
- 包含大量 Radix UI 组件和其他第三方库

### playground-ui

- 基于 Vite 构建
- 使用 Tailwind CSS
- 使用 React 19
- 有较好的设计理念和组件结构
- 同样依赖于 Radix UI 组件

### lumosai-nui-new (目标项目)

- 基于 Tauri 2.0 和 React 18
- 使用 Vite 作为构建工具
- 目前只有基础结构，缺乏 UI 组件和样式系统
- 能够打包为桌面应用

## 迁移目标

1. 创建一个统一的 UI 组件库，继承两个现有库的最佳实践
2. 解决现有的构建和配置问题
3. 优化性能和用户体验
4. 提供一个可以打包为桌面应用的解决方案
5. 保持组件的可重用性和可维护性

## 迁移策略

### 第一阶段：基础设置 (1-2周) [已完成]

1. **项目结构设置** [已完成]
   - 在 `lumosai-nui-new` 中创建合适的目录结构
   - 设置 `src/components` 目录用于UI组件
   - 设置 `src/styles` 目录用于样式文件
   - 创建 `src/lib` 目录用于工具函数和共享逻辑

2. **依赖项管理** [已完成]
   - 添加必要的依赖，包括但不限于：
     - Tailwind CSS 和相关插件
     - Radix UI 组件
     - React Hook Form
     - Zod 用于验证
     - Lucide React 用于图标
     - 其他必要的第三方库

3. **样式系统设置** [已完成]
   - 配置 Tailwind CSS
   - 集成 CSS 变量系统
   - 实现主题切换功能
   - 设置全局样式基础

4. **Tauri 配置优化** [已完成]
   - 完善 `tauri.conf.json` 配置
   - 设置应用窗口属性
   - 配置应用安全策略
   - 设置构建选项

### 第二阶段：核心组件迁移 (2-3周) [进行中]

1. **基础组件** [已完成]
   - 按钮 (Button) ✅
   - 输入框 (Input) ✅
   - 选择器 (Select) ✅
   - 开关 (Switch) ✅
   - 复选框 (Checkbox) ✅
   - 标签 (Label) ✅
   - 卡片 (Card) ✅

2. **布局组件** [已完成]
   - 网格 (Grid) - 使用 CSS Grid 替代 ✅
   - 弹性布局 (Flex) ✅
   - 容器 (Container) ✅
   - 分隔线 (Separator) ✅
   - 滚动区域 (ScrollArea) ✅

3. **导航组件** [部分完成]
   - 标签页 (Tabs) ✅
   - 导航菜单 (NavigationMenu) ✅
   - 侧边栏 (Sidebar) - 待实现
   - 面包屑 (Breadcrumb) - 待实现

4. **反馈组件** [部分完成]
   - 提示 (Toast) ✅
   - 对话框 (Dialog) ✅
   - 弹出框 (Popover) ✅
   - 工具提示 (Tooltip) ✅
   - 加载指示器 (LoadingIndicator) ✅

### 第三阶段：高级组件迁移 (3-4周)

1. **数据展示组件** [部分完成]
   - 表格 (Table) ✅
   - 列表 (List) ✅
   - 树形控件 (Tree) ✅
   - 图表 (Charts) ✅
   - 日期选择器 (DatePicker)

2. **编辑器组件**
   - 代码编辑器 (CodeEditor)
   - Markdown 编辑器
   - 富文本编辑器

3. **专业组件**
   - 流程图 (FlowChart)
   - 拖拽界面 (DragAndDrop)
   - 分割面板 (ResizablePanels)
   - 聊天界面 (ChatInterface)

4. **集成 Tauri 特性**
   - 文件系统访问
   - 系统通知
   - 系统菜单
   - 全局快捷键

### 第四阶段：优化和测试 (2-3周) [部分完成]

1. **性能优化**
   - 组件懒加载
   - 代码分割
   - 资源优化
   - 减少不必要的渲染

2. **可访问性优化**
   - 键盘导航
   - 屏幕阅读器支持
   - 高对比度主题
   - WAI-ARIA 实践

3. **测试** [部分完成]
   - 单元测试 ✅ (为以下组件添加了测试: Button, Dialog, Toast, Tooltip, ScrollArea, NavigationMenu, LoadingIndicator, Table, List, Tree, Charts)
   - 集成测试
   - 端到端测试
   - 可访问性测试

4. **文档**
   - 组件使用文档
   - API 参考
   - 示例和最佳实践
   - 主题定制指南

## 技术栈选择

1. **核心框架** [已完成]
   - React 18+ ✅
   - TypeScript ✅
   - Tauri 2.0 ✅

2. **构建工具** [已完成]
   - Vite ✅
   - PostCSS ✅

3. **样式方案** [已完成]
   - Tailwind CSS ✅
   - CSS 变量 ✅
   - CSS Modules (必要时) ✅

4. **组件库基础** [已完成]
   - Radix UI (无样式组件) ✅
   - Headless UI (必要时)

5. **状态管理** [部分完成]
   - React Context ✅
   - Zustand (复杂状态)

6. **表单处理**
   - React Hook Form
   - Zod

7. **其他工具** [已完成]
   - date-fns (日期处理)
   - clsx/tailwind-merge (条件类名) ✅
   - Lucide React (图标) ✅

## 迁移注意事项

1. **API 兼容性**
   - 尽可能保持与现有 API 的兼容性，减少迁移成本
   - 在无法兼容的情况下，提供明确的迁移指南

2. **样式一致性**
   - 确保迁移后的组件视觉效果与原有一致
   - 建立设计令牌系统，确保跨组件的一致性

3. **性能考量**
   - 避免不必要的渲染
   - 优化大型列表和表格
   - 考虑代码分割和懒加载

4. **Tauri 特性利用**
   - 充分利用 Tauri 提供的原生功能
   - 考虑桌面端特有的交互模式

## 时间线

- **第一阶段**：1-2周 [已完成]
- **第二阶段**：2-3周 [进行中]
- **第三阶段**：3-4周
- **第四阶段**：2-3周 [部分完成]

总计：8-12周

## 后续计划

1. 持续集成和部署流程建设
2. 组件库版本管理策略
3. 设计系统文档和规范
4. 性能监控和优化机制

## 结论

通过将 `lumosai_ui` 和 `playground-ui` 的最佳实践整合到新的 `lumosai-nui-new` 项目中，我们将创建一个更加统一、高效且现代化的 UI 框架。这个迁移不仅解决了现有项目中的问题，还将为未来的发展提供更好的基础。 