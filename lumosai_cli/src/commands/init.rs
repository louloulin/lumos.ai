use std::path::{Path, PathBuf};
use std::fs;
use crate::error::{CliError, CliResult};
use crate::template::{TemplateManager, TemplateType};
use crate::util::{check_rust_toolchain, create_dir_all};
use dialoguer::{Input, Select, Confirm};
use colored::Colorize;

/// 初始化一个新的Lumos AI项目
pub async fn run(
    name: Option<String>,
    template_str: Option<String>,
    template_url: Option<String>,
    output: Option<PathBuf>,
) -> CliResult<()> {
    // 检查Rust工具链
    check_rust_toolchain()?;
    
    // 获取项目名称
    let project_name = match name {
        Some(n) => n,
        None => {
            match Input::<String>::new()
                .with_prompt("项目名称")
                .default("my_lumosai_project".to_string())
                .interact() {
                    Ok(input) => input,
                    Err(e) => return Err(CliError::Interaction(e.to_string()))
                }
        }
    };
    
    // 创建模板管理器
    let template_manager = TemplateManager::new()?;
    
    // 如果提供了远程模板URL，先下载模板
    if let Some(url) = template_url {
        let template_name = match &template_str {
            Some(name) => name.clone(),
            None => {
                // 从URL提取模板名称，例如从GitHub仓库名或URL的最后一部分
                let name = if url.contains("github.com") {
                    // 从GitHub URL中提取仓库名
                    url.split('/')
                       .last()
                       .unwrap_or("custom-template")
                       .replace(".git", "")
                } else {
                    // 从其他URL中提取最后一部分
                    url.split('/')
                       .last()
                       .unwrap_or("custom-template")
                       .split('.')
                       .next()
                       .unwrap_or("custom-template")
                       .to_string()
                };
                
                println!("{}", format!("从URL提取的模板名称: {}", name).bright_blue());
                name
            }
        };
        
        // 下载模板
        template_manager.download_template(&url, &template_name)?;
        
        // 设置模板类型为已下载的模板
        let template_type = TemplateType::Custom(template_name);
        
        // 确定输出目录并创建项目
        let output_dir = get_output_directory(output, &project_name)?;
        template_manager.create_project(&template_type, &project_name, &output_dir)?;
        
        print_completion_message(&output_dir);
        return Ok(());
    }
    
    // 获取模板类型
    let template_type = match template_str {
        Some(t) => TemplateType::from_str(&t),
        None => {
            // 修复：确保有效的选项索引范围
            let choices = &["agent", "workflow", "rag", "custom"];
            
            let selection = match Select::new()
                .with_prompt("选择模板类型")
                .items(choices)
                .default(0)
                .interact() {
                    Ok(sel) => sel,
                    Err(e) => return Err(CliError::Interaction(e.to_string()))
                };
                
            // 这里不再使用match避免可能的越界问题
            if selection < choices.len() {
                match choices[selection] {
                    "agent" => TemplateType::Agent,
                    "workflow" => TemplateType::Workflow,
                    "rag" => TemplateType::Rag,
                    "custom" => {
                        let custom_type = match Input::<String>::new()
                            .with_prompt("输入自定义模板名称")
                            .interact() {
                                Ok(input) => input,
                                Err(e) => return Err(CliError::Interaction(e.to_string()))
                            };
                        TemplateType::Custom(custom_type)
                    },
                    _ => TemplateType::Agent, // 默认情况
                }
            } else {
                // 如果选择索引无效，默认使用Agent模板
                println!("{}", "无效选择，默认使用Agent模板".bright_yellow());
                TemplateType::Agent
            }
        }
    };
    
    // 确定输出目录
    let output_dir = get_output_directory(output, &project_name)?;
    
    println!("{}", format!("初始化 {} 项目: {}", template_type.name(), project_name).bright_blue());
    
    // 使用模板管理器创建项目
    template_manager.create_project(&template_type, &project_name, &output_dir)?;
    
    print_completion_message(&output_dir);
    
    Ok(())
}

// 辅助函数：确定输出目录
fn get_output_directory(output: Option<PathBuf>, project_name: &str) -> CliResult<PathBuf> {
    match output {
        Some(path) => Ok(path),
        None => {
            let current_dir = std::env::current_dir()
                .map_err(|e| CliError::Io(e))?;
                
            let dir_input = match Input::<String>::new()
                .with_prompt("输出目录")
                .default(current_dir.join(project_name).to_string_lossy().to_string())
                .interact() {
                    Ok(input) => input,
                    Err(e) => return Err(CliError::Interaction(e.to_string()))
                };
                
            let output_dir = PathBuf::from(dir_input);
            
            // 检查输出目录是否存在
            if output_dir.exists() {
                // 检查目录是否为空
                let is_empty = match fs::read_dir(&output_dir) {
                    Ok(mut entries) => entries.next().is_none(),
                    Err(_) => false,
                };
                
                if !is_empty {
                    let confirm = match Confirm::new()
                        .with_prompt(format!("目录 {} 不为空，是否继续? 这可能会覆盖现有文件", output_dir.display()))
                        .default(false)
                        .interact() {
                            Ok(result) => result,
                            Err(e) => return Err(CliError::Interaction(e.to_string()))
                        };
                        
                    if !confirm {
                        return Err(CliError::canceled("初始化已取消"));
                    }
                }
            } else {
                // 创建输出目录
                create_dir_all(&output_dir)?;
            }
            
            Ok(output_dir)
        }
    }
}

// 辅助函数：打印完成信息
fn print_completion_message(output_dir: &Path) {
    println!("{}", format!("项目已创建: {}", output_dir.display()).bright_green());
    println!("{}", "要开始开发，请运行:".bright_blue());
    println!("{}", format!("  cd {}", output_dir.display()).bright_cyan());
    println!("{}", "  cargo run".bright_cyan());
    println!("{}", "  # 或者使用开发服务器:".bright_blue());
    println!("{}", "  lumos dev".bright_cyan());
} 