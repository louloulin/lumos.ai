#!/usr/bin/env python3
"""
Replace MockLlmProvider with real Zhipu AI provider in test files.

This script:
1. Finds all files using MockLlmProvider
2. Replaces MockLlmProvider::new() with create_test_zhipu_provider_arc()
3. Updates imports to use test_helpers
4. Preserves file structure and formatting
"""

import os
import re
import sys
from pathlib import Path

def should_skip_file(filepath):
    """Check if file should be skipped"""
    skip_patterns = [
        'target/',
        '.git/',
        'mock.rs',  # Don't modify the mock implementation itself
        'test_helpers.rs',  # Don't modify the helper file
    ]
    
    for pattern in skip_patterns:
        if pattern in str(filepath):
            return True
    return False

def replace_mock_in_file(filepath):
    """Replace MockLlmProvider with Zhipu provider in a single file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        modified = False
        
        # Check if file uses MockLlmProvider
        if 'MockLlmProvider' not in content:
            return False
        
        # Pattern 1: Replace MockLlmProvider::new(vec![...])
        # with create_test_zhipu_provider_arc()
        pattern1 = r'MockLlmProvider::new\(vec!\[[\s\S]*?\]\)'
        if re.search(pattern1, content):
            # For simple cases, just replace with test helper
            content = re.sub(
                r'Arc::new\(MockLlmProvider::new\(vec!\[[\s\S]*?\]\)\)',
                'create_test_zhipu_provider_arc()',
                content
            )
            content = re.sub(
                r'MockLlmProvider::new\(vec!\[[\s\S]*?\]\)',
                'create_test_zhipu_provider()',
                content
            )
            modified = True
        
        # Pattern 2: Update imports
        # Add test_helpers import if not present
        if 'use crate::llm::' in content or 'use lumosai_core::llm::' in content:
            # Check if MockLlmProvider is imported
            if 'MockLlmProvider' in content:
                # Replace MockLlmProvider import with test_helpers
                content = re.sub(
                    r'use (crate|lumosai_core)::llm::\{([^}]*?)MockLlmProvider,?\s*([^}]*?)\}',
                    lambda m: f'use {m.group(1)}::llm::{{{m.group(2)}{m.group(3)}}}\nuse {m.group(1)}::llm::test_helpers::{{create_test_zhipu_provider, create_test_zhipu_provider_arc}};',
                    content
                )
                
                # Also handle single import case
                content = re.sub(
                    r'use (crate|lumosai_core)::llm::MockLlmProvider;',
                    r'use \1::llm::test_helpers::{create_test_zhipu_provider, create_test_zhipu_provider_arc};',
                    content
                )
                modified = True
        
        # Pattern 3: Replace struct definitions that use MockLlmProvider
        content = re.sub(
            r'struct MockLlmProvider\s*\{[\s\S]*?\}',
            '// MockLlmProvider replaced with real Zhipu provider via test_helpers',
            content
        )
        
        # Pattern 4: Replace impl blocks for MockLlmProvider
        content = re.sub(
            r'impl MockLlmProvider\s*\{[\s\S]*?\n\}',
            '// MockLlmProvider implementation replaced with real Zhipu provider',
            content
        )
        
        # Pattern 5: Replace #[async_trait] impl LlmProvider for MockLlmProvider
        content = re.sub(
            r'#\[async_trait\]\s*impl LlmProvider for MockLlmProvider\s*\{[\s\S]*?\n\}',
            '// MockLlmProvider LlmProvider implementation replaced with real Zhipu provider',
            content
        )
        
        # Only write if content changed
        if content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        
        return False
        
    except Exception as e:
        print(f"❌ Error processing {filepath}: {e}")
        return False

def main():
    """Main function"""
    print("🔄 Replacing MockLlmProvider with Zhipu AI provider...")
    print("=" * 60)
    
    # Get all Rust files
    rust_files = []
    for root, dirs, files in os.walk('.'):
        # Skip target and .git directories
        dirs[:] = [d for d in dirs if d not in ['target', '.git']]
        
        for file in files:
            if file.endswith('.rs'):
                filepath = Path(root) / file
                if not should_skip_file(filepath):
                    rust_files.append(filepath)
    
    print(f"📁 Found {len(rust_files)} Rust files to check")
    print()
    
    modified_files = []
    skipped_files = []
    
    for filepath in rust_files:
        if replace_mock_in_file(filepath):
            modified_files.append(filepath)
            print(f"✅ Modified: {filepath}")
        else:
            # Check if file contains MockLlmProvider
            try:
                with open(filepath, 'r', encoding='utf-8') as f:
                    if 'MockLlmProvider' in f.read():
                        skipped_files.append(filepath)
                        print(f"⏭️  Skipped: {filepath} (manual review needed)")
            except:
                pass
    
    print()
    print("=" * 60)
    print(f"📊 Summary:")
    print(f"  - Total files checked: {len(rust_files)}")
    print(f"  - Files modified: {len(modified_files)}")
    print(f"  - Files skipped: {len(skipped_files)}")
    
    if modified_files:
        print()
        print("✅ Modified files:")
        for f in modified_files:
            print(f"  - {f}")
    
    if skipped_files:
        print()
        print("⚠️  Files needing manual review:")
        for f in skipped_files:
            print(f"  - {f}")
    
    print()
    print("🎉 Replacement complete!")
    print()
    print("📝 Next steps:")
    print("  1. Review the changes: git diff")
    print("  2. Run tests: cargo test --workspace")
    print("  3. Fix any compilation errors manually")
    print("  4. Commit changes: git add -A && git commit")

if __name__ == '__main__':
    main()

