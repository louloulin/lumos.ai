use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::error::{CliError, CliResult};
use crate::util;

/// 模板类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateType {
    /// 智能体模板
    Agent,
    
    /// 工作流模板
    Workflow,
    
    /// RAG模板
    Rag,
    
    /// 自定义模板
    Custom(String),
}

impl TemplateType {
    /// 从字符串创建模板类型
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "agent" => TemplateType::Agent,
            "workflow" => TemplateType::Workflow,
            "rag" => TemplateType::Rag,
            _ => TemplateType::Custom(s.to_string()),
        }
    }
    
    /// 获取模板类型名称
    pub fn name(&self) -> String {
        match self {
            TemplateType::Agent => "agent".to_string(),
            TemplateType::Workflow => "workflow".to_string(),
            TemplateType::Rag => "rag".to_string(),
            TemplateType::Custom(name) => name.clone(),
        }
    }
}

/// 模板管理器
pub struct TemplateManager {
    /// 模板目录
    template_dir: PathBuf,
}

impl TemplateManager {
    /// 创建模板管理器
    pub fn new() -> CliResult<Self> {
        let template_dir = Self::get_template_dir()?;
        Ok(TemplateManager { template_dir })
    }
    
    /// 获取模板目录
    fn get_template_dir() -> CliResult<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| CliError::template("无法获取用户主目录"))?;
            
        let template_dir = home_dir.join(".lomusai").join("templates");
        
        // 确保模板目录存在
        if !template_dir.exists() {
            fs::create_dir_all(&template_dir)
                .map_err(|e| CliError::Io(e))?;
        }
        
        Ok(template_dir)
    }
    
    /// 获取模板目录
    pub fn template_directory(&self) -> &PathBuf {
        &self.template_dir
    }
    
    /// 创建基础项目结构
    pub fn create_project(
        &self,
        template_type: &TemplateType,
        project_name: &str,
        output_dir: &Path,
    ) -> CliResult<()> {
        // 创建项目目录
        util::create_dir_all(output_dir)?;
        
        // 创建Cargo.toml
        self.create_cargo_toml(template_type, project_name, output_dir)?;
        
        // 创建src目录
        let src_dir = output_dir.join("src");
        util::create_dir_all(&src_dir)?;
        
        // 创建main.rs
        self.create_main_rs(template_type, &src_dir)?;
        
        // 创建.gitignore
        self.create_gitignore(output_dir)?;
        
        // 创建README.md
        self.create_readme(template_type, project_name, output_dir)?;
        
        // 如果是RAG模板，创建数据目录
        if *template_type == TemplateType::Rag {
            let data_dir = output_dir.join("data");
            util::create_dir_all(&data_dir)?;
            
            // 添加示例文件
            fs::write(
                data_dir.join("example.md"),
                "# 示例文档\n\n这是一个示例文档，用于RAG应用测试。",
            ).map_err(|e| CliError::Io(e))?;
        }
        
        Ok(())
    }
    
    /// 创建Cargo.toml
    fn create_cargo_toml(
        &self,
        template_type: &TemplateType,
        project_name: &str,
        output_dir: &Path,
    ) -> CliResult<()> {
        let crate_name = project_name.replace('-', "_");
        
        // 基础依赖
        let mut dependencies = vec![
            "lomusai_core = \"0.1.0\"",
            "tokio = { version = \"1.34\", features = [\"full\"] }",
            "serde = { version = \"1.0\", features = [\"derive\"] }",
            "serde_json = \"1.0\"",
            "anyhow = \"1.0\"",
            "tracing = \"0.1\"",
            "tracing-subscriber = \"0.3\"",
        ];
        
        // 根据模板类型添加特定依赖
        match template_type {
            TemplateType::Agent => {
                // 智能体特定依赖
            },
            TemplateType::Workflow => {
                // 工作流特定依赖
            },
            TemplateType::Rag => {
                // RAG特定依赖
                dependencies.push("lomusai_rag = \"0.1.0\"");
                dependencies.push("lomusai_stores = \"0.1.0\"");
            },
            TemplateType::Custom(_) => {
                // 无特定依赖
            },
        }
        
        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
{}
"#,
            crate_name,
            dependencies.join("\n"),
        );
        
        fs::write(output_dir.join("Cargo.toml"), cargo_toml)
            .map_err(|e| CliError::Io(e))
    }
    
    /// 创建main.rs
    fn create_main_rs(&self, template_type: &TemplateType, src_dir: &Path) -> CliResult<()> {
        let content = match template_type {
            TemplateType::Agent => {
                r#"use anyhow::Result;
use lomusai_core::agent::{Agent, AgentExecutor};
use lomusai_core::llm::{LLMService, OpenAI};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // 设置日志
    tracing_subscriber::fmt::init();
    
    // 初始化LLM服务
    let api_key = env::var("OPENAI_API_KEY").expect("需要设置OPENAI_API_KEY环境变量");
    let llm = OpenAI::new(api_key);
    
    // 创建智能体
    let agent = Agent::builder()
        .name("助手")
        .description("一个有用的AI助手")
        .llm(llm)
        .build();
        
    // 创建执行器并运行
    let executor = AgentExecutor::new(agent);
    let response = executor.run("今天天气怎么样?").await?;
    
    println!("回复: {}", response);
    
    Ok(())
}
"#
            },
            TemplateType::Workflow => {
                r#"use anyhow::Result;
use lomusai_core::workflow::{Workflow, WorkflowBuilder, Step};
use lomusai_core::llm::{LLMService, OpenAI};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // 设置日志
    tracing_subscriber::fmt::init();
    
    // 初始化LLM服务
    let api_key = env::var("OPENAI_API_KEY").expect("需要设置OPENAI_API_KEY环境变量");
    let llm = OpenAI::new(api_key);
    
    // 创建工作流
    let workflow = WorkflowBuilder::new()
        .name("示例工作流")
        .description("处理用户查询的简单工作流")
        .llm(llm.clone())
        .add_step(Step::new("理解查询", |input, ctx| {
            Box::pin(async move {
                // 解析用户查询
                Ok(format!("用户询问: {}", input))
            })
        }))
        .add_step(Step::new("生成回复", move |input, ctx| {
            let llm = llm.clone();
            Box::pin(async move {
                // 使用LLM生成回复
                let prompt = format!("请回复以下查询: {}", input);
                let response = llm.generate(&prompt, None).await?;
                Ok(response)
            })
        }))
        .build();
    
    // 运行工作流
    let result = workflow.execute("今天天气怎么样?", None).await?;
    println!("工作流结果: {}", result);
    
    Ok(())
}
"#
            },
            TemplateType::Rag => {
                r#"use anyhow::Result;
use lomusai_core::llm::{LLMService, OpenAI};
use lomusai_rag::{
    embedding::{Embeddings, OpenAIEmbeddings},
    document::{Document, DocumentLoader},
    retriever::Retriever,
    store::MemoryVectorStore,
};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // 设置日志
    tracing_subscriber::fmt::init();
    
    // 获取API密钥
    let api_key = env::var("OPENAI_API_KEY").expect("需要设置OPENAI_API_KEY环境变量");
    
    // 初始化LLM和Embeddings
    let llm = OpenAI::new(api_key.clone());
    let embeddings = OpenAIEmbeddings::new(api_key);
    
    // 加载文档
    let docs = DocumentLoader::load(Path::new("data/example.md")).await?;
    println!("加载了 {} 个文档", docs.len());
    
    // 创建向量存储
    let store = MemoryVectorStore::new(embeddings);
    
    // 添加文档到存储
    store.add_documents(&docs).await?;
    
    // 创建检索器
    let retriever = Retriever::new(store, 2); // 检索前2个最相关文档
    
    // 用户查询
    let query = "示例文档的内容是什么?";
    
    // 检索相关文档
    let contexts = retriever.retrieve(query).await?;
    println!("找到 {} 个相关文档", contexts.len());
    
    // 构建提示
    let prompt = format!(
        "请根据以下上下文回答问题:\n\n上下文: {}\n\n问题: {}",
        contexts.iter().map(|c| c.content.clone()).collect::<Vec<_>>().join("\n\n"),
        query
    );
    
    // 生成回答
    let response = llm.generate(&prompt, None).await?;
    println!("回答: {}", response);
    
    Ok(())
}
"#
            },
            TemplateType::Custom(_) => {
                r#"use anyhow::Result;
use lomusai_core::llm::{LLMService, OpenAI};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // 设置日志
    tracing_subscriber::fmt::init();
    
    // 初始化LLM服务
    let api_key = env::var("OPENAI_API_KEY").expect("需要设置OPENAI_API_KEY环境变量");
    let llm = OpenAI::new(api_key);
    
    // 自定义项目启动代码
    println!("Hello, Lomus AI!");
    
    Ok(())
}
"#
            }
        };
        
        fs::write(src_dir.join("main.rs"), content)
            .map_err(|e| CliError::Io(e))
    }
    
    /// 创建.gitignore
    fn create_gitignore(&self, output_dir: &Path) -> CliResult<()> {
        let content = r#"/target
**/*.rs.bk
Cargo.lock
.env
.env.local
.DS_Store
.idea/
.vscode/
"#;
        
        fs::write(output_dir.join(".gitignore"), content)
            .map_err(|e| CliError::Io(e))
    }
    
    /// 创建README.md
    fn create_readme(
        &self,
        template_type: &TemplateType,
        project_name: &str,
        output_dir: &Path,
    ) -> CliResult<()> {
        let template_name = match template_type {
            TemplateType::Agent => "智能体",
            TemplateType::Workflow => "工作流",
            TemplateType::Rag => "RAG检索增强生成",
            TemplateType::Custom(_) => "自定义",
        };
        
        let content = format!(
            r#"# {}

这是一个使用Lomus AI框架创建的{}项目。

## 运行项目

```bash
# 设置OpenAI API密钥
export OPENAI_API_KEY=你的API密钥

# 运行项目
cargo run
```

## 开发

```bash
# 使用Lomus CLI启动开发服务器
lomus dev
```

## 构建

```bash
# 构建优化版本
lomus build
```

## 部署

```bash
# 部署到本地
lomus deploy

# 部署到Docker
lomus deploy --target docker
```
"#,
            project_name,
            template_name,
        );
        
        fs::write(output_dir.join("README.md"), content)
            .map_err(|e| CliError::Io(e))
    }
    
    /// 从远程仓库下载模板
    pub fn download_template(&self, template_url: &str, template_name: &str) -> CliResult<()> {
        println!("正在从 {} 下载模板...", template_url);
        
        // 创建临时目录
        let temp_dir = tempfile::tempdir()
            .map_err(|e| CliError::template(format!("创建临时目录失败: {}", e)))?;
            
        // 确定模板目录
        let template_dir = self.template_dir.join(template_name);
        
        // 确保模板目录存在
        if !template_dir.exists() {
            util::create_dir_all(&template_dir)?;
        }
        
        // 检查URL类型，支持git或http下载
        if template_url.starts_with("https://github.com") || template_url.starts_with("git@github.com") {
            // 使用git克隆仓库
            let status = std::process::Command::new("git")
                .args(["clone", "--depth", "1", template_url, temp_dir.path().to_str().unwrap()])
                .status()
                .map_err(|e| CliError::template(format!("Git克隆失败: {}", e)))?;
                
            if !status.success() {
                return Err(CliError::template("Git克隆失败"));
            }
            
            // 复制模板文件到模板目录
            util::copy_dir_all(temp_dir.path(), &template_dir)?;
            
            println!("模板下载成功，已保存到: {}", template_dir.display());
            
            return Ok(());
        } else if template_url.starts_with("http://") || template_url.starts_with("https://") {
            // 从HTTP URL下载ZIP文件
            #[cfg(feature = "reqwest")]
            {
                use std::io::Write;
                
                // 下载ZIP文件
                let response = reqwest::blocking::get(template_url)
                    .map_err(|e| CliError::template(format!("下载模板失败: {}", e)))?;
                    
                let zip_data = response.bytes()
                    .map_err(|e| CliError::template(format!("读取下载内容失败: {}", e)))?;
                    
                // 保存到临时文件
                let zip_path = temp_dir.path().join("template.zip");
                let mut file = std::fs::File::create(&zip_path)
                    .map_err(|e| CliError::Io(e))?;
                    
                file.write_all(&zip_data)
                    .map_err(|e| CliError::Io(e))?;
                    
                // 解压缩
                let file = std::fs::File::open(&zip_path)
                    .map_err(|e| CliError::Io(e))?;
                    
                let mut archive = zip::ZipArchive::new(file)
                    .map_err(|e| CliError::template(format!("解压缩失败: {}", e)))?;
                    
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i)
                        .map_err(|e| CliError::template(format!("读取ZIP文件失败: {}", e)))?;
                        
                    let outpath = template_dir.join(file.name());
                    
                    if file.name().ends_with('/') {
                        util::create_dir_all(&outpath)?;
                    } else {
                        if let Some(p) = outpath.parent() {
                            if !p.exists() {
                                util::create_dir_all(p)?;
                            }
                        }
                        
                        let mut outfile = std::fs::File::create(&outpath)
                            .map_err(|e| CliError::Io(e))?;
                            
                        std::io::copy(&mut file, &mut outfile)
                            .map_err(|e| CliError::Io(e))?;
                    }
                }
                
                println!("模板下载并解压成功，已保存到: {}", template_dir.display());
                
                return Ok(());
            }
            
            #[cfg(not(feature = "reqwest"))]
            {
                return Err(CliError::template("不支持HTTP下载。请启用reqwest特性，或使用git仓库URL"));
            }
        }
        
        Err(CliError::template(format!("不支持的模板URL: {}", template_url)))
    }
    
    /// 列出可用的模板
    pub fn list_templates(&self) -> CliResult<Vec<String>> {
        if !self.template_dir.exists() {
            return Ok(vec![]);
        }
        
        let mut templates = Vec::new();
        
        for entry in fs::read_dir(&self.template_dir)
            .map_err(|e| CliError::Io(e))?
        {
            let entry = entry.map_err(|e| CliError::Io(e))?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    templates.push(name.to_string());
                }
            }
        }
        
        Ok(templates)
    }
}