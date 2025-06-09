# LumosAI 发布指南

本文档描述了 LumosAI 项目的完整发布流程，包括版本管理、构建、测试、发布和通知等各个环节。

## 📋 目录

- [发布概述](#发布概述)
- [发布准备](#发布准备)
- [版本管理](#版本管理)
- [发布流程](#发布流程)
- [自动化发布](#自动化发布)
- [手动发布](#手动发布)
- [发布后操作](#发布后操作)
- [回滚流程](#回滚流程)
- [故障排除](#故障排除)

## 🎯 发布概述

LumosAI 采用语义化版本控制和自动化发布流程，支持：

- **自动化 CI/CD**: GitHub Actions 驱动的完整发布流程
- **多平台构建**: Linux、macOS、Windows 平台支持
- **多渠道发布**: GitHub Releases、crates.io、文档站点
- **质量保证**: 全面的测试、检查和验证
- **通知系统**: Slack、Discord、邮件等多渠道通知

## 🔧 发布准备

### 环境要求

确保开发环境满足以下要求：

```bash
# Rust 工具链
rustc --version  # >= 1.70.0
cargo --version

# Git 配置
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# 必要工具
cargo install cargo-audit
cargo install cargo-outdated
cargo install cargo-tarpaulin
```

### 权限配置

确保具有以下权限：

- **GitHub**: 仓库写权限，可创建 Release
- **Crates.io**: 包发布权限
- **文档站点**: 部署权限

### 环境变量

配置必要的环境变量：

```bash
# GitHub Token (用于创建 Release)
export GITHUB_TOKEN="your_github_token"

# Crates.io Token (用于发布包)
export CARGO_REGISTRY_TOKEN="your_crates_token"

# 通知配置 (可选)
export SLACK_WEBHOOK_URL="your_slack_webhook"
export DISCORD_WEBHOOK_URL="your_discord_webhook"
export EMAIL_RECIPIENTS="team@lumosai.dev"
```

## 📊 版本管理

### 语义化版本

LumosAI 遵循 [语义化版本](https://semver.org/lang/zh-CN/) 规范：

- **主版本号 (MAJOR)**: 不兼容的 API 变更
- **次版本号 (MINOR)**: 向后兼容的功能新增  
- **修订号 (PATCH)**: 向后兼容的问题修复

### 版本管理工具

使用内置的版本管理工具：

```bash
# 查看当前版本
python3 scripts/version-manager.py show

# 检查版本一致性
python3 scripts/version-manager.py check

# 更新版本
python3 scripts/version-manager.py update 1.2.3

# 递增版本
python3 scripts/version-manager.py bump patch  # 1.0.0 -> 1.0.1
python3 scripts/version-manager.py bump minor  # 1.0.0 -> 1.1.0
python3 scripts/version-manager.py bump major  # 1.0.0 -> 2.0.0
```

## 🚀 发布流程

### 1. 发布前检查

运行发布前检查脚本：

```bash
chmod +x scripts/pre-release-check.sh
./scripts/pre-release-check.sh
```

检查项目包括：
- Git 状态和分支
- 版本一致性
- 代码格式和 Clippy
- 测试套件
- 文档构建
- 安全审计
- 依赖检查
- 发布构建

### 2. 准备发布

```bash
# 1. 确保在正确的分支
git checkout main
git pull origin main

# 2. 更新版本号
python3 scripts/version-manager.py bump minor

# 3. 更新变更日志
# 编辑 CHANGELOG.md，添加新版本的变更记录

# 4. 提交版本更改
git add .
git commit -m "chore: prepare release v1.1.0"
```

### 3. 创建发布

```bash
# 使用发布脚本
chmod +x scripts/release.sh
./scripts/release.sh minor  # 或 patch、major、1.2.3
```

发布脚本会自动：
- 运行所有检查
- 更新版本号
- 构建发布版本
- 创建 Git 标签
- 发布到 crates.io (可选)

## 🤖 自动化发布

### GitHub Actions 发布

推送标签触发自动发布：

```bash
# 创建并推送标签
git tag v1.1.0
git push origin v1.1.0
```

或使用 GitHub 界面手动触发：

1. 访问 GitHub Actions 页面
2. 选择 "Release" 工作流
3. 点击 "Run workflow"
4. 输入版本号并运行

### 发布工作流

自动发布工作流包括：

1. **验证**: 检查版本格式和发布条件
2. **CI 检查**: 运行完整的 CI 测试套件
3. **构建**: 多平台构建发布产物
4. **发布**: 发布到 GitHub 和 crates.io
5. **文档**: 更新 API 文档
6. **通知**: 发送发布通知

## 🔨 手动发布

### 发布到 crates.io

```bash
# 登录 crates.io
cargo login

# 按依赖顺序发布包
cargo publish --package lumos_macro
cargo publish --package lumosai_core
cargo publish --package lumosai_vector
cargo publish --package lumosai_evals
cargo publish --package lumosai_rag
cargo publish --package lumosai_network
cargo publish --package lumosai_cli
cargo publish  # 主包
```

### 创建 GitHub Release

```bash
# 使用 GitHub CLI
gh release create v1.1.0 \
    --title "LumosAI v1.1.0" \
    --notes-file CHANGELOG.md \
    target/release/lumosai-*
```

### 更新文档

```bash
# 构建并部署文档
cargo doc --all-features --workspace --no-deps
# 部署到文档站点
```

## 📢 发布后操作

### 自动通知

发布完成后自动运行通知脚本：

```bash
chmod +x scripts/post-release-notify.sh
./scripts/post-release-notify.sh
```

通知包括：
- Slack/Discord 消息
- 邮件通知
- 社交媒体内容准备
- 包管理器更新

### 手动操作

1. **社交媒体**: 发布准备好的 Twitter/LinkedIn 内容
2. **社区通知**: 在相关论坛和社区发布公告
3. **博客文章**: 撰写发布博客文章
4. **用户文档**: 更新用户指南和教程

## 🔄 回滚流程

### 快速回滚

如果发现严重问题，可以快速回滚：

```bash
# 1. 删除有问题的标签
git tag -d v1.1.0
git push origin :refs/tags/v1.1.0

# 2. 从 crates.io 撤回 (仅限 72 小时内)
cargo yank --version 1.1.0

# 3. 删除 GitHub Release
gh release delete v1.1.0
```

### 修复发布

```bash
# 1. 修复问题
git checkout main
# 进行必要的修复

# 2. 发布补丁版本
./scripts/release.sh patch

# 3. 取消撤回
cargo unyank --version 1.1.0  # 如果之前撤回了
```

## 🔍 故障排除

### 常见问题

**1. 版本不一致**
```bash
# 检查并修复版本不一致
python3 scripts/version-manager.py check
python3 scripts/version-manager.py update 1.1.0
```

**2. 测试失败**
```bash
# 运行特定测试
cargo test --package lumosai_core
cargo test --test integration_tests
```

**3. 构建失败**
```bash
# 清理并重新构建
cargo clean
cargo build --release --all-features
```

**4. 发布权限问题**
```bash
# 检查 crates.io 权限
cargo owner --list lumosai

# 重新登录
cargo login
```

### 调试工具

```bash
# 检查发布状态
cargo search lumosai

# 查看包信息
cargo info lumosai

# 检查依赖树
cargo tree

# 安全审计
cargo audit
```

## 📚 相关资源

- [语义化版本](https://semver.org/lang/zh-CN/)
- [Cargo 发布指南](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)

## 🤝 贡献

如果您发现发布流程中的问题或有改进建议，请：

1. 创建 Issue 描述问题
2. 提交 Pull Request 改进流程
3. 更新相关文档

---

*最后更新: 2024-01-XX*
