use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::error::{CliError, CliResult};
use crate::util::create_dir_all;
use std::fmt;
use colored::Colorize;

/// 模板类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateType {
    Agent,
    Workflow,
    Rag,
    Custom(String),
}

impl TemplateType {
    /// 将字符串转换为模板类型
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "agent" => TemplateType::Agent,
            "workflow" => TemplateType::Workflow,
            "rag" => TemplateType::Rag,
            _ => TemplateType::Custom(s.to_string()),
        }
    }
    
    /// 获取模板类型的名称
    pub fn name(&self) -> String {
        match self {
            TemplateType::Agent => "Agent".to_string(),
            TemplateType::Workflow => "Workflow".to_string(),
            TemplateType::Rag => "RAG".to_string(),
            TemplateType::Custom(name) => format!("Custom ({})", name),
        }
    }
    
    /// 获取模板类型的目录名称
    pub fn dir_name(&self) -> String {
        match self {
            TemplateType::Agent => "agent".to_string(),
            TemplateType::Workflow => "workflow".to_string(),
            TemplateType::Rag => "rag".to_string(),
            TemplateType::Custom(name) => name.clone(),
        }
    }
}

impl fmt::Display for TemplateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// 模板文件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

/// 模板定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub version: String,
    pub template_type: String,
    pub files: Vec<TemplateFile>,
    pub dependencies: HashMap<String, String>,
}

impl Template {
    /// 从模板文件加载模板
    pub fn load(template_type: &TemplateType) -> CliResult<Self> {
        // 获取模板目录路径
        let template_dir = Self::get_template_dir()?;
        let type_dir = template_dir.join(template_type.dir_name());
        
        // 确保模板目录存在
        if !type_dir.exists() {
            return Err(CliError::template_not_found(&template_type.name()));
        }
        
        // 读取模板配置
        let config_path = type_dir.join("template.json");
        if !config_path.exists() {
            return Err(CliError::template_config_not_found(&config_path));
        }
        
        // 解析模板配置
        let template_json = fs::read_to_string(&config_path)
            .map_err(|e| CliError::io_error(e, &config_path))?;
            
        let template: Template = serde_json::from_str(&template_json)
            .map_err(|e| CliError::template_parse_error(e))?;
            
        Ok(template)
    }
    
    /// 获取模板目录
    fn get_template_dir() -> CliResult<PathBuf> {
        // 获取用户主目录
        let home_dir = dirs::home_dir()
            .ok_or_else(|| CliError::Other("无法确定用户主目录".to_string()))?;
            
        // 构建模板目录路径
        let template_dir = home_dir.join(".lumosai").join("templates");
        
        // 确保目录存在
        if !template_dir.exists() {
            create_dir_all(&template_dir)?;
        }
        
        Ok(template_dir)
    }
    
    /// 应用模板到指定目录
    pub fn apply(&self, project_name: &str, output_dir: &Path) -> CliResult<()> {
        // 确保输出目录存在
        if !output_dir.exists() {
            create_dir_all(output_dir)?;
        }
        
        // 处理所有模板文件
        for file in &self.files {
            // 替换模板变量
            let content = self.process_template_content(&file.content, project_name);
            
            // 构建文件路径，替换模板变量
            let file_path_str = file.path.replace("{{project_name}}", project_name);
            let file_path = output_dir.join(file_path_str);
            
            // 确保父目录存在
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    create_dir_all(parent)?;
                }
            }
            
            // 写入文件内容
            fs::write(&file_path, content)
                .map_err(|e| CliError::io_error(e, &file_path))?;
                
            println!("{}", format!("创建文件: {}", file_path.display()).bright_green());
        }
        
        // 创建Cargo.toml（如果依赖不为空）
        if !self.dependencies.is_empty() {
            self.create_cargo_toml(project_name, output_dir)?;
        }
        
        Ok(())
    }
    
    /// 处理模板内容，替换变量
    fn process_template_content(&self, content: &str, project_name: &str) -> String {
        content
            .replace("{{project_name}}", project_name)
            .replace("{{template_name}}", &self.name)
            .replace("{{template_version}}", &self.version)
    }
    
    /// 创建Cargo.toml文件
    fn create_cargo_toml(&self, project_name: &str, output_dir: &Path) -> CliResult<()> {
        let cargo_path = output_dir.join("Cargo.toml");
        
        let mut cargo_content = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
            project_name
        );
        
        // 添加依赖项
        for (dep, version) in &self.dependencies {
            cargo_content.push_str(&format!("{} = \"{}\"\n", dep, version));
        }
        
        // 写入Cargo.toml
        fs::write(&cargo_path, cargo_content)
            .map_err(|e| CliError::io_error(e, &cargo_path))?;
            
        println!("{}", format!("创建文件: {}", cargo_path.display()).bright_green());
        
        Ok(())
    }
}

/// 模板管理器
pub struct TemplateManager {
    template_dir: PathBuf,
}

impl TemplateManager {
    /// 创建新的模板管理器
    pub fn new() -> CliResult<Self> {
        let template_dir = Template::get_template_dir()?;
        Ok(TemplateManager { template_dir })
    }
    
    /// 列出所有可用模板
    pub fn list_templates(&self) -> CliResult<Vec<(String, String)>> {
        // 检查模板目录是否存在
        if !self.template_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut templates = Vec::new();
        
        // 读取模板目录下的所有子目录
        for entry in fs::read_dir(&self.template_dir)
            .map_err(|e| CliError::io_error(e, &self.template_dir))? {
                
            let entry = entry.map_err(|e| CliError::io_error(e, &self.template_dir))?;
            let path = entry.path();
            
            // 只处理目录
            if path.is_dir() {
                let config_path = path.join("template.json");
                
                // 如果存在模板配置文件，读取基本信息
                if config_path.exists() {
                    let template_json = fs::read_to_string(&config_path)
                        .map_err(|e| CliError::io_error(e, &config_path))?;
                        
                    if let Ok(template) = serde_json::from_str::<Template>(&template_json) {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                            
                        templates.push((name, template.description));
                    }
                }
            }
        }
        
        Ok(templates)
    }
    
    /// 下载模板
    pub fn download_template(&self, url: &str, name: &str) -> CliResult<()> {
        // 在实际实现中，这里应该从URL下载模板
        println!("{}", format!("下载模板 {}: {}", name, url).bright_blue());
        
        // 创建模板目录
        let template_path = self.template_dir.join(name);
        if !template_path.exists() {
            create_dir_all(&template_path)?;
        }
        
        // 这里应该实现实际的模板下载逻辑
        // 暂时创建一个基本模板以供测试
        self.create_basic_template(&template_path, name)?;
        
        println!("{}", format!("模板 {} 已下载到 {}", name, template_path.display()).bright_green());
        
        Ok(())
    }
    
    /// 创建基本模板（临时实现）
    fn create_basic_template(&self, template_path: &Path, name: &str) -> CliResult<()> {
        // 创建基本的模板配置
        let template = Template {
            name: name.to_string(),
            description: format!("从URL下载的 {} 模板", name),
            version: "0.1.0".to_string(),
            template_type: "custom".to_string(),
            files: vec![
                TemplateFile {
                    path: "src/main.rs".to_string(),
                    content: r#"fn main() {
    println!("Hello from {{project_name}}!");
}
"#.to_string(),
                },
                TemplateFile {
                    path: "README.md".to_string(),
                    content: r#"# {{project_name}}

A Lumos AI project created from the {{template_name}} template.

## Getting Started

```bash
cargo run
```
"#.to_string(),
                },
            ],
            dependencies: {
                let mut deps = HashMap::new();
                deps.insert("lumosai".to_string(), "0.1.0".to_string());
                deps
            },
        };
        
        // 保存模板配置
        let config_path = template_path.join("template.json");
        let config_json = serde_json::to_string_pretty(&template)
            .map_err(|e| CliError::template_parse_error(e))?;
            
        fs::write(&config_path, config_json)
            .map_err(|e| CliError::io_error(e, &config_path))?;
            
        Ok(())
    }
    
    /// 移除模板
    pub fn remove_template(&self, name: &str) -> CliResult<()> {
        let template_path = self.template_dir.join(name);
        
        // 检查模板是否存在
        if !template_path.exists() {
            return Err(CliError::template_not_found(name));
        }
        
        // 删除模板目录
        fs::remove_dir_all(&template_path)
            .map_err(|e| CliError::io_error(e, &template_path))?;
            
        println!("{}", format!("模板 {} 已删除", name).bright_green());
        
        Ok(())
    }
    
    /// 使用模板创建项目
    pub fn create_project(&self, template_type: &TemplateType, project_name: &str, output_dir: &Path) -> CliResult<()> {
        // 加载模板
        let template = match Template::load(template_type) {
            Ok(t) => t,
            Err(e) => {
                // 如果找不到模板，尝试创建基本项目
                println!("{}", format!("找不到 {} 模板: {}", template_type.name(), e).bright_yellow());
                println!("{}", "创建基本项目...".bright_blue());
                return self.create_basic_project(project_name, output_dir);
            }
        };
        
        // 应用模板
        template.apply(project_name, output_dir)?;
        
        Ok(())
    }
    
    /// 创建基本项目
    fn create_basic_project(&self, project_name: &str, output_dir: &Path) -> CliResult<()> {
        // 确保输出目录存在
        if !output_dir.exists() {
            create_dir_all(output_dir)?;
        }
        
        // 创建src目录
        let src_dir = output_dir.join("src");
        create_dir_all(&src_dir)?;
        
        // 创建main.rs
        let main_content = format!(
            r#"fn main() {{
    println!("Hello from {}!");
}}
"#,
            project_name
        );
        
        let main_path = src_dir.join("main.rs");
        fs::write(&main_path, main_content)
            .map_err(|e| CliError::io_error(e, &main_path))?;
            
        println!("{}", format!("创建文件: {}", main_path.display()).bright_green());
        
        // 创建Cargo.toml
        let cargo_content = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
lumosai = "0.1.0"
"#,
            project_name
        );
        
        let cargo_path = output_dir.join("Cargo.toml");
        fs::write(&cargo_path, cargo_content)
            .map_err(|e| CliError::io_error(e, &cargo_path))?;
            
        println!("{}", format!("创建文件: {}", cargo_path.display()).bright_green());
        
        // 创建README.md
        let readme_content = format!(
            r#"# {}

A Lumos AI project.

## Getting Started

```bash
cargo run
```
"#,
            project_name
        );
        
        let readme_path = output_dir.join("README.md");
        fs::write(&readme_path, readme_content)
            .map_err(|e| CliError::io_error(e, &readme_path))?;
            
        println!("{}", format!("创建文件: {}", readme_path.display()).bright_green());
        
        Ok(())
    }
} 