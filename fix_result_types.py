#!/usr/bin/env python3
"""
修复 Result 类型问题的脚本
将 Result<T, Box<dyn std::error::Error>> 替换为 std::result::Result<T, Box<dyn std::error::Error>>
"""

import os
import re
import glob

def fix_result_types_in_file(file_path):
    """修复单个文件中的 Result 类型"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 替换模式
        patterns = [
            # 函数返回类型
            (r'-> Result<([^>]+), Box<dyn std::error::Error([^>]*)>>', r'-> std::result::Result<\1, Box<dyn std::error::Error\2>>'),
            # 变量类型声明
            (r': Result<([^>]+), Box<dyn std::error::Error([^>]*)>>', r': std::result::Result<\1, Box<dyn std::error::Error\2>>'),
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
            print(f"✅ 修复了 {file_path}")
            return True
        else:
            return False
            
    except Exception as e:
        print(f"❌ 修复 {file_path} 时出错: {e}")
        return False

def main():
    """主函数"""
    print("🔧 修复 Result 类型问题...")
    
    # 需要修复的文件模式
    file_patterns = [
        "examples/*.rs",
        "lumosai_core/examples/*.rs",
        "tests/**/*.rs",
    ]
    
    fixed_count = 0
    total_count = 0
    
    for pattern in file_patterns:
        for file_path in glob.glob(pattern, recursive=True):
            if os.path.isfile(file_path):
                total_count += 1
                if fix_result_types_in_file(file_path):
                    fixed_count += 1
    
    print(f"\n📊 修复完成:")
    print(f"  - 检查文件: {total_count}")
    print(f"  - 修复文件: {fixed_count}")
    
    if fixed_count > 0:
        print("\n🎉 Result 类型问题修复完成!")
    else:
        print("\n✨ 没有发现需要修复的 Result 类型问题")

if __name__ == "__main__":
    main()
