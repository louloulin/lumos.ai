#!/bin/bash
# DeepSeek API 设置脚本 (Bash)
# 用于快速设置 DeepSeek API Key 环境变量

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 显示帮助信息
show_help() {
    echo -e "${CYAN}🔑 DeepSeek API 设置脚本${NC}"
    echo -e "${CYAN}========================${NC}"
    echo ""
    echo -e "${YELLOW}用法:${NC}"
    echo -e "  ./setup_deepseek_api.sh [选项]"
    echo ""
    echo -e "${YELLOW}选项:${NC}"
    echo -e "  -k, --api-key <key>    设置 DeepSeek API Key"
    echo -e "  -p, --permanent        永久设置环境变量"
    echo -e "  -v, --verify           验证 API Key 设置"
    echo -e "  -h, --help             显示此帮助信息"
    echo ""
    echo -e "${YELLOW}示例:${NC}"
    echo -e "  ${GREEN}# 临时设置 API Key${NC}"
    echo -e "  ./setup_deepseek_api.sh -k sk-your-api-key-here"
    echo ""
    echo -e "  ${GREEN}# 永久设置 API Key${NC}"
    echo -e "  ./setup_deepseek_api.sh -k sk-your-api-key-here -p"
    echo ""
    echo -e "  ${GREEN}# 验证当前设置${NC}"
    echo -e "  ./setup_deepseek_api.sh -v"
    echo ""
    echo -e "${YELLOW}获取 API Key:${NC}"
    echo -e "  1. 访问 https://platform.deepseek.com/"
    echo -e "  2. 注册并登录账户"
    echo -e "  3. 在 API 管理页面创建新的 API Key"
    echo ""
}

# 检查 API Key 格式
test_api_key_format() {
    local key="$1"
    
    if [[ -z "$key" ]]; then
        return 1
    fi
    
    # 检查 API Key 格式
    if [[ "$key" =~ ^sk-[a-zA-Z0-9]{32,64}$ ]]; then
        return 0
    fi
    
    return 1
}

# 设置 API Key
set_api_key() {
    local key="$1"
    local permanent="$2"
    
    echo -e "${YELLOW}🔧 设置 DeepSeek API Key...${NC}"
    
    # 验证 API Key 格式
    if ! test_api_key_format "$key"; then
        echo -e "${RED}❌ API Key 格式无效！${NC}"
        echo -e "${RED}   API Key 应该以 'sk-' 开头，后跟 32-64 个字符${NC}"
        return 1
    fi
    
    # 设置临时环境变量
    export DEEPSEEK_API_KEY="$key"
    
    if [[ "$permanent" == "true" ]]; then
        # 检测 shell 类型
        if [[ -n "$ZSH_VERSION" ]]; then
            shell_rc="$HOME/.zshrc"
        elif [[ -n "$BASH_VERSION" ]]; then
            shell_rc="$HOME/.bashrc"
        else
            shell_rc="$HOME/.profile"
        fi
        
        echo -e "${YELLOW}⚠️  正在设置永久环境变量...${NC}"
        
        # 检查是否已经存在
        if grep -q "DEEPSEEK_API_KEY" "$shell_rc" 2>/dev/null; then
            # 更新现有的设置
            if [[ "$OSTYPE" == "darwin"* ]]; then
                # macOS
                sed -i '' "s/export DEEPSEEK_API_KEY=.*/export DEEPSEEK_API_KEY=\"$key\"/" "$shell_rc"
            else
                # Linux
                sed -i "s/export DEEPSEEK_API_KEY=.*/export DEEPSEEK_API_KEY=\"$key\"/" "$shell_rc"
            fi
            echo -e "${GREEN}✅ 更新了 $shell_rc 中的 API Key${NC}"
        else
            # 添加新的设置
            echo "export DEEPSEEK_API_KEY=\"$key\"" >> "$shell_rc"
            echo -e "${GREEN}✅ 添加了 API Key 到 $shell_rc${NC}"
        fi
        
        echo -e "${YELLOW}   重新加载 shell 配置: source $shell_rc${NC}"
    else
        echo -e "${GREEN}✅ 临时环境变量设置成功！${NC}"
        echo -e "${YELLOW}   仅在当前会话中有效${NC}"
    fi
    
    # 显示设置的 API Key（部分隐藏）
    local masked_key="${key:0:8}...${key: -8}"
    echo -e "${CYAN}   API Key: $masked_key${NC}"
    
    return 0
}

# 验证 API Key 设置
test_api_key_setup() {
    echo -e "${YELLOW}🧪 验证 API Key 设置...${NC}"
    
    # 检查环境变量
    if [[ -z "$DEEPSEEK_API_KEY" ]]; then
        echo -e "${RED}❌ DEEPSEEK_API_KEY 环境变量未设置${NC}"
        return 1
    fi
    
    # 验证格式
    if ! test_api_key_format "$DEEPSEEK_API_KEY"; then
        echo -e "${RED}❌ API Key 格式无效${NC}"
        return 1
    fi
    
    # 显示当前设置
    local masked_key="${DEEPSEEK_API_KEY:0:8}...${DEEPSEEK_API_KEY: -8}"
    echo -e "${GREEN}✅ API Key 已正确设置: $masked_key${NC}"
    
    # 检查是否为永久设置
    local shell_rc
    if [[ -n "$ZSH_VERSION" ]]; then
        shell_rc="$HOME/.zshrc"
    elif [[ -n "$BASH_VERSION" ]]; then
        shell_rc="$HOME/.bashrc"
    else
        shell_rc="$HOME/.profile"
    fi
    
    if grep -q "DEEPSEEK_API_KEY" "$shell_rc" 2>/dev/null; then
        echo -e "${GREEN}✅ 永久环境变量已设置${NC}"
    else
        echo -e "${YELLOW}⚠️  仅设置了临时环境变量${NC}"
    fi
    
    return 0
}

# 测试 LumosAI 示例
test_lumosai_example() {
    echo -e "${YELLOW}🚀 测试 LumosAI 示例...${NC}"
    
    # 检查是否在正确的目录
    if [[ ! -f "Cargo.toml" ]]; then
        echo -e "${RED}❌ 请在 LumosAI 项目根目录运行此脚本${NC}"
        return 1
    fi
    
    # 检查示例文件是否存在
    if [[ ! -f "examples/real_deepseek_api_validation.rs" ]]; then
        echo -e "${RED}❌ 找不到真实 API 验证示例文件${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ 可以运行以下命令测试 API:${NC}"
    echo -e "${CYAN}   cargo run --example real_deepseek_api_validation${NC}"
    
    return 0
}

# 解析命令行参数
API_KEY=""
PERMANENT="false"
VERIFY="false"
HELP="false"

while [[ $# -gt 0 ]]; do
    case $1 in
        -k|--api-key)
            API_KEY="$2"
            shift 2
            ;;
        -p|--permanent)
            PERMANENT="true"
            shift
            ;;
        -v|--verify)
            VERIFY="true"
            shift
            ;;
        -h|--help)
            HELP="true"
            shift
            ;;
        *)
            echo -e "${RED}❌ 未知参数: $1${NC}"
            echo -e "${YELLOW}   使用 -h 或 --help 查看使用说明${NC}"
            exit 1
            ;;
    esac
done

# 主逻辑
if [[ "$HELP" == "true" ]]; then
    show_help
    exit 0
fi

echo -e "${CYAN}🔑 DeepSeek API 设置脚本${NC}"
echo -e "${CYAN}========================${NC}"
echo ""

if [[ "$VERIFY" == "true" ]]; then
    if test_api_key_setup; then
        test_lumosai_example
    fi
    exit 0
fi

if [[ -z "$API_KEY" ]]; then
    echo -e "${RED}❌ 请提供 API Key${NC}"
    echo -e "${YELLOW}   使用 -h 或 --help 参数查看使用说明${NC}"
    exit 1
fi

# 设置 API Key
if set_api_key "$API_KEY" "$PERMANENT"; then
    echo ""
    echo -e "${GREEN}🎉 设置完成！${NC}"
    echo ""
    
    # 验证设置
    test_api_key_setup > /dev/null
    
    # 提供下一步指导
    echo -e "${YELLOW}📋 下一步:${NC}"
    echo -e "  ${GREEN}1. 运行验证示例:${NC}"
    echo -e "     ${CYAN}cargo run --example real_deepseek_api_validation${NC}"
    echo ""
    echo -e "  ${GREEN}2. 查看更多示例:${NC}"
    echo -e "     ${CYAN}cargo run --example simple_api_validation${NC}"
    echo ""
    echo -e "  ${GREEN}3. 阅读文档:${NC}"
    echo -e "     ${CYAN}docs/DEEPSEEK_API_SETUP.md${NC}"
    echo ""
    
    if [[ "$PERMANENT" != "true" ]]; then
        echo -e "${BLUE}💡 提示: 使用 -p 或 --permanent 参数可以永久设置环境变量${NC}"
    fi
else
    echo ""
    echo -e "${RED}❌ 设置失败，请检查 API Key 格式${NC}"
    exit 1
fi
