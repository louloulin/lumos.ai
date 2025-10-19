//! 容器管理工具演示程序

use lumosai_core::tool::builtin::container::*;
use lumosai_core::tool::{ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("�� 容器管理工具演示\n");
    println!("{}", "=".repeat(80));

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();
    let tools = get_all_container_tools();

    // 测试 1: Docker 管理
    println!("\n🐋 测试 1: Docker 管理");
    println!("{}", "-".repeat(80));
    let docker_tool = &tools[0];

    for (desc, params) in vec![
        ("场景 1: 列出容器", json!({"operation": "list"})),
        (
            "场景 2: 启动容器",
            json!({"operation": "start", "container_name": "web-server"}),
        ),
        (
            "场景 3: 查看详情",
            json!({"operation": "inspect", "container_name": "web-server"}),
        ),
    ] {
        println!("\n{}", desc);
        match docker_tool.execute(params, context.clone(), &options).await {
            Ok(r) if r["success"] == true => {
                println!("  ✅ 操作: {}", r["operation"]);
                if let Some(c) = r.get("containers") {
                    println!("  容器数: {}", r["total"]);
                }
            }
            Ok(r) => println!("  ❌ 失败: {}", r["error"]),
            Err(e) => println!("  ❌ 错误: {}", e),
        }
    }

    // 测试 2: 镜像管理
    println!("\n\n🖼️  测试 2: 镜像管理");
    println!("{}", "-".repeat(80));
    let image_tool = &tools[1];

    for (desc, params) in vec![
        (
            "场景 1: 列出镜像",
            json!({"operation": "list", "image_name": "nginx"}),
        ),
        (
            "场景 2: 构建镜像",
            json!({"operation": "build", "image_name": "myapp", "dockerfile_path": "./Dockerfile"}),
        ),
        (
            "场景 3: 推送镜像",
            json!({"operation": "push", "image_name": "myapp", "tag": "v1.0"}),
        ),
    ] {
        println!("\n{}", desc);
        match image_tool.execute(params, context.clone(), &options).await {
            Ok(r) if r["success"] == true => {
                println!("  ✅ 操作: {} - {}", r["operation"], r["image_name"])
            }
            Ok(r) => println!("  ❌ 失败: {}", r["error"]),
            Err(e) => println!("  ❌ 错误: {}", e),
        }
    }

    // 测试 3: 容器编排
    println!("\n\n🎭 测试 3: 容器编排");
    println!("{}", "-".repeat(80));
    let orch_tool = &tools[2];

    for (desc, params) in vec![
        (
            "场景 1: 部署服务",
            json!({"operation": "deploy", "service_name": "web", "image": "nginx:latest", "replicas": 3}),
        ),
        (
            "场景 2: 扩容服务",
            json!({"operation": "scale", "service_name": "web", "replicas": 5}),
        ),
        (
            "场景 3: 列出服务",
            json!({"operation": "list", "service_name": "web"}),
        ),
    ] {
        println!("\n{}", desc);
        match orch_tool.execute(params, context.clone(), &options).await {
            Ok(r) if r["success"] == true => {
                println!("  ✅ 操作: {} - {}", r["operation"], r["service_name"])
            }
            Ok(r) => println!("  ❌ 失败: {}", r["error"]),
            Err(e) => println!("  ❌ 错误: {}", e),
        }
    }

    // 测试 4: 参数验证
    println!("\n\n🔍 测试 4: 参数验证");
    println!("{}", "-".repeat(80));

    println!("\n测试 4.1: 无效操作");
    if let Ok(r) = docker_tool
        .execute(json!({"operation": "invalid"}), context.clone(), &options)
        .await
    {
        if r["success"] == false {
            println!("  ✅ 正确拒绝: {}", r["error"]);
        }
    }

    println!("\n测试 4.2: 缺少参数");
    if let Ok(r) = image_tool
        .execute(
            json!({"operation": "build", "image_name": "test"}),
            context.clone(),
            &options,
        )
        .await
    {
        if r["success"] == false {
            println!("  ✅ 正确拒绝: {}", r["error"]);
        }
    }

    // 总结
    println!("\n\n{}", "=".repeat(80));
    println!("✅ 容器管理工具演示完成！");
    println!("{}", "=".repeat(80));
    println!("\n📊 工具统计:");
    println!("  - Docker 管理: docker_manager");
    println!("  - 镜像管理: image_manager");
    println!("  - 容器编排: orchestration");
    println!("\n💡 使用建议:");
    println!("  1. Docker 管理: 管理容器生命周期");
    println!("  2. 镜像管理: 构建和分发镜像");
    println!("  3. 容器编排: 部署多容器应用");
    println!("{}", "=".repeat(80));
}
