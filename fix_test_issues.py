#!/usr/bin/env python3
"""
修复测试问题的脚本
"""

import os
import re
import glob

def fix_await_on_build(file_path):
    """修复 .build().await 问题"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 修复 Agent::builder().build().await 模式
        patterns = [
            (r'\.build\(\)\s*\.await\s*\.unwrap\(\)', r'.build().unwrap()'),
            (r'\.build\(\)\s*\.await\s*\?', r'.build()?'),
            (r'\.build\(\)\s*\.await', r'.build()'),
        ]
        
        modified = False
        for pattern, replacement in patterns:
            new_content = re.sub(pattern, replacement, content)
            if new_content != content:
                content = new_content
                modified = True
        
        if modified:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ 修复了 {file_path} 中的 .await 问题")
            return True
        return False
            
    except Exception as e:
        print(f"❌ 修复 {file_path} 时出错: {e}")
        return False

def fix_generate_simple_calls(file_path):
    """修复 generate_simple 调用问题"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 检查是否有 generate_simple 调用但没有方法定义
        if 'generate_simple' in content and 'AgentTrait' not in content:
            # 这是调用文件，不需要修改
            return False
        
        return False
            
    except Exception as e:
        print(f"❌ 修复 {file_path} 时出错: {e}")
        return False

def fix_rag_test_params(file_path):
    """修复 RAG 测试参数顺序问题"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 修复 create_test_rag_system 调用
        # 从 create_test_rag_system(storage, "name") 改为 create_test_rag_system("name", storage)
        pattern = r'create_test_rag_system\(([^,]+),\s*"([^"]+)"\)'
        replacement = r'create_test_rag_system("\2", \1)'
        
        new_content = re.sub(pattern, replacement, content)
        if new_content != content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f"✅ 修复了 {file_path} 中的 RAG 测试参数")
            return True
        
        return False
            
    except Exception as e:
        print(f"❌ 修复 {file_path} 时出错: {e}")
        return False

def fix_type_mismatches(file_path):
    """修复类型不匹配问题"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 修复 Agent 类型不匹配
        patterns = [
            # 修复 Arc<dyn Agent> vs Arc<dyn AgentTrait> 问题
            (r'Arc<dyn Agent>', r'Arc<dyn AgentTrait>'),
            # 修复其他常见类型问题
            (r'&Arc<dyn AgentTrait>', r'Arc<dyn AgentTrait>'),
        ]
        
        modified = False
        for pattern, replacement in patterns:
            new_content = re.sub(pattern, replacement, content)
            if new_content != content:
                content = new_content
                modified = True
        
        if modified:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ 修复了 {file_path} 中的类型不匹配")
            return True
        
        return False
            
    except Exception as e:
        print(f"❌ 修复 {file_path} 时出错: {e}")
        return False

def main():
    """主函数"""
    print("🔧 修复测试问题...")
    
    # 需要修复的文件模式
    file_patterns = [
        "tests/**/*.rs",
        "examples/*.rs",
        "lumosai_core/examples/*.rs",
    ]
    
    fixed_count = 0
    total_count = 0
    
    for pattern in file_patterns:
        for file_path in glob.glob(pattern, recursive=True):
            if os.path.isfile(file_path):
                total_count += 1
                
                # 应用各种修复
                fixes = [
                    fix_await_on_build(file_path),
                    fix_generate_simple_calls(file_path),
                    fix_rag_test_params(file_path),
                    fix_type_mismatches(file_path),
                ]
                
                if any(fixes):
                    fixed_count += 1
    
    print(f"\n📊 修复完成:")
    print(f"  - 检查文件: {total_count}")
    print(f"  - 修复文件: {fixed_count}")
    
    if fixed_count > 0:
        print("\n🎉 测试问题修复完成!")
    else:
        print("\n✨ 没有发现需要修复的测试问题")

if __name__ == "__main__":
    main()
