# 👥 社区资源

**获取帮助、参与讨论、贡献代码**

## 🆘 获取帮助

遇到问题？这里有多种获取帮助的方式，按响应速度排序：

### 🚀 立即获得帮助
- **📖 在线文档**: 浏览完整的技术文档
- **🔍 搜索功能**: 使用页面搜索查找相关内容
- **❓ FAQ**: 查看常见问题和解答

### 💬 社区支持（24小时内响应）
- **[GitHub Issues](https://github.com/louloulin/lumos.ai/issues)** - 报告bug和功能请求
- **[GitHub Discussions](https://github.com/louloulin/lumos.ai/discussions)** - 技术讨论和经验分享

### 📧 官方支持（工作日响应）
- **技术支持**: support@lumosai.com
- **商业合作**: business@lumosai.com

---

## ❓ 常见问题 (FAQ)

### 安装和配置

**Q: LumosAI支持哪些操作系统？**
A: 支持Linux、macOS和Windows。推荐使用Ubuntu 20.04+或macOS 12+。

**Q: 需要什么版本的Rust？**
A: 需要Rust 1.70或更高版本。运行`rustc --version`检查版本。

**Q: 如何设置API密钥？**
A: 设置环境变量`export OPENAI_API_KEY="your-key"`或参考[配置指南](../reference/configuration/README.md)。

### 使用问题

**Q: 支持哪些AI模型？**
A: 支持OpenAI GPT系列、Anthropic Claude系列、DeepSeek等。详见[模型配置](../reference/configuration/models.md)。

**Q: 如何提高Agent响应速度？**
A: 1) 使用更快的模型 2) 限制max_tokens 3) 启用流式响应 4) 优化网络连接。

**Q: Agent记忆是如何工作的？**
A: Agent自动保存对话历史，支持设置记忆长度限制和手动清理。详见[内存管理](../reference/api/memory.md)。

### 技术问题

**Q: 编译时出现SSL错误怎么办？**
A: 安装系统SSL库：Ubuntu `sudo apt install libssl-dev`，macOS `brew install openssl`。

**Q: 如何处理网络连接问题？**
A: 检查网络连接，设置代理，或使用国内API端点。详见[故障排除](../learn/getting-started/troubleshooting.md)。

**Q: 程序占用内存过多怎么办？**
A: 限制对话历史长度，定期清理内存，使用流式处理。

---

## 🐛 报告问题

### Bug报告模板

使用以下模板提交Bug报告：

```markdown
**Bug描述**
简要描述遇到的问题

**复现步骤**
1. 运行 '...'
2. 点击 '....'
3. 滚动到 '....'
4. 看到错误

**期望行为**
描述你期望发生的情况

**实际行为**
描述实际发生的情况

**环境信息**
- 操作系统: [例如 Ubuntu 20.04]
- Rust版本: [例如 1.75.0]
- LumosAI版本: [例如 0.2.0]
- 模型提供商: [例如 OpenAI]

**附加信息**
- 错误日志
- 最小复现代码
- 相关配置
```

### 功能请求模板

```markdown
**功能描述**
简要描述你希望添加的功能

**使用场景**
描述这个功能的使用场景和价值

**解决方案**
描述你期望的实现方式

**替代方案**
描述你考虑过的其他解决方案

**附加信息**
任何相关的信息或参考资料
```

**提交链接**: [新建Issue](https://github.com/louloulin/lumos.ai/issues/new/choose)

---

## 💬 参与讨论

### GitHub Discussions

我们鼓励在GitHub Discussions中进行技术讨论：

- **💡 想法和分享**: 分享使用经验和创意
- **❓ 问答**: 提出技术问题并获得社区帮助
- **📖 最佳实践**: 讨论开发模式和最佳实践
- **🎯 功能讨论**: 讨论新功能的想法和需求

### 讨论分类

| 分类 | 描述 | 适合内容 |
|------|------|----------|
| **General** | 一般讨论 | 使用经验、想法分享 |
| **Q&A** | 问答 | 技术问题、疑难解答 |
| **Ideas** | 功能想法 | 新功能建议、改进方案 |
| **Show and Tell** | 成果展示 | 项目分享、使用案例 |

**参与链接**: [GitHub Discussions](https://github.com/louloulin/lumos.ai/discussions)

---

## 🌟 社区贡献

我们欢迎各种形式的贡献！

### 贡献类型

- **🐛 Bug修复**: 发现并修复问题
- **✨ 新功能**: 开发新的功能特性
- **📖 文档改进**: 完善文档和示例
- **🧪 测试**: 增加测试覆盖率
- **🎨 示例**: 创建实用的示例代码
- **🌐 国际化**: 翻译文档和界面

### 贡献流程

1. **🍴 Fork项目**: 在GitHub上fork仓库
2. **🌿 创建分支**: `git checkout -b feature/amazing-feature`
3. **✅ 编写代码**: 实现你的想法
4. **🧪 添加测试**: 确保代码质量
5. **📝 更新文档**: 说明变更内容
6. **💾 提交代码**: `git commit -m 'Add amazing feature'`
7. **📤 推送分支**: `git push origin feature/amazing-feature`
8. **🔄 创建PR**: 提交Pull Request

**详细指南**: [贡献指南](../contribute/development.md)

---

## 🏆 贡献者认可

### 贡献者列表

感谢所有为LumosAI做出贡献的开发者！

<a href="https://github.com/louloulin/lumos.ai/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=louloulin/lumos.ai" />
</a>

### 贡献统计

- **总贡献者**: 50+
- **代码提交**: 1000+
- **解决Issues**: 200+
- **合并PR**: 300+

### 成为核心贡献者

活跃的贡献者有机会成为核心团队成员：
- 持续高质量的贡献
- 积极参与社区讨论
- 帮助新用户解决问题
- 推动项目发展方向

---

## 📚 学习资源

### 官方资源

- **[文档中心](../README.md)** - 完整的技术文档
- **[API参考](../reference/api/README.md)** - 详细的API文档
- **[示例集合](../learn/examples/README.md)** - 丰富的代码示例
- **[最佳实践](../learn/guides/best-practices.md)** - 开发经验总结

### 社区资源

- **[Awesome LumosAI](https://github.com/louloulin/awesome-lumosai)** - 社区整理的资源集合
- **[LumosAI Showcases](https://github.com/louloulin/lumosai-showcases)** - 社区项目展示
- **[用户案例](https://lumosai.com/cases)** - 实际应用案例

### 外部学习

- **[Rust学习资源](https://www.rust-lang.org/learn)** - Rust语言学习
- **[AI开发指南](https://github.com/eugeneyan/applied-ml)** - 应用机器学习
- **[异步编程指南](https://rust-lang.github.io/async-book/)** - Rust异步编程

---

## 🌐 社区平台

### 代码托管
- **GitHub仓库**: https://github.com/louloulin/lumos.ai
- **文档站点**: https://docs.lumosai.com
- **Crates.io**: https://crates.io/crates/lumosai

### 社交媒体
- **Twitter**: [@LumosAI](https://twitter.com/lumosai) - 项目动态
- **YouTube**: [LumosAI频道](https://youtube.com/lumosai) - 视频教程
- **Blog**: https://blog.lumosai.com - 技术文章

### 即时交流
- **Discord**: [邀请链接](https://discord.gg/lumosai) - 实时讨论
- **微信群**: 扫描官网二维码加入
- **QQ群**: 123456789 (LumosAI技术交流)

---

## 📋 社区准则

### 行为准则

我们致力于建立一个友善、包容的社区环境：

- **尊重他人**: 尊重不同的观点和经验
- **建设性反馈**: 提供建设性的意见和建议
- **友善沟通**: 使用友善和专业的语言
- **耐心帮助**: 耐心帮助新用户解决问题

### 禁止行为

- 恶意攻击或人身攻击
- 发布垃圾信息或无关内容
- 侵犯他人知识产权
- 违反法律法规的行为

**详细准则**: [行为准则](../contribute/code-of-conduct.md)

---

## 🎯 路线图参与

### 功能规划

我们透明地管理项目路线图，欢迎社区参与：

- **季度规划**: 每季度发布功能规划
- **优先级投票**: 社区投票决定功能优先级
- **RFC讨论**: 重要功能变更的RFC讨论
- **Beta测试**: 新功能的社区测试

### 如何参与

1. **关注路线图**: 查看[项目路线图](https://github.com/louloulin/lumosai/projects)
2. **参与讨论**: 在GitHub Issues中参与讨论
3. **测试新功能**: 加入Beta测试计划
4. **提供反馈**: 分享使用体验和改进建议

---

## 📞 联系我们

### 商务合作

- **邮箱**: business@lumosai.com
- **微信**: lumosai-business
- **电话**: +86-xxx-xxxx-xxxx

### 媒体合作

- **邮箱**: media@lumosai.com
- **联系人**: 市场部

### 技术支持

- **邮箱**: support@lumosai.com
- **响应时间**: 工作日24小时内

---

**🚀 加入LumosAI社区，一起构建更好的AI应用开发体验！**

*[← 返回文档首页](../README.md)*