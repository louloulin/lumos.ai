#!/usr/bin/env python3
"""
Milvus 实现验证脚本
验证 Milvus 集成的完整性和功能
"""

import os
import sys
import subprocess
from pathlib import Path
from typing import List, Dict, Any

def run_command(cmd: List[str], cwd: str = None) -> tuple[int, str, str]:
    """运行命令并返回结果"""
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=60
        )
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return 1, "", "Command timed out"
    except Exception as e:
        return 1, "", str(e)

def check_file_exists(file_path: str) -> bool:
    """检查文件是否存在"""
    return Path(file_path).exists()

def validate_file_structure() -> Dict[str, Any]:
    """验证文件结构"""
    print("🔍 验证文件结构...")
    
    required_files = [
        "lumosai_vector/milvus/Cargo.toml",
        "lumosai_vector/milvus/src/lib.rs",
        "lumosai_vector/milvus/src/config.rs",
        "lumosai_vector/milvus/src/error.rs",
        "lumosai_vector/milvus/src/storage.rs",
        "lumosai_vector/milvus/src/client.rs",
        "lumosai_vector/milvus/src/types.rs",
        "lumosai_vector/milvus/README.md",
        "lumosai_vector/milvus/examples/basic_usage.rs",
        "lumosai_vector/milvus/examples/batch_operations.rs",
        "lumosai_vector/milvus/examples/collection_management.rs",
        "lumosai_vector/milvus/tests/integration_tests.rs",
    ]
    
    missing_files = []
    existing_files = []
    
    for file_path in required_files:
        if check_file_exists(file_path):
            existing_files.append(file_path)
        else:
            missing_files.append(file_path)
    
    return {
        "total": len(required_files),
        "existing": len(existing_files),
        "missing": len(missing_files),
        "missing_files": missing_files,
        "success": len(missing_files) == 0
    }

def validate_cargo_config() -> Dict[str, Any]:
    """验证 Cargo 配置"""
    print("📦 验证 Cargo 配置...")
    
    # 检查 Milvus crate 的 Cargo.toml
    milvus_cargo_path = "lumosai_vector/milvus/Cargo.toml"
    if not check_file_exists(milvus_cargo_path):
        return {"success": False, "error": "Milvus Cargo.toml not found"}
    
    # 检查工作空间 Cargo.toml
    workspace_cargo_path = "lumosai_vector/Cargo.toml"
    if not check_file_exists(workspace_cargo_path):
        return {"success": False, "error": "Workspace Cargo.toml not found"}
    
    # 检查工作空间是否包含 milvus 特性
    try:
        with open(workspace_cargo_path, 'r') as f:
            content = f.read()
            
        has_milvus_dep = 'lumosai-vector-milvus' in content
        has_milvus_feature = 'milvus = ["lumosai-vector-milvus"]' in content
        has_milvus_in_all = '"milvus"' in content and 'all = [' in content
        
        return {
            "success": has_milvus_dep and has_milvus_feature and has_milvus_in_all,
            "has_dependency": has_milvus_dep,
            "has_feature": has_milvus_feature,
            "has_all_feature": has_milvus_in_all
        }
    except Exception as e:
        return {"success": False, "error": str(e)}

def validate_api_structure() -> Dict[str, Any]:
    """验证 API 结构"""
    print("🔧 验证 API 结构...")
    
    # 检查主要模块导出
    lib_rs_path = "lumosai_vector/milvus/src/lib.rs"
    if not check_file_exists(lib_rs_path):
        return {"success": False, "error": "lib.rs not found"}
    
    try:
        with open(lib_rs_path, 'r') as f:
            content = f.read()
        
        required_exports = [
            "pub use storage::MilvusStorage;",
            "pub use config::{MilvusConfig, MilvusConfigBuilder};",
            "pub use error::{MilvusError, MilvusResult};",
            "pub use client::MilvusClient;",
            "pub use types::*;",
        ]
        
        missing_exports = []
        for export in required_exports:
            if export not in content:
                missing_exports.append(export)
        
        has_vector_storage_trait = "VectorStorage" in content
        has_async_trait = "#[async_trait]" in content or "async_trait" in content
        
        return {
            "success": len(missing_exports) == 0 and has_vector_storage_trait,
            "missing_exports": missing_exports,
            "has_vector_storage_trait": has_vector_storage_trait,
            "has_async_trait": has_async_trait
        }
    except Exception as e:
        return {"success": False, "error": str(e)}

def validate_examples() -> Dict[str, Any]:
    """验证示例文件"""
    print("📝 验证示例文件...")
    
    examples = [
        "lumosai_vector/milvus/examples/basic_usage.rs",
        "lumosai_vector/milvus/examples/batch_operations.rs", 
        "lumosai_vector/milvus/examples/collection_management.rs"
    ]
    
    valid_examples = []
    invalid_examples = []
    
    for example in examples:
        if check_file_exists(example):
            try:
                with open(example, 'r') as f:
                    content = f.read()
                
                # 检查示例是否包含必要的导入和主函数
                has_main = "#[tokio::main]" in content and "async fn main()" in content
                has_milvus_import = "lumosai_vector_milvus" in content
                has_vector_storage = "VectorStorage" in content
                
                if has_main and has_milvus_import and has_vector_storage:
                    valid_examples.append(example)
                else:
                    invalid_examples.append(example)
            except Exception:
                invalid_examples.append(example)
        else:
            invalid_examples.append(example)
    
    return {
        "success": len(invalid_examples) == 0,
        "total": len(examples),
        "valid": len(valid_examples),
        "invalid": len(invalid_examples),
        "invalid_examples": invalid_examples
    }

def validate_documentation() -> Dict[str, Any]:
    """验证文档"""
    print("📚 验证文档...")
    
    readme_path = "lumosai_vector/milvus/README.md"
    if not check_file_exists(readme_path):
        return {"success": False, "error": "README.md not found"}
    
    try:
        with open(readme_path, 'r') as f:
            content = f.read()
        
        required_sections = [
            "# 🚀 LumosAI Milvus Integration",
            "## ✨ Features",
            "## 🚀 Quick Start",
            "## 🔧 Configuration",
            "## 📊 Index Types",
            "## 🔍 Advanced Search",
            "## 📈 Performance Optimization",
        ]
        
        missing_sections = []
        for section in required_sections:
            if section not in content:
                missing_sections.append(section)
        
        has_code_examples = "```rust" in content
        has_installation = "Cargo.toml" in content
        
        return {
            "success": len(missing_sections) == 0 and has_code_examples,
            "missing_sections": missing_sections,
            "has_code_examples": has_code_examples,
            "has_installation": has_installation,
            "word_count": len(content.split())
        }
    except Exception as e:
        return {"success": False, "error": str(e)}

def validate_compilation() -> Dict[str, Any]:
    """验证编译"""
    print("🔨 验证编译...")
    
    # 检查基本编译
    code, stdout, stderr = run_command(
        ["cargo", "check", "--manifest-path", "lumosai_vector/milvus/Cargo.toml"],
        cwd="."
    )
    
    if code != 0:
        return {
            "success": False,
            "error": "Compilation failed",
            "stdout": stdout,
            "stderr": stderr
        }
    
    # 检查示例编译
    examples_result = []
    examples = ["basic_usage", "batch_operations", "collection_management"]
    
    for example in examples:
        code, stdout, stderr = run_command(
            ["cargo", "check", "--example", example, "--manifest-path", "lumosai_vector/milvus/Cargo.toml"],
            cwd="."
        )
        
        examples_result.append({
            "name": example,
            "success": code == 0,
            "stderr": stderr if code != 0 else ""
        })
    
    failed_examples = [ex for ex in examples_result if not ex["success"]]
    
    return {
        "success": len(failed_examples) == 0,
        "examples_total": len(examples),
        "examples_passed": len(examples) - len(failed_examples),
        "failed_examples": failed_examples
    }

def validate_integration() -> Dict[str, Any]:
    """验证集成"""
    print("🔗 验证集成...")
    
    # 检查是否能在工作空间中找到 milvus 特性
    code, stdout, stderr = run_command(
        ["cargo", "check", "--features", "milvus", "--manifest-path", "lumosai_vector/Cargo.toml"],
        cwd="."
    )
    
    return {
        "success": code == 0,
        "stdout": stdout,
        "stderr": stderr
    }

def main():
    """主函数"""
    print("🚀 Milvus 实现验证")
    print("=" * 50)
    
    # 检查是否在正确的目录
    if not Path("Cargo.toml").exists():
        print("❌ 错误: 请在项目根目录运行此脚本")
        sys.exit(1)
    
    results = {}
    
    # 1. 验证文件结构
    results["file_structure"] = validate_file_structure()
    
    # 2. 验证 Cargo 配置
    results["cargo_config"] = validate_cargo_config()
    
    # 3. 验证 API 结构
    results["api_structure"] = validate_api_structure()
    
    # 4. 验证示例文件
    results["examples"] = validate_examples()
    
    # 5. 验证文档
    results["documentation"] = validate_documentation()
    
    # 6. 验证编译
    results["compilation"] = validate_compilation()
    
    # 7. 验证集成
    results["integration"] = validate_integration()
    
    # 汇总结果
    print("\n📊 验证结果汇总")
    print("=" * 50)
    
    total_checks = len(results)
    passed_checks = sum(1 for result in results.values() if result["success"])
    
    for check_name, result in results.items():
        status = "✅" if result["success"] else "❌"
        print(f"{status} {check_name.replace('_', ' ').title()}")
        
        if not result["success"]:
            if "error" in result:
                print(f"   错误: {result['error']}")
            if "missing_files" in result:
                for file in result["missing_files"][:3]:  # 只显示前3个
                    print(f"   缺失文件: {file}")
            if "missing_exports" in result:
                for export in result["missing_exports"][:3]:
                    print(f"   缺失导出: {export}")
            if "failed_examples" in result:
                for example in result["failed_examples"][:2]:
                    print(f"   失败示例: {example['name']}")
    
    print(f"\n📈 总体结果: {passed_checks}/{total_checks} ({passed_checks/total_checks*100:.1f}%)")
    
    if passed_checks == total_checks:
        print("🎉 所有验证通过！Milvus 集成实现完成。")
        
        print("\n🚀 下一步:")
        print("1. 启动 Milvus 服务进行测试")
        print("2. 运行示例: cargo run --example basic_usage --manifest-path lumosai_vector/milvus/Cargo.toml")
        print("3. 运行集成测试: cargo test --manifest-path lumosai_vector/milvus/Cargo.toml")
        print("4. 更新 plan7.md 标记 Milvus 实现完成")
        
        return 0
    else:
        print("❌ 部分验证失败，请检查上述错误并修复。")
        return 1

if __name__ == "__main__":
    sys.exit(main())
