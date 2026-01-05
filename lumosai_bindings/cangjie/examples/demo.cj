// Lumos.ai Cangjie绑定演示
// 展示如何使用Cangjie开发高性能AI Agent

import lumosai_cangjie.*

// 演示ENGRAM记忆系统
func demonstrateMemory() {
    print("=== ENGRAM记忆系统演示 ===")

    let memory = Lumos.memoryService()

    // 存储不同类型的记忆
    let episodicId = memory.store(
        "今天我参加了AI会议，学到了很多关于大语言模型的知识",
        .Episodic
    )

    let semanticId = memory.store(
        "Rust是一门系统编程语言，具有内存安全和零成本抽象的特性",
        .Semantic
    )

    let proceduralId = memory.store(
        "要编译Rust程序，首先运行cargo build命令，然后运行cargo run",
        .Procedural
    )

    print("存储了3条记忆：")
    print("- 事件记忆: \(episodicId)")
    print("- 事实记忆: \(semanticId)")
    print("- 过程记忆: \(proceduralId)")

    // 检索记忆
    let results = memory.retrieve("Rust编程")
    switch results {
    case .success(let entries):
        print("\n检索到 \(entries.count) 条相关记忆:")
        for entry in entries {
            print("- [\(entry.memoryType.toString())] \(entry.content.prefix(50))...")
        }
    case .failure(let error):
        print("检索失败: \(error)")
    }

    // 获取统计信息
    let stats = memory.stats()
    switch stats {
    case .success(let info):
        print("\n记忆统计:")
        print("- 总条目数: \(info.totalEntries)")
        print("- 事件记忆: \(info.episodicCount)")
        print("- 事实记忆: \(info.semanticCount)")
        print("- 过程记忆: \(info.proceduralCount)")
        print("- 平均重要性: \(String(format: "%.2f", info.avgImportance))")
        print("- 健康评分: \(String(format: "%.2f", info.healthScore))")
    case .failure(let error):
        print("获取统计失败: \(error)")
    }
}

// 演示智能Agent
func demonstrateAgent() {
    print("\n=== 智能Agent演示 ===")

    // 创建Agent
    let agent = Lumos.quickAgent("助手", "一个友好的AI助手")

    // 配置Agent
    let configuredAgent = SmartAgent(config: AgentConfig(
        name: "高级助手",
        description: "具备记忆能力和工具使用的AI助手",
        model: "gpt-4",
        temperature: 0.7,
        maxTokens: 2000,
        enableMemory: true,
        tools: ["calculator", "web_search"]
    ))

    // 添加工具
    let _ = configuredAgent.addTool("calculator")
    let _ = configuredAgent.addTool("web_search")

    // 生成响应
    let response = configuredAgent.generate("帮我计算15 + 27的结果")
    switch response {
    case .success(let result):
        print("Agent响应: \(result)")
    case .failure(let error):
        print("生成响应失败: \(error)")
    }

    // 获取Agent信息
    let info = configuredAgent.info()
    print("\nAgent信息:")
    print("- 名称: \(info.name)")
    print("- 描述: \(info.description)")
    print("- 模型: \(info.model)")
    print("- 工具数量: \(info.tools.count)")
}

// 演示工具使用
func demonstrateTools() {
    print("\n=== 工具使用演示 ===")

    // 使用计算器工具
    let calculator = BuiltinTools.calculator()
    let calcResult = calculator.execute("15 + 27")
    switch calcResult {
    case .success(let result):
        print("计算器结果: \(result)")
    case .failure(let error):
        print("计算失败: \(error)")
    }

    // 使用网络搜索工具
    let search = BuiltinTools.webSearch()
    let searchResult = search.execute("最新的AI新闻")
    switch searchResult {
    case .success(let result):
        print("搜索结果: \(result)")
    case .failure(let error):
        print("搜索失败: \(error)")
    }

    // 获取工具信息
    let calcInfo = calculator.info()
    print("\n计算器工具信息:")
    print("- 名称: \(calcInfo.name)")
    print("- 描述: \(calcInfo.description)")
}

// 综合演示
func comprehensiveDemo() {
    print("==========================================")
    print("🌟 Lumos.ai Cangjie绑定综合演示")
    print("==========================================")

    demonstrateMemory()
    demonstrateAgent()
    demonstrateTools()

    print("\n==========================================")
    print("✅ 演示完成！")
    print("==========================================")
}

// 主函数
func main() {
    print("🚀 启动Lumos.ai Cangjie演示...")

    comprehensiveDemo()

    print("\n🎉 感谢使用Lumos.ai Cangjie绑定！")
    print("📚 更多信息请访问: https://docs.lumosai.com/cangjie")
}

// 错误处理扩展
extension LumosError: CustomStringConvertible {
    var description: String {
        switch self {
        case .agentError(let msg): return "Agent错误: \(msg)"
        case .memoryError(let msg): return "记忆错误: \(msg)"
        case .toolError(let msg): return "工具错误: \(msg)"
        case .networkError(let msg): return "网络错误: \(msg)"
        }
    }
}

// 异步支持示例（如果Cangjie支持async）
// async func asyncDemo() async {
//     let agent = Lumos.quickAgent("异步助手", "支持异步操作的助手")
//     let response = await agent.generateAsync("异步问候")
//     print("异步响应: \(response)")
// }



