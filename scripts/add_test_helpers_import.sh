#!/bin/bash

# Add test_helpers import to files that use create_test_zhipu_provider_arc

FILES=(
    "lumosai_core/src/agent/mastra_compat.rs"
    "lumosai_core/src/agent/streaming.rs"
    "lumosai_core/src/agent/enhanced_streaming_demo.rs"
    "lumosai_core/src/agent/plan4_api_tests.rs"
    "lumosai_core/src/agent/simplified_api_tests.rs"
    "lumosai_core/src/agent/websocket_demo.rs"
    "lumosai_core/src/agent/websocket.rs"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        # Check if already has the import
        if ! grep -q "use crate::llm::test_helpers" "$file"; then
            # Find the #[cfg(test)] line
            if grep -q "#\[cfg(test)\]" "$file"; then
                # Add import after "use super::*;"
                sed -i '' '/#\[cfg(test)\]/,/use super::\*;/ {
                    /use super::\*;/a\
    use crate::llm::test_helpers::create_test_zhipu_provider_arc;
                }' "$file"
                echo "✅ Added import to: $file"
            else
                echo "⚠️  No #[cfg(test)] found in: $file"
            fi
        else
            echo "✓ Already has import: $file"
        fi
    else
        echo "❌ File not found: $file"
    fi
done

echo ""
echo "🎉 Done!"

