#!/bin/bash

# LumosAI 发布前检查脚本
# 确保项目在发布前满足所有质量要求

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查计数器
CHECKS_PASSED=0
CHECKS_FAILED=0
CHECKS_WARNING=0

# 记录检查结果
record_check() {
    local status=$1
    local message=$2
    
    case $status in
        "pass")
            log_success "$message"
            ((CHECKS_PASSED++))
            ;;
        "fail")
            log_error "$message"
            ((CHECKS_FAILED++))
            ;;
        "warn")
            log_warning "$message"
            ((CHECKS_WARNING++))
            ;;
    esac
}

# 检查 Git 状态
check_git_status() {
    log_info "检查 Git 状态..."
    
    # 检查是否在 Git 仓库中
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        record_check "fail" "当前目录不是 Git 仓库"
        return 1
    fi
    
    # 检查是否有未提交的更改
    if ! git diff-index --quiet HEAD --; then
        record_check "warn" "工作目录有未提交的更改"
    else
        record_check "pass" "工作目录干净"
    fi
    
    # 检查是否在正确的分支
    current_branch=$(git branch --show-current)
    if [[ "$current_branch" != "main" && "$current_branch" != "master" ]]; then
        record_check "warn" "当前分支不是 main/master: $current_branch"
    else
        record_check "pass" "在正确的发布分支: $current_branch"
    fi
    
    # 检查是否与远程同步
    if git status --porcelain=v1 2>/dev/null | grep -q "^##.*ahead"; then
        record_check "warn" "本地分支领先于远程分支"
    elif git status --porcelain=v1 2>/dev/null | grep -q "^##.*behind"; then
        record_check "fail" "本地分支落后于远程分支"
        return 1
    else
        record_check "pass" "与远程分支同步"
    fi
}

# 检查版本一致性
check_version_consistency() {
    log_info "检查版本一致性..."
    
    # 获取根 Cargo.toml 的版本
    root_version=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    
    if [[ -z "$root_version" ]]; then
        record_check "fail" "无法读取根 Cargo.toml 的版本"
        return 1
    fi
    
    log_info "根版本: $root_version"
    
    # 检查所有子包的版本
    local inconsistent_packages=()
    
    find . -name "Cargo.toml" -not -path "./target/*" | while read -r cargo_file; do
        if [[ "$cargo_file" != "./Cargo.toml" ]]; then
            package_version=$(grep '^version = ' "$cargo_file" | sed 's/version = "\(.*\)"/\1/')
            package_name=$(grep '^name = ' "$cargo_file" | sed 's/name = "\(.*\)"/\1/')
            
            if [[ "$package_version" != "$root_version" ]]; then
                echo "$package_name:$package_version" >> /tmp/inconsistent_packages
            fi
        fi
    done
    
    if [[ -f /tmp/inconsistent_packages ]]; then
        record_check "fail" "版本不一致的包:"
        while read -r line; do
            log_error "  $line (期望: $root_version)"
        done < /tmp/inconsistent_packages
        rm -f /tmp/inconsistent_packages
        return 1
    else
        record_check "pass" "所有包版本一致: $root_version"
    fi
}

# 检查代码格式
check_code_format() {
    log_info "检查代码格式..."
    
    if cargo fmt --all -- --check > /dev/null 2>&1; then
        record_check "pass" "代码格式正确"
    else
        record_check "fail" "代码格式不正确，请运行 'cargo fmt'"
        return 1
    fi
}

# 运行 Clippy 检查
check_clippy() {
    log_info "运行 Clippy 检查..."
    
    local clippy_output
    clippy_output=$(cargo clippy --all-features --workspace -- -D warnings 2>&1)
    local clippy_exit_code=$?
    
    if [[ $clippy_exit_code -eq 0 ]]; then
        record_check "pass" "Clippy 检查通过"
    else
        record_check "fail" "Clippy 检查失败"
        echo "$clippy_output"
        return 1
    fi
}

# 运行测试
check_tests() {
    log_info "运行测试套件..."
    
    # 运行单元测试
    if cargo test --all-features --workspace > /dev/null 2>&1; then
        record_check "pass" "单元测试通过"
    else
        record_check "fail" "单元测试失败"
        return 1
    fi
    
    # 运行文档测试
    if cargo test --doc --all-features --workspace > /dev/null 2>&1; then
        record_check "pass" "文档测试通过"
    else
        record_check "warn" "文档测试失败"
    fi
}

# 检查文档
check_documentation() {
    log_info "检查文档..."
    
    # 构建文档
    if cargo doc --all-features --workspace --no-deps > /dev/null 2>&1; then
        record_check "pass" "文档构建成功"
    else
        record_check "fail" "文档构建失败"
        return 1
    fi
    
    # 检查 README 文件
    if [[ -f "README.md" ]]; then
        record_check "pass" "README.md 存在"
    else
        record_check "warn" "README.md 不存在"
    fi
    
    # 检查 CHANGELOG 文件
    if [[ -f "CHANGELOG.md" ]]; then
        record_check "pass" "CHANGELOG.md 存在"
    else
        record_check "warn" "CHANGELOG.md 不存在"
    fi
    
    # 检查 LICENSE 文件
    if [[ -f "LICENSE" || -f "LICENSE.md" || -f "LICENSE.txt" ]]; then
        record_check "pass" "LICENSE 文件存在"
    else
        record_check "warn" "LICENSE 文件不存在"
    fi
}

# 安全审计
check_security() {
    log_info "运行安全审计..."
    
    # 检查是否安装了 cargo-audit
    if ! command -v cargo-audit &> /dev/null; then
        log_info "安装 cargo-audit..."
        cargo install cargo-audit
    fi
    
    # 运行安全审计
    if cargo audit > /dev/null 2>&1; then
        record_check "pass" "安全审计通过"
    else
        record_check "warn" "安全审计发现问题"
    fi
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    # 检查是否有重复依赖
    local duplicate_deps
    duplicate_deps=$(cargo tree --duplicates 2>/dev/null | grep -v "^$" | wc -l)
    
    if [[ $duplicate_deps -eq 0 ]]; then
        record_check "pass" "无重复依赖"
    else
        record_check "warn" "发现 $duplicate_deps 个重复依赖"
    fi
    
    # 检查过时的依赖
    if command -v cargo-outdated &> /dev/null; then
        local outdated_deps
        outdated_deps=$(cargo outdated --root-deps-only 2>/dev/null | grep -c "->")
        
        if [[ $outdated_deps -eq 0 ]]; then
            record_check "pass" "所有依赖都是最新的"
        else
            record_check "warn" "发现 $outdated_deps 个过时的依赖"
        fi
    fi
}

# 检查构建
check_build() {
    log_info "检查发布构建..."
    
    # 清理之前的构建
    cargo clean > /dev/null 2>&1
    
    # 构建发布版本
    if cargo build --release --all-features --workspace > /dev/null 2>&1; then
        record_check "pass" "发布构建成功"
    else
        record_check "fail" "发布构建失败"
        return 1
    fi
    
    # 检查二进制文件大小
    local binary_size
    if [[ -f "target/release/lumosai" ]]; then
        binary_size=$(du -h target/release/lumosai | cut -f1)
        log_info "主二进制文件大小: $binary_size"
        record_check "pass" "主二进制文件构建成功"
    else
        record_check "warn" "主二进制文件不存在"
    fi
}

# 检查示例
check_examples() {
    log_info "检查示例..."
    
    # 构建所有示例
    if cargo build --examples --all-features > /dev/null 2>&1; then
        record_check "pass" "示例构建成功"
    else
        record_check "warn" "示例构建失败"
    fi
}

# 检查基准测试
check_benchmarks() {
    log_info "检查基准测试..."
    
    # 检查是否有基准测试
    if find . -name "*.rs" -path "*/benches/*" | grep -q .; then
        if cargo bench --no-run > /dev/null 2>&1; then
            record_check "pass" "基准测试编译成功"
        else
            record_check "warn" "基准测试编译失败"
        fi
    else
        record_check "warn" "未找到基准测试"
    fi
}

# 检查发布配置
check_release_config() {
    log_info "检查发布配置..."
    
    # 检查 Cargo.toml 中的发布相关字段
    local required_fields=("name" "version" "authors" "description" "license")
    
    for field in "${required_fields[@]}"; do
        if grep -q "^$field = " Cargo.toml; then
            record_check "pass" "Cargo.toml 包含 $field 字段"
        else
            record_check "warn" "Cargo.toml 缺少 $field 字段"
        fi
    done
    
    # 检查发布配置文件
    if [[ -f "release.toml" ]]; then
        record_check "pass" "发布配置文件存在"
    else
        record_check "warn" "发布配置文件不存在"
    fi
}

# 主函数
main() {
    log_info "开始发布前检查..."
    echo "========================================"
    
    # 运行所有检查
    check_git_status
    check_version_consistency
    check_code_format
    check_clippy
    check_tests
    check_documentation
    check_security
    check_dependencies
    check_build
    check_examples
    check_benchmarks
    check_release_config
    
    echo "========================================"
    log_info "检查完成"
    
    # 显示结果摘要
    echo
    log_info "检查结果摘要:"
    log_success "通过: $CHECKS_PASSED"
    if [[ $CHECKS_WARNING -gt 0 ]]; then
        log_warning "警告: $CHECKS_WARNING"
    fi
    if [[ $CHECKS_FAILED -gt 0 ]]; then
        log_error "失败: $CHECKS_FAILED"
    fi
    
    # 决定是否可以发布
    if [[ $CHECKS_FAILED -eq 0 ]]; then
        echo
        log_success "✅ 所有关键检查通过，可以进行发布！"
        if [[ $CHECKS_WARNING -gt 0 ]]; then
            log_warning "⚠️  有 $CHECKS_WARNING 个警告，建议修复后再发布"
        fi
        exit 0
    else
        echo
        log_error "❌ 有 $CHECKS_FAILED 个关键检查失败，请修复后再发布"
        exit 1
    fi
}

# 显示帮助信息
show_help() {
    cat << EOF
LumosAI 发布前检查脚本

用法:
    $0 [选项]

选项:
    -h, --help    显示此帮助信息

检查项目:
    - Git 状态和分支
    - 版本一致性
    - 代码格式
    - Clippy 检查
    - 测试套件
    - 文档构建
    - 安全审计
    - 依赖检查
    - 发布构建
    - 示例构建
    - 基准测试
    - 发布配置

EOF
}

# 处理命令行参数
case "${1:-}" in
    -h|--help)
        show_help
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac
