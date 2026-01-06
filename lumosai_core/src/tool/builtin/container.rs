//! 容器管理工具模块
//!
//! 提供 Docker 管理、镜像管理和容器编排等功能。

use crate::error::Result;
use lumos_macro::tool;
use serde_json::{json, Value};

/// Docker 管理（容器、网络、卷）
#[tool(name = "docker_manager", description = "Docker 管理（容器、网络、卷）")]
async fn docker_manager(
    operation: String,
    container_name: Option<String>,
    image: Option<String>,
) -> Result<Value> {
    let valid_ops = [
        "list", "start", "stop", "restart", "remove", "inspect", "logs",
    ];
    let op_lower = operation.to_lowercase();

    if !valid_ops.contains(&op_lower.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的操作: {}. 支持: list, start, stop, restart, remove, inspect, logs", operation)
        }));
    }

    if ["start", "stop", "restart", "remove", "inspect", "logs"].contains(&op_lower.as_str())
        && container_name.is_none()
    {
        return Ok(
            json!({"success": false, "error": format!("{} 操作需要 container_name", operation)}),
        );
    }

    let mut result = json!({"success": true, "operation": op_lower, "timestamp": chrono::Utc::now().to_rfc3339()});

    match op_lower.as_str() {
        "list" => {
            result["containers"] = json!([
                {"id": "abc123", "name": "web-server", "image": "nginx:latest", "status": "running", "ports": ["80:80"]},
                {"id": "def456", "name": "database", "image": "postgres:15", "status": "running", "ports": ["5432:5432"]}
            ]);
            result["total"] = json!(2);
        }
        "inspect" => {
            result["container_name"] = json!(container_name.clone().unwrap_or_default());
            result["details"] = json!({
                "id": "abc123",
                "name": container_name.unwrap_or_default(),
                "image": image.unwrap_or_else(|| "nginx:latest".to_string()),
                "status": "running",
                "cpu_usage": "15.5%",
                "memory_usage": "256MB"
            });
        }
        "logs" => {
            result["container_name"] = json!(container_name.unwrap_or_default());
            result["logs"] = json!(["[INFO] Server started", "[INFO] Request received"]);
        }
        _ => {
            result["container_name"] = json!(container_name.unwrap_or_default());
            result["status"] = json!(format!("{} successful", op_lower));
        }
    }

    Ok(result)
}

/// 镜像管理（构建、推送、拉取）
#[tool(name = "image_manager", description = "镜像管理（构建、推送、拉取）")]
async fn image_manager(
    operation: String,
    image_name: String,
    tag: Option<String>,
    dockerfile_path: Option<String>,
    registry: Option<String>,
) -> Result<Value> {
    let tag = tag.unwrap_or_else(|| "latest".to_string());
    let registry = registry.unwrap_or_else(|| "docker.io".to_string());
    let valid_ops = ["list", "build", "push", "pull", "remove", "inspect"];
    let op_lower = operation.to_lowercase();

    if !valid_ops.contains(&op_lower.as_str()) {
        return Ok(json!({"success": false, "error": format!("不支持的操作: {}", operation)}));
    }

    if op_lower == "build" && dockerfile_path.is_none() {
        return Ok(json!({"success": false, "error": "build 操作需要 dockerfile_path"}));
    }

    let mut result = json!({"success": true, "operation": op_lower, "image_name": image_name, "tag": tag, "timestamp": chrono::Utc::now().to_rfc3339()});

    match op_lower.as_str() {
        "list" => {
            result["images"] = json!([
                {"repository": "nginx", "tag": "latest", "size": "142MB"},
                {"repository": "postgres", "tag": "15", "size": "376MB"}
            ]);
        }
        "build" => {
            result["dockerfile_path"] = json!(dockerfile_path.unwrap_or_default());
            result["image_id"] = json!("sha256:abc123");
            result["size"] = json!("256MB");
        }
        "push" | "pull" => {
            result["registry"] = json!(registry);
            result["status"] = json!("success");
        }
        "inspect" => {
            result["details"] = json!({"id": "sha256:abc123", "size": "256MB", "layers": 12});
        }
        _ => {}
    }

    Ok(result)
}

/// 容器编排（部署、扩缩容、服务）
#[tool(name = "orchestration", description = "容器编排（部署、扩缩容、服务）")]
async fn orchestration(
    operation: String,
    service_name: String,
    replicas: Option<i64>,
    image: Option<String>,
) -> Result<Value> {
    let valid_ops = ["deploy", "scale", "update", "remove", "list", "inspect"];
    let op_lower = operation.to_lowercase();

    if !valid_ops.contains(&op_lower.as_str()) {
        return Ok(json!({"success": false, "error": format!("不支持的操作: {}", operation)}));
    }

    if op_lower == "scale" && replicas.is_none() {
        return Ok(json!({"success": false, "error": "scale 操作需要 replicas"}));
    }

    let mut result = json!({"success": true, "operation": op_lower, "service_name": service_name, "timestamp": chrono::Utc::now().to_rfc3339()});

    match op_lower.as_str() {
        "deploy" => {
            result["image"] = json!(image.unwrap_or_else(|| "nginx:latest".to_string()));
            result["replicas"] = json!(replicas.unwrap_or(3));
            result["status"] = json!("deployed");
        }
        "scale" => {
            result["previous_replicas"] = json!(3);
            result["new_replicas"] = json!(replicas.unwrap_or(3));
            result["status"] = json!("scaled");
        }
        "list" => {
            result["services"] = json!([
                {"name": "web-service", "image": "nginx:latest", "replicas": "3/3", "status": "running"},
                {"name": "api-service", "image": "node:18", "replicas": "5/5", "status": "running"}
            ]);
        }
        "inspect" => {
            result["details"] = json!({"name": service_name, "replicas": {"desired": 3, "current": 3}, "status": "running"});
        }
        _ => {
            result["status"] = json!("success");
        }
    }

    Ok(result)
}

pub fn get_all_container_tools() -> Vec<Box<dyn crate::tool::Tool>> {
    vec![
        docker_manager_tool(),
        image_manager_tool(),
        orchestration_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{ToolExecutionContext, ToolExecutionOptions};

    #[tokio::test]
    async fn test_docker_manager() {
        let tool = docker_manager_tool();
        let context = ToolExecutionContext::default();
        let options = ToolExecutionOptions::default();
        let params = json!({"operation": "list"});
        let result = tool.execute(params, context, &options).await.unwrap();
        assert_eq!(result["success"], true);
    }
}
