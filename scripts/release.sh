#!/bin/bash

# LumosAI 发布脚本
# 用于自动化版本发布流程

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

# 检查必要的工具
check_dependencies() {
    log_info "检查依赖工具..."
    
    local missing_tools=()
    
    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi
    
    if ! command -v git &> /dev/null; then
        missing_tools+=("git")
    fi
    
    if ! command -v jq &> /dev/null; then
        missing_tools+=("jq")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "缺少必要工具: ${missing_tools[*]}"
        log_error "请安装缺少的工具后重试"
        exit 1
    fi
    
    log_success "所有依赖工具已安装"
}

# 检查工作目录状态
check_working_directory() {
    log_info "检查工作目录状态..."
    
    # 检查是否在 git 仓库中
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        log_error "当前目录不是 Git 仓库"
        exit 1
    fi
    
    # 检查是否有未提交的更改
    if ! git diff-index --quiet HEAD --; then
        log_warning "工作目录有未提交的更改"
        read -p "是否继续发布? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "发布已取消"
            exit 0
        fi
    fi
    
    log_success "工作目录状态检查完成"
}

# 获取当前版本
get_current_version() {
    grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'
}

# 验证版本格式
validate_version() {
    local version=$1
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
        log_error "无效的版本格式: $version"
        log_error "版本格式应为: major.minor.patch 或 major.minor.patch-prerelease"
        exit 1
    fi
}

# 更新版本号
update_version() {
    local new_version=$1
    log_info "更新版本号到 $new_version..."
    
    # 更新根 Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    
    # 更新所有子包的版本号
    find . -name "Cargo.toml" -not -path "./target/*" | while read -r cargo_file; do
        if [ "$cargo_file" != "./Cargo.toml" ]; then
            sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" "$cargo_file"
        fi
    done
    
    # 清理备份文件
    find . -name "Cargo.toml.bak" -delete
    
    log_success "版本号已更新到 $new_version"
}

# 运行测试
run_tests() {
    log_info "运行测试套件..."
    
    # 运行所有测试
    if ! cargo test --all-features --workspace; then
        log_error "测试失败，发布已中止"
        exit 1
    fi
    
    # 运行 clippy 检查
    if ! cargo clippy --all-features --workspace -- -D warnings; then
        log_warning "Clippy 检查发现警告，但继续发布"
    fi
    
    # 检查格式
    if ! cargo fmt --all -- --check; then
        log_error "代码格式检查失败，请运行 'cargo fmt' 修复"
        exit 1
    fi
    
    log_success "所有测试通过"
}

# 构建发布版本
build_release() {
    log_info "构建发布版本..."
    
    # 清理之前的构建
    cargo clean
    
    # 构建发布版本
    if ! cargo build --release --all-features --workspace; then
        log_error "发布版本构建失败"
        exit 1
    fi
    
    log_success "发布版本构建完成"
}

# 生成变更日志
generate_changelog() {
    local version=$1
    local changelog_file="CHANGELOG.md"
    
    log_info "生成变更日志..."
    
    # 如果变更日志文件不存在，创建它
    if [ ! -f "$changelog_file" ]; then
        cat > "$changelog_file" << EOF
# 变更日志

所有重要的项目变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

EOF
    fi
    
    # 获取上一个标签
    local last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    
    # 生成提交日志
    local commits=""
    if [ -n "$last_tag" ]; then
        commits=$(git log --oneline "$last_tag"..HEAD)
    else
        commits=$(git log --oneline)
    fi
    
    # 创建临时变更日志条目
    local temp_entry=$(mktemp)
    cat > "$temp_entry" << EOF

## [$version] - $(date +%Y-%m-%d)

### 新增
- 新功能和特性

### 变更
- 现有功能的变更

### 修复
- Bug 修复

### 提交记录
$commits

EOF
    
    # 将新条目插入到变更日志的开头
    local temp_changelog=$(mktemp)
    head -n 6 "$changelog_file" > "$temp_changelog"
    cat "$temp_entry" >> "$temp_changelog"
    tail -n +7 "$changelog_file" >> "$temp_changelog"
    mv "$temp_changelog" "$changelog_file"
    
    rm "$temp_entry"
    
    log_success "变更日志已生成"
}

# 创建 Git 标签
create_git_tag() {
    local version=$1
    local tag_name="v$version"
    
    log_info "创建 Git 标签 $tag_name..."
    
    # 提交版本更改
    git add .
    git commit -m "chore: bump version to $version"
    
    # 创建标签
    git tag -a "$tag_name" -m "Release $version"
    
    log_success "Git 标签 $tag_name 已创建"
}

# 发布到 crates.io
publish_to_crates() {
    log_info "发布到 crates.io..."
    
    # 检查是否已登录
    if ! cargo login --help &> /dev/null; then
        log_error "请先使用 'cargo login' 登录到 crates.io"
        exit 1
    fi
    
    # 发布各个包（按依赖顺序）
    local packages=(
        "lumos_macro"
        "lumosai_core"
        "lumosai_vector/core"
        "lumosai_vector"
        "lumosai_evals"
        "lumosai_rag"
        "lumosai_network"
        "lumosai_cli"
        "."
    )
    
    for package in "${packages[@]}"; do
        log_info "发布包: $package"
        if [ "$package" = "." ]; then
            cargo publish --allow-dirty
        else
            cargo publish --package "$(basename "$package")" --allow-dirty
        fi
        
        # 等待一段时间让 crates.io 处理
        sleep 10
    done
    
    log_success "所有包已发布到 crates.io"
}

# 主函数
main() {
    local release_type=${1:-"patch"}
    local current_version
    local new_version
    
    log_info "开始 LumosAI 发布流程..."
    log_info "发布类型: $release_type"
    
    # 检查依赖
    check_dependencies
    
    # 检查工作目录
    check_working_directory
    
    # 获取当前版本
    current_version=$(get_current_version)
    log_info "当前版本: $current_version"
    
    # 计算新版本
    case $release_type in
        "major")
            new_version=$(echo "$current_version" | awk -F. '{print ($1+1)".0.0"}')
            ;;
        "minor")
            new_version=$(echo "$current_version" | awk -F. '{print $1"."($2+1)".0"}')
            ;;
        "patch")
            new_version=$(echo "$current_version" | awk -F. '{print $1"."$2"."($3+1)}')
            ;;
        *)
            # 自定义版本号
            new_version=$release_type
            ;;
    esac
    
    # 验证版本格式
    validate_version "$new_version"
    
    log_info "新版本: $new_version"
    
    # 确认发布
    read -p "确认发布版本 $new_version? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "发布已取消"
        exit 0
    fi
    
    # 更新版本号
    update_version "$new_version"
    
    # 运行测试
    run_tests
    
    # 构建发布版本
    build_release
    
    # 生成变更日志
    generate_changelog "$new_version"
    
    # 创建 Git 标签
    create_git_tag "$new_version"
    
    # 询问是否发布到 crates.io
    read -p "是否发布到 crates.io? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        publish_to_crates
    fi
    
    log_success "发布完成！"
    log_info "版本: $new_version"
    log_info "标签: v$new_version"
    log_info "请使用 'git push origin main --tags' 推送到远程仓库"
}

# 显示帮助信息
show_help() {
    cat << EOF
LumosAI 发布脚本

用法:
    $0 [release_type]

参数:
    release_type    发布类型，可选值：
                   - patch    补丁版本 (默认)
                   - minor    次要版本
                   - major    主要版本
                   - x.y.z    自定义版本号

示例:
    $0              # 发布补丁版本
    $0 minor        # 发布次要版本
    $0 major        # 发布主要版本
    $0 1.2.3        # 发布自定义版本

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
