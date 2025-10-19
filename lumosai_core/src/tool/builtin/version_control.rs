//! 版本控制工具模块
//!
//! 提供 Git 状态查询、操作和仓库分析等功能。
//!
//! ## 工具列表
//!
//! - `git_status_tool`: Git 状态查询（分支、变更、提交）
//! - `git_operation_tool`: Git 操作（提交、推送、拉取）
//! - `repository_analyzer_tool`: 仓库分析（统计、贡献者、历史）

use crate::error::Result;
use lumos_macro::tool;
use serde_json::{json, Value};

/// Git 状态查询（分支、变更、提交）
#[tool(
    name = "git_status",
    description = "Git 状态查询（分支、变更、提交）"
)]
async fn git_status(
    repository_path: String,
    show_untracked: Option<bool>,
    show_ignored: Option<bool>,
) -> Result<Value> {
    let show_untracked = show_untracked.unwrap_or(true);
    let show_ignored = show_ignored.unwrap_or(false);

    // Mock 实现：生成模拟的 Git 状态
    let mut result = json!({
        "success": true,
        "repository_path": repository_path,
        "branch": {
            "current": "main",
            "upstream": "origin/main",
            "ahead": 2,
            "behind": 0,
            "status": "ahead"
        },
        "changes": {
            "modified": 5,
            "added": 3,
            "deleted": 1,
            "renamed": 0,
            "copied": 0
        },
        "staged": {
            "files": 4,
            "insertions": 245,
            "deletions": 67
        },
        "unstaged": {
            "files": 4,
            "insertions": 128,
            "deletions": 32
        },
        "conflicts": 0,
        "stashes": 2,
        "last_commit": {
            "hash": "a1b2c3d4",
            "author": "John Doe",
            "date": "2025-10-19T10:30:00Z",
            "message": "feat: add new feature"
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if show_untracked {
        result["untracked"] = json!({
            "files": 2,
            "list": ["new_file.rs", "temp.txt"]
        });
    }

    if show_ignored {
        result["ignored"] = json!({
            "files": 15,
            "patterns": ["target/", "*.log", ".DS_Store"]
        });
    }

    Ok(result)
}

/// Git 操作（提交、推送、拉取）
#[tool(
    name = "git_operation",
    description = "Git 操作（提交、推送、拉取）"
)]
async fn git_operation(
    repository_path: String,
    operation: String,
    message: Option<String>,
    branch: Option<String>,
    force: Option<bool>,
) -> Result<Value> {
    let force = force.unwrap_or(false);
    let branch = branch.unwrap_or_else(|| "main".to_string());

    let valid_operations = vec!["commit", "push", "pull", "fetch", "merge", "rebase", "checkout"];
    let operation_lower = operation.to_lowercase();
    if !valid_operations.contains(&operation_lower.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的 Git 操作: {}. 支持的操作: commit, push, pull, fetch, merge, rebase, checkout", operation)
        }));
    }

    // 对于 commit 操作，message 是必需的
    if operation_lower == "commit" && message.is_none() {
        return Ok(json!({
            "success": false,
            "error": "commit 操作需要提供 message 参数"
        }));
    }

    // Mock 实现：生成模拟的操作结果
    let mut result = json!({
        "success": true,
        "repository_path": repository_path,
        "operation": operation_lower,
        "branch": branch,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    match operation_lower.as_str() {
        "commit" => {
            result["commit"] = json!({
                "hash": "e5f6g7h8",
                "message": message.unwrap_or_default(),
                "files_changed": 4,
                "insertions": 245,
                "deletions": 67,
                "author": "John Doe <john@example.com>"
            });
        }
        "push" => {
            result["push"] = json!({
                "remote": "origin",
                "branch": branch,
                "commits_pushed": 2,
                "force": force,
                "status": "success"
            });
        }
        "pull" => {
            result["pull"] = json!({
                "remote": "origin",
                "branch": branch,
                "commits_pulled": 3,
                "files_changed": 8,
                "conflicts": 0,
                "status": "success"
            });
        }
        "fetch" => {
            result["fetch"] = json!({
                "remote": "origin",
                "branches_updated": 2,
                "tags_updated": 1,
                "status": "success"
            });
        }
        "merge" => {
            result["merge"] = json!({
                "source_branch": branch,
                "target_branch": "main",
                "commits_merged": 5,
                "conflicts": 0,
                "status": "success"
            });
        }
        "rebase" => {
            result["rebase"] = json!({
                "onto": branch,
                "commits_rebased": 3,
                "conflicts": 0,
                "status": "success"
            });
        }
        "checkout" => {
            result["checkout"] = json!({
                "branch": branch,
                "previous_branch": "develop",
                "status": "success"
            });
        }
        _ => {}
    }

    Ok(result)
}

/// 仓库分析（统计、贡献者、历史）
#[tool(
    name = "repository_analyzer",
    description = "仓库分析（统计、贡献者、历史）"
)]
async fn repository_analyzer(
    repository_path: String,
    analysis_type: Option<String>,
    time_range_days: Option<i64>,
) -> Result<Value> {
    let analysis_type = analysis_type.unwrap_or_else(|| "all".to_string());
    let time_range_days = time_range_days.unwrap_or(30);

    let valid_types = vec!["all", "statistics", "contributors", "history", "files", "branches"];
    let type_lower = analysis_type.to_lowercase();
    if !valid_types.contains(&type_lower.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的分析类型: {}. 支持的类型: all, statistics, contributors, history, files, branches", analysis_type)
        }));
    }

    if time_range_days < 1 || time_range_days > 365 {
        return Ok(json!({
            "success": false,
            "error": format!("时间范围必须在 1-365 天之间，当前值: {}", time_range_days)
        }));
    }

    // Mock 实现：生成模拟的仓库分析数据
    let mut result = json!({
        "success": true,
        "repository_path": repository_path,
        "analysis_type": type_lower,
        "time_range_days": time_range_days,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if type_lower == "all" || type_lower == "statistics" {
        result["statistics"] = json!({
            "total_commits": 1250,
            "total_contributors": 15,
            "total_files": 342,
            "total_lines": 45678,
            "code_lines": 38234,
            "comment_lines": 5123,
            "blank_lines": 2321,
            "languages": {
                "Rust": 65.5,
                "TypeScript": 25.3,
                "JavaScript": 7.2,
                "Other": 2.0
            }
        });
    }

    if type_lower == "all" || type_lower == "contributors" {
        result["contributors"] = json!([
            {
                "name": "John Doe",
                "email": "john@example.com",
                "commits": 450,
                "insertions": 12345,
                "deletions": 5678,
                "percentage": 36.0
            },
            {
                "name": "Jane Smith",
                "email": "jane@example.com",
                "commits": 320,
                "insertions": 8765,
                "deletions": 3456,
                "percentage": 25.6
            },
            {
                "name": "Bob Johnson",
                "email": "bob@example.com",
                "commits": 280,
                "insertions": 7654,
                "deletions": 2987,
                "percentage": 22.4
            }
        ]);
    }

    if type_lower == "all" || type_lower == "history" {
        result["history"] = json!({
            "commits_per_day": 4.2,
            "peak_commit_day": "2025-10-15",
            "peak_commits": 25,
            "active_days": 180,
            "inactive_days": 15,
            "commit_trend": "increasing"
        });
    }

    if type_lower == "all" || type_lower == "files" {
        result["files"] = json!({
            "most_changed": [
                {"path": "src/main.rs", "changes": 125},
                {"path": "src/lib.rs", "changes": 98},
                {"path": "README.md", "changes": 67}
            ],
            "largest": [
                {"path": "src/core/engine.rs", "lines": 2345},
                {"path": "src/api/server.rs", "lines": 1876},
                {"path": "src/utils/helpers.rs", "lines": 1234}
            ]
        });
    }

    if type_lower == "all" || type_lower == "branches" {
        result["branches"] = json!({
            "total": 12,
            "active": 5,
            "merged": 45,
            "stale": 2,
            "list": [
                {"name": "main", "commits": 850, "last_commit": "2025-10-19"},
                {"name": "develop", "commits": 320, "last_commit": "2025-10-18"},
                {"name": "feature/new-api", "commits": 45, "last_commit": "2025-10-17"}
            ]
        });
    }

    Ok(result)
}

/// 获取所有版本控制工具
pub fn get_all_version_control_tools() -> Vec<Box<dyn crate::tool::Tool>> {
    vec![
        git_status_tool(),
        git_operation_tool(),
        repository_analyzer_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{ToolExecutionContext, ToolExecutionOptions};

    #[tokio::test]
    async fn test_git_status() {
        let tool = git_status_tool();
        let context = ToolExecutionContext::default();
        let options = ToolExecutionOptions::default();

        let params = json!({
            "repository_path": "/path/to/repo"
        });

        let result = tool.execute(params, context, &options).await.unwrap();
        assert_eq!(result["success"], true);
        assert!(result["branch"].is_object());
    }

    #[tokio::test]
    async fn test_git_operation() {
        let tool = git_operation_tool();
        let context = ToolExecutionContext::default();
        let options = ToolExecutionOptions::default();

        let params = json!({
            "repository_path": "/path/to/repo",
            "operation": "commit",
            "message": "test commit"
        });

        let result = tool.execute(params, context, &options).await.unwrap();
        assert_eq!(result["success"], true);
    }

    #[tokio::test]
    async fn test_repository_analyzer() {
        let tool = repository_analyzer_tool();
        let context = ToolExecutionContext::default();
        let options = ToolExecutionOptions::default();

        let params = json!({
            "repository_path": "/path/to/repo",
            "analysis_type": "statistics"
        });

        let result = tool.execute(params, context, &options).await.unwrap();
        assert_eq!(result["success"], true);
        assert!(result["statistics"].is_object());
    }
}

