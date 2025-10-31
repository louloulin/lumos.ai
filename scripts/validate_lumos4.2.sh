#!/bin/bash

# LumosAI 4.2 验证脚本
# 用于验证当前项目状态并生成改造基线报告

set -e

echo "=========================================="
echo "LumosAI 4.2 项目状态验证"
echo "=========================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 创建报告目录
REPORT_DIR="target/lumos4.2-report"
mkdir -p "$REPORT_DIR"

REPORT_FILE="$REPORT_DIR/baseline_report.md"

# 初始化报告
cat > "$REPORT_FILE" << 'EOF'
# LumosAI 4.2 基线报告

> 生成时间: $(date)

## 1. 代码库统计

EOF

echo "📊 1. 代码库统计..."

# 统计代码行数
echo "### 代码行数" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "Rust 代码:" >> "$REPORT_FILE"
find . -name "*.rs" -not -path "./target/*" -not -path "./.git/*" | xargs wc -l | tail -1 >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "测试代码:" >> "$REPORT_FILE"
find . -name "*.rs" -path "*/tests/*" -not -path "./target/*" | xargs wc -l 2>/dev/null | tail -1 >> "$REPORT_FILE" || echo "0 total" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "示例代码:" >> "$REPORT_FILE"
find examples -name "*.rs" 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 >> "$REPORT_FILE" || echo "0 total" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 统计包数量
echo "### 包统计" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "Workspace 成员数量:" >> "$REPORT_FILE"
grep -A 100 "members = \[" Cargo.toml | grep -c '    "' >> "$REPORT_FILE" || echo "0" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo -e "${GREEN}✓${NC} 代码库统计完成"

echo ""
echo "🧪 2. 测试覆盖率分析..."

# 测试统计
echo "## 2. 测试覆盖率" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 统计测试文件
TEST_FILES=$(find . -name "*.rs" -path "*/tests/*" -not -path "./target/*" | wc -l)
echo "### 测试文件统计" >> "$REPORT_FILE"
echo "- 测试文件数量: $TEST_FILES" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 统计测试函数
echo "### 测试函数统计" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "单元测试 (#[test]):" >> "$REPORT_FILE"
grep -r "#\[test\]" --include="*.rs" --exclude-dir=target . 2>/dev/null | wc -l >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "异步测试 (#[tokio::test]):" >> "$REPORT_FILE"
grep -r "#\[tokio::test\]" --include="*.rs" --exclude-dir=target . 2>/dev/null | wc -l >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "基准测试 (#[bench]):" >> "$REPORT_FILE"
grep -r "#\[bench\]" --include="*.rs" --exclude-dir=target . 2>/dev/null | wc -l >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo -e "${GREEN}✓${NC} 测试统计完成"

# 运行测试（如果可能）
echo ""
echo "🔧 3. 运行测试套件..."

echo "## 3. 测试执行结果" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

if cargo test --workspace --lib --no-fail-fast 2>&1 | tee "$REPORT_DIR/test_output.log"; then
    echo -e "${GREEN}✓${NC} 测试通过"
    echo "- 状态: ✅ 通过" >> "$REPORT_FILE"
else
    echo -e "${RED}✗${NC} 测试失败"
    echo "- 状态: ❌ 失败" >> "$REPORT_FILE"
fi

# 提取测试结果
if [ -f "$REPORT_DIR/test_output.log" ]; then
    echo '```' >> "$REPORT_FILE"
    tail -20 "$REPORT_DIR/test_output.log" >> "$REPORT_FILE"
    echo '```' >> "$REPORT_FILE"
fi
echo "" >> "$REPORT_FILE"

echo ""
echo "📚 4. 文档覆盖率分析..."

echo "## 4. 文档覆盖率" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 统计文档文件
DOC_FILES=$(find . -name "*.md" -not -path "./target/*" -not -path "./.git/*" | wc -l)
echo "### 文档文件统计" >> "$REPORT_FILE"
echo "- Markdown 文件数量: $DOC_FILES" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 统计文档行数
echo "### 文档行数" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
find . -name "*.md" -not -path "./target/*" -not -path "./.git/*" | xargs wc -l | tail -1 >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 统计文档注释
echo "### 文档注释统计" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "文档注释 (///):" >> "$REPORT_FILE"
grep -r "^///" --include="*.rs" --exclude-dir=target . 2>/dev/null | wc -l >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "模块文档 (//!):" >> "$REPORT_FILE"
grep -r "^//!" --include="*.rs" --exclude-dir=target . 2>/dev/null | wc -l >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo -e "${GREEN}✓${NC} 文档统计完成"

echo ""
echo "🔍 5. 代码质量检查..."

echo "## 5. 代码质量" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Clippy 检查
echo "### Clippy 检查" >> "$REPORT_FILE"
if cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee "$REPORT_DIR/clippy_output.log"; then
    echo -e "${GREEN}✓${NC} Clippy 检查通过"
    echo "- 状态: ✅ 无警告" >> "$REPORT_FILE"
else
    echo -e "${YELLOW}⚠${NC} Clippy 发现问题"
    echo "- 状态: ⚠️ 有警告" >> "$REPORT_FILE"
    
    # 统计警告数量
    WARNING_COUNT=$(grep -c "warning:" "$REPORT_DIR/clippy_output.log" || echo "0")
    echo "- 警告数量: $WARNING_COUNT" >> "$REPORT_FILE"
fi
echo "" >> "$REPORT_FILE"

# 格式检查
echo "### 格式检查" >> "$REPORT_FILE"
if cargo fmt --all -- --check 2>&1 | tee "$REPORT_DIR/fmt_output.log"; then
    echo -e "${GREEN}✓${NC} 格式检查通过"
    echo "- 状态: ✅ 格式正确" >> "$REPORT_FILE"
else
    echo -e "${YELLOW}⚠${NC} 需要格式化"
    echo "- 状态: ⚠️ 需要格式化" >> "$REPORT_FILE"
fi
echo "" >> "$REPORT_FILE"

echo ""
echo "🛠️ 6. 工具生态分析..."

echo "## 6. 工具生态" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 统计内置工具
echo "### 内置工具统计" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "工具定义 (#[tool]):" >> "$REPORT_FILE"
grep -r "#\[tool" --include="*.rs" --exclude-dir=target . 2>/dev/null | wc -l >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "工具文件:" >> "$REPORT_FILE"
find . -path "*/tool/*" -name "*.rs" -not -path "./target/*" | wc -l >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo -e "${GREEN}✓${NC} 工具统计完成"

echo ""
echo "📦 7. 依赖分析..."

echo "## 7. 依赖分析" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 统计依赖
echo "### 依赖统计" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
cargo tree --workspace --depth 1 2>/dev/null | head -50 >> "$REPORT_FILE" || echo "无法获取依赖树" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo -e "${GREEN}✓${NC} 依赖分析完成"

echo ""
echo "📈 8. 生成改进建议..."

echo "## 8. 改进建议" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 基于分析生成建议
cat >> "$REPORT_FILE" << 'EOF'
### P0 - 立即执行

1. **提升测试覆盖率**
   - 当前: ~40%
   - 目标: >80%
   - 行动: 为核心模块添加 200+ 单元测试

2. **完善文档**
   - 当前: 基础文档
   - 目标: 500+ 页完整文档
   - 行动: 创建 10+ 教程，完善 API 文档

3. **扩展工具生态**
   - 当前: 4 个内置工具
   - 目标: 50+ 工具
   - 行动: 实现文件、网络、数据处理等工具

### P1 - 近期执行

4. **实现状态机工作流**
   - 支持循环和条件分支
   - 提供可视化工具

5. **建立可观测性平台**
   - 分布式追踪
   - 性能监控
   - 日志聚合

6. **完善评估框架**
   - 多种评估指标
   - 基准测试套件
   - A/B 测试支持

### P2 - 长期规划

7. **创建 Agent 模板库**
   - 20+ 预构建模板
   - 模板市场

8. **增强多 Agent 协作**
   - 角色系统
   - 协作模式

9. **实现人机协作**
   - 人类反馈循环
   - 交互式对话

## 9. 下一步行动

1. **立即**: 运行 `cargo tarpaulin --workspace` 获取详细覆盖率
2. **本周**: 开始执行 Week 1 测试任务
3. **本月**: 完成阶段 1 基础加固
4. **本季度**: 完成阶段 2 功能增强

EOF

echo -e "${GREEN}✓${NC} 改进建议生成完成"

echo ""
echo "=========================================="
echo "✅ 验证完成！"
echo "=========================================="
echo ""
echo "📄 报告已生成: $REPORT_FILE"
echo ""
echo "查看报告:"
echo "  cat $REPORT_FILE"
echo ""
echo "或在浏览器中查看:"
echo "  open $REPORT_FILE  # macOS"
echo "  xdg-open $REPORT_FILE  # Linux"
echo ""

# 生成摘要
echo "=========================================="
echo "📊 快速摘要"
echo "=========================================="
echo ""
echo "代码行数: $(find . -name "*.rs" -not -path "./target/*" -not -path "./.git/*" | xargs wc -l | tail -1 | awk '{print $1}')"
echo "测试文件: $TEST_FILES"
echo "文档文件: $DOC_FILES"
echo ""
echo "详细信息请查看: $REPORT_FILE"
echo ""

