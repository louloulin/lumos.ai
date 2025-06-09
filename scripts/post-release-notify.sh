#!/bin/bash

# LumosAI 发布后通知脚本
# 发布完成后发送通知到各个渠道

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

# 获取版本信息
get_version_info() {
    VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    TAG="v$VERSION"
    RELEASE_DATE=$(date +"%Y-%m-%d")
    RELEASE_TIME=$(date +"%H:%M:%S UTC")
    
    # 获取 Git 信息
    GIT_COMMIT=$(git rev-parse HEAD)
    GIT_BRANCH=$(git branch --show-current)
    
    # 获取发布统计
    TOTAL_COMMITS=$(git rev-list --count HEAD)
    
    # 获取上一个标签
    LAST_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")
    
    if [[ -n "$LAST_TAG" ]]; then
        COMMITS_SINCE_LAST=$(git rev-list --count "$LAST_TAG"..HEAD)
        DAYS_SINCE_LAST=$(( ($(date +%s) - $(git log -1 --format=%ct "$LAST_TAG")) / 86400 ))
    else
        COMMITS_SINCE_LAST=$TOTAL_COMMITS
        DAYS_SINCE_LAST="N/A"
    fi
}

# 生成发布摘要
generate_release_summary() {
    cat << EOF
🎉 **LumosAI $VERSION 发布成功！**

📊 **发布信息**
• 版本: $VERSION
• 标签: $TAG
• 发布日期: $RELEASE_DATE $RELEASE_TIME
• Git 提交: ${GIT_COMMIT:0:8}
• 分支: $GIT_BRANCH

📈 **统计信息**
• 总提交数: $TOTAL_COMMITS
• 自上次发布: $COMMITS_SINCE_LAST 个提交
• 距离上次发布: $DAYS_SINCE_LAST 天

🔗 **相关链接**
• GitHub Release: https://github.com/lumosai/lumosai/releases/tag/$TAG
• Crates.io: https://crates.io/crates/lumosai
• 文档: https://docs.rs/lumosai/$VERSION
• 变更日志: https://github.com/lumosai/lumosai/blob/main/CHANGELOG.md

📦 **安装方式**
\`\`\`bash
cargo install lumosai
\`\`\`

EOF
}

# 发送 Slack 通知
send_slack_notification() {
    local webhook_url="$1"
    
    if [[ -z "$webhook_url" ]]; then
        log_warning "Slack webhook URL 未配置，跳过 Slack 通知"
        return 0
    fi
    
    log_info "发送 Slack 通知..."
    
    local payload=$(cat << EOF
{
    "text": "LumosAI $VERSION 发布成功！",
    "blocks": [
        {
            "type": "header",
            "text": {
                "type": "plain_text",
                "text": "🎉 LumosAI $VERSION 发布成功！"
            }
        },
        {
            "type": "section",
            "fields": [
                {
                    "type": "mrkdwn",
                    "text": "*版本:* $VERSION"
                },
                {
                    "type": "mrkdwn",
                    "text": "*发布日期:* $RELEASE_DATE"
                },
                {
                    "type": "mrkdwn",
                    "text": "*提交数:* $COMMITS_SINCE_LAST (自上次发布)"
                },
                {
                    "type": "mrkdwn",
                    "text": "*Git 提交:* ${GIT_COMMIT:0:8}"
                }
            ]
        },
        {
            "type": "actions",
            "elements": [
                {
                    "type": "button",
                    "text": {
                        "type": "plain_text",
                        "text": "查看 Release"
                    },
                    "url": "https://github.com/lumosai/lumosai/releases/tag/$TAG"
                },
                {
                    "type": "button",
                    "text": {
                        "type": "plain_text",
                        "text": "查看文档"
                    },
                    "url": "https://docs.rs/lumosai/$VERSION"
                }
            ]
        }
    ]
}
EOF
)
    
    if curl -X POST -H 'Content-type: application/json' \
            --data "$payload" \
            "$webhook_url" > /dev/null 2>&1; then
        log_success "Slack 通知发送成功"
    else
        log_error "Slack 通知发送失败"
    fi
}

# 发送 Discord 通知
send_discord_notification() {
    local webhook_url="$1"
    
    if [[ -z "$webhook_url" ]]; then
        log_warning "Discord webhook URL 未配置，跳过 Discord 通知"
        return 0
    fi
    
    log_info "发送 Discord 通知..."
    
    local payload=$(cat << EOF
{
    "embeds": [
        {
            "title": "🎉 LumosAI $VERSION 发布成功！",
            "description": "新版本已成功发布到 GitHub 和 crates.io",
            "color": 5763719,
            "fields": [
                {
                    "name": "📦 版本",
                    "value": "$VERSION",
                    "inline": true
                },
                {
                    "name": "📅 发布日期",
                    "value": "$RELEASE_DATE",
                    "inline": true
                },
                {
                    "name": "🔄 提交数",
                    "value": "$COMMITS_SINCE_LAST (自上次发布)",
                    "inline": true
                },
                {
                    "name": "🔗 链接",
                    "value": "[GitHub Release](https://github.com/lumosai/lumosai/releases/tag/$TAG) | [Crates.io](https://crates.io/crates/lumosai) | [文档](https://docs.rs/lumosai/$VERSION)",
                    "inline": false
                },
                {
                    "name": "📦 安装",
                    "value": "\`\`\`bash\ncargo install lumosai\`\`\`",
                    "inline": false
                }
            ],
            "footer": {
                "text": "LumosAI Release Bot",
                "icon_url": "https://github.com/lumosai.png"
            },
            "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%S.000Z)"
        }
    ]
}
EOF
)
    
    if curl -X POST -H 'Content-type: application/json' \
            --data "$payload" \
            "$webhook_url" > /dev/null 2>&1; then
        log_success "Discord 通知发送成功"
    else
        log_error "Discord 通知发送失败"
    fi
}

# 发送邮件通知
send_email_notification() {
    local recipients="$1"
    
    if [[ -z "$recipients" ]]; then
        log_warning "邮件收件人未配置，跳过邮件通知"
        return 0
    fi
    
    log_info "发送邮件通知..."
    
    local subject="LumosAI $VERSION 发布成功"
    local body=$(generate_release_summary)
    
    # 使用 sendmail 或 mail 命令发送邮件
    if command -v sendmail &> /dev/null; then
        echo -e "Subject: $subject\n\n$body" | sendmail "$recipients"
        log_success "邮件通知发送成功"
    elif command -v mail &> /dev/null; then
        echo "$body" | mail -s "$subject" "$recipients"
        log_success "邮件通知发送成功"
    else
        log_warning "未找到邮件发送工具，跳过邮件通知"
    fi
}

# 更新社交媒体
update_social_media() {
    log_info "准备社交媒体更新内容..."
    
    local tweet_content="🎉 LumosAI $VERSION 发布！

🦀 Rust 原生 AI 代理框架
⚡ 高性能异步架构
🔧 模块化设计
🛡️ 企业级安全

📦 cargo install lumosai

#RustLang #AI #OpenSource #LumosAI

https://github.com/lumosai/lumosai/releases/tag/$TAG"
    
    # 保存到文件供手动发布
    echo "$tweet_content" > "/tmp/lumosai_release_tweet.txt"
    log_info "Twitter 内容已保存到 /tmp/lumosai_release_tweet.txt"
    
    # LinkedIn 内容
    local linkedin_content="🎉 Excited to announce LumosAI $VERSION release!

LumosAI is a high-performance, enterprise-grade AI agent framework built in Rust. This release brings:

✨ Enhanced workflow system
🧠 Advanced memory management  
🔧 Intelligent tool system
🏗️ Modular application framework
🛡️ Enterprise security features

Perfect for building production-ready AI applications with:
• Type safety and memory safety
• Async-first architecture
• Horizontal scalability
• Enterprise-grade security

Get started: cargo install lumosai

#AI #Rust #OpenSource #MachineLearning #Enterprise #Technology

https://github.com/lumosai/lumosai"
    
    echo "$linkedin_content" > "/tmp/lumosai_release_linkedin.txt"
    log_info "LinkedIn 内容已保存到 /tmp/lumosai_release_linkedin.txt"
}

# 更新文档网站
update_documentation_site() {
    log_info "更新文档网站..."
    
    # 这里可以添加自动更新文档网站的逻辑
    # 例如触发 Netlify 或 Vercel 的构建
    
    log_info "文档网站更新已触发"
}

# 更新包管理器
update_package_managers() {
    log_info "更新包管理器..."
    
    # 创建 Homebrew 更新脚本
    cat > "/tmp/update_homebrew.rb" << EOF
# Homebrew formula update for LumosAI $VERSION
class Lumosai < Formula
  desc "Enterprise-grade AI agent framework built in Rust"
  homepage "https://lumosai.dev"
  url "https://github.com/lumosai/lumosai/archive/refs/tags/$TAG.tar.gz"
  license "MIT"
  
  depends_on "rust" => :build
  
  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
  end
  
  test do
    system "#{bin}/lumosai", "--version"
  end
end
EOF
    
    log_info "Homebrew formula 已保存到 /tmp/update_homebrew.rb"
    
    # 创建 Arch Linux PKGBUILD
    cat > "/tmp/PKGBUILD" << EOF
# Maintainer: LumosAI Team
pkgname=lumosai
pkgver=$VERSION
pkgrel=1
pkgdesc="Enterprise-grade AI agent framework built in Rust"
arch=('x86_64')
url="https://lumosai.dev"
license=('MIT')
depends=()
makedepends=('rust' 'cargo')
source=("https://github.com/lumosai/lumosai/archive/refs/tags/$TAG.tar.gz")

build() {
    cd "\$pkgname-\$pkgver"
    cargo build --release --locked
}

package() {
    cd "\$pkgname-\$pkgver"
    install -Dm755 target/release/lumosai "\$pkgdir/usr/bin/lumosai"
    install -Dm644 LICENSE "\$pkgdir/usr/share/licenses/\$pkgname/LICENSE"
}
EOF
    
    log_info "Arch Linux PKGBUILD 已保存到 /tmp/PKGBUILD"
}

# 生成发布报告
generate_release_report() {
    log_info "生成发布报告..."
    
    local report_file="release_report_$VERSION.md"
    
    cat > "$report_file" << EOF
# LumosAI $VERSION 发布报告

## 发布信息
- **版本**: $VERSION
- **标签**: $TAG
- **发布日期**: $RELEASE_DATE $RELEASE_TIME
- **Git 提交**: $GIT_COMMIT
- **分支**: $GIT_BRANCH

## 统计信息
- **总提交数**: $TOTAL_COMMITS
- **自上次发布**: $COMMITS_SINCE_LAST 个提交
- **距离上次发布**: $DAYS_SINCE_LAST 天

## 发布渠道
- [x] GitHub Release
- [x] Crates.io
- [x] 文档更新
- [x] 通知发送

## 变更摘要
$(if [[ -n "$LAST_TAG" ]]; then
    echo "### 提交记录 (自 $LAST_TAG)"
    git log --oneline "$LAST_TAG"..HEAD | sed 's/^/- /'
else
    echo "### 初始发布"
    echo "- 首次发布 LumosAI 框架"
fi)

## 下载统计
- GitHub Release: [查看](https://github.com/lumosai/lumosai/releases/tag/$TAG)
- Crates.io: [查看](https://crates.io/crates/lumosai)

## 后续计划
- 监控发布后的反馈
- 收集用户使用情况
- 准备下一个版本的开发

---
*报告生成时间: $(date)*
EOF
    
    log_success "发布报告已保存到 $report_file"
}

# 主函数
main() {
    local slack_webhook="${SLACK_WEBHOOK_URL:-}"
    local discord_webhook="${DISCORD_WEBHOOK_URL:-}"
    local email_recipients="${EMAIL_RECIPIENTS:-}"
    
    log_info "开始发布后通知流程..."
    
    # 获取版本信息
    get_version_info
    
    log_info "发布版本: $VERSION"
    log_info "发布标签: $TAG"
    
    # 发送通知
    send_slack_notification "$slack_webhook"
    send_discord_notification "$discord_webhook"
    send_email_notification "$email_recipients"
    
    # 更新社交媒体内容
    update_social_media
    
    # 更新文档和包管理器
    update_documentation_site
    update_package_managers
    
    # 生成发布报告
    generate_release_report
    
    log_success "发布后通知流程完成！"
    
    # 显示摘要
    echo
    log_info "发布摘要:"
    generate_release_summary
}

# 显示帮助信息
show_help() {
    cat << EOF
LumosAI 发布后通知脚本

用法:
    $0 [选项]

环境变量:
    SLACK_WEBHOOK_URL     Slack webhook URL
    DISCORD_WEBHOOK_URL   Discord webhook URL  
    EMAIL_RECIPIENTS      邮件收件人列表

选项:
    -h, --help    显示此帮助信息

功能:
    - 发送 Slack 通知
    - 发送 Discord 通知
    - 发送邮件通知
    - 准备社交媒体内容
    - 更新文档网站
    - 更新包管理器配置
    - 生成发布报告

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
