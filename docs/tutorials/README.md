# LumosAI 渐进式教程

## 🎯 学习路径

本教程系列将带您从零开始，逐步掌握LumosAI的所有功能。每个教程都建立在前一个的基础上，形成完整的学习路径。

## 📚 教程结构

### 🌱 初级教程 (Beginner)
适合完全没有LumosAI经验的开发者

1. **[Hello World](./beginner/01-hello-world.md)** (5分钟)
   - 安装LumosAI
   - 创建第一个Agent
   - 生成第一个响应

2. **[快速API入门](./beginner/02-quick-api.md)** (10分钟)
   - 使用`Agent::quick()`
   - 添加工具
   - 基础配置

3. **[工具使用基础](./beginner/03-basic-tools.md)** (15分钟)
   - 内置工具库
   - 工具调用
   - 工具组合

4. **[内存管理入门](./beginner/04-memory-basics.md)** (15分钟)
   - 对话记忆
   - 上下文管理
   - 记忆配置

### 🚀 中级教程 (Intermediate)
适合有基础经验，想要深入学习的开发者

5. **[构建器模式详解](./intermediate/05-builder-pattern.md)** (20分钟)
   - AgentBuilder详解
   - 高级配置选项
   - 最佳实践

6. **[自定义工具开发](./intermediate/06-custom-tools.md)** (30分钟)
   - 工具trait实现
   - 参数验证
   - 异步工具

7. **[工作流基础](./intermediate/07-workflow-basics.md)** (25分钟)
   - 工作流概念
   - 步骤定义
   - 条件执行

8. **[DSL宏使用](./intermediate/08-dsl-macros.md)** (30分钟)
   - agent!宏
   - workflow!宏
   - tools!宏

### 🎓 高级教程 (Advanced)
适合有丰富经验，需要掌握高级功能的开发者

9. **[复杂工作流设计](./advanced/09-complex-workflows.md)** (45分钟)
   - 并行执行
   - 条件分支
   - 错误处理

10. **[多Agent协作](./advanced/10-multi-agent.md)** (40分钟)
    - Agent间通信
    - 任务分配
    - 协作模式

11. **[自定义LLM集成](./advanced/11-custom-llm.md)** (35分钟)
    - LlmProvider trait
    - 自定义适配器
    - 性能优化

12. **[向量存储和RAG](./advanced/12-vector-rag.md)** (50分钟)
    - 向量存储配置
    - RAG管道
    - 知识库构建

### 🏭 生产级教程 (Production)
适合准备部署到生产环境的开发者

13. **[Web服务集成](./production/13-web-service.md)** (60分钟)
    - HTTP API封装
    - 异步处理
    - 错误处理

14. **[微服务架构](./production/14-microservice.md)** (75分钟)
    - 服务拆分
    - 通信协议
    - 负载均衡

15. **[监控和日志](./production/15-monitoring.md)** (45分钟)
    - 性能监控
    - 日志记录
    - 错误追踪

16. **[部署和运维](./production/16-deployment.md)** (60分钟)
    - 容器化部署
    - 配置管理
    - 扩容策略

## 🎯 学习建议

### 按顺序学习
建议按照编号顺序学习，每个教程都建立在前面的基础上。

### 动手实践
每个教程都包含完整的代码示例，建议跟着动手实践。

### 循序渐进
不要急于跳到高级教程，扎实的基础很重要。

### 参考文档
遇到问题时，参考[API文档](../api-reference/)和[最佳实践](../best-practices/)。

## 📊 学习进度追踪

- [ ] 初级教程 (1-4)
  - [ ] Hello World
  - [ ] 快速API入门
  - [ ] 工具使用基础
  - [ ] 内存管理入门

- [ ] 中级教程 (5-8)
  - [ ] 构建器模式详解
  - [ ] 自定义工具开发
  - [ ] 工作流基础
  - [ ] DSL宏使用

- [ ] 高级教程 (9-12)
  - [ ] 复杂工作流设计
  - [ ] 多Agent协作
  - [ ] 自定义LLM集成
  - [ ] 向量存储和RAG

- [ ] 生产级教程 (13-16)
  - [ ] Web服务集成
  - [ ] 微服务架构
  - [ ] 监控和日志
  - [ ] 部署和运维

## 🔗 相关资源

- [API选择指南](../api-choice-guide.md)
- [示例代码库](../../examples/)
- [最佳实践](../best-practices/)
- [常见问题](../faq.md)
- [社区论坛](https://github.com/lumosai/community)

## 💡 学习小贴士

1. **设置开发环境**: 确保Rust和相关工具已正确安装
2. **准备API密钥**: 某些教程需要LLM API密钥
3. **创建练习项目**: 为每个教程创建独立的练习项目
4. **记录学习笔记**: 记录重要概念和个人理解
5. **参与社区讨论**: 在社区中分享经验和提问

## 🆘 获取帮助

如果在学习过程中遇到问题：

1. 查看[常见问题](../faq.md)
2. 搜索[GitHub Issues](https://github.com/lumosai/lumos.ai/issues)
3. 在[社区论坛](https://github.com/lumosai/community)提问
4. 查看[示例代码](../../examples/)寻找类似实现

开始您的LumosAI学习之旅吧！🚀
