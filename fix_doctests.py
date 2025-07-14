#!/usr/bin/env python3
"""
修复LumosAI项目中的文档测试问题
将所有 'lumos' 替换为 'lumosai'，并修复Result类型
"""

import os
import re

def fix_doctest_file(filepath):
    """修复单个文件中的文档测试"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 记录是否有修改
        modified = False
        
        # 1. 替换 use lumos:: 为 use lumosai::
        new_content = re.sub(r'use lumos::', 'use lumosai::', content)
        if new_content != content:
            modified = True
            content = new_content
        
        # 2. 替换 lumos:: 调用为 lumosai::
        new_content = re.sub(r'lumos::', 'lumosai::', content)
        if new_content != content:
            modified = True
            content = new_content
        
        # 3. 修复 Result<()> 为完整的Result类型
        new_content = re.sub(
            r'async fn main\(\) -> Result<\(\)>',
            'async fn main() -> std::result::Result<(), Box<dyn std::error::Error>>',
            content
        )
        if new_content != content:
            modified = True
            content = new_content
        
        # 如果有修改，写回文件
        if modified:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ 修复了 {filepath}")
            return True
        else:
            print(f"⏭️  跳过 {filepath} (无需修改)")
            return False
            
    except Exception as e:
        print(f"❌ 修复 {filepath} 时出错: {e}")
        return False

def main():
    """主函数"""
    print("🔧 开始修复LumosAI文档测试...")
    
    # 需要修复的文件列表
    files_to_fix = [
        'src/agent.rs',
        'src/events.rs', 
        'src/orchestration.rs',
        'src/rag.rs',
        'src/session.rs',
        'src/vector.rs'
    ]
    
    fixed_count = 0
    total_count = len(files_to_fix)
    
    for filepath in files_to_fix:
        if os.path.exists(filepath):
            if fix_doctest_file(filepath):
                fixed_count += 1
        else:
            print(f"⚠️  文件不存在: {filepath}")
    
    print(f"\n📊 修复完成: {fixed_count}/{total_count} 个文件被修复")
    print("🎉 文档测试修复完成!")

if __name__ == "__main__":
    main()
