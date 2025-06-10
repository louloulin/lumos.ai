#!/bin/bash

# Lumos AI 框架功能演示运行脚本
# 
# 此脚本用于运行所有已实现的演示示例
# 确保在运行前已经正确配置了开发环境

set -e

echo "🚀 Lumos AI 框架功能演示"
echo "========================="
echo ""

# 检查 Rust 环境
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: 未找到 Cargo。请先安装 Rust 开发环境。"
    echo "   安装命令: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust 环境检查通过"
echo "   Cargo 版本: $(cargo --version)"
echo ""

# 检查项目结构
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误: 未找到 Cargo.toml 文件。请确保在项目根目录运行此脚本。"
    exit 1
fi

echo "✅ 项目结构检查通过"
echo ""

# 编译项目
echo "🔨 编译项目..."
if ! cargo build --examples; then
    echo "❌ 编译失败。请检查代码错误。"
    exit 1
fi

echo "✅ 编译成功"
echo ""

# 运行演示函数
run_demo() {
    local demo_name=$1
    local demo_file=$2
    local description=$3
    
    echo "📋 运行演示: $demo_name"
    echo "   描述: $description"
    echo "   文件: examples/$demo_file"
    echo "   ----------------------------------------"
    
    if cargo run --example "$demo_file"; then
        echo "   ✅ $demo_name 演示完成"
    else
        echo "   ❌ $demo_name 演示失败"
        return 1
    fi
    
    echo ""
    echo "   按 Enter 继续下一个演示..."
    read -r
    echo ""
}

# 阶段一：基础功能演示
echo "🎯 阶段一：基础功能演示"
echo "======================"
echo ""

run_demo "基础 Agent 创建" "basic_agent" "展示如何创建和配置基础的 AI Agent"

run_demo "工具集成" "tool_integration" "演示自定义工具创建和 Agent 工具集成"

run_demo "记忆系统" "memory_system" "展示对话记忆和记忆管理功能"

run_demo "流式响应" "streaming_response" "演示实时流式输出和事件处理"

echo "🎉 阶段一演示完成！"
echo ""

# 阶段二：高级功能演示
echo "🎯 阶段二：高级功能演示"
echo "======================"
echo ""

run_demo "RAG 系统" "rag_system" "展示检索增强生成系统和知识库构建"

run_demo "向量存储" "vector_storage" "演示多种向量存储后端和性能测试"

run_demo "多代理工作流" "multi_agent_workflow" "展示复杂的多代理协作和工作流编排"

run_demo "事件驱动架构" "event_driven_architecture" "演示事件驱动的代理协作系统"

echo "🎉 阶段二演示完成！"
echo ""

# 阶段三：企业级功能演示
echo "🎯 阶段三：企业级功能演示"
echo "========================"
echo ""

run_demo "监控与遥测" "monitoring_telemetry" "展示全面的性能监控和遥测系统"

run_demo "安全与审计" "security_audit" "演示企业级安全控制和审计系统"

run_demo "多租户架构" "multi_tenant" "展示企业级多租户功能和资源隔离"

run_demo "云原生部署" "cloud_native_deployment" "演示Kubernetes部署和云原生最佳实践"

echo "🎉 阶段三演示完成！"
echo ""

# 总结
echo "🏆 所有演示完成！"
echo "=================="
echo ""
echo "已成功运行的功能演示："
echo ""
echo "🎯 阶段一：基础功能"
echo "✅ 基础 Agent 创建和配置"
echo "✅ 工具系统集成"
echo "✅ 记忆系统管理"
echo "✅ 流式响应处理"
echo ""
echo "🎯 阶段二：高级功能"
echo "✅ RAG 系统构建"
echo "✅ 向量存储操作"
echo "✅ 多代理工作流编排"
echo "✅ 事件驱动架构"
echo ""
echo "🎯 阶段三：企业级功能"
echo "✅ 监控与遥测系统"
echo "✅ 安全与审计控制"
echo "✅ 多租户架构"
echo "✅ 云原生部署"
echo ""
echo "🎯 下一步建议："
echo "1. 查看各个演示的源代码了解实现细节"
echo "2. 根据需要修改配置和参数"
echo "3. 集成到您的实际项目中"
echo "4. 参考 demo.md 文档了解更多功能"
echo ""
echo "📚 相关文档："
echo "- demo.md - 完整功能演示文档"
echo "- examples/ - 所有演示源代码"
echo "- lumosai_core/ - 核心框架代码"
echo ""
echo "感谢使用 Lumos AI 框架！🚀"
