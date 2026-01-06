#!/bin/bash
# scripts/verify_crates_structure.sh
# 验证 crates 目录结构是否正确创建

set -e

echo "🔍 Verifying crates directory structure..."
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 计数器
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

# 检查函数
check_dir() {
    local dir=$1
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    if [ -d "$dir" ]; then
        echo -e "${GREEN}✓${NC} Directory exists: $dir"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        return 0
    else
        echo -e "${RED}✗${NC} Directory missing: $dir"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        return 1
    fi
}

# 检查文件
check_file() {
    local file=$1
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} File exists: $file"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        return 0
    else
        echo -e "${RED}✗${NC} File missing: $file"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        return 1
    fi
}

echo "📁 Checking main crates directory..."
check_dir "crates"
echo ""

echo "📁 Checking layer directories..."
check_dir "crates/core"
check_dir "crates/runtime"
check_dir "crates/services"
check_dir "crates/extensions"
check_dir "crates/tools"
check_dir "crates/bindings"
check_dir "crates/integrations"
echo ""

echo "📁 Checking Core Layer crates..."
check_dir "crates/core/lumosai-types"
check_dir "crates/core/lumosai-error"
check_dir "crates/core/lumosai-config"
check_dir "crates/core/lumosai-logger"
echo ""

echo "📁 Checking Runtime Layer crates..."
check_dir "crates/runtime/lumosai-agent"
check_dir "crates/runtime/lumosai-workflow"
check_dir "crates/runtime/lumosai-tool"
check_dir "crates/runtime/lumosai-memory"
check_dir "crates/runtime/lumosai-llm"
check_dir "crates/runtime/lumosai-rag"
check_dir "crates/runtime/lumosai-vector"
echo ""

echo "📁 Checking Services Layer crates..."
check_dir "crates/services/lumosai-mcp"
check_dir "crates/services/lumosai-network"
check_dir "crates/services/lumosai-auth"
check_dir "crates/services/lumosai-security"
check_dir "crates/services/lumosai-telemetry"
check_dir "crates/services/lumosai-enterprise"
echo ""

echo "📁 Checking Extensions Layer crates..."
check_dir "crates/extensions/lumosai-multimodal"
check_dir "crates/extensions/lumosai-voice"
check_dir "crates/extensions/lumosai-cloud"
check_dir "crates/extensions/lumosai-evals"
echo ""

echo "📁 Checking Tools Layer crates..."
check_dir "crates/tools/lumosai-cli"
check_dir "crates/tools/lumosai-macro"
check_dir "crates/tools/lumosai-derive"
echo ""

echo "📁 Checking Bindings Layer crates..."
check_dir "crates/bindings/lumosai-bindings"
echo ""

echo "📄 Checking README files..."
check_file "crates/README.md"
check_file "crates/integrations/README.md"
echo ""

# 统计目录数量
TOTAL_DIRS=$(find crates -type d | wc -l | tr -d ' ')
echo "📊 Statistics:"
echo "  Total directories: $TOTAL_DIRS"
echo "  Total checks: $TOTAL_CHECKS"
echo "  Passed: $PASSED_CHECKS"
echo "  Failed: $FAILED_CHECKS"
echo ""

# 最终结果
if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed!${NC}"
    echo ""
    echo "📋 Directory structure summary:"
    echo "  - Core Layer: 4 crates"
    echo "  - Runtime Layer: 7 crates"
    echo "  - Services Layer: 6 crates"
    echo "  - Extensions Layer: 4 crates"
    echo "  - Tools Layer: 3 crates"
    echo "  - Bindings Layer: 1 crate"
    echo "  - Total: 25 crates"
    echo ""
    exit 0
else
    echo -e "${RED}❌ Some checks failed!${NC}"
    echo "Please fix the missing directories/files and run again."
    echo ""
    exit 1
fi

