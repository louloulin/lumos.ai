#!/usr/bin/env python3
"""
LumosAI 发布配置更新工具
用于更新 Cargo.toml 中的发布相关配置
"""

import os
import re
import sys
import toml
import argparse
from pathlib import Path
from typing import Dict, List, Optional

class ReleaseConfigUpdater:
    """发布配置更新器"""
    
    def __init__(self, workspace_root: Path):
        self.workspace_root = workspace_root
        self.cargo_toml_path = workspace_root / "Cargo.toml"
        
    def update_package_metadata(self, 
                               description: Optional[str] = None,
                               repository: Optional[str] = None,
                               homepage: Optional[str] = None,
                               documentation: Optional[str] = None,
                               keywords: Optional[List[str]] = None,
                               categories: Optional[List[str]] = None,
                               license: Optional[str] = None,
                               readme: Optional[str] = None):
        """更新包元数据"""
        
        if not self.cargo_toml_path.exists():
            raise FileNotFoundError(f"未找到 Cargo.toml: {self.cargo_toml_path}")
        
        # 读取现有配置
        with open(self.cargo_toml_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 解析 TOML
        try:
            config = toml.loads(content)
        except toml.TomlDecodeError as e:
            raise ValueError(f"解析 Cargo.toml 失败: {e}")
        
        # 确保 package 部分存在
        if 'package' not in config:
            config['package'] = {}
        
        package = config['package']
        
        # 更新字段
        if description:
            package['description'] = description
            print(f"✅ 更新描述: {description}")
        
        if repository:
            package['repository'] = repository
            print(f"✅ 更新仓库: {repository}")
        
        if homepage:
            package['homepage'] = homepage
            print(f"✅ 更新主页: {homepage}")
        
        if documentation:
            package['documentation'] = documentation
            print(f"✅ 更新文档: {documentation}")
        
        if keywords:
            package['keywords'] = keywords
            print(f"✅ 更新关键词: {keywords}")
        
        if categories:
            package['categories'] = categories
            print(f"✅ 更新分类: {categories}")
        
        if license:
            package['license'] = license
            print(f"✅ 更新许可证: {license}")
        
        if readme:
            package['readme'] = readme
            print(f"✅ 更新 README: {readme}")
        
        # 写回文件
        with open(self.cargo_toml_path, 'w', encoding='utf-8') as f:
            toml.dump(config, f)
        
        print(f"✅ 配置已更新到 {self.cargo_toml_path}")
    
    def add_release_profile(self):
        """添加发布配置文件"""
        
        with open(self.cargo_toml_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        config = toml.loads(content)
        
        # 添加发布配置
        if 'profile' not in config:
            config['profile'] = {}
        
        if 'release' not in config['profile']:
            config['profile']['release'] = {}
        
        release_profile = {
            'opt-level': 3,
            'lto': True,
            'codegen-units': 1,
            'panic': 'abort',
            'strip': True,
        }
        
        config['profile']['release'].update(release_profile)
        
        # 添加发布优化配置
        if 'release-with-debug' not in config['profile']:
            config['profile']['release-with-debug'] = {
                'inherits': 'release',
                'debug': True,
                'strip': False,
            }
        
        with open(self.cargo_toml_path, 'w', encoding='utf-8') as f:
            toml.dump(config, f)
        
        print("✅ 已添加发布配置文件")
    
    def update_workspace_metadata(self):
        """更新工作空间元数据"""
        
        with open(self.cargo_toml_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        config = toml.loads(content)
        
        # 添加工作空间元数据
        if 'workspace' not in config:
            config['workspace'] = {}
        
        if 'metadata' not in config['workspace']:
            config['workspace']['metadata'] = {}
        
        metadata = {
            'release': {
                'tag-format': 'v{{version}}',
                'tag-message': 'Release {{version}}',
                'pre-release-replacements': [
                    {
                        'file': 'CHANGELOG.md',
                        'search': '## \\[未发布\\]',
                        'replace': '## [未发布]\n\n## [{{version}}] - {{date}}'
                    }
                ],
                'pre-release-commit-message': 'chore: prepare release {{version}}',
                'post-release-commit-message': 'chore: bump version to {{version}}',
            }
        }
        
        config['workspace']['metadata'].update(metadata)
        
        with open(self.cargo_toml_path, 'w', encoding='utf-8') as f:
            toml.dump(config, f)
        
        print("✅ 已更新工作空间元数据")
    
    def add_package_exclude(self, exclude_patterns: List[str]):
        """添加包排除模式"""
        
        with open(self.cargo_toml_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        config = toml.loads(content)
        
        if 'package' not in config:
            config['package'] = {}
        
        # 默认排除模式
        default_excludes = [
            "/.github/",
            "/scripts/",
            "/docs/",
            "/examples/",
            "/tests/",
            "/benches/",
            "*.md",
            "*.toml",
            ".gitignore",
            ".gitattributes",
        ]
        
        all_excludes = list(set(default_excludes + exclude_patterns))
        config['package']['exclude'] = all_excludes
        
        with open(self.cargo_toml_path, 'w', encoding='utf-8') as f:
            toml.dump(config, f)
        
        print(f"✅ 已添加排除模式: {all_excludes}")
    
    def update_all_packages(self, **kwargs):
        """更新所有子包的配置"""
        
        # 查找所有 Cargo.toml 文件
        cargo_files = list(self.workspace_root.glob("*/Cargo.toml"))
        cargo_files.extend(list(self.workspace_root.glob("*/*/Cargo.toml")))
        
        for cargo_file in cargo_files:
            if cargo_file == self.cargo_toml_path:
                continue
            
            print(f"📦 更新包配置: {cargo_file.parent.name}")
            
            with open(cargo_file, 'r', encoding='utf-8') as f:
                content = f.read()
            
            try:
                config = toml.loads(content)
            except toml.TomlDecodeError:
                print(f"⚠️  跳过无效的 TOML 文件: {cargo_file}")
                continue
            
            if 'package' not in config:
                continue
            
            package = config['package']
            
            # 更新字段
            for key, value in kwargs.items():
                if value and key in ['repository', 'homepage', 'documentation', 'license']:
                    package[key] = value
            
            # 更新关键词和分类（添加包特定的）
            if 'keywords' in kwargs and kwargs['keywords']:
                base_keywords = kwargs['keywords']
                package_name = package.get('name', '')
                
                # 根据包名添加特定关键词
                if 'core' in package_name:
                    package['keywords'] = base_keywords + ['core', 'framework']
                elif 'cli' in package_name:
                    package['keywords'] = base_keywords + ['cli', 'command-line']
                elif 'rag' in package_name:
                    package['keywords'] = base_keywords + ['rag', 'retrieval']
                elif 'vector' in package_name:
                    package['keywords'] = base_keywords + ['vector', 'embedding']
                else:
                    package['keywords'] = base_keywords
            
            with open(cargo_file, 'w', encoding='utf-8') as f:
                toml.dump(config, f)
    
    def validate_config(self) -> bool:
        """验证发布配置"""
        
        print("🔍 验证发布配置...")
        
        with open(self.cargo_toml_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        config = toml.loads(content)
        
        # 检查必需字段
        required_fields = ['name', 'version', 'authors', 'description', 'license']
        missing_fields = []
        
        package = config.get('package', {})
        for field in required_fields:
            if field not in package:
                missing_fields.append(field)
        
        if missing_fields:
            print(f"❌ 缺少必需字段: {missing_fields}")
            return False
        
        # 检查版本格式
        version = package.get('version', '')
        if not re.match(r'^\d+\.\d+\.\d+(-[a-zA-Z0-9]+)?$', version):
            print(f"❌ 无效的版本格式: {version}")
            return False
        
        # 检查许可证
        license_value = package.get('license', '')
        valid_licenses = ['MIT', 'Apache-2.0', 'BSD-2-Clause', 'BSD-3-Clause']
        if license_value not in valid_licenses:
            print(f"⚠️  许可证可能需要检查: {license_value}")
        
        print("✅ 配置验证通过")
        return True

def main():
    parser = argparse.ArgumentParser(description='LumosAI 发布配置更新工具')
    parser.add_argument('--workspace', type=Path, default=Path.cwd(),
                       help='工作空间根目录')
    
    subparsers = parser.add_subparsers(dest='command', help='可用命令')
    
    # 更新元数据命令
    metadata_parser = subparsers.add_parser('metadata', help='更新包元数据')
    metadata_parser.add_argument('--description', help='包描述')
    metadata_parser.add_argument('--repository', help='仓库 URL')
    metadata_parser.add_argument('--homepage', help='主页 URL')
    metadata_parser.add_argument('--documentation', help='文档 URL')
    metadata_parser.add_argument('--keywords', nargs='+', help='关键词列表')
    metadata_parser.add_argument('--categories', nargs='+', help='分类列表')
    metadata_parser.add_argument('--license', help='许可证')
    metadata_parser.add_argument('--readme', help='README 文件路径')
    metadata_parser.add_argument('--all-packages', action='store_true',
                                help='更新所有子包')
    
    # 添加配置文件命令
    subparsers.add_parser('profile', help='添加发布配置文件')
    
    # 更新工作空间命令
    subparsers.add_parser('workspace', help='更新工作空间元数据')
    
    # 添加排除模式命令
    exclude_parser = subparsers.add_parser('exclude', help='添加包排除模式')
    exclude_parser.add_argument('patterns', nargs='+', help='排除模式列表')
    
    # 验证配置命令
    subparsers.add_parser('validate', help='验证发布配置')
    
    # 完整设置命令
    setup_parser = subparsers.add_parser('setup', help='完整发布配置设置')
    setup_parser.add_argument('--description', 
                             default='企业级 AI 代理框架',
                             help='包描述')
    setup_parser.add_argument('--repository',
                             default='https://github.com/lumosai/lumosai',
                             help='仓库 URL')
    setup_parser.add_argument('--homepage',
                             default='https://lumosai.dev',
                             help='主页 URL')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    try:
        updater = ReleaseConfigUpdater(args.workspace)
        
        if args.command == 'metadata':
            kwargs = {k: v for k, v in vars(args).items() 
                     if v is not None and k not in ['command', 'workspace', 'all_packages']}
            
            if args.all_packages:
                updater.update_all_packages(**kwargs)
            else:
                updater.update_package_metadata(**kwargs)
        
        elif args.command == 'profile':
            updater.add_release_profile()
        
        elif args.command == 'workspace':
            updater.update_workspace_metadata()
        
        elif args.command == 'exclude':
            updater.add_package_exclude(args.patterns)
        
        elif args.command == 'validate':
            if not updater.validate_config():
                sys.exit(1)
        
        elif args.command == 'setup':
            print("🚀 设置完整发布配置...")
            
            # 更新包元数据
            updater.update_package_metadata(
                description=args.description,
                repository=args.repository,
                homepage=args.homepage,
                documentation=f"{args.repository.replace('github.com', 'docs.rs')}/lumosai",
                keywords=['ai', 'agent', 'rust', 'framework', 'enterprise'],
                categories=['development-tools', 'api-bindings', 'science'],
                license='MIT',
                readme='README.md'
            )
            
            # 添加发布配置文件
            updater.add_release_profile()
            
            # 更新工作空间元数据
            updater.update_workspace_metadata()
            
            # 添加排除模式
            updater.add_package_exclude([])
            
            # 更新所有子包
            updater.update_all_packages(
                repository=args.repository,
                homepage=args.homepage,
                license='MIT',
                keywords=['ai', 'agent', 'rust']
            )
            
            # 验证配置
            updater.validate_config()
            
            print("✅ 发布配置设置完成！")
    
    except Exception as e:
        print(f"错误: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
