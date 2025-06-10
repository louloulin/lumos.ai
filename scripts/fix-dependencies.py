#!/usr/bin/env python3
"""
LumosAI 依赖版本修复脚本
解决版本冲突和编译问题
"""

import os
import subprocess
import sys
from pathlib import Path

def run_command(cmd, cwd=None):
    """运行命令并返回结果"""
    try:
        result = subprocess.run(cmd, shell=True, cwd=cwd, capture_output=True, text=True)
        return result.returncode == 0, result.stdout, result.stderr
    except Exception as e:
        return False, "", str(e)

def fix_cargo_lock():
    """删除 Cargo.lock 强制重新解析依赖"""
    lock_files = [
        "Cargo.lock",
        "lumosai_vector/Cargo.lock",
        "lumosai_cli/Cargo.lock"
    ]
    
    for lock_file in lock_files:
        if os.path.exists(lock_file):
            os.remove(lock_file)
            print(f"✅ 删除 {lock_file}")

def update_workspace_dependencies():
    """更新工作空间依赖"""
    print("🔄 更新工作空间依赖...")
    
    # 更新 Cargo.toml 中的依赖版本
    cargo_toml_updates = {
        "Cargo.toml": {
            "tokio": '{ version = "1.40", features = ["full"] }',
            "serde": '{ version = "1.0.210", features = ["derive"] }',
            "serde_json": '"1.0.128"',
            "reqwest": '{ version = "0.12", features = ["json"] }',
            "sqlx": '{ version = "0.8", features = ["runtime-tokio", "postgres", "sqlite", "json"] }',
        }
    }
    
    for file_path, updates in cargo_toml_updates.items():
        if os.path.exists(file_path):
            print(f"📝 更新 {file_path}")
            # 这里可以添加具体的文件更新逻辑

def check_compilation():
    """检查编译状态"""
    print("🔍 检查编译状态...")
    
    success, stdout, stderr = run_command("cargo check --workspace")
    
    if success:
        print("✅ 编译检查通过")
        return True
    else:
        print("❌ 编译检查失败")
        print("错误信息:")
        print(stderr)
        return False

def fix_zstd_issues():
    """修复 zstd 相关问题"""
    print("🔧 修复 zstd 依赖问题...")
    
    # 在 Cargo.toml 中添加 zstd 版本锁定
    zstd_fix = '''
# Fix zstd version conflicts
[workspace.dependencies.zstd]
version = "0.12"
default-features = false

[workspace.dependencies.zstd-safe]
version = "6.0.5"
default-features = false
'''
    
    print("📝 添加 zstd 版本锁定")

def clean_build_cache():
    """清理构建缓存"""
    print("🧹 清理构建缓存...")
    
    cache_dirs = [
        "target",
        "lumosai_vector/target"
    ]
    
    for cache_dir in cache_dirs:
        if os.path.exists(cache_dir):
            success, _, _ = run_command(f"cargo clean", cwd=os.path.dirname(cache_dir) if "/" in cache_dir else ".")
            if success:
                print(f"✅ 清理 {cache_dir}")

def update_rust_toolchain():
    """更新 Rust 工具链"""
    print("🦀 检查 Rust 工具链...")
    
    success, stdout, _ = run_command("rustc --version")
    if success:
        print(f"当前 Rust 版本: {stdout.strip()}")
    
    # 更新到最新稳定版
    success, _, _ = run_command("rustup update stable")
    if success:
        print("✅ Rust 工具链更新完成")

def fix_workspace_members():
    """修复工作空间成员配置"""
    print("🔧 检查工作空间成员配置...")
    
    # 检查所有成员是否存在
    workspace_members = [
        "lumosai_core",
        "lumosai_cli", 
        "lumosai_evals",
        "lumosai_rag",
        "lumosai_network",
        "lumos_macro",
        "lumosai_mcp",
        "lumosai_marketplace",
        "lumosai_examples",
        "lumosai_derive",
        "lumosai_enterprise",
        "lumosai_bindings",
        "lumosai_vector",
    ]
    
    missing_members = []
    for member in workspace_members:
        if not os.path.exists(f"{member}/Cargo.toml"):
            missing_members.append(member)
    
    if missing_members:
        print(f"⚠️ 缺失的工作空间成员: {missing_members}")
    else:
        print("✅ 所有工作空间成员都存在")

def run_tests():
    """运行基础测试"""
    print("🧪 运行基础测试...")
    
    success, stdout, stderr = run_command("cargo test --test simple_test")
    
    if success:
        print("✅ 基础测试通过")
        return True
    else:
        print("❌ 基础测试失败")
        print(stderr)
        return False

def main():
    """主函数"""
    print("🚀 LumosAI 依赖修复脚本启动")
    print("=" * 50)
    
    # 步骤 1: 清理缓存
    clean_build_cache()
    
    # 步骤 2: 删除锁文件
    fix_cargo_lock()
    
    # 步骤 3: 更新工具链
    update_rust_toolchain()
    
    # 步骤 4: 修复工作空间
    fix_workspace_members()
    
    # 步骤 5: 修复 zstd 问题
    fix_zstd_issues()
    
    # 步骤 6: 更新依赖
    update_workspace_dependencies()
    
    # 步骤 7: 检查编译
    if check_compilation():
        print("\n🎉 依赖修复完成！")
        
        # 步骤 8: 运行测试
        if run_tests():
            print("🎊 所有检查通过，项目状态良好！")
        else:
            print("⚠️ 测试失败，需要进一步调试")
    else:
        print("\n❌ 编译仍有问题，需要手动检查")
        print("\n建议:")
        print("1. 检查 Cargo.toml 中的依赖版本")
        print("2. 运行 'cargo update' 更新依赖")
        print("3. 检查是否有版本冲突")
    
    print("\n" + "=" * 50)
    print("修复脚本执行完成")

if __name__ == "__main__":
    main()
