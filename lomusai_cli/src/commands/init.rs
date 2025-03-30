use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;
use crate::error::{CliError, CliResult};
use crate::template::{TemplateManager, TemplateType};
use crate::util::{check_rust_toolchain, create_dir_all};
use dialoguer::{Input, Select, Confirm};
use colored::Colorize;

/// 初始化一个新的Lomus AI项目
pub async fn run(
    name: Option<String>,
    template_str: Option<String>,
    output: Option<PathBuf>,
) -> CliResult<()> {
    // 检查Rust工具链
    check_rust_toolchain()?;
    
    // 获取项目名称
    let project_name = match name {
        Some(n) => n,
        None => Input::<String>::new()
            .with_prompt("项目名称")
            .default("my_lomusai_project".to_string())
            .interact()?
    };
    
    // 获取模板类型
    let template_type = match template_str {
        Some(t) => TemplateType::from_str(&t),
        None => {
            let choices = &["agent", "workflow", "rag", "custom"];
            let selection = Select::new()
                .with_prompt("选择模板类型")
                .items(choices)
                .default(0)
                .interact()?;
                
            match selection {
                0 => TemplateType::Agent,
                1 => TemplateType::Workflow,
                2 => TemplateType::Rag,
                3 => {
                    let custom_type = Input::<String>::new()
                        .with_prompt("输入自定义模板名称")
                        .interact()?;
                    TemplateType::Custom(custom_type)
                },
                _ => unreachable!(),
            }
        }
    };
    
    // 确定输出目录
    let output_dir = match output {
        Some(path) => path,
        None => {
            let current_dir = std::env::current_dir()
                .map_err(|e| CliError::Io(e))?;
                
            let dir_input = Input::<String>::new()
                .with_prompt("输出目录")
                .default(current_dir.join(&project_name).to_string_lossy().to_string())
                .interact()?;
                
            PathBuf::from(dir_input)
        }
    };
    
    // 检查输出目录是否存在
    if output_dir.exists() {
        // 检查目录是否为空
        let is_empty = match fs::read_dir(&output_dir) {
            Ok(mut entries) => entries.next().is_none(),
            Err(_) => false,
        };
        
        if !is_empty {
            let confirm = Confirm::new()
                .with_prompt(format!("目录 {} 不为空，是否继续? 这可能会覆盖现有文件", output_dir.display()))
                .default(false)
                .interact()?;
                
            if !confirm {
                return Err(CliError::canceled("初始化已取消"));
            }
        }
    } else {
        // 创建输出目录
        create_dir_all(&output_dir)?;
    }
    
    println!("{}", format!("初始化 {} 项目: {}", template_type.name(), project_name).bright_blue());
    
    // 使用模板管理器创建项目
    let template_manager = TemplateManager::new()?;
    template_manager.create_project(&template_type, &project_name, &output_dir)?;
    
    println!("{}", format!("项目已创建: {}", output_dir.display()).bright_green());
    println!("{}", "要开始开发，请运行:".bright_blue());
    println!("{}", format!("  cd {}", output_dir.display()).bright_cyan());
    println!("{}", "  cargo run".bright_cyan());
    println!("{}", "  # 或者使用开发服务器:".bright_blue());
    println!("{}", "  lomus dev".bright_cyan());
    
    Ok(())
}

// 创建基本项目结构
fn create_basic_project(
    output_path: &Path,
    template_type: &TemplateType,
    variables: &HashMap<String, String>
) -> CliResult<()> {
    let project_name = variables.get("project_name").unwrap();
    let crate_name = variables.get("crate_name").unwrap();
    
    // 创建目录结构
    let src_dir = output_path.join("src");
    fs::create_dir_all(&src_dir).map_err(CliError::Io)?;
    
    // 创建Cargo.toml
    let cargo_toml_content = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
lomusai_core = "0.1.0"
tokio = {{ version = "1.34", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
"#, crate_name);

    fs::write(output_path.join("Cargo.toml"), cargo_toml_content)
        .map_err(CliError::Io)?;
    
    // 根据模板类型创建不同的main.rs
    let main_rs_content = match template_type {
        TemplateType::Agent => r#"use lomusai_core::agent::{Agent, AgentConfig};
use lomusai_core::llm::provider::OpenAI;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("正在启动Agent...");
    
    // 创建LLM提供者
    let llm = OpenAI::new()?;
    
    // 配置Agent
    let config = AgentConfig::default()
        .with_name("测试Agent")
        .with_description("这是一个测试Agent");
    
    // 创建Agent
    let mut agent = Agent::new(config, llm);
    
    // 运行Agent
    let response = agent.run("你好，世界!").await?;
    println!("Agent响应: {}", response);
    
    Ok(())
}
"#,
        TemplateType::Workflow => r#"use lomusai_core::workflow::{Workflow, Step};
use lomusai_core::llm::provider::OpenAI;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("正在启动Workflow...");
    
    // 创建LLM提供者
    let llm = OpenAI::new()?;
    
    // 创建工作流
    let mut workflow = Workflow::new("测试工作流", llm);
    
    // 添加步骤
    workflow.add_step(Step::new("step1", "总结以下内容"));
    workflow.add_step(Step::new("step2", "将步骤1的结果翻译成英文"));
    
    // 运行工作流
    let input = "这是一个测试工作流，用于演示Lomus AI的工作流功能。";
    let result = workflow.run(input).await?;
    
    println!("工作流结果: {}", result);
    
    Ok(())
}
"#,
        TemplateType::Rag => r#"use lomusai_core::rag::{RagEngine, DocumentStore};
use lomusai_core::llm::provider::OpenAI;
use lomusai_core::vector::provider::SimpleVector;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("正在启动RAG引擎...");
    
    // 创建LLM提供者
    let llm = OpenAI::new()?;
    
    // 创建向量存储
    let vector_store = SimpleVector::new()?;
    
    // 创建文档存储
    let doc_store = DocumentStore::new(vector_store);
    
    // 创建RAG引擎
    let mut rag = RagEngine::new(llm, doc_store);
    
    // 添加文档
    rag.add_document("文档1", "这是一个测试文档，用于演示Lomus AI的RAG功能。").await?;
    
    // 查询
    let query = "RAG功能是什么?";
    let result = rag.query(query).await?;
    
    println!("查询结果: {}", result);
    
    Ok(())
}
"#,
        TemplateType::Custom(_) => r#"fn main() {
    println!("Hello, Lomus AI!");
}
"#,
    };
    
    fs::write(src_dir.join("main.rs"), main_rs_content)
        .map_err(CliError::Io)?;
    
    // 创建.gitignore
    let gitignore_content = r#"/target
**/*.rs.bk
Cargo.lock
.env
"#;
    
    fs::write(output_path.join(".gitignore"), gitignore_content)
        .map_err(CliError::Io)?;
    
    // 创建README.md
    let readme_content = format!(r#"# {}

这是一个使用 Lomus AI 框架创建的项目。

## 运行

```bash
cargo run
```

## 开发

```bash
lomus dev
```
"#, project_name);
    
    fs::write(output_path.join("README.md"), readme_content)
        .map_err(CliError::Io)?;
    
    Ok(())
} 