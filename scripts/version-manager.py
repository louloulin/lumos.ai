#!/usr/bin/env python3
"""
LumosAI 版本管理工具
用于管理多包工作空间的版本同步
"""

import os
import re
import sys
import json
import argparse
import subprocess
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass

@dataclass
class PackageInfo:
    """包信息"""
    name: str
    path: Path
    version: str
    dependencies: List[str]

class VersionManager:
    """版本管理器"""
    
    def __init__(self, workspace_root: Path):
        self.workspace_root = workspace_root
        self.packages: Dict[str, PackageInfo] = {}
        self._discover_packages()
    
    def _discover_packages(self):
        """发现工作空间中的所有包"""
        # 读取工作空间配置
        workspace_toml = self.workspace_root / "Cargo.toml"
        if not workspace_toml.exists():
            raise FileNotFoundError("未找到工作空间 Cargo.toml 文件")
        
        # 解析工作空间成员
        with open(workspace_toml, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 简单的 TOML 解析（查找 members）
        members_match = re.search(r'members\s*=\s*\[(.*?)\]', content, re.DOTALL)
        if not members_match:
            raise ValueError("未找到工作空间成员配置")
        
        members_str = members_match.group(1)
        members = [m.strip().strip('"\'') for m in members_str.split(',') if m.strip()]
        
        # 添加根包
        self._add_package(self.workspace_root)
        
        # 添加成员包
        for member in members:
            if member.startswith('#'):  # 跳过注释行
                continue
            member_path = self.workspace_root / member
            if member_path.exists():
                self._add_package(member_path)
    
    def _add_package(self, package_path: Path):
        """添加包到管理列表"""
        cargo_toml = package_path / "Cargo.toml"
        if not cargo_toml.exists():
            return
        
        with open(cargo_toml, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 解析包名和版本
        name_match = re.search(r'name\s*=\s*["\']([^"\']+)["\']', content)
        version_match = re.search(r'version\s*=\s*["\']([^"\']+)["\']', content)
        
        if not name_match or not version_match:
            return
        
        name = name_match.group(1)
        version = version_match.group(1)
        
        # 查找内部依赖
        dependencies = []
        dep_pattern = r'(lumosai_\w+|lumos_\w+)\s*=\s*\{[^}]*path\s*='
        for match in re.finditer(dep_pattern, content):
            dependencies.append(match.group(1))
        
        self.packages[name] = PackageInfo(
            name=name,
            path=package_path,
            version=version,
            dependencies=dependencies
        )
    
    def get_current_versions(self) -> Dict[str, str]:
        """获取所有包的当前版本"""
        return {name: pkg.version for name, pkg in self.packages.items()}
    
    def update_version(self, package_name: str, new_version: str) -> bool:
        """更新指定包的版本"""
        if package_name not in self.packages:
            print(f"错误: 未找到包 '{package_name}'")
            return False
        
        package = self.packages[package_name]
        cargo_toml = package.path / "Cargo.toml"
        
        # 读取文件内容
        with open(cargo_toml, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 更新版本号
        new_content = re.sub(
            r'(version\s*=\s*["\'])[^"\']+(["\'])',
            f'\\g<1>{new_version}\\g<2>',
            content,
            count=1
        )
        
        # 写回文件
        with open(cargo_toml, 'w', encoding='utf-8') as f:
            f.write(new_content)
        
        # 更新内存中的版本信息
        self.packages[package_name].version = new_version
        
        print(f"✅ 已更新 {package_name} 版本到 {new_version}")
        return True
    
    def update_all_versions(self, new_version: str) -> bool:
        """更新所有包的版本"""
        success = True
        for package_name in self.packages:
            if not self.update_version(package_name, new_version):
                success = False
        return success
    
    def check_version_consistency(self) -> Tuple[bool, List[str]]:
        """检查版本一致性"""
        versions = set(pkg.version for pkg in self.packages.values())
        
        if len(versions) == 1:
            return True, []
        
        # 收集不一致的包
        inconsistent = []
        for name, pkg in self.packages.items():
            inconsistent.append(f"{name}: {pkg.version}")
        
        return False, inconsistent
    
    def get_dependency_order(self) -> List[str]:
        """获取依赖顺序（用于发布）"""
        # 简单的拓扑排序
        visited = set()
        order = []
        
        def visit(package_name: str):
            if package_name in visited or package_name not in self.packages:
                return
            
            visited.add(package_name)
            
            # 先访问依赖
            for dep in self.packages[package_name].dependencies:
                visit(dep)
            
            order.append(package_name)
        
        # 访问所有包
        for package_name in self.packages:
            visit(package_name)
        
        return order
    
    def validate_version(self, version: str) -> bool:
        """验证版本格式"""
        pattern = r'^\d+\.\d+\.\d+(-[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?$'
        return bool(re.match(pattern, version))
    
    def bump_version(self, bump_type: str, current_version: str) -> str:
        """递增版本号"""
        parts = current_version.split('-')[0].split('.')
        major, minor, patch = map(int, parts)
        
        if bump_type == 'major':
            return f"{major + 1}.0.0"
        elif bump_type == 'minor':
            return f"{major}.{minor + 1}.0"
        elif bump_type == 'patch':
            return f"{major}.{minor}.{patch + 1}"
        else:
            raise ValueError(f"无效的版本递增类型: {bump_type}")
    
    def generate_release_notes(self, version: str) -> str:
        """生成发布说明"""
        try:
            # 获取上一个标签
            result = subprocess.run(
                ['git', 'describe', '--tags', '--abbrev=0'],
                capture_output=True,
                text=True,
                cwd=self.workspace_root
            )
            
            if result.returncode == 0:
                last_tag = result.stdout.strip()
                # 获取提交日志
                log_result = subprocess.run(
                    ['git', 'log', f'{last_tag}..HEAD', '--oneline'],
                    capture_output=True,
                    text=True,
                    cwd=self.workspace_root
                )
                
                if log_result.returncode == 0:
                    commits = log_result.stdout.strip().split('\n')
                    
                    notes = f"# Release {version}\n\n"
                    notes += f"## Changes since {last_tag}\n\n"
                    
                    for commit in commits:
                        if commit.strip():
                            notes += f"- {commit}\n"
                    
                    return notes
            
        except Exception as e:
            print(f"警告: 无法生成发布说明: {e}")
        
        return f"# Release {version}\n\n## Changes\n\n- 版本更新到 {version}\n"

def main():
    parser = argparse.ArgumentParser(description='LumosAI 版本管理工具')
    parser.add_argument('--workspace', type=Path, default=Path.cwd(),
                       help='工作空间根目录')
    
    subparsers = parser.add_subparsers(dest='command', help='可用命令')
    
    # 显示版本命令
    subparsers.add_parser('show', help='显示所有包的版本')
    
    # 检查一致性命令
    subparsers.add_parser('check', help='检查版本一致性')
    
    # 更新版本命令
    update_parser = subparsers.add_parser('update', help='更新版本')
    update_parser.add_argument('version', help='新版本号')
    update_parser.add_argument('--package', help='指定包名（默认更新所有包）')
    
    # 递增版本命令
    bump_parser = subparsers.add_parser('bump', help='递增版本')
    bump_parser.add_argument('type', choices=['major', 'minor', 'patch'],
                           help='递增类型')
    
    # 发布顺序命令
    subparsers.add_parser('order', help='显示发布顺序')
    
    # 生成发布说明命令
    notes_parser = subparsers.add_parser('notes', help='生成发布说明')
    notes_parser.add_argument('version', help='版本号')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    try:
        vm = VersionManager(args.workspace)
        
        if args.command == 'show':
            print("📦 包版本信息:")
            for name, version in vm.get_current_versions().items():
                print(f"  {name}: {version}")
        
        elif args.command == 'check':
            consistent, issues = vm.check_version_consistency()
            if consistent:
                print("✅ 所有包版本一致")
            else:
                print("❌ 版本不一致:")
                for issue in issues:
                    print(f"  {issue}")
        
        elif args.command == 'update':
            if not vm.validate_version(args.version):
                print(f"错误: 无效的版本格式 '{args.version}'")
                return
            
            if args.package:
                vm.update_version(args.package, args.version)
            else:
                vm.update_all_versions(args.version)
        
        elif args.command == 'bump':
            # 获取当前版本（使用第一个包的版本）
            current_versions = vm.get_current_versions()
            if not current_versions:
                print("错误: 未找到任何包")
                return
            
            current_version = list(current_versions.values())[0]
            new_version = vm.bump_version(args.type, current_version)
            
            print(f"递增版本: {current_version} -> {new_version}")
            vm.update_all_versions(new_version)
        
        elif args.command == 'order':
            order = vm.get_dependency_order()
            print("📋 发布顺序:")
            for i, package in enumerate(order, 1):
                print(f"  {i}. {package}")
        
        elif args.command == 'notes':
            notes = vm.generate_release_notes(args.version)
            print(notes)
    
    except Exception as e:
        print(f"错误: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
