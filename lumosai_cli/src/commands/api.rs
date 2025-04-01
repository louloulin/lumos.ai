use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Write};
use colored::Colorize;
use crate::error::{CliResult, CliError};
use std::env;

/// 生成API端点
pub async fn run(
    project_dir: Option<PathBuf>,
    output: Option<PathBuf>,
    agents: Option<Vec<String>>,
) -> CliResult<()> {
    let project_dir = match project_dir {
        Some(dir) => dir,
        None => env::current_dir().map_err(|e| CliError::io("获取当前目录失败", e))?,
    };

    // 检查项目目录
    if !project_dir.exists() {
        return Err(CliError::path_not_found(
            project_dir.to_string_lossy().to_string(),
            "项目目录不存在",
        ));
    }

    println!("{}", "正在生成API端点...".bright_cyan());
    println!("{}", format!("项目目录: {}", project_dir.display()).bright_blue());

    // 确定输出目录
    let output_dir = match output {
        Some(dir) => dir,
        None => project_dir.join("src").join("api"),
    };

    println!("{}", format!("输出目录: {}", output_dir.display()).bright_blue());

    // 创建输出目录（如果不存在）
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)
            .map_err(|e| CliError::io(format!("无法创建输出目录: {}", output_dir.display()), e))?;
    }

    // 获取所有代理信息
    let agents_dir = project_dir.join("src").join("agents");
    let available_agents = if agents_dir.exists() {
        find_agents(&agents_dir)?
    } else {
        Vec::new()
    };

    // 如果指定了特定代理，过滤代理列表
    let agents_to_process = match agents {
        Some(specified_agents) => {
            // 验证所有指定的代理都存在
            for agent in &specified_agents {
                if !available_agents.contains(agent) {
                    return Err(CliError::invalid_input(
                        format!("指定的代理 '{}' 不存在", agent),
                        "请检查代理名称或添加该代理"
                    ));
                }
            }
            specified_agents
        },
        None => available_agents,
    };

    if agents_to_process.is_empty() {
        println!("{}", "警告: 未找到任何代理".bright_yellow());
        println!("{}", "请先创建至少一个代理，或指定有效的代理".bright_yellow());
        return Ok(());
    }

    println!("{}", format!("找到 {} 个代理:", agents_to_process.len()).bright_blue());
    for agent in &agents_to_process {
        println!("  - {}", agent);
    }

    // 生成API模块文件
    generate_api_module(&output_dir, &agents_to_process)?;

    // 为每个代理生成API端点
    for agent in &agents_to_process {
        generate_agent_api(&output_dir, agent, &project_dir)?;
    }

    println!("{}", "API端点生成完成！".bright_green());
    println!("{}", format!("API文件位于: {}", output_dir.display()).bright_green());

    Ok(())
}

/// 查找项目中的所有代理
fn find_agents(agents_dir: &Path) -> CliResult<Vec<String>> {
    let mut agents = Vec::new();

    if !agents_dir.exists() {
        return Ok(agents);
    }

    for entry in fs::read_dir(agents_dir)
        .map_err(|e| CliError::io(format!("无法读取代理目录: {}", agents_dir.display()), e))? {
        let entry = entry.map_err(|e| CliError::io("读取目录条目失败", e))?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    // 检查是否存在agent.rs文件来确认这是一个代理目录
                    if path.join("agent.rs").exists() || path.join("mod.rs").exists() {
                        agents.push(name_str.to_string());
                    }
                }
            }
        }
    }

    Ok(agents)
}

/// 生成API模块文件 (mod.rs)
fn generate_api_module(output_dir: &Path, agents: &[String]) -> CliResult<()> {
    let mod_path = output_dir.join("mod.rs");
    let mut content = String::new();
    
    // 添加代理模块声明
    for agent in agents {
        content.push_str(&format!("pub mod {};\n", agent));
    }
    
    // 添加API模块代码
    content.push_str("\nuse actix_web::{web, App, HttpServer, Responder, HttpResponse};\nuse serde::{Serialize, Deserialize};\n\n");
    
    // 添加响应结构
    content.push_str("#[derive(Serialize)]\nstruct ApiResponse<T> {\n    success: bool,\n    data: Option<T>,\n    error: Option<String>,\n}\n\n");
    
    // 添加路由配置函数
    content.push_str("pub fn configure_routes(cfg: &mut web::ServiceConfig) {\n");
    
    // 为每个代理添加路由
    for agent in agents {
        content.push_str(&format!("    cfg.service(web::scope(\"/{}\").configure({}::configure_routes));\n", agent, agent));
    }
    
    // 添加API端点信息
    content.push_str("    cfg.service(api_info);\n");
    content.push_str("}\n\n");
    
    // 添加API信息端点
    content.push_str("#[derive(Serialize)]\nstruct ApiInfo {\n    version: String,\n    agents: Vec<String>,\n}\n\n");
    content.push_str("#[actix_web::get(\"/\")]\nasync fn api_info() -> impl Responder {\n");
    content.push_str("    let info = ApiInfo {\n");
    content.push_str("        version: env!(\"CARGO_PKG_VERSION\").to_string(),\n");
    content.push_str("        agents: vec![\n");
    
    for agent in agents {
        content.push_str(&format!("            \"{}\".to_string(),\n", agent));
    }
    
    content.push_str("        ],\n");
    content.push_str("    };\n\n");
    content.push_str("    HttpResponse::Ok().json(ApiResponse {\n");
    content.push_str("        success: true,\n");
    content.push_str("        data: Some(info),\n");
    content.push_str("        error: None,\n");
    content.push_str("    })\n");
    content.push_str("}\n");
    
    // 写入文件
    fs::write(&mod_path, content)
        .map_err(|e| CliError::io(format!("无法写入文件: {}", mod_path.display()), e))?;
    
    println!("{}", format!("生成API模块: {}", mod_path.display()).bright_green());
    
    Ok(())
}

/// 为特定代理生成API端点
fn generate_agent_api(output_dir: &Path, agent_name: &str, project_dir: &Path) -> CliResult<()> {
    let agent_api_path = output_dir.join(format!("{}.rs", agent_name));
    let agent_dir = project_dir.join("src").join("agents").join(agent_name);
    
    // 读取代理定义文件以提取信息
    let agent_def_path = if agent_dir.join("agent.rs").exists() {
        agent_dir.join("agent.rs")
    } else if agent_dir.join("mod.rs").exists() {
        agent_dir.join("mod.rs")
    } else {
        return Err(CliError::path_not_found(
            agent_dir.to_string_lossy().to_string(),
            format!("代理 '{}' 的定义文件不存在", agent_name)
        ));
    };
    
    // 生成API文件内容
    let mut content = String::new();
    
    // 导入必要的依赖
    content.push_str("use actix_web::{web, HttpResponse, Responder};\n");
    content.push_str("use serde::{Serialize, Deserialize};\n");
    content.push_str("use crate::agents::*;\n");
    content.push_str(&format!("use crate::agents::{}::*;\n", agent_name));
    content.push_str("use crate::api::ApiResponse;\n\n");
    
    // 添加请求和响应结构
    content.push_str("#[derive(Deserialize)]\nstruct ChatRequest {\n    message: String,\n    conversation_id: Option<String>,\n}\n\n");
    
    content.push_str("#[derive(Serialize)]\nstruct ChatResponse {\n    response: String,\n    conversation_id: String,\n}\n\n");
    
    // 配置路由
    content.push_str("pub fn configure_routes(cfg: &mut web::ServiceConfig) {\n");
    content.push_str("    cfg.service(chat)\n");
    content.push_str("        .service(info);\n");
    content.push_str("}\n\n");
    
    // 添加聊天端点
    content.push_str("#[actix_web::post(\"/chat\")]\nasync fn chat(req: web::Json<ChatRequest>) -> impl Responder {\n");
    content.push_str(&format!("    let agent = {}Agent::new();\n", agent_name.replace('-', "_")));
    content.push_str("    \n");
    content.push_str("    match agent.process_message(&req.message, req.conversation_id.clone()).await {\n");
    content.push_str("        Ok((response, conversation_id)) => {\n");
    content.push_str("            HttpResponse::Ok().json(ApiResponse {\n");
    content.push_str("                success: true,\n");
    content.push_str("                data: Some(ChatResponse {\n");
    content.push_str("                    response,\n");
    content.push_str("                    conversation_id,\n");
    content.push_str("                }),\n");
    content.push_str("                error: None,\n");
    content.push_str("            })\n");
    content.push_str("        },\n");
    content.push_str("        Err(e) => {\n");
    content.push_str("            HttpResponse::InternalServerError().json(ApiResponse::<()> {\n");
    content.push_str("                success: false,\n");
    content.push_str("                data: None,\n");
    content.push_str("                error: Some(e.to_string()),\n");
    content.push_str("            })\n");
    content.push_str("        }\n");
    content.push_str("    }\n");
    content.push_str("}\n\n");
    
    // 添加代理信息端点
    content.push_str("#[derive(Serialize)]\nstruct AgentInfo {\n    name: String,\n    description: String,\n}\n\n");
    content.push_str("#[actix_web::get(\"/\")]\nasync fn info() -> impl Responder {\n");
    content.push_str(&format!("    let agent = {}Agent::new();\n", agent_name.replace('-', "_")));
    content.push_str("    \n");
    content.push_str("    let info = AgentInfo {\n");
    content.push_str(&format!("        name: \"{}\".to_string(),\n", agent_name));
    content.push_str("        description: agent.description().to_string(),\n");
    content.push_str("    };\n");
    content.push_str("    \n");
    content.push_str("    HttpResponse::Ok().json(ApiResponse {\n");
    content.push_str("        success: true,\n");
    content.push_str("        data: Some(info),\n");
    content.push_str("        error: None,\n");
    content.push_str("    })\n");
    content.push_str("}\n");
    
    // 写入文件
    fs::write(&agent_api_path, content)
        .map_err(|e| CliError::io(format!("无法写入文件: {}", agent_api_path.display()), e))?;
    
    println!("{}", format!("为代理 '{}' 生成API: {}", agent_name, agent_api_path.display()).bright_green());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_run_with_invalid_directory() {
        let result = run(
            Some(PathBuf::from("/非常不太可能存在的路径/abcdef")),
            None,
            None
        ).await;
        
        assert!(result.is_err());
        
        // 检查错误类型
        if let Err(e) = result {
            assert!(e.to_string().contains("项目目录不存在"));
        }
    }
    
    #[tokio::test]
    async fn test_run_with_no_agents() {
        // 创建临时项目目录
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        // 创建基本目录结构
        fs::create_dir_all(temp_path.join("src").join("agents")).unwrap();
        
        // 运行API生成命令，应该返回Ok但不生成任何文件
        let result = run(
            Some(temp_path.to_path_buf()),
            None,
            None
        ).await;
        
        assert!(result.is_ok());
        
        // 检查API目录是否存在
        let api_dir = temp_path.join("src").join("api");
        assert!(api_dir.exists());
        
        // 但是没有生成任何代理文件
        let entries = fs::read_dir(api_dir).unwrap().count();
        assert_eq!(entries, 0);
    }
    
    #[tokio::test]
    async fn test_run_with_invalid_specified_agent() {
        // 创建临时项目目录
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        // 创建基本目录结构
        fs::create_dir_all(temp_path.join("src").join("agents")).unwrap();
        
        // 指定不存在的代理
        let result = run(
            Some(temp_path.to_path_buf()),
            None,
            Some(vec!["non_existent_agent".to_string()])
        ).await;
        
        assert!(result.is_err());
        
        // 检查错误类型
        if let Err(e) = result {
            assert!(e.to_string().contains("指定的代理 'non_existent_agent' 不存在"));
        }
    }
    
    #[test]
    fn test_find_agents_with_empty_directory() {
        // 创建临时目录
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        // 目录不存在时应返回空列表
        let non_existent_dir = temp_path.join("non_existent");
        let result = find_agents(&non_existent_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
        
        // 创建空的agents目录
        let agents_dir = temp_path.join("agents");
        fs::create_dir_all(&agents_dir).unwrap();
        
        // 空目录应返回空列表
        let result = find_agents(&agents_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
    
    #[test]
    fn test_find_agents_with_valid_agents() {
        // 创建临时目录
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        let agents_dir = temp_path.join("agents");
        fs::create_dir_all(&agents_dir).unwrap();
        
        // 创建一个有效的代理目录，带有agent.rs
        let agent1_dir = agents_dir.join("agent1");
        fs::create_dir_all(&agent1_dir).unwrap();
        fs::write(agent1_dir.join("agent.rs"), "// Test agent").unwrap();
        
        // 创建另一个有效的代理目录，带有mod.rs
        let agent2_dir = agents_dir.join("agent2");
        fs::create_dir_all(&agent2_dir).unwrap();
        fs::write(agent2_dir.join("mod.rs"), "// Test agent").unwrap();
        
        // 创建一个无效的代理目录，不带有agent.rs或mod.rs
        let invalid_dir = agents_dir.join("invalid");
        fs::create_dir_all(&invalid_dir).unwrap();
        
        // 应该找到2个有效代理
        let result = find_agents(&agents_dir);
        assert!(result.is_ok());
        
        let agents = result.unwrap();
        assert_eq!(agents.len(), 2);
        assert!(agents.contains(&"agent1".to_string()));
        assert!(agents.contains(&"agent2".to_string()));
        assert!(!agents.contains(&"invalid".to_string()));
    }
} 