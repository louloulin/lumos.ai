#!/usr/bin/env python3
"""
Simple script to replace MockLlmProvider with Zhipu AI provider
"""

import os
import re
import sys

def process_file(filepath):
    """Process a single file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original = content
        
        # Skip if no MockLlmProvider
        if 'MockLlmProvider' not in content:
            return False
        
        # Skip mock.rs and test_helpers.rs
        if 'mock.rs' in filepath or 'test_helpers.rs' in filepath:
            return False
        
        # Skip files with struct MockLlmProvider (need manual review)
        if 'struct MockLlmProvider' in content:
            print(f"  ⚠️  Skipped (has struct definition): {filepath}")
            return False
        
        # Replace Arc::new(MockLlmProvider::new(...))
        content = re.sub(
            r'Arc::new\(MockLlmProvider::new\(vec!\[[^\]]*\]\)\)',
            'create_test_zhipu_provider_arc()',
            content
        )
        
        # Replace MockLlmProvider::new(...)
        content = re.sub(
            r'MockLlmProvider::new\(vec!\[[^\]]*\]\)',
            'create_test_zhipu_provider()',
            content
        )
        
        # Add test_helpers import if needed
        if 'create_test_zhipu_provider' in content:
            # Check if already imported
            if 'test_helpers' not in content:
                # Add import after llm imports
                if 'use crate::llm::' in content:
                    content = re.sub(
                        r'(use crate::llm::\{[^}]+\};)',
                        r'\1\n    use crate::llm::test_helpers::{create_test_zhipu_provider, create_test_zhipu_provider_arc};',
                        content,
                        count=1
                    )
                elif 'use lumosai_core::llm::' in content:
                    content = re.sub(
                        r'(use lumosai_core::llm::\{[^}]+\};)',
                        r'\1\n    use lumosai_core::llm::test_helpers::{create_test_zhipu_provider, create_test_zhipu_provider_arc};',
                        content,
                        count=1
                    )
            
            # Remove MockLlmProvider from imports
            content = re.sub(r', MockLlmProvider', '', content)
            content = re.sub(r'MockLlmProvider, ', '', content)
            content = re.sub(r'use (crate|lumosai_core)::llm::MockLlmProvider;\n', '', content)
        
        # Write if changed
        if content != original:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        
        return False
        
    except Exception as e:
        print(f"  ❌ Error: {filepath}: {e}")
        return False

def main():
    print("🔄 Replacing MockLlmProvider with Zhipu AI provider...")
    print("=" * 60)
    
    # Find all Rust files with MockLlmProvider
    modified = 0
    total = 0
    
    for root, dirs, files in os.walk('.'):
        # Skip target and .git
        dirs[:] = [d for d in dirs if d not in ['target', '.git']]
        
        for file in files:
            if file.endswith('.rs'):
                filepath = os.path.join(root, file)
                
                # Check if file contains MockLlmProvider
                try:
                    with open(filepath, 'r', encoding='utf-8') as f:
                        if 'MockLlmProvider' in f.read():
                            total += 1
                            print(f"\n📝 Processing: {filepath}")
                            if process_file(filepath):
                                modified += 1
                                print(f"  ✅ Modified")
                            else:
                                print(f"  ⏭️  Skipped")
                except:
                    pass
    
    print("\n" + "=" * 60)
    print(f"📊 Summary:")
    print(f"  - Files with MockLlmProvider: {total}")
    print(f"  - Files modified: {modified}")
    print(f"\n🎉 Done!")

if __name__ == '__main__':
    main()

