//! 版本控制工具演示程序

use lumosai_core::tool::builtin::version_control::*;
use lumosai_core::tool::{ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("🔧 版本控制工具演示\n");
    println!("{}", "=".repeat(80));
    println!();

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // 测试 1: Git 状态查询
    println!("📊 测试 1: Git 状态查询");
    println!("{}", "-".repeat(80));
    
    let tools = get_all_version_control_tools();
    let status_tool = &tools[0];
    
    println!("工具名称: {}", status_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", status_tool.description());
    println!();

    let status_scenarios = vec![
        ("场景 1: 基本状态查询", json!({
            "repository_path": "/path/to/lumosai"
        })),
        ("场景 2: 包含未跟踪文件", json!({
            "repository_path": "/path/to/lumosai",
            "show_untracked": true
        })),
        ("场景 3: 包含忽略文件", json!({
            "repository_path": "/path/to/lumosai",
            "show_untracked": true,
            "show_ignored": true
        })),
    ];

    for (desc, params) in status_scenarios {
        println!("{}", desc);
        
        match status_tool.execute(params, context.clone(), &options).await {
            Ok(result) => {
                if result["success"] == true {
                    println!("  ✅ 仓库路径: {}", result["repository_path"]);
                    
                    if let Some(branch) = result.get("branch") {
                        println!("  🌿 分支信息:");
                        println!("    当前分支: {}", branch["current"]);
                        println!("    上游分支: {}", branch["upstream"]);
                        println!("    领先: {} 个提交", branch["ahead"]);
                        println!("    落后: {} 个提交", branch["behind"]);
                        println!("    状态: {}", branch["status"]);
                    }
                    
                    if let Some(changes) = result.get("changes") {
                        println!("  📝 变更统计:");
                        println!("    修改: {} 个文件", changes["modified"]);
                        println!("    新增: {} 个文件", changes["added"]);
                        println!("    删除: {} 个文件", changes["deleted"]);
                    }
                    
                    if let Some(staged) = result.get("staged") {
                        println!("  ✨ 暂存区:");
                        println!("    文件数: {}", staged["files"]);
                        println!("    新增行: {}", staged["insertions"]);
                        println!("    删除行: {}", staged["deletions"]);
                    }
                    
                    if let Some(untracked) = result.get("untracked") {
                        println!("  ❓ 未跟踪文件: {} 个", untracked["files"]);
                    }
                    
                    if let Some(ignored) = result.get("ignored") {
                        println!("  🚫 忽略文件: {} 个", ignored["files"]);
                    }
                } else {
                    println!("  ❌ 查询失败: {}", result["error"]);
                }
            }
            Err(e) => println!("  ❌ 查询失败: {}", e),
        }
        println!();
    }

    // 测试 2: Git 操作
    println!("⚡ 测试 2: Git 操作");
    println!("{}", "-".repeat(80));
    
    let operation_tool = &tools[1];
    
    println!("工具名称: {}", operation_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", operation_tool.description());
    println!();

    let operation_scenarios = vec![
        ("场景 1: 提交变更", json!({
            "repository_path": "/path/to/lumosai",
            "operation": "commit",
            "message": "feat: add version control tools"
        })),
        ("场景 2: 推送到远程", json!({
            "repository_path": "/path/to/lumosai",
            "operation": "push",
            "branch": "main"
        })),
        ("场景 3: 拉取更新", json!({
            "repository_path": "/path/to/lumosai",
            "operation": "pull",
            "branch": "main"
        })),
        ("场景 4: 获取远程更新", json!({
            "repository_path": "/path/to/lumosai",
            "operation": "fetch"
        })),
        ("场景 5: 合并分支", json!({
            "repository_path": "/path/to/lumosai",
            "operation": "merge",
            "branch": "feature/new-tools"
        })),
    ];

    for (desc, params) in operation_scenarios {
        println!("{}", desc);
        
        match operation_tool.execute(params, context.clone(), &options).await {
            Ok(result) => {
                if result["success"] == true {
                    println!("  ✅ 操作: {}", result["operation"]);
                    println!("  分支: {}", result["branch"]);
                    
                    if let Some(commit) = result.get("commit") {
                        println!("  📝 提交信息:");
                        println!("    哈希: {}", commit["hash"]);
                        println!("    消息: {}", commit["message"]);
                        println!("    文件变更: {}", commit["files_changed"]);
                        println!("    新增行: {}", commit["insertions"]);
                        println!("    删除行: {}", commit["deletions"]);
                    }
                    
                    if let Some(push) = result.get("push") {
                        println!("  📤 推送信息:");
                        println!("    远程: {}", push["remote"]);
                        println!("    提交数: {}", push["commits_pushed"]);
                        println!("    状态: {}", push["status"]);
                    }
                    
                    if let Some(pull) = result.get("pull") {
                        println!("  📥 拉取信息:");
                        println!("    远程: {}", pull["remote"]);
                        println!("    提交数: {}", pull["commits_pulled"]);
                        println!("    文件变更: {}", pull["files_changed"]);
                        println!("    冲突: {}", pull["conflicts"]);
                    }
                    
                    if let Some(merge) = result.get("merge") {
                        println!("  🔀 合并信息:");
                        println!("    源分支: {}", merge["source_branch"]);
                        println!("    目标分支: {}", merge["target_branch"]);
                        println!("    提交数: {}", merge["commits_merged"]);
                        println!("    冲突: {}", merge["conflicts"]);
                    }
                } else {
                    println!("  ❌ 操作失败: {}", result["error"]);
                }
            }
            Err(e) => println!("  ❌ 操作失败: {}", e),
        }
        println!();
    }

    // 测试 3: 仓库分析
    println!("📈 测试 3: 仓库分析");
    println!("{}", "-".repeat(80));
    
    let analyzer_tool = &tools[2];
    
    println!("工具名称: {}", analyzer_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", analyzer_tool.description());
    println!();

    let analysis_scenarios = vec![
        ("场景 1: 完整分析", json!({
            "repository_path": "/path/to/lumosai",
            "analysis_type": "all",
            "time_range_days": 30
        })),
        ("场景 2: 统计信息", json!({
            "repository_path": "/path/to/lumosai",
            "analysis_type": "statistics"
        })),
        ("场景 3: 贡献者分析", json!({
            "repository_path": "/path/to/lumosai",
            "analysis_type": "contributors"
        })),
        ("场景 4: 历史趋势", json!({
            "repository_path": "/path/to/lumosai",
            "analysis_type": "history",
            "time_range_days": 90
        })),
    ];

    for (desc, params) in analysis_scenarios {
        println!("{}", desc);
        
        match analyzer_tool.execute(params, context.clone(), &options).await {
            Ok(result) => {
                if result["success"] == true {
                    println!("  ✅ 分析类型: {}", result["analysis_type"]);
                    println!("  时间范围: {} 天", result["time_range_days"]);
                    
                    if let Some(stats) = result.get("statistics") {
                        println!("  📊 统计信息:");
                        println!("    总提交数: {}", stats["total_commits"]);
                        println!("    贡献者数: {}", stats["total_contributors"]);
                        println!("    文件总数: {}", stats["total_files"]);
                        println!("    代码行数: {}", stats["code_lines"]);
                        println!("    语言分布:");
                        if let Some(langs) = stats["languages"].as_object() {
                            for (lang, percent) in langs {
                                println!("      {}: {}%", lang, percent);
                            }
                        }
                    }
                    
                    if let Some(contributors) = result.get("contributors") {
                        if let Some(list) = contributors.as_array() {
                            println!("  👥 贡献者 (前 3 名):");
                            for (i, contributor) in list.iter().take(3).enumerate() {
                                println!("    {}. {} - {} 次提交 ({}%)",
                                    i + 1,
                                    contributor["name"],
                                    contributor["commits"],
                                    contributor["percentage"]
                                );
                            }
                        }
                    }
                    
                    if let Some(history) = result.get("history") {
                        println!("  📅 历史趋势:");
                        println!("    日均提交: {}", history["commits_per_day"]);
                        println!("    活跃天数: {}", history["active_days"]);
                        println!("    提交趋势: {}", history["commit_trend"]);
                    }
                } else {
                    println!("  ❌ 分析失败: {}", result["error"]);
                }
            }
            Err(e) => println!("  ❌ 分析失败: {}", e),
        }
        println!();
    }

    // 测试 4: 参数验证
    println!("🔍 测试 4: 参数验证");
    println!("{}", "-".repeat(80));
    
    println!("测试 4.1: 无效的 Git 操作");
    let params = json!({
        "repository_path": "/path/to/repo",
        "operation": "invalid_operation"
    });

    match operation_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    println!("测试 4.2: commit 操作缺少 message");
    let params = json!({
        "repository_path": "/path/to/repo",
        "operation": "commit"
    });

    match operation_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    // 测试 5: 批量获取工具
    println!("📦 测试 5: 获取所有版本控制工具");
    println!("{}", "-".repeat(80));
    
    let all_tools = get_all_version_control_tools();
    println!("总共 {} 个版本控制工具:\n", all_tools.len());
    
    for (i, tool) in all_tools.iter().enumerate() {
        println!("{}. {} - {}", 
            i + 1, 
            tool.name().unwrap_or("unknown"), 
            tool.description()
        );
    }
    println!();

    // 总结
    println!("{}", "=".repeat(80));
    println!("✅ 版本控制工具演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 工具统计:");
    println!("  - Git 状态查询: git_status");
    println!("  - Git 操作: git_operation");
    println!("  - 仓库分析: repository_analyzer");
    println!();
    println!("💡 使用建议:");
    println!("  1. Git 状态: 查询仓库当前状态和变更");
    println!("  2. Git 操作: 执行提交、推送、拉取等操作");
    println!("  3. 仓库分析: 分析仓库统计、贡献者和历史");
    println!();
    println!("🚀 下一步:");
    println!("  - 集成真实的 Git 库（如 git2-rs）");
    println!("  - 添加更多 Git 操作（cherry-pick, stash, tag）");
    println!("  - 支持多仓库管理和批量操作");
    println!("{}", "=".repeat(80));
}

