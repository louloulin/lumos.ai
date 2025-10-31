#!/bin/bash
#
# Replace MockLlmProvider with Zhipu AI provider in all test files
#
# Usage: ./scripts/replace_mock_providers.sh

set -e

echo "🔄 Replacing MockLlmProvider with Zhipu AI provider..."
echo "=" | tr '=' '=' | head -c 60; echo

# Counter
total_files=0
modified_files=0

# Find all Rust files containing MockLlmProvider
files=$(grep -r "MockLlmProvider" --include="*.rs" . 2>/dev/null | \
        grep -v "target/" | \
        grep -v ".git/" | \
        grep -v "mock.rs" | \
        grep -v "test_helpers.rs" | \
        cut -d: -f1 | \
        sort -u)

for file in $files; do
    total_files=$((total_files + 1))
    
    # Skip if file doesn't exist
    if [ ! -f "$file" ]; then
        continue
    fi
    
    echo "📝 Processing: $file"
    
    # Create backup
    cp "$file" "$file.bak"
    
    # Replace patterns
    modified=0
    
    # Pattern 1: Arc::new(MockLlmProvider::new(vec![...]))
    if grep -q "Arc::new(MockLlmProvider::new(vec!\[" "$file"; then
        sed -i '' 's/Arc::new(MockLlmProvider::new(vec!\[\]))/create_test_zhipu_provider_arc()/g' "$file"
        sed -i '' 's/Arc::new(MockLlmProvider::new(vec!\[.*\]))/create_test_zhipu_provider_arc()/g' "$file"
        modified=1
    fi
    
    # Pattern 2: MockLlmProvider::new(vec![...])
    if grep -q "MockLlmProvider::new(vec!\[" "$file"; then
        sed -i '' 's/MockLlmProvider::new(vec!\[\])/create_test_zhipu_provider()/g' "$file"
        sed -i '' 's/MockLlmProvider::new(vec!\[.*\])/create_test_zhipu_provider()/g' "$file"
        modified=1
    fi
    
    # Pattern 3: Update imports - remove MockLlmProvider, add test_helpers
    if grep -q "use.*MockLlmProvider" "$file"; then
        # For crate::llm imports
        if grep -q "use crate::llm::" "$file"; then
            # Check if test_helpers is already imported
            if ! grep -q "use crate::llm::test_helpers" "$file"; then
                # Add test_helpers import after llm imports
                sed -i '' '/use crate::llm::/a\
    use crate::llm::test_helpers::{create_test_zhipu_provider, create_test_zhipu_provider_arc};
' "$file"
            fi
            # Remove MockLlmProvider from imports
            sed -i '' 's/, MockLlmProvider//g' "$file"
            sed -i '' 's/MockLlmProvider, //g' "$file"
            sed -i '' 's/use crate::llm::MockLlmProvider;//d' "$file"
            modified=1
        fi
        
        # For lumosai_core::llm imports
        if grep -q "use lumosai_core::llm::" "$file"; then
            # Check if test_helpers is already imported
            if ! grep -q "use lumosai_core::llm::test_helpers" "$file"; then
                # Add test_helpers import after llm imports
                sed -i '' '/use lumosai_core::llm::/a\
    use lumosai_core::llm::test_helpers::{create_test_zhipu_provider, create_test_zhipu_provider_arc};
' "$file"
            fi
            # Remove MockLlmProvider from imports
            sed -i '' 's/, MockLlmProvider//g' "$file"
            sed -i '' 's/MockLlmProvider, //g' "$file"
            sed -i '' 's/use lumosai_core::llm::MockLlmProvider;//d' "$file"
            modified=1
        fi
    fi
    
    # Pattern 4: Remove MockLlmProvider struct definitions
    if grep -q "struct MockLlmProvider" "$file"; then
        # This is complex, skip for now and mark for manual review
        echo "  ⚠️  Contains MockLlmProvider struct definition - needs manual review"
        # Restore backup
        mv "$file.bak" "$file"
        continue
    fi
    
    # Check if file was actually modified
    if [ $modified -eq 1 ]; then
        if ! diff -q "$file" "$file.bak" > /dev/null 2>&1; then
            echo "  ✅ Modified"
            modified_files=$((modified_files + 1))
            rm "$file.bak"
        else
            echo "  ⏭️  No changes needed"
            rm "$file.bak"
        fi
    else
        echo "  ⏭️  Skipped (no patterns matched)"
        rm "$file.bak"
    fi
done

echo
echo "=" | tr '=' '=' | head -c 60; echo
echo "📊 Summary:"
echo "  - Total files processed: $total_files"
echo "  - Files modified: $modified_files"
echo
echo "🎉 Replacement complete!"
echo
echo "📝 Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Run tests: cargo test --workspace 2>&1 | grep 'test result'"
echo "  3. Fix any compilation errors"
echo "  4. Commit: git add -A && git commit -m 'Replace MockLlmProvider with Zhipu AI'"

