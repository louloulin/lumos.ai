use std::path::{Path, PathBuf};
use std::fs;
use std::process::{Command, Stdio};
use colored::Colorize;
use dialoguer::{Input, Select, Confirm, MultiSelect};
use crate::error::{CliError, CliResult};
use crate::util::create_dir_all;

pub struct CreateOptions {
    /// 项目名称
    pub name: String,
    /// 项目目录
    pub project_dir: PathBuf,
    /// 组件列表
    pub components: Vec<String>,
    /// LLM提供商
    pub llm_provider: Option<String>,
    /// LLM API密钥
    pub llm_api_key: Option<String>,
    /// 是否添加示例代码
    pub add_example: bool,
}

/// 创建新项目
pub async fn run(
    name: Option<String>,
    components: Option<Vec<String>>,
    llm_provider: Option<String>,
    llm_api_key: Option<String>,
    project_name: Option<String>,
    add_example: bool,
) -> CliResult<()> {
    // 获取项目名称
    let project_name = match project_name.or(name) {
        Some(n) => n,
        None => {
            Input::<String>::new()
                .with_prompt("项目名称")
                .default("my-lumosai-project".to_string())
                .interact()?
        }
    };
    
    // 项目目录
    let current_dir = std::env::current_dir()?;
    let project_dir = current_dir.join(&project_name);
    
    // 检查目录是否存在
    if project_dir.exists() {
        let is_empty = fs::read_dir(&project_dir)?.next().is_none();
        
        if !is_empty {
            let overwrite = Confirm::new()
                .with_prompt(format!("目录 {} 已存在且不为空。是否继续? (这可能会覆盖文件)", project_dir.display()))
                .default(false)
                .interact()?;
                
            if !overwrite {
                return Err(CliError::Canceled("创建操作已取消".to_string()));
            }
        }
    } else {
        create_dir_all(&project_dir)?;
    }
    
    // 组件选择
    let components = match components {
        Some(c) => c,
        None => {
            let choices = &[
                "agents - 代理",
                "tools - 工具",
                "workflows - 工作流",
                "rag - 检索增强生成",
            ];
            
            let defaults = vec![true, true, true, false];
            
            let selections = MultiSelect::new()
                .with_prompt("选择要包含的组件")
                .items(&choices[..])
                .defaults(&defaults)
                .interact()?;
                
            selections.iter().map(|&i| choices[i].split(" - ").next().unwrap().to_string()).collect()
        }
    };
    
    // LLM提供商选择
    let llm_provider = match llm_provider {
        Some(p) => p,
        None => {
            let choices = &[
                "openai - OpenAI (GPT-4, GPT-3.5)",
                "anthropic - Anthropic (Claude)",
                "deepseek - DeepSeek (DeepSeek-Chat, DeepSeek-Reasoner)",
                "gemini - Google Gemini",
                "local - 本地模型",
            ];
            
            let selection = Select::new()
                .with_prompt("选择默认LLM提供商")
                .items(&choices[..])
                .default(0)
                .interact()?;
                
            choices[selection].split(" - ").next().unwrap().to_string()
        }
    };
    
    // 获取API密钥
    let llm_api_key = match llm_api_key {
        Some(key) => Some(key),
        None => {
            if llm_provider != "local" {
                let key = Input::<String>::new()
                    .with_prompt(format!("{} API 密钥 (可选，稍后可在项目中设置)", llm_provider))
                    .allow_empty(true)
                    .interact()?;
                    
                if key.is_empty() { None } else { Some(key) }
            } else {
                None
            }
        }
    };
    
    // 创建选项
    let options = CreateOptions {
        name: project_name.clone(),
        project_dir: project_dir.clone(),
        components,
        llm_provider: Some(llm_provider),
        llm_api_key,
        add_example,
    };
    
    // 执行创建
    create_project(&options).await?;
    
    println!("{}", format!("项目 '{}' 创建成功!", project_name).bright_green());
    println!("{}", "要开始开发，请运行:".bright_blue());
    println!("{}", format!("  cd {}", project_name).bright_cyan());
    println!("{}", "  cargo build".bright_cyan());
    println!("{}", "  lumosai dev".bright_cyan());
    
    Ok(())
}

/// 创建项目
async fn create_project(options: &CreateOptions) -> CliResult<()> {
    // 1. 创建基本文件结构
    create_cargo_toml(options)?;
    create_basic_structure(options)?;
    
    // 2. 根据选择的组件创建对应文件
    if options.components.contains(&"agents".to_string()) {
        create_agents_files(options)?;
    }
    
    if options.components.contains(&"tools".to_string()) {
        create_tools_files(options)?;
    }
    
    if options.components.contains(&"workflows".to_string()) {
        create_workflows_files(options)?;
    }
    
    if options.components.contains(&"rag".to_string()) {
        create_rag_files(options)?;
    }
    
    // 3. 创建示例代码
    if options.add_example {
        create_example_files(options)?;
    }
    
    // 4. 初始化Git仓库
    init_git_repo(&options.project_dir)?;
    
    // 5. 安装依赖
    install_dependencies(options).await?;
    
    Ok(())
}

/// 创建Cargo.toml文件
fn create_cargo_toml(options: &CreateOptions) -> CliResult<()> {
    let cargo_toml_path = options.project_dir.join("Cargo.toml");
    
    let mut dependencies = vec![
        "lumosai_core = \"0.1.0\"".to_string(),
        "tokio = { version = \"1.29\", features = [\"full\"] }".to_string(),
        "serde = { version = \"1.0\", features = [\"derive\"] }".to_string(),
        "serde_json = \"1.0\"".to_string(),
    ];
    
    if options.components.contains(&"agents".to_string()) {
        dependencies.push("lumos_macro = \"0.1.0\"".to_string());
    }
    
    if options.components.contains(&"rag".to_string()) {
        dependencies.push("lumosai_rag = \"0.1.0\"".to_string());
    }
    
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
{}
"#,
        options.name.replace("-", "_"),
        dependencies.join("\n")
    );
    
    fs::write(cargo_toml_path, cargo_toml)?;
    
    Ok(())
}

/// 创建基本文件结构
fn create_basic_structure(options: &CreateOptions) -> CliResult<()> {
    // 创建src目录
    let src_dir = options.project_dir.join("src");
    create_dir_all(&src_dir)?;
    
    // 创建main.rs
    let main_rs_path = src_dir.join("main.rs");
    let main_rs_content = r#"use lumosai_core::{Result, Error};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Lumosai项目启动中...");
    
    // 这里是您的代码
    
    Ok(())
}
"#;
    
    fs::write(main_rs_path, main_rs_content)?;
    
    // 创建.env文件
    let env_path = options.project_dir.join(".env");
    let mut env_content = String::new();
    
    if let Some(provider) = &options.llm_provider {
        env_content.push_str(&format!("LLM_PROVIDER={}\n", provider));
    }
    
    if let Some(api_key) = &options.llm_api_key {
        env_content.push_str(&format!("LLM_API_KEY={}\n", api_key));
    }
    
    fs::write(env_path, env_content)?;
    
    // 创建.gitignore
    let gitignore_path = options.project_dir.join(".gitignore");
    let gitignore_content = r#"/target
**/*.rs.bk
.env
Cargo.lock
.vscode/
.idea/
.DS_Store
*.db
"#;
    fs::write(gitignore_path, gitignore_content)?;
    
    Ok(())
}

/// 创建代理相关文件
fn create_agents_files(options: &CreateOptions) -> CliResult<()> {
    let agents_dir = options.project_dir.join("src").join("agents");
    create_dir_all(&agents_dir)?;
    
    // 创建mod.rs
    let mod_rs_path = agents_dir.join("mod.rs");
    let mod_rs_content = r#"//! 代理模块

pub mod assistant;
"#;
    fs::write(mod_rs_path, mod_rs_content)?;
    
    // 创建示例代理
    let assistant_rs_path = agents_dir.join("assistant.rs");
    let assistant_rs_content = r#"//! 助手代理

use lumosai_core::{Result, Error};
use lumosai_core::agent::{Agent, SimpleAgent};
use lumosai_core::llm::{LlmProvider, OpenAiAdapter};
use std::sync::Arc;

/// 获取助手代理
pub fn get_assistant() -> Result<impl Agent> {
    // 从环境变量获取API密钥
    let api_key = std::env::var("LLM_API_KEY").map_err(|_| Error::Configuration("未设置LLM_API_KEY环境变量".into()))?;
    
    // 创建LLM适配器
    let llm = Arc::new(OpenAiAdapter::new(
        &api_key,
        "gpt-4",
    ));
    
    // 创建代理
    let agent = SimpleAgent::new(
        "assistant",
        "你是一个有用的助手，能够回答问题并提供帮助。",
        llm,
    );
    
    Ok(agent)
}
"#;
    
    fs::write(assistant_rs_path, assistant_rs_content)?;
    
    Ok(())
}

/// 创建工具相关文件
fn create_tools_files(options: &CreateOptions) -> CliResult<()> {
    let tools_dir = options.project_dir.join("src").join("tools");
    create_dir_all(&tools_dir)?;
    
    // 创建mod.rs
    let mod_rs_path = tools_dir.join("mod.rs");
    let mod_rs_content = r#"//! 工具模块

pub mod calculator;
"#;
    fs::write(mod_rs_path, mod_rs_content)?;
    
    // 创建示例工具
    let calculator_rs_path = tools_dir.join("calculator.rs");
    let calculator_rs_content = r#"//! 计算器工具

use lumosai_core::{Result, Error};
use lumosai_core::tool::{Tool, FunctionTool};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 计算器参数
#[derive(Deserialize)]
pub struct CalculatorParams {
    /// 运算符
    pub operation: String,
    /// 第一个操作数
    pub a: f64,
    /// 第二个操作数
    pub b: f64,
}

/// 计算器结果
#[derive(Serialize)]
pub struct CalculatorResult {
    /// 计算结果
    pub result: f64,
}

/// 获取计算器工具
pub fn get_calculator() -> Arc<dyn Tool> {
    Arc::new(FunctionTool::new(
        "calculator",
        "执行基本的数学计算",
        |params: CalculatorParams| async move {
            let result = match params.operation.as_str() {
                "add" => params.a + params.b,
                "subtract" => params.a - params.b,
                "multiply" => params.a * params.b,
                "divide" => {
                    if params.b == 0.0 {
                        return Err(Error::Tool("除数不能为零".into()));
                    }
                    params.a / params.b
                },
                _ => return Err(Error::Tool("不支持的操作".into())),
            };
            
            Ok(CalculatorResult { result })
        },
    ))
}
"#;
    
    fs::write(calculator_rs_path, calculator_rs_content)?;
    
    Ok(())
}

/// 创建工作流相关文件
fn create_workflows_files(options: &CreateOptions) -> CliResult<()> {
    let workflows_dir = options.project_dir.join("src").join("workflows");
    create_dir_all(&workflows_dir)?;
    
    // 创建mod.rs
    let mod_rs_path = workflows_dir.join("mod.rs");
    let mod_rs_content = r#"//! 工作流模块

pub mod simple_workflow;
"#;
    fs::write(mod_rs_path, mod_rs_content)?;
    
    // 创建示例工作流
    let workflow_rs_path = workflows_dir.join("simple_workflow.rs");
    let workflow_rs_content = r#"//! 简单工作流示例

use lumosai_core::{Result, Error};
use lumosai_core::workflow::{Workflow, WorkflowStep};
use crate::agents::assistant::get_assistant;

/// 获取简单工作流
pub async fn get_simple_workflow() -> Result<Workflow> {
    let mut workflow = Workflow::new(
        "simple_workflow",
        "简单的顺序工作流示例",
    );
    
    // 获取助手代理
    let assistant = get_assistant()?;
    
    // 添加步骤
    workflow.add_step(WorkflowStep::new(
        "generate_response",
        assistant,
        |input| async move {
            // 这里可以对输入进行处理
            Ok(input)
        },
    ));
    
    Ok(workflow)
}
"#;
    
    fs::write(workflow_rs_path, workflow_rs_content)?;
    
    Ok(())
}

/// 创建RAG相关文件
fn create_rag_files(options: &CreateOptions) -> CliResult<()> {
    let rag_dir = options.project_dir.join("src").join("rag");
    create_dir_all(&rag_dir)?;
    
    // 创建mod.rs
    let mod_rs_path = rag_dir.join("mod.rs");
    let mod_rs_content = r#"//! RAG模块

pub mod document;
pub mod embeddings;
"#;
    fs::write(mod_rs_path, mod_rs_content)?;
    
    // 创建示例RAG文件
    let document_rs_path = rag_dir.join("document.rs");
    let document_rs_content = r#"//! 文档处理

use lumosai_core::{Result, Error};
use lumosai_rag::document::{Document, Chunker, TextChunker};
use std::path::Path;

/// 加载并分块文档
pub async fn load_and_chunk_document<P: AsRef<Path>>(path: P) -> Result<Vec<Document>> {
    // 读取文件内容
    let content = std::fs::read_to_string(path)?;
    
    // 创建分块器
    let chunker = TextChunker::new(500, 100);
    
    // 分块
    let chunks = chunker.chunk(&content)?;
    
    Ok(chunks)
}
"#;
    
    fs::write(document_rs_path, document_rs_content)?;
    
    let embeddings_rs_path = rag_dir.join("embeddings.rs");
    let embeddings_rs_content = r#"//! 嵌入生成

use lumosai_core::{Result, Error};
use lumosai_rag::embeddings::{EmbeddingProvider, OpenAiEmbedding};
use lumosai_rag::document::Document;
use lumosai_rag::store::{VectorStore, SimpleVectorStore};

/// 创建嵌入提供者
pub fn create_embedding_provider() -> Result<impl EmbeddingProvider> {
    // 从环境变量获取API密钥
    let api_key = std::env::var("LLM_API_KEY").map_err(|_| Error::Configuration("未设置LLM_API_KEY环境变量".into()))?;
    
    // 创建OpenAI嵌入提供者
    let embedding_provider = OpenAiEmbedding::new(&api_key, "text-embedding-ada-002");
    
    Ok(embedding_provider)
}

/// 创建向量存储
pub fn create_vector_store() -> Result<impl VectorStore> {
    // 创建简单的内存向量存储
    let store = SimpleVectorStore::new();
    
    Ok(store)
}
"#;
    
    fs::write(embeddings_rs_path, embeddings_rs_content)?;
    
    Ok(())
}

/// 创建示例文件
fn create_example_files(options: &CreateOptions) -> CliResult<()> {
    let examples_dir = options.project_dir.join("examples");
    create_dir_all(&examples_dir)?;
    
    // 创建简单的示例
    let basic_example_path = examples_dir.join("basic_agent.rs");
    let basic_example_content = r#"//! 基本代理示例

use lumosai_core::{Result, Error};

#[tokio::main]
async fn main() -> Result<()> {
    // 导入代理
    use myproject::agents::assistant::get_assistant;
    
    // 获取代理
    let mut agent = get_assistant()?;
    
    // 运行代理
    let response = agent.run("计算15加27的结果").await?;
    
    println!("代理回答: {}", response);
    
    Ok(())
}
"#.replace("myproject", &options.name.replace("-", "_"));
    
    fs::write(basic_example_path, basic_example_content)?;
    
    if options.components.contains(&"tools".to_string()) {
        let tool_example_path = examples_dir.join("using_tools.rs");
        let tool_example_content = r#"//! 使用工具的示例

use lumosai_core::{Result, Error};

#[tokio::main]
async fn main() -> Result<()> {
    // 导入
    use myproject::agents::assistant::get_assistant;
    use myproject::tools::calculator::get_calculator;
    
    // 获取代理
    let mut agent = get_assistant()?;
    
    // 添加工具
    agent.add_tool(get_calculator());
    
    // 运行代理
    let response = agent.run("计算24乘以7的结果").await?;
    
    println!("代理回答: {}", response);
    
    Ok(())
}
"#.replace("myproject", &options.name.replace("-", "_"));
        
        fs::write(tool_example_path, tool_example_content)?;
    }
    
    Ok(())
}

/// 初始化Git仓库
fn init_git_repo(project_dir: &Path) -> CliResult<()> {
    println!("{}", "初始化Git仓库...".bright_blue());
    
    let status = Command::new("git")
        .args(["init"])
        .current_dir(project_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
        
    match status {
        Ok(status) if status.success() => {
            println!("{}", "Git仓库初始化成功".bright_green());
            Ok(())
        },
        Ok(_) => {
            println!("{}", "Git仓库初始化失败".bright_yellow());
            Ok(()) // 继续执行，不中断流程
        },
        Err(_) => {
            println!("{}", "未找到git命令，跳过Git初始化".bright_yellow());
            Ok(()) // 继续执行，不中断流程
        }
    }
}

/// 安装依赖
async fn install_dependencies(options: &CreateOptions) -> CliResult<()> {
    println!("{}", "开始安装依赖...".bright_blue());
    println!("{}", "这可能需要一些时间，请耐心等待".bright_blue());
    
    let status = Command::new("cargo")
        .args(["check"])
        .current_dir(&options.project_dir)
        .status()?;
        
    if status.success() {
        println!("{}", "依赖安装成功".bright_green());
    } else {
        println!("{}", "依赖安装可能出现问题，请稍后手动运行 'cargo build'".bright_yellow());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_options() {
        let options = CreateOptions {
            name: "test-project".to_string(),
            project_dir: PathBuf::from("/tmp/test-project"),
            components: vec!["agents".to_string(), "tools".to_string()],
            llm_provider: Some("openai".to_string()),
            llm_api_key: Some("test-key".to_string()),
            add_example: true,
        };
        
        assert_eq!(options.name, "test-project");
        assert_eq!(options.project_dir, PathBuf::from("/tmp/test-project"));
        assert_eq!(options.components, vec!["agents", "tools"]);
        assert_eq!(options.llm_provider, Some("openai".to_string()));
        assert_eq!(options.llm_api_key, Some("test-key".to_string()));
        assert_eq!(options.add_example, true);
    }
    
    #[test]
    fn test_create_cargo_toml() {
        // 创建临时目录
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        let options = CreateOptions {
            name: "test-project".to_string(),
            project_dir: temp_path.to_path_buf(),
            components: vec!["agents".to_string(), "rag".to_string()],
            llm_provider: Some("openai".to_string()),
            llm_api_key: None,
            add_example: false,
        };
        
        let result = create_cargo_toml(&options);
        assert!(result.is_ok());
        
        // 检查Cargo.toml是否已创建
        let cargo_toml_path = temp_path.join("Cargo.toml");
        assert!(cargo_toml_path.exists());
        
        // 读取文件内容并检查
        let content = fs::read_to_string(cargo_toml_path).unwrap();
        assert!(content.contains("name = \"test_project\""));
        assert!(content.contains("lumosai_core = \"0.1.0\""));
        assert!(content.contains("lumos_macro = \"0.1.0\""));  // 由于包含agents组件
        assert!(content.contains("lumosai_rag = \"0.1.0\""));  // 由于包含rag组件
    }
    
    #[test]
    fn test_create_basic_structure() {
        // 创建临时目录
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        let options = CreateOptions {
            name: "test-project".to_string(),
            project_dir: temp_path.to_path_buf(),
            components: vec![],
            llm_provider: Some("openai".to_string()),
            llm_api_key: Some("test-api-key".to_string()),
            add_example: false,
        };
        
        let result = create_basic_structure(&options);
        assert!(result.is_ok());
        
        // 检查基本文件是否已创建
        let src_dir = temp_path.join("src");
        assert!(src_dir.exists());
        
        let main_rs_path = src_dir.join("main.rs");
        assert!(main_rs_path.exists());
        
        let env_path = temp_path.join(".env");
        assert!(env_path.exists());
        
        let gitignore_path = temp_path.join(".gitignore");
        assert!(gitignore_path.exists());
        
        // 检查.env文件内容
        let env_content = fs::read_to_string(env_path).unwrap();
        assert!(env_content.contains("LLM_PROVIDER=openai"));
        assert!(env_content.contains("LLM_API_KEY=test-api-key"));
    }
    
    #[test]
    fn test_create_agents_files() {
        // 创建临时目录
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        let options = CreateOptions {
            name: "test-project".to_string(),
            project_dir: temp_path.to_path_buf(),
            components: vec!["agents".to_string()],
            llm_provider: Some("openai".to_string()),
            llm_api_key: None,
            add_example: false,
        };
        
        let result = create_agents_files(&options);
        assert!(result.is_ok());
        
        // 检查代理文件是否已创建
        let agents_dir = temp_path.join("src").join("agents");
        assert!(agents_dir.exists());
        
        let mod_rs_path = agents_dir.join("mod.rs");
        assert!(mod_rs_path.exists());
        
        let assistant_rs_path = agents_dir.join("assistant.rs");
        assert!(assistant_rs_path.exists());
        
        // 检查文件内容
        let assistant_content = fs::read_to_string(assistant_rs_path).unwrap();
        assert!(assistant_content.contains("pub fn get_assistant() -> Result<impl Agent>"));
    }
    
    #[test]
    fn test_create_example_files() {
        // 创建临时目录
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        let options = CreateOptions {
            name: "test-project".to_string(),
            project_dir: temp_path.to_path_buf(),
            components: vec!["agents".to_string(), "tools".to_string()],
            llm_provider: None,
            llm_api_key: None,
            add_example: true,
        };
        
        let result = create_example_files(&options);
        assert!(result.is_ok());
        
        // 检查示例文件是否已创建
        let examples_dir = temp_path.join("examples");
        assert!(examples_dir.exists());
        
        let basic_example_path = examples_dir.join("basic_agent.rs");
        assert!(basic_example_path.exists());
        
        let tool_example_path = examples_dir.join("using_tools.rs");
        assert!(tool_example_path.exists());
        
        // 检查文件内容，确保项目名称已正确替换
        let basic_content = fs::read_to_string(basic_example_path).unwrap();
        assert!(basic_content.contains("use test_project::agents::"));
    }
} 