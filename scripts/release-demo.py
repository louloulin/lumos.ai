#!/usr/bin/env python3
"""
LumosAI 发布演示脚本
演示发布系统的各个功能
"""

import os
import sys
import subprocess
from pathlib import Path

def run_command(cmd, description):
    """运行命令并显示结果"""
    print(f"\n🔄 {description}")
    print(f"命令: {' '.join(cmd)}")
    print("-" * 50)
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        
        if result.returncode == 0:
            print(f"✅ 成功")
            if result.stdout.strip():
                print(f"输出:\n{result.stdout}")
        else:
            print(f"❌ 失败 (退出码: {result.returncode})")
            if result.stderr.strip():
                print(f"错误:\n{result.stderr}")
        
        return result.returncode == 0
    
    except subprocess.TimeoutExpired:
        print("⏰ 命令超时")
        return False
    except Exception as e:
        print(f"💥 异常: {e}")
        return False

def demo_version_management():
    """演示版本管理功能"""
    print("\n" + "=" * 60)
    print("📦 版本管理演示")
    print("=" * 60)
    
    # 显示当前版本
    run_command(
        ["python", "scripts/version-manager.py", "show"],
        "显示所有包的当前版本"
    )
    
    # 检查版本一致性
    run_command(
        ["python", "scripts/version-manager.py", "check"],
        "检查版本一致性"
    )
    
    # 显示发布顺序
    run_command(
        ["python", "scripts/version-manager.py", "order"],
        "显示包的发布顺序"
    )

def demo_build_system():
    """演示构建系统"""
    print("\n" + "=" * 60)
    print("🔨 构建系统演示")
    print("=" * 60)
    
    # 检查代码格式
    run_command(
        ["cargo", "fmt", "--check"],
        "检查代码格式"
    )
    
    # 运行 Clippy 检查
    run_command(
        ["cargo", "clippy", "--workspace", "--", "-D", "warnings"],
        "运行 Clippy 代码检查"
    )
    
    # 构建项目
    run_command(
        ["cargo", "build", "--workspace"],
        "构建整个工作空间"
    )

def demo_testing():
    """演示测试系统"""
    print("\n" + "=" * 60)
    print("🧪 测试系统演示")
    print("=" * 60)
    
    # 运行单元测试
    run_command(
        ["cargo", "test", "--workspace", "--lib"],
        "运行单元测试"
    )
    
    # 构建文档
    run_command(
        ["cargo", "doc", "--workspace", "--no-deps"],
        "构建 API 文档"
    )
    
    # 构建示例
    run_command(
        ["cargo", "build", "--examples"],
        "构建示例项目"
    )

def demo_release_preparation():
    """演示发布准备"""
    print("\n" + "=" * 60)
    print("🚀 发布准备演示")
    print("=" * 60)
    
    # 显示项目信息
    print("\n📊 项目统计信息:")
    
    # 统计 Rust 文件
    rust_files = list(Path(".").rglob("*.rs"))
    rust_files = [f for f in rust_files if "target" not in str(f)]
    print(f"  • Rust 源文件: {len(rust_files)} 个")
    
    # 统计代码行数
    total_lines = 0
    for file in rust_files:
        try:
            with open(file, 'r', encoding='utf-8') as f:
                lines = len([line for line in f if line.strip() and not line.strip().startswith('//')])
                total_lines += lines
        except:
            continue
    
    print(f"  • 代码行数: {total_lines:,} 行")
    
    # 统计包数量
    cargo_files = list(Path(".").rglob("Cargo.toml"))
    cargo_files = [f for f in cargo_files if "target" not in str(f)]
    print(f"  • 包数量: {len(cargo_files)} 个")
    
    # 显示工作空间成员
    print(f"\n📦 工作空间成员:")
    workspace_members = [
        "lumosai_core", "lumosai_cli", "lumosai_evals", "lumosai_rag",
        "lumosai_network", "lumosai_vector", "lumos_macro", "lumosai_mcp",
        "lumosai_marketplace", "lumosai_examples", "lumosai_derive",
        "lumosai_enterprise", "lumosai_bindings"
    ]
    
    for i, member in enumerate(workspace_members, 1):
        print(f"  {i:2d}. {member}")

def demo_release_features():
    """演示发布功能特性"""
    print("\n" + "=" * 60)
    print("✨ 发布功能特性")
    print("=" * 60)
    
    features = [
        "🔄 自动化版本管理 - 统一管理多包版本",
        "🧪 全面质量检查 - 代码格式、测试、文档",
        "🔨 多平台构建 - Linux、macOS、Windows 支持",
        "📦 多渠道发布 - GitHub、crates.io、文档站点",
        "🔒 安全审计 - 依赖漏洞检查",
        "📊 质量报告 - 覆盖率、性能、代码质量",
        "🚀 CI/CD 集成 - GitHub Actions 自动化",
        "📢 发布通知 - Slack、Discord、邮件通知",
        "📚 文档生成 - API 文档自动更新",
        "🔄 回滚支持 - 快速回滚机制",
    ]
    
    for feature in features:
        print(f"  • {feature}")

def demo_release_workflow():
    """演示发布工作流"""
    print("\n" + "=" * 60)
    print("🔄 发布工作流演示")
    print("=" * 60)
    
    workflow_steps = [
        ("1️⃣", "发布前检查", "运行 scripts/pre-release-check.sh"),
        ("2️⃣", "版本更新", "python scripts/version-manager.py bump patch"),
        ("3️⃣", "质量检查", "python scripts/quality-check.py"),
        ("4️⃣", "构建发布", "cargo build --release --workspace"),
        ("5️⃣", "运行测试", "cargo test --workspace"),
        ("6️⃣", "创建标签", "git tag v0.1.1"),
        ("7️⃣", "推送标签", "git push origin v0.1.1"),
        ("8️⃣", "自动发布", "GitHub Actions 自动触发"),
        ("9️⃣", "发布通知", "运行 scripts/post-release-notify.sh"),
    ]
    
    print("\n发布工作流步骤:")
    for step, title, command in workflow_steps:
        print(f"  {step} {title}")
        print(f"     命令: {command}")
        print()

def show_release_files():
    """显示发布相关文件"""
    print("\n" + "=" * 60)
    print("📁 发布系统文件")
    print("=" * 60)
    
    release_files = [
        ("scripts/release.sh", "主发布脚本"),
        ("scripts/version-manager.py", "版本管理工具"),
        ("scripts/pre-release-check.sh", "发布前检查"),
        ("scripts/post-release-notify.sh", "发布后通知"),
        ("scripts/quality-check.py", "质量检查工具"),
        ("scripts/update-release-config.py", "配置更新工具"),
        (".github/workflows/ci.yml", "CI 工作流"),
        (".github/workflows/release.yml", "发布工作流"),
        ("release.toml", "发布配置文件"),
        ("deny.toml", "依赖检查配置"),
        ("CHANGELOG.md", "变更日志"),
        ("docs/RELEASE_GUIDE.md", "发布指南"),
    ]
    
    print("\n发布系统包含以下文件:")
    for file_path, description in release_files:
        status = "✅" if Path(file_path).exists() else "❌"
        print(f"  {status} {file_path:<35} - {description}")

def main():
    """主函数"""
    print("🎉 LumosAI 发布系统演示")
    print("=" * 60)
    print("这个演示将展示 LumosAI 发布系统的各个功能")
    
    # 检查是否在正确的目录
    if not Path("Cargo.toml").exists():
        print("❌ 错误: 请在项目根目录运行此脚本")
        sys.exit(1)
    
    try:
        # 演示各个功能
        demo_version_management()
        demo_build_system()
        demo_testing()
        demo_release_preparation()
        demo_release_features()
        demo_release_workflow()
        show_release_files()
        
        print("\n" + "=" * 60)
        print("🎊 发布系统演示完成！")
        print("=" * 60)
        
        print("\n📚 下一步:")
        print("  1. 查看发布指南: docs/RELEASE_GUIDE.md")
        print("  2. 运行发布前检查: scripts/pre-release-check.sh")
        print("  3. 执行发布: scripts/release.sh patch")
        print("  4. 查看 GitHub Actions 工作流")
        
        print("\n🔗 相关链接:")
        print("  • GitHub: https://github.com/lumosai/lumosai")
        print("  • 文档: https://docs.rs/lumosai")
        print("  • Crates.io: https://crates.io/crates/lumosai")
        
    except KeyboardInterrupt:
        print("\n\n⏹️  演示被用户中断")
    except Exception as e:
        print(f"\n\n💥 演示过程中发生错误: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
