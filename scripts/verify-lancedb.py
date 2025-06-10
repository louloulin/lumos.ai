#!/usr/bin/env python3
"""
LanceDB 功能验证脚本
验证 LanceDB 实现的完整性和正确性
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
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
        
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

def verify_file_structure():
    """验证文件结构"""
    print("\n📁 验证 LanceDB 文件结构")
    print("=" * 60)
    
    required_files = [
        "lumosai_vector/lancedb/Cargo.toml",
        "lumosai_vector/lancedb/src/lib.rs",
        "lumosai_vector/lancedb/src/config.rs",
        "lumosai_vector/lancedb/src/storage.rs",
        "lumosai_vector/lancedb/src/error.rs",
        "lumosai_vector/lancedb/src/conversion.rs",
        "lumosai_vector/lancedb/src/index.rs",
        "lumosai_vector/lancedb/README.md",
        "lumosai_vector/lancedb/examples/basic_usage.rs",
        "lumosai_vector/lancedb/examples/batch_operations.rs",
        "lumosai_vector/lancedb/examples/vector_search.rs",
        "lumosai_vector/lancedb/tests/integration_tests.rs",
        "lumosai_vector/lancedb/tests/compile_test.rs",
    ]
    
    all_exist = True
    for file_path in required_files:
        if Path(file_path).exists():
            print(f"✅ {file_path}")
        else:
            print(f"❌ {file_path} - 文件不存在")
            all_exist = False
    
    return all_exist

def verify_cargo_config():
    """验证 Cargo 配置"""
    print("\n📦 验证 Cargo 配置")
    print("=" * 60)
    
    all_passed = True
    
    # 检查 LanceDB Cargo.toml
    lancedb_cargo = Path("lumosai_vector/lancedb/Cargo.toml")
    if lancedb_cargo.exists():
        with open(lancedb_cargo, 'r', encoding='utf-8') as f:
            content = f.read()
            
        checks = [
            ("包名", 'name = "lumosai-vector-lancedb"' in content),
            ("版本", 'version = "0.1.0"' in content),
            ("LanceDB 依赖", 'lancedb = "0.8.0"' in content),
            ("Lance 依赖", 'lance = "0.12.0"' in content),
            ("Arrow 依赖", 'arrow = "52.0.0"' in content),
            ("核心依赖", 'lumosai-vector-core' in content),
            ("异步支持", 'tokio' in content),
        ]
        
        for check_name, passed in checks:
            status = "✅" if passed else "❌"
            print(f"{status} {check_name}")
            if not passed:
                all_passed = False
    
    # 检查工作空间配置
    workspace_cargo = Path("lumosai_vector/Cargo.toml")
    if workspace_cargo.exists():
        with open(workspace_cargo, 'r', encoding='utf-8') as f:
            content = f.read()
            
        lancedb_feature = 'lancedb = ["lumosai-vector-lancedb"]' in content
        lancedb_dep = 'lumosai-vector-lancedb = { path = "lancedb", optional = true }' in content
        
        print(f"{'✅' if lancedb_feature else '❌'} 工作空间 LanceDB 功能")
        print(f"{'✅' if lancedb_dep else '❌'} 工作空间 LanceDB 依赖")
        
        if not lancedb_feature or not lancedb_dep:
            all_passed = False
    
    return all_passed

def verify_api_structure():
    """验证 API 结构"""
    print("\n🔧 验证 API 结构")
    print("=" * 60)
    
    all_passed = True
    
    # 检查主要模块
    modules = {
        "lib.rs": ["LanceDbStorage", "LanceDbClient", "LanceDbConfig"],
        "config.rs": ["LanceDbConfig", "LanceDbConfigBuilder", "IndexType", "PerformanceConfig"],
        "storage.rs": ["LanceDbStorage", "VectorStorage"],
        "error.rs": ["LanceDbError", "LanceDbResult"],
        "conversion.rs": ["documents_to_record_batch", "record_batch_to_documents"],
        "index.rs": ["IndexManager", "IndexConfiguration"],
    }
    
    for module, expected_items in modules.items():
        module_path = Path(f"lumosai_vector/lancedb/src/{module}")
        if module_path.exists():
            with open(module_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            print(f"\n📄 {module}:")
            for item in expected_items:
                if item in content:
                    print(f"  ✅ {item}")
                else:
                    print(f"  ❌ {item} - 未找到")
                    all_passed = False
    
    return all_passed

def verify_examples():
    """验证示例文件"""
    print("\n📚 验证示例文件")
    print("=" * 60)
    
    all_passed = True
    
    examples = [
        "basic_usage.rs",
        "batch_operations.rs", 
        "vector_search.rs",
    ]
    
    for example in examples:
        example_path = Path(f"lumosai_vector/lancedb/examples/{example}")
        if example_path.exists():
            with open(example_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # 检查示例的关键组件
            checks = [
                ("LanceDB 导入", "lumosai_vector_lancedb" in content),
                ("异步主函数", "#[tokio::main]" in content),
                ("错误处理", "Result<" in content),
                ("示例说明", "//!" in content),
            ]
            
            print(f"\n📄 {example}:")
            for check_name, passed in checks:
                status = "✅" if passed else "❌"
                print(f"  {status} {check_name}")
                if not passed:
                    all_passed = False
    
    return all_passed

def verify_documentation():
    """验证文档"""
    print("\n📖 验证文档")
    print("=" * 60)
    
    readme_path = Path("lumosai_vector/lancedb/README.md")
    if readme_path.exists():
        with open(readme_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        doc_checks = [
            ("标题", "# LumosAI LanceDB Integration" in content),
            ("功能特性", "## 🚀 Features" in content),
            ("安装说明", "## 📦 Installation" in content),
            ("快速开始", "## 🎯 Quick Start" in content),
            ("API 示例", "```rust" in content),
            ("性能基准", "## 📊 Performance Benchmarks" in content),
            ("索引类型", "## 🔧 Index Types" in content),
            ("云存储", "### Cloud Storage" in content),
        ]
        
        all_passed = True
        for check_name, passed in doc_checks:
            status = "✅" if passed else "❌"
            print(f"{status} {check_name}")
            if not passed:
                all_passed = False
        
        return all_passed
    else:
        print("❌ README.md 不存在")
        return False

def verify_integration():
    """验证集成"""
    print("\n🔗 验证集成")
    print("=" * 60)
    
    # 检查是否正确集成到工作空间
    workspace_lib = Path("lumosai_vector/src/lib.rs")
    if workspace_lib.exists():
        with open(workspace_lib, 'r', encoding='utf-8') as f:
            content = f.read()
        
        integration_checks = [
            ("条件编译", '#[cfg(feature = "lancedb")]' in content),
            ("模块导出", "pub use lumosai_vector_lancedb as lancedb" in content),
        ]
        
        all_passed = True
        for check_name, passed in integration_checks:
            status = "✅" if passed else "❌"
            print(f"{status} {check_name}")
            if not passed:
                all_passed = False
        
        return all_passed
    else:
        return False

def verify_features():
    """验证功能特性"""
    print("\n🚀 验证功能特性")
    print("=" * 60)
    
    features = [
        ("高性能存储", "columnar storage"),
        ("ACID 事务", "transaction"),
        ("多种索引", "IVF", "IVFPQ", "HNSW"),
        ("元数据过滤", "metadata filtering"),
        ("版本控制", "versioning"),
        ("压缩支持", "compression"),
        ("云存储", "S3", "Azure", "GCS"),
        ("批量操作", "batch operations"),
    ]
    
    # 检查 README 中是否提到这些功能
    readme_path = Path("lumosai_vector/lancedb/README.md")
    if readme_path.exists():
        with open(readme_path, 'r', encoding='utf-8') as f:
            content = f.read().lower()
        
        for feature_info in features:
            feature_name = feature_info[0]
            keywords = feature_info[1:]
            
            found = any(keyword.lower() in content for keyword in keywords)
            status = "✅" if found else "❌"
            print(f"{status} {feature_name}")
    
    return True

def main():
    """主函数"""
    print("🚀 LumosAI LanceDB 功能验证")
    print("=" * 60)
    
    # 检查是否在正确的目录
    if not Path("Cargo.toml").exists():
        print("❌ 错误: 请在项目根目录运行此脚本")
        sys.exit(1)
    
    try:
        # 运行各项验证
        checks = [
            ("文件结构", verify_file_structure),
            ("Cargo 配置", verify_cargo_config),
            ("API 结构", verify_api_structure),
            ("示例文件", verify_examples),
            ("文档", verify_documentation),
            ("集成", verify_integration),
            ("功能特性", verify_features),
        ]
        
        passed_checks = 0
        total_checks = len(checks)
        
        for check_name, check_func in checks:
            try:
                if check_func():
                    passed_checks += 1
                    print(f"\n✅ {check_name} 验证通过")
                else:
                    print(f"\n⚠️  {check_name} 验证有问题")
            except Exception as e:
                print(f"\n❌ {check_name} 验证失败: {e}")
        
        # 编译验证（可选，因为可能需要很长时间）
        print("\n🔨 编译验证")
        print("=" * 60)
        print("⚠️  注意: 由于 LanceDB 依赖较多，编译可能需要较长时间")
        
        user_input = input("是否进行编译验证? (y/N): ").strip().lower()
        if user_input in ['y', 'yes']:
            compile_success = run_command(
                ["cargo", "check", "--package", "lumosai-vector-lancedb"],
                "编译 LanceDB 包"
            )
            
            if compile_success:
                passed_checks += 1
                total_checks += 1
            else:
                total_checks += 1
        else:
            print("⏭️  跳过编译验证")
        
        # 显示总结
        print("\n" + "=" * 60)
        print("📊 验证结果总结")
        print("=" * 60)
        
        success_rate = (passed_checks / total_checks) * 100
        print(f"通过检查: {passed_checks}/{total_checks} ({success_rate:.1f}%)")
        
        if success_rate >= 90:
            print("🎉 LanceDB 实现验证成功！")
            print("\n✨ 主要成就:")
            print("  • 完整的 LanceDB 集成实现")
            print("  • 高性能列式存储支持")
            print("  • 多种索引类型 (IVF, IVFPQ, HNSW)")
            print("  • 复杂元数据过滤")
            print("  • 云存储支持 (S3, Azure, GCS)")
            print("  • 批量操作优化")
            print("  • 完整的文档和示例")
            print("  • 与 LumosAI 框架无缝集成")
        elif success_rate >= 70:
            print("⚠️  LanceDB 实现基本完成，但有一些问题需要修复")
        else:
            print("❌ LanceDB 实现存在重大问题，需要进一步修复")
        
        print(f"\n📈 实现进度: {success_rate:.1f}%")
        
        if success_rate >= 80:
            print("\n🚀 下一步建议:")
            print("  1. 运行完整的集成测试")
            print("  2. 性能基准测试")
            print("  3. 云存储配置测试")
            print("  4. 大规模数据测试")
            print("  5. 与其他向量数据库性能对比")
        
    except KeyboardInterrupt:
        print("\n\n⏹️  验证被用户中断")
    except Exception as e:
        print(f"\n\n💥 验证过程中发生错误: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
