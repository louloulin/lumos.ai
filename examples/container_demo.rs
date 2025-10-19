//! 容器管理工具演示程序

use lumosai_core::tool::builtin::container::*;
use lumosai_core::tool::{ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("🐳 容器管理工具演示\n");
    println!("{}", "=".repeat(80));
    println!();

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // 测试 1: Docker 管理
    println!("🔧 测试 1: Docker 管理");
    println!("{}", "-".repeat(80));

    let tools = get_all_container_tools();
    let docker_tool = &tools[0];

    println!("工具名称: {}", docker_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", docker_tool.description());
    println!();

    // 场景 1: 列出所有容器
    println!("场景 1: 列出所有容器");
    let params = json!({"operation": "list"});
    match docker_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == true {
                println!("  ✅ 操作: {}", result["operation"]);
                if let Some(containers) = result.get("containers") {
                    if let Some(list) = containers.as_array() {
                        println!("  📦 容器列表 ({} 个):", list.len());
                        for container in list {
                            println!("    - {} ({}) - 状态: {}",
                                container["name"], container["image"], container["status"]);
                        }
                    }
                }
            }
        }
        Err(e) => println!("  ❌ 操作失败: {}", e),
    }
    println!();

    // 场景 2: 启动容器
    println!("场景 2: 启动容器");
    let params = json!({"operation": "start", "container_name": "web-server"});
    match docker_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == true {
                println!("  ✅ 容器名称: {}", result["container_name"]);
                println!("  状态: {}", result["status"]);
            }
        }
        Err(e) => println!("  ❌ 操作失败: {}", e),
    }
    println!();

    // 测试 2: 镜像管理
    println!("🖼️  测试 2: 镜像管理");
    println!("{}", "-".repeat(80));

    let image_tool = &tools[1];
    println!("工具名称: {}", image_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", image_tool.description());
    println!();

    // 场景 1: 列出所有镜像
    println!("场景 1: 列出所有镜像");
    let params = json!({"operation": "list"});
    match image_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == true {
                println!("  ✅ 操作: {}", result["operation"]);
                if let Some(images) = result.get("images") {
                    if let Some(list) = images.as_array() {
                        println!("  🖼️  镜像列表 ({} 个):", list.len());
                        for image in list {
                            println!("    - {}:{} - {}",
                                image["repository"], image["tag"], image["size"]);
                        }
                    }
                }
            }
        }
        Err(e) => println!("  ❌ 操作失败: {}", e),
    }
    println!();

    // 场景 2: 构建镜像
    println!("场景 2: 构建镜像");
    let params = json!({
        "operation": "build",
        "image_name": "myapp",
        "tag": "v1.0.0",
        "dockerfile_path": "./Dockerfile"
    });
    match image_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == true {
                println!("  ✅ 镜像名称: {}", result["image_name"]);
                println!("  构建时间: {} 秒", result["build_time_seconds"]);
                println!("  镜像 ID: {}", result["image_id"]);
                println!("  大小: {}", result["size"]);
            }
        }
        Err(e) => println!("  ❌ 操作失败: {}", e),
    }
    println!();

    // 测试 3: 容器编排
    println!("🎭 测试 3: 容器编排");
    println!("{}", "-".repeat(80));

    let orchestration_tool = &tools[2];
    println!("工具名称: {}", orchestration_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", orchestration_tool.description());
    println!();

    // 场景 1: 列出所有服务
    println!("场景 1: 列出所有服务");
    let params = json!({"operation": "list"});
    match orchestration_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == true {
                println!("  ✅ 操作: {}", result["operation"]);
                if let Some(services) = result.get("services") {
                    if let Some(list) = services.as_array() {
                        println!("  🎭 服务列表 ({} 个):", list.len());
                        for service in list {
                            println!("    - {} ({}) - {} 副本 - 状态: {}",
                                service["name"], service["image"],
                                service["replicas"], service["status"]);
                        }
                    }
                }
            }
        }
        Err(e) => println!("  ❌ 操作失败: {}", e),
    }
    println!();


    // 场景 2: 部署服务
    println!("场景 2: 部署服务");
    let params = json!({
        "operation": "deploy",
        "service_name": "web-service",
        "image": "nginx:latest",
        "replicas": 3
    });
    match orchestration_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == true {
                println!("  ✅ 服务名称: {}", result["service_name"]);
                println!("  镜像: {}", result["image"]);
                println!("  副本数: {}", result["replicas"]);
            }
        }
        Err(e) => println!("  ❌ 操作失败: {}", e),
    }
    println!();

    // 场景 3: 扩容服务
    println!("场景 3: 扩容服务");
    let params = json!({
        "operation": "scale",
        "service_name": "api-service",
        "replicas": 5
    });
    match orchestration_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == true {
                println!("  ✅ 服务名称: {}", result["service_name"]);
                println!("  之前副本数: {}", result["previous_replicas"]);
                println!("  新副本数: {}", result["new_replicas"]);
            }
        }
        Err(e) => println!("  ❌ 操作失败: {}", e),
    }
    println!();

    // 测试 4: 参数验证
    println!("🔍 测试 4: 参数验证");
    println!("{}", "-".repeat(80));

    println!("测试 4.1: 无效的 Docker 操作");
    let params = json!({"operation": "invalid_operation"});
    match docker_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    println!("测试 4.2: start 操作缺少 container_name");
    let params = json!({"operation": "start"});
    match docker_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    // 测试 5: 批量获取工具
    println!("📦 测试 5: 获取所有容器管理工具");
    println!("{}", "-".repeat(80));

    let all_tools = get_all_container_tools();
    println!("总共 {} 个容器管理工具:\n", all_tools.len());

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
    println!("✅ 容器管理工具演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 工具统计:");
    println!("  - Docker 管理: docker_manager");
    println!("  - 镜像管理: image_manager");
    println!("  - 容器编排: orchestration");
    println!();
    println!("💡 使用建议:");
    println!("  1. Docker 管理: 管理容器生命周期和资源");
    println!("  2. 镜像管理: 构建、推送、拉取镜像");
    println!("  3. 容器编排: 部署和管理多容器应用");
    println!();
    println!("🚀 下一步:");
    println!("  - 集成真实的 Docker API（如 bollard）");
    println!("  - 添加 Kubernetes 支持");
    println!("  - 支持 Docker Compose 和 Swarm");
    println!("{}", "=".repeat(80));
}
