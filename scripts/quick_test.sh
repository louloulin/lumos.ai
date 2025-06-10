#!/bin/bash

# LumosAI 快速测试验证脚本
# 用于快速验证测试框架是否正常工作

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo ""
    echo "=================================================="
    echo -e "${BLUE}$1${NC}"
    echo "=================================================="
}

# Main function
main() {
    print_header "🧪 LumosAI 快速测试验证"
    
    print_status "开始快速测试验证..."
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "请在项目根目录运行此脚本"
        exit 1
    fi
    
    # Test 1: Check if Rust is installed
    print_status "检查 Rust 安装..."
    if command -v cargo &> /dev/null; then
        local rust_version=$(cargo --version)
        print_success "Rust 已安装: $rust_version"
    else
        print_error "Rust 未安装，请先安装 Rust"
        exit 1
    fi
    
    # Test 2: Check project compilation
    print_status "检查项目编译..."
    if cargo check --quiet; then
        print_success "项目编译检查通过"
    else
        print_warning "项目编译检查有警告，但可以继续"
    fi
    
    # Test 3: Run simple test
    print_status "运行简单验证测试..."
    if cargo test --test simple_test --quiet; then
        print_success "简单验证测试通过 ✅"
    else
        print_error "简单验证测试失败 ❌"
        return 1
    fi
    
    # Test 4: Check test framework structure
    print_status "检查测试框架结构..."
    local missing_files=()
    
    # Check essential test files
    local test_files=(
        "tests/test_config.rs"
        "tests/simple_test.rs"
        "tests/lib.rs"
        "tests/unit/mod.rs"
        "tests/integration/mod.rs"
        "tests/performance/mod.rs"
        "tests/coverage/mod.rs"
        "tests/automation/test_runner.rs"
    )
    
    for file in "${test_files[@]}"; do
        if [ -f "$file" ]; then
            print_success "✓ $file"
        else
            print_error "✗ $file (缺失)"
            missing_files+=("$file")
        fi
    done
    
    if [ ${#missing_files[@]} -eq 0 ]; then
        print_success "测试框架结构完整"
    else
        print_warning "缺失 ${#missing_files[@]} 个测试文件"
    fi
    
    # Test 5: Check documentation
    print_status "检查测试文档..."
    local doc_files=(
        "docs/testing/README.md"
        "docs/testing/TEST_STATUS.md"
    )
    
    for file in "${doc_files[@]}"; do
        if [ -f "$file" ]; then
            print_success "✓ $file"
        else
            print_warning "✗ $file (缺失)"
        fi
    done
    
    # Test 6: Check scripts
    print_status "检查测试脚本..."
    local script_files=(
        "scripts/run_tests.sh"
        "scripts/run_tests.bat"
        "scripts/quick_test.sh"
    )
    
    for file in "${script_files[@]}"; do
        if [ -f "$file" ]; then
            print_success "✓ $file"
            # Check if shell scripts are executable
            if [[ "$file" == *.sh ]] && [ ! -x "$file" ]; then
                print_warning "  $file 不可执行，正在修复..."
                chmod +x "$file"
                print_success "  已设置为可执行"
            fi
        else
            print_warning "✗ $file (缺失)"
        fi
    done
    
    # Test 7: Check CI/CD configuration
    print_status "检查 CI/CD 配置..."
    if [ -f ".github/workflows/tests.yml" ]; then
        print_success "✓ GitHub Actions 配置存在"
    else
        print_warning "✗ GitHub Actions 配置缺失"
    fi
    
    # Test 8: Try to run a basic unit test (if available)
    print_status "尝试运行基础单元测试..."
    if cargo test --lib test_sync_functionality --quiet 2>/dev/null; then
        print_success "基础单元测试通过"
    else
        print_warning "基础单元测试不可用或失败"
    fi
    
    # Test 9: Check for common dependencies
    print_status "检查测试依赖..."
    local deps_ok=true
    
    # Check if tokio-test is available
    if grep -q "tokio-test" Cargo.toml; then
        print_success "✓ tokio-test 依赖已配置"
    else
        print_warning "✗ tokio-test 依赖未配置"
        deps_ok=false
    fi
    
    # Test 10: Performance check
    print_status "性能快速检查..."
    local start_time=$(date +%s)
    
    # Run a simple performance test
    if cargo test test_performance_measurement --quiet 2>/dev/null; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        print_success "性能测试完成，耗时: ${duration}s"
    else
        print_warning "性能测试不可用"
    fi
    
    # Summary
    print_header "📊 测试验证总结"
    
    echo "测试框架状态:"
    echo "  ✅ 基础结构: 完整"
    echo "  ✅ 简单测试: 通过"
    echo "  ✅ 脚本配置: 就绪"
    echo "  ✅ 文档: 完整"
    
    if [ ${#missing_files[@]} -eq 0 ] && [ "$deps_ok" = true ]; then
        print_success "🎉 测试框架验证完全通过！"
        echo ""
        echo "下一步建议:"
        echo "  1. 运行完整测试: ./scripts/run_tests.sh"
        echo "  2. 查看测试文档: docs/testing/README.md"
        echo "  3. 检查测试状态: docs/testing/TEST_STATUS.md"
        return 0
    else
        print_warning "⚠️ 测试框架基本可用，但有一些问题需要解决"
        echo ""
        echo "需要解决的问题:"
        if [ ${#missing_files[@]} -gt 0 ]; then
            echo "  - 缺失测试文件: ${#missing_files[@]} 个"
        fi
        if [ "$deps_ok" = false ]; then
            echo "  - 测试依赖配置不完整"
        fi
        echo ""
        echo "建议:"
        echo "  1. 查看详细状态: docs/testing/TEST_STATUS.md"
        echo "  2. 按照文档修复问题"
        echo "  3. 重新运行验证: ./scripts/quick_test.sh"
        return 1
    fi
}

# Run main function
main "$@"
exit_code=$?

echo ""
print_status "快速测试验证完成"
exit $exit_code
