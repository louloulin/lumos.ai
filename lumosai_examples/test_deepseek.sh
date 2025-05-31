#!/bin/bash

# DeepSeek Agent 测试脚本
# 用于验证 DeepSeek 集成是否正常工作

echo "🧪 DeepSeek Agent 测试脚本"
echo "=========================="

# 检查是否设置了 API 密钥
if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo "❌ 错误：未设置 DEEPSEEK_API_KEY 环境变量"
    echo "请设置您的 DeepSeek API 密钥："
    echo "export DEEPSEEK_API_KEY=\"your-api-key\""
    exit 1
fi

echo "✅ 找到 DeepSeek API 密钥"

# 检查 Rust 环境
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误：未找到 Cargo (Rust 包管理器)"
    echo "请安装 Rust: https://rustup.rs/"
    exit 1
fi

echo "✅ Rust 环境检查通过"

# 进入示例目录
cd "$(dirname "$0")"

echo "🔧 编译 DeepSeek Agent 示例..."

# 编译示例
if cargo build --example deepseek_agent_demo; then
    echo "✅ 编译成功"
else
    echo "❌ 编译失败"
    exit 1
fi

echo "🚀 运行 DeepSeek Agent 演示..."
echo "注意：这将调用 DeepSeek API，可能产生费用"
echo "按 Ctrl+C 可以随时停止"
echo ""

# 运行示例
cargo run --example deepseek_agent_demo

echo ""
echo "🎉 测试完成！"
