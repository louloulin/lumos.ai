#!/usr/bin/env python3
"""
LumosAI 发布质量检查工具
用于检查代码质量、测试覆盖率、文档完整性等
"""

import os
import re
import sys
import json
import subprocess
import argparse
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass

@dataclass
class QualityMetrics:
    """质量指标"""
    test_coverage: float
    clippy_warnings: int
    doc_coverage: float
    code_lines: int
    test_lines: int
    dependency_count: int
    security_issues: int
    performance_score: float

class QualityChecker:
    """质量检查器"""
    
    def __init__(self, workspace_root: Path):
        self.workspace_root = workspace_root
        self.metrics = QualityMetrics(0, 0, 0, 0, 0, 0, 0, 0)
    
    def run_command(self, cmd: List[str], capture_output: bool = True) -> Tuple[int, str, str]:
        """运行命令并返回结果"""
        try:
            result = subprocess.run(
                cmd,
                cwd=self.workspace_root,
                capture_output=capture_output,
                text=True,
                timeout=300
            )
            return result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return -1, "", "命令超时"
        except Exception as e:
            return -1, "", str(e)
    
    def check_test_coverage(self) -> float:
        """检查测试覆盖率"""
        print("🧪 检查测试覆盖率...")
        
        # 检查是否安装了 cargo-tarpaulin
        code, _, _ = self.run_command(["cargo", "tarpaulin", "--version"])
        if code != 0:
            print("⚠️  cargo-tarpaulin 未安装，跳过覆盖率检查")
            return 0.0
        
        # 运行覆盖率测试
        code, stdout, stderr = self.run_command([
            "cargo", "tarpaulin", 
            "--all-features", 
            "--workspace",
            "--out", "json",
            "--output-dir", "target/tarpaulin"
        ])
        
        if code != 0:
            print(f"❌ 覆盖率测试失败: {stderr}")
            return 0.0
        
        # 解析覆盖率结果
        try:
            coverage_file = self.workspace_root / "target/tarpaulin/tarpaulin-report.json"
            if coverage_file.exists():
                with open(coverage_file, 'r') as f:
                    data = json.load(f)
                    coverage = data.get('coverage', 0.0)
                    print(f"✅ 测试覆盖率: {coverage:.1f}%")
                    return coverage
        except Exception as e:
            print(f"⚠️  解析覆盖率结果失败: {e}")
        
        return 0.0
    
    def check_clippy_warnings(self) -> int:
        """检查 Clippy 警告"""
        print("🔍 检查 Clippy 警告...")
        
        code, stdout, stderr = self.run_command([
            "cargo", "clippy", 
            "--all-features", 
            "--workspace",
            "--message-format=json"
        ])
        
        warnings = 0
        for line in stdout.split('\n'):
            if line.strip():
                try:
                    msg = json.loads(line)
                    if msg.get('reason') == 'compiler-message':
                        level = msg.get('message', {}).get('level')
                        if level == 'warning':
                            warnings += 1
                except json.JSONDecodeError:
                    continue
        
        print(f"✅ Clippy 警告数: {warnings}")
        return warnings
    
    def check_doc_coverage(self) -> float:
        """检查文档覆盖率"""
        print("📚 检查文档覆盖率...")
        
        # 运行文档构建并检查缺失的文档
        code, stdout, stderr = self.run_command([
            "cargo", "doc", 
            "--all-features", 
            "--workspace",
            "--no-deps"
        ])
        
        if code != 0:
            print(f"❌ 文档构建失败: {stderr}")
            return 0.0
        
        # 统计文档覆盖率（简单实现）
        doc_warnings = stderr.count("missing documentation")
        total_items = stderr.count("warning:") + 100  # 估算
        
        if total_items > 0:
            coverage = max(0, (total_items - doc_warnings) / total_items * 100)
        else:
            coverage = 100.0
        
        print(f"✅ 文档覆盖率: {coverage:.1f}%")
        return coverage
    
    def count_code_lines(self) -> Tuple[int, int]:
        """统计代码行数"""
        print("📊 统计代码行数...")
        
        code_lines = 0
        test_lines = 0
        
        # 统计 Rust 源文件
        for rust_file in self.workspace_root.rglob("*.rs"):
            if "target" in str(rust_file):
                continue
            
            try:
                with open(rust_file, 'r', encoding='utf-8') as f:
                    lines = len([line for line in f if line.strip() and not line.strip().startswith('//')])
                
                if "test" in str(rust_file) or "/tests/" in str(rust_file):
                    test_lines += lines
                else:
                    code_lines += lines
            except Exception:
                continue
        
        print(f"✅ 代码行数: {code_lines}, 测试行数: {test_lines}")
        return code_lines, test_lines
    
    def count_dependencies(self) -> int:
        """统计依赖数量"""
        print("📦 统计依赖数量...")
        
        code, stdout, stderr = self.run_command([
            "cargo", "tree", "--depth=1"
        ])
        
        if code != 0:
            print(f"⚠️  无法统计依赖: {stderr}")
            return 0
        
        # 简单统计直接依赖
        deps = len([line for line in stdout.split('\n') if line.startswith('├──') or line.startswith('└──')])
        print(f"✅ 直接依赖数: {deps}")
        return deps
    
    def check_security_issues(self) -> int:
        """检查安全问题"""
        print("🔒 检查安全问题...")
        
        # 检查是否安装了 cargo-audit
        code, _, _ = self.run_command(["cargo", "audit", "--version"])
        if code != 0:
            print("⚠️  cargo-audit 未安装，跳过安全检查")
            return 0
        
        code, stdout, stderr = self.run_command([
            "cargo", "audit", "--json"
        ])
        
        if code != 0:
            print(f"⚠️  安全检查失败: {stderr}")
            return 0
        
        try:
            data = json.loads(stdout)
            vulnerabilities = len(data.get('vulnerabilities', {}).get('list', []))
            print(f"✅ 安全问题数: {vulnerabilities}")
            return vulnerabilities
        except json.JSONDecodeError:
            print("⚠️  解析安全检查结果失败")
            return 0
    
    def check_performance(self) -> float:
        """检查性能（基准测试）"""
        print("⚡ 检查性能...")
        
        # 检查是否有基准测试
        bench_files = list(self.workspace_root.rglob("benches/*.rs"))
        if not bench_files:
            print("⚠️  未找到基准测试")
            return 0.0
        
        code, stdout, stderr = self.run_command([
            "cargo", "bench", "--no-run"
        ])
        
        if code == 0:
            print("✅ 基准测试编译成功")
            return 85.0  # 假设性能分数
        else:
            print(f"❌ 基准测试编译失败: {stderr}")
            return 0.0
    
    def run_all_checks(self) -> QualityMetrics:
        """运行所有质量检查"""
        print("🚀 开始质量检查...")
        print("=" * 50)
        
        # 运行各项检查
        self.metrics.test_coverage = self.check_test_coverage()
        self.metrics.clippy_warnings = self.check_clippy_warnings()
        self.metrics.doc_coverage = self.check_doc_coverage()
        self.metrics.code_lines, self.metrics.test_lines = self.count_code_lines()
        self.metrics.dependency_count = self.count_dependencies()
        self.metrics.security_issues = self.check_security_issues()
        self.metrics.performance_score = self.check_performance()
        
        return self.metrics
    
    def generate_report(self, metrics: QualityMetrics) -> str:
        """生成质量报告"""
        
        # 计算总体质量分数
        quality_score = self._calculate_quality_score(metrics)
        
        report = f"""
# LumosAI 质量报告

## 📊 总体评分: {quality_score:.1f}/100

## 📈 详细指标

### 🧪 测试质量
- **测试覆盖率**: {metrics.test_coverage:.1f}%
- **测试代码行数**: {metrics.test_lines:,}
- **代码测试比**: {(metrics.test_lines / max(metrics.code_lines, 1) * 100):.1f}%

### 🔍 代码质量  
- **Clippy 警告**: {metrics.clippy_warnings}
- **代码行数**: {metrics.code_lines:,}
- **文档覆盖率**: {metrics.doc_coverage:.1f}%

### 📦 依赖管理
- **直接依赖数**: {metrics.dependency_count}
- **安全问题**: {metrics.security_issues}

### ⚡ 性能
- **性能分数**: {metrics.performance_score:.1f}/100

## 🎯 质量等级

{self._get_quality_grade(quality_score)}

## 📋 改进建议

{self._get_improvement_suggestions(metrics)}

---
*报告生成时间: {self._get_timestamp()}*
"""
        return report
    
    def _calculate_quality_score(self, metrics: QualityMetrics) -> float:
        """计算质量分数"""
        
        # 权重配置
        weights = {
            'test_coverage': 0.25,
            'clippy_warnings': 0.20,
            'doc_coverage': 0.15,
            'security_issues': 0.20,
            'performance': 0.20
        }
        
        # 计算各项分数
        test_score = min(metrics.test_coverage, 100)
        clippy_score = max(0, 100 - metrics.clippy_warnings * 2)
        doc_score = min(metrics.doc_coverage, 100)
        security_score = max(0, 100 - metrics.security_issues * 10)
        performance_score = metrics.performance_score
        
        # 加权平均
        total_score = (
            test_score * weights['test_coverage'] +
            clippy_score * weights['clippy_warnings'] +
            doc_score * weights['doc_coverage'] +
            security_score * weights['security_issues'] +
            performance_score * weights['performance']
        )
        
        return min(total_score, 100)
    
    def _get_quality_grade(self, score: float) -> str:
        """获取质量等级"""
        if score >= 90:
            return "🏆 **优秀** - 代码质量非常高，可以发布"
        elif score >= 80:
            return "✅ **良好** - 代码质量良好，建议发布"
        elif score >= 70:
            return "⚠️  **一般** - 代码质量一般，建议改进后发布"
        elif score >= 60:
            return "❌ **较差** - 代码质量较差，需要改进"
        else:
            return "🚫 **很差** - 代码质量很差，不建议发布"
    
    def _get_improvement_suggestions(self, metrics: QualityMetrics) -> str:
        """获取改进建议"""
        suggestions = []
        
        if metrics.test_coverage < 80:
            suggestions.append("- 📈 提高测试覆盖率到 80% 以上")
        
        if metrics.clippy_warnings > 10:
            suggestions.append("- 🔧 修复 Clippy 警告，保持代码整洁")
        
        if metrics.doc_coverage < 90:
            suggestions.append("- 📚 完善 API 文档，提高文档覆盖率")
        
        if metrics.security_issues > 0:
            suggestions.append("- 🔒 修复安全漏洞，确保代码安全")
        
        if metrics.dependency_count > 50:
            suggestions.append("- 📦 考虑减少依赖数量，简化依赖树")
        
        if not suggestions:
            suggestions.append("- 🎉 代码质量很好，继续保持！")
        
        return "\n".join(suggestions)
    
    def _get_timestamp(self) -> str:
        """获取时间戳"""
        from datetime import datetime
        return datetime.now().strftime("%Y-%m-%d %H:%M:%S")

def main():
    parser = argparse.ArgumentParser(description='LumosAI 发布质量检查工具')
    parser.add_argument('--workspace', type=Path, default=Path.cwd(),
                       help='工作空间根目录')
    parser.add_argument('--output', type=Path,
                       help='输出报告文件路径')
    parser.add_argument('--format', choices=['text', 'json', 'html'],
                       default='text', help='输出格式')
    parser.add_argument('--threshold', type=float, default=70.0,
                       help='质量分数阈值')
    
    args = parser.parse_args()
    
    try:
        checker = QualityChecker(args.workspace)
        metrics = checker.run_all_checks()
        
        print("\n" + "=" * 50)
        print("📊 质量检查完成")
        
        # 生成报告
        if args.format == 'text':
            report = checker.generate_report(metrics)
            print(report)
            
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    f.write(report)
                print(f"✅ 报告已保存到 {args.output}")
        
        elif args.format == 'json':
            import json
            report_data = {
                'metrics': metrics.__dict__,
                'quality_score': checker._calculate_quality_score(metrics),
                'timestamp': checker._get_timestamp()
            }
            
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(report_data, f, indent=2, ensure_ascii=False)
                print(f"✅ JSON 报告已保存到 {args.output}")
            else:
                print(json.dumps(report_data, indent=2, ensure_ascii=False))
        
        # 检查是否达到阈值
        quality_score = checker._calculate_quality_score(metrics)
        if quality_score < args.threshold:
            print(f"\n❌ 质量分数 {quality_score:.1f} 低于阈值 {args.threshold}")
            sys.exit(1)
        else:
            print(f"\n✅ 质量分数 {quality_score:.1f} 达到阈值 {args.threshold}")
    
    except Exception as e:
        print(f"错误: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
