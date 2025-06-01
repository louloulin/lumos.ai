use clap::Args;
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use colored::Colorize;
use tokio::process::Command;

use crate::error::{CliResult, CliError};
use crate::util::{is_lumos_project, ensure_dir_exists};

/// 代理可视化配置选项
#[derive(Args, Debug)]
pub struct VisualizeOptions {
    /// 项目目录
    #[arg(long)]
    project_dir: Option<PathBuf>,
    
    /// 要可视化的代理ID
    #[arg(long)]
    agent: Option<String>,
    
    /// 要可视化的工作流ID
    #[arg(long)]
    workflow: Option<String>,
    
    /// 输出文件路径
    #[arg(long)]
    output: Option<PathBuf>,
    
    /// 可视化格式 (png, svg, pdf, html)
    #[arg(long, default_value = "svg")]
    format: String,
    
    /// 包含性能信息
    #[arg(long)]
    include_performance: bool,
    
    /// 交互式模式 (需要浏览器支持)
    #[arg(long)]
    interactive: bool,
}

impl Default for VisualizeOptions {
    fn default() -> Self {
        Self {
            project_dir: None,
            agent: None,
            workflow: None,
            output: None,
            format: "svg".to_string(),
            include_performance: false,
            interactive: false,
        }
    }
}

/// 运行可视化命令
pub async fn run(options: VisualizeOptions) -> CliResult<()> {
    // 解析项目目录
    let project_dir = match &options.project_dir {
        Some(dir) => dir.clone(),
        None => env::current_dir().map_err(|e| CliError::io("获取当前目录失败", e))?,
    };

    // 检查项目目录是否存在
    if !project_dir.exists() {
        return Err(CliError::path_not_found(
            project_dir.to_string_lossy().to_string(),
            "项目目录不存在",
        ));
    }

    // 检查是否是Lumosai项目
    if !is_lumos_project(&project_dir) {
        println!("{}", "警告: 当前目录不是标准的Lumosai项目目录结构".bright_yellow());
    }

    // 检查代理和工作流参数
    if options.agent.is_none() && options.workflow.is_none() {
        return Err(CliError::Other("请指定要可视化的代理或工作流".to_string()));
    }

    // 确定输出格式
    let format = validate_format(&options.format)?;
    
    // 确定输出文件路径
    let output_path = determine_output_path(&options, &project_dir, &format)?;
    
    println!("{}", "生成可视化图表...".bright_blue());
    println!("{}", format!("项目目录: {}", project_dir.display()).bright_blue());
    
    if let Some(agent) = &options.agent {
        println!("{}", format!("代理: {}", agent).bright_blue());
        
        // 生成代理可视化
        generate_agent_visualization(
            &project_dir,
            agent,
            &output_path,
            &format,
            options.include_performance,
            options.interactive,
        ).await?;
    }
    
    if let Some(workflow) = &options.workflow {
        println!("{}", format!("工作流: {}", workflow).bright_blue());
        
        // 生成工作流可视化
        generate_workflow_visualization(
            &project_dir,
            workflow,
            &output_path,
            &format,
            options.include_performance,
            options.interactive,
        ).await?;
    }
    
    println!("{}", format!("可视化图表已生成: {}", output_path.display()).bright_green());
    
    // 如果是交互式模式且输出格式是html，尝试在浏览器中打开
    if options.interactive && format == "html" {
        if let Err(e) = open::that(&output_path) {
            println!("{}", format!("无法在浏览器中打开: {}", e).bright_yellow());
        }
    }
    
    Ok(())
}

/// 验证并返回有效的格式
fn validate_format(format: &str) -> CliResult<String> {
    let valid_formats = ["png", "svg", "pdf", "html"];
    let format = format.to_lowercase();
    
    if valid_formats.contains(&format.as_str()) {
        Ok(format)
    } else {
        Err(CliError::other(format!(
            "不支持的格式: {}。支持的格式: png, svg, pdf, html",
            format
        )))
    }
}

/// 确定输出文件路径
fn determine_output_path(
    options: &VisualizeOptions,
    project_dir: &Path,
    format: &str,
) -> CliResult<PathBuf> {
    if let Some(output) = &options.output {
        return Ok(output.clone());
    }
    
    // 创建输出目录
    let vis_dir = project_dir.join("visualizations");
    ensure_dir_exists(&vis_dir)?;
    
    // 根据代理或工作流名生成文件名
    let filename = if let Some(agent) = &options.agent {
        format!("agent-{}.{}", agent, format)
    } else if let Some(workflow) = &options.workflow {
        format!("workflow-{}.{}", workflow, format)
    } else {
        format!("lumosai-visualization.{}", format)
    };
    
    Ok(vis_dir.join(filename))
}

/// 生成代理可视化
async fn generate_agent_visualization(
    project_dir: &Path,
    agent_id: &str,
    output_path: &Path,
    format: &str,
    include_performance: bool,
    interactive: bool,
) -> CliResult<()> {
    // 检查代理是否存在
    let agent_dir = project_dir.join("src").join("agents").join(agent_id);
    if !agent_dir.exists() {
        return Err(CliError::path_not_found(
            agent_dir.to_string_lossy().to_string(),
            format!("代理 '{}' 不存在", agent_id),
        ));
    }
    
    // 读取代理结构并生成可视化
    // 这里我们使用一个内部的脚本来生成Graphviz DOT文件，然后转换为指定格式
    
    // 1. 生成DOT文件
    let dot_file = generate_agent_dot_file(project_dir, agent_id, include_performance).await?;
    
    // 2. 使用Graphviz将DOT文件转换为指定格式
    convert_dot_to_format(&dot_file, output_path, format, interactive).await?;
    
    // 3. 清理临时DOT文件
    let _ = fs::remove_file(&dot_file);
    
    Ok(())
}

/// 生成工作流可视化
async fn generate_workflow_visualization(
    project_dir: &Path,
    workflow_id: &str,
    output_path: &Path,
    format: &str,
    include_performance: bool,
    interactive: bool,
) -> CliResult<()> {
    // 检查工作流是否存在
    let workflow_dir = project_dir.join("src").join("workflows").join(workflow_id);
    if !workflow_dir.exists() {
        return Err(CliError::path_not_found(
            workflow_dir.to_string_lossy().to_string(),
            format!("工作流 '{}' 不存在", workflow_id),
        ));
    }
    
    // 读取工作流结构并生成可视化
    // 这里我们使用一个内部的脚本来生成Graphviz DOT文件，然后转换为指定格式
    
    // 1. 生成DOT文件
    let dot_file = generate_workflow_dot_file(project_dir, workflow_id, include_performance).await?;
    
    // 2. 使用Graphviz将DOT文件转换为指定格式
    convert_dot_to_format(&dot_file, output_path, format, interactive).await?;
    
    // 3. 清理临时DOT文件
    let _ = fs::remove_file(&dot_file);
    
    Ok(())
}

/// 生成代理的DOT文件
async fn generate_agent_dot_file(
    project_dir: &Path,
    agent_id: &str,
    include_performance: bool,
) -> CliResult<PathBuf> {
    let agent_dir = project_dir.join("src").join("agents").join(agent_id);
    let dot_file_path = project_dir.join("visualizations").join(format!("agent-{}.dot", agent_id));
    
    // 读取代理文件
    let agent_file = agent_dir.join("agent.rs");
    let mod_file = agent_dir.join("mod.rs");
    
    let agent_code = if agent_file.exists() {
        fs::read_to_string(&agent_file).map_err(|e| CliError::io_error(e, &agent_file))?
    } else if mod_file.exists() {
        fs::read_to_string(&mod_file).map_err(|e| CliError::io_error(e, &mod_file))?
    } else {
        return Err(CliError::path_not_found(
            agent_dir.to_string_lossy().to_string(),
            format!("代理 '{}' 的主文件不存在", agent_id),
        ));
    };
    
    // 解析代理结构
    let (tools, llms, functions) = parse_agent_code(&agent_code);
    
    // 生成DOT文件内容
    let mut dot_content = String::from("digraph agent {\n");
    dot_content.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n");
    dot_content.push_str(&format!("  agent [label=\"{}\", fillcolor=lightgreen];\n", agent_id));
    
    // 添加工具节点
    for (i, tool) in tools.iter().enumerate() {
        dot_content.push_str(&format!("  tool{} [label=\"Tool: {}\", fillcolor=lightyellow];\n", i, tool));
        dot_content.push_str(&format!("  agent -> tool{};\n", i));
    }
    
    // 添加LLM节点
    for (i, llm) in llms.iter().enumerate() {
        dot_content.push_str(&format!("  llm{} [label=\"LLM: {}\", fillcolor=lightpink];\n", i, llm));
        dot_content.push_str(&format!("  agent -> llm{};\n", i));
    }
    
    // 添加函数节点
    for (i, func) in functions.iter().enumerate() {
        dot_content.push_str(&format!("  func{} [label=\"Function: {}\", fillcolor=lightgrey];\n", i, func));
        dot_content.push_str(&format!("  agent -> func{};\n", i));
    }
    
    // 如果包含性能信息
    if include_performance {
        dot_content.push_str("  subgraph cluster_performance {\n");
        dot_content.push_str("    label=\"Performance\";\n");
        dot_content.push_str("    style=filled;\n");
        dot_content.push_str("    fillcolor=aliceblue;\n");
        dot_content.push_str("    avg_latency [label=\"Avg. Latency: N/A\", shape=none];\n");
        dot_content.push_str("    throughput [label=\"Throughput: N/A\", shape=none];\n");
        dot_content.push_str("  }\n");
    }
    
    dot_content.push_str("}\n");
    
    // 写入DOT文件
    fs::write(&dot_file_path, dot_content).map_err(|e| CliError::io_error(e, &dot_file_path))?;
    
    Ok(dot_file_path)
}

/// 生成工作流的DOT文件
async fn generate_workflow_dot_file(
    project_dir: &Path,
    workflow_id: &str,
    include_performance: bool,
) -> CliResult<PathBuf> {
    let workflow_dir = project_dir.join("src").join("workflows").join(workflow_id);
    let dot_file_path = project_dir.join("visualizations").join(format!("workflow-{}.dot", workflow_id));
    
    // 读取工作流文件
    let workflow_file = workflow_dir.join("workflow.rs");
    let mod_file = workflow_dir.join("mod.rs");
    
    let workflow_code = if workflow_file.exists() {
        fs::read_to_string(&workflow_file).map_err(|e| CliError::io_error(e, &workflow_file))?
    } else if mod_file.exists() {
        fs::read_to_string(&mod_file).map_err(|e| CliError::io_error(e, &mod_file))?
    } else {
        return Err(CliError::path_not_found(
            workflow_dir.to_string_lossy().to_string(),
            format!("工作流 '{}' 的主文件不存在", workflow_id),
        ));
    };
    
    // 解析工作流结构
    let steps = parse_workflow_code(&workflow_code);
    
    // 生成DOT文件内容
    let mut dot_content = String::from("digraph workflow {\n");
    dot_content.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n");
    dot_content.push_str(&format!("  workflow [label=\"{}\", fillcolor=lightgreen];\n", workflow_id));
    
    // 添加步骤节点
    for (i, step) in steps.iter().enumerate() {
        dot_content.push_str(&format!("  step{} [label=\"Step {}: {}\", fillcolor=lightyellow];\n", i, i + 1, step));
        
        if i == 0 {
            dot_content.push_str(&format!("  workflow -> step{};\n", i));
        } else {
            dot_content.push_str(&format!("  step{} -> step{};\n", i - 1, i));
        }
    }
    
    // 如果包含性能信息
    if include_performance {
        dot_content.push_str("  subgraph cluster_performance {\n");
        dot_content.push_str("    label=\"Performance\";\n");
        dot_content.push_str("    style=filled;\n");
        dot_content.push_str("    fillcolor=aliceblue;\n");
        dot_content.push_str("    avg_duration [label=\"Avg. Duration: N/A\", shape=none];\n");
        dot_content.push_str("    success_rate [label=\"Success Rate: N/A\", shape=none];\n");
        dot_content.push_str("  }\n");
    }
    
    dot_content.push_str("}\n");
    
    // 写入DOT文件
    fs::write(&dot_file_path, dot_content).map_err(|e| CliError::io_error(e, &dot_file_path))?;
    
    Ok(dot_file_path)
}

/// 将DOT文件转换为指定格式
async fn convert_dot_to_format(
    dot_file: &Path,
    output_path: &Path,
    format: &str,
    interactive: bool,
) -> CliResult<()> {
    // 检查是否安装了Graphviz
    if !check_graphviz().await {
        return Err(CliError::dependency("Graphviz", "请安装Graphviz以生成可视化图表"));
    }
    
    // 如果是交互式HTML格式，使用不同的转换方法
    if format == "html" && interactive {
        return generate_interactive_html(dot_file, output_path).await;
    }
    
    // 使用Graphviz的dot命令转换
    let mut cmd = Command::new("dot");
    cmd.args([
        "-T", format,
        "-o", output_path.to_str().unwrap_or("output"),
        dot_file.to_str().unwrap_or("input.dot"),
    ]);
    
    let status = cmd.status().await.map_err(|e| CliError::io(
        format!("执行Graphviz dot命令失败: {}", e).as_str(), 
        e
    ))?;
    
    if !status.success() {
        return Err(CliError::failed::<std::io::Error>("转换DOT文件失败", None));
    }
    
    Ok(())
}

/// 检查是否安装了Graphviz
async fn check_graphviz() -> bool {
    Command::new("dot")
        .arg("-V")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// 解析代理代码
fn parse_agent_code(code: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut tools = Vec::new();
    let mut llms = Vec::new();
    let mut functions = Vec::new();
    
    // 这是一个简化的解析实现，实际中应该使用更复杂的语法分析
    
    // 查找工具的简单实现
    for line in code.lines() {
        if line.contains("use") && line.contains("tool") {
            if let Some(tool_name) = extract_name_from_use(line) {
                tools.push(tool_name);
            }
        } else if line.contains("use") && (line.contains("llm") || line.contains("openai") || line.contains("anthropic")) {
            if let Some(llm_name) = extract_name_from_use(line) {
                llms.push(llm_name);
            }
        } else if line.contains("fn ") && !line.contains("impl") && !line.contains("//") {
            if let Some(func_name) = extract_function_name(line) {
                functions.push(func_name);
            }
        }
    }
    
    (tools, llms, functions)
}

/// 解析工作流代码
fn parse_workflow_code(code: &str) -> Vec<String> {
    let mut steps = Vec::new();
    
    // 简化的解析实现
    for line in code.lines() {
        if line.contains("step") || line.contains("Step") || line.contains("task") || line.contains("Task") {
            if let Some(step_name) = extract_step_name(line) {
                steps.push(step_name);
            }
        }
    }
    
    steps
}

/// 从use语句中提取名称
fn extract_name_from_use(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split("::").collect();
    if let Some(last_part) = parts.last() {
        let clean_part = last_part.trim().trim_end_matches(';').trim_end_matches('}');
        return Some(clean_part.to_string());
    }
    None
}

/// 从函数声明中提取函数名
fn extract_function_name(line: &str) -> Option<String> {
    let line = line.trim();

    // 跳过注释行
    if line.starts_with("//") {
        return None;
    }

    // 查找 "fn " 关键字的位置
    if let Some(fn_pos) = line.find("fn ") {
        let after_fn = &line[fn_pos + 3..];
        let name_end = after_fn.find('(').unwrap_or(after_fn.len());
        let name = after_fn[..name_end].trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }

    None
}

/// 从步骤声明中提取步骤名
fn extract_step_name(line: &str) -> Option<String> {
    // 查找常见的步骤声明模式
    let patterns = [
        "step", "Step", "task", "Task",
        ".step(", ".task(", "add_step(", "add_task(",
    ];
    
    for pattern in patterns {
        if let Some(pos) = line.find(pattern) {
            let after_pattern = &line[pos + pattern.len()..];
            if let Some(quote_pos) = after_pattern.find('"') {
                let after_quote = &after_pattern[quote_pos + 1..];
                if let Some(end_quote_pos) = after_quote.find('"') {
                    return Some(after_quote[..end_quote_pos].to_string());
                }
            } else if let Some(open_paren) = after_pattern.find('(') {
                let after_open = &after_pattern[open_paren + 1..];
                if let Some(close_paren) = after_open.find(')') {
                    let potential_name = after_open[..close_paren].trim();
                    if !potential_name.is_empty() {
                        return Some(potential_name.to_string());
                    }
                }
            }
        }
    }
    
    // 如果找不到明确的步骤名称，使用行内容的摘要
    if line.len() > 10 {
        Some(format!("{}...", &line[..10]))
    } else {
        Some(line.to_string())
    }
}

/// 生成交互式HTML可视化
async fn generate_interactive_html(
    dot_file: &Path,
    output_path: &Path,
) -> CliResult<()> {
    // 读取DOT文件内容
    let dot_content = fs::read_to_string(dot_file)
        .map_err(|e| CliError::io_error(e, dot_file))?;
    
    // 转义DOT内容以安全嵌入到JavaScript中
    let escaped_dot_content = dot_content
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('$', "\\$");
    
    // 创建交互式HTML
    let html_content = format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Lumosai Visualization</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script src="https://unpkg.com/@hpcc-js/wasm@1.12.13/dist/index.min.js"></script>
    <script src="https://unpkg.com/d3-graphviz@4.0.0/build/d3-graphviz.js"></script>
    <style>
        body {{ margin: 0; padding: 20px; font-family: Arial, sans-serif; }}
        #graph {{ width: 100%; height: 800px; border: 1px solid #ccc; }}
        .controls {{ margin-bottom: 20px; }}
        button {{ padding: 8px 12px; margin-right: 10px; cursor: pointer; }}
    </style>
</head>
<body>
    <h1>Lumosai 可视化</h1>
    <div class="controls">
        <button id="zoomIn">放大</button>
        <button id="zoomOut">缩小</button>
        <button id="resetZoom">重置</button>
        <button id="download">下载 SVG</button>
    </div>
    <div id="graph"></div>
    
    <script>
        // 初始化图形
        const graphviz = d3.select("#graph").graphviz()
            .zoom(true)
            .fit(true);
            
        // 渲染图形
        const dotSource = `{}`;
        graphviz.renderDot(dotSource);
        
        // 控制按钮
        document.getElementById("zoomIn").onclick = () => {{
            graphviz.zoomRelativeToMouse(1.2);
        }};
        
        document.getElementById("zoomOut").onclick = () => {{
            graphviz.zoomRelativeToMouse(0.8);
        }};
        
        document.getElementById("resetZoom").onclick = () => {{
            graphviz.resetZoom();
            graphviz.fit(true).center();
        }};
        
        document.getElementById("download").onclick = () => {{
            const svg = document.querySelector("#graph svg");
            const serializer = new XMLSerializer();
            let source = serializer.serializeToString(svg);
            
            // 添加命名空间
            if (!source.match(/^<svg[^>]+xmlns="http:\/\/www\.w3\.org\/2000\/svg"/)) {{
                source = source.replace(/^<svg/, '<svg xmlns="http://www.w3.org/2000/svg"');
            }}
            
            // 添加XML声明
            source = '<?xml version="1.0" standalone="no"?>\r\n' + source;
            
            // 将SVG源转换为URI数据模式
            const svgBlob = new Blob([source], {{ type: "image/svg+xml;charset=utf-8" }});
            const svgUrl = URL.createObjectURL(svgBlob);
            
            // 创建下载链接
            const downloadLink = document.createElement("a");
            downloadLink.href = svgUrl;
            downloadLink.download = "lumosai_visualization.svg";
            document.body.appendChild(downloadLink);
            downloadLink.click();
            document.body.removeChild(downloadLink);
        }};
    </script>
</body>
</html>
"##,
        escaped_dot_content
    );
    
    // 写入HTML文件
    fs::write(output_path, html_content)
        .map_err(|e| CliError::io_error(e, output_path))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    
    #[test]
    fn test_default_options() {
        let options = VisualizeOptions::default();
        assert_eq!(options.format, "svg");
        assert_eq!(options.include_performance, false);
        assert_eq!(options.interactive, false);
        assert_eq!(options.agent, None);
        assert_eq!(options.workflow, None);
    }
    
    #[test]
    fn test_validate_format() {
        assert!(validate_format("png").is_ok());
        assert!(validate_format("svg").is_ok());
        assert!(validate_format("pdf").is_ok());
        assert!(validate_format("html").is_ok());
        assert!(validate_format("PNG").is_ok()); // 大写也应该有效
        
        assert!(validate_format("unknown").is_err());
    }
    
    #[test]
    fn test_extract_function_name() {
        assert_eq!(extract_function_name("fn test() {"), Some("test".to_string()));
        assert_eq!(extract_function_name("    fn hello_world() -> String {"), Some("hello_world".to_string()));
        assert_eq!(extract_function_name("pub fn process(input: &str) -> Result<String, Error> {"), Some("process".to_string()));
        assert_eq!(extract_function_name("// fn comment() {"), None);
    }
    
    #[test]
    fn test_extract_name_from_use() {
        assert_eq!(extract_name_from_use("use lumosai::tools::calculator;"), Some("calculator".to_string()));
        assert_eq!(extract_name_from_use("use crate::agents::assistant::Agent;"), Some("Agent".to_string()));
    }
    
    #[test]
    fn test_create_minimal_project_structure() -> CliResult<()> {
        // 创建临时项目目录
        let temp_dir = temp_dir().join("lumosai_vis_test");
        let src_dir = temp_dir.join("src");
        let agents_dir = src_dir.join("agents");
        let agent_dir = agents_dir.join("test_agent");
        
        // 清理可能存在的旧目录
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        // 创建目录结构
        fs::create_dir_all(&agent_dir).unwrap();
        
        // 创建Cargo.toml
        fs::write(
            temp_dir.join("Cargo.toml"),
            r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
lumosai = "0.1.0"
"#,
        ).unwrap();
        
        // 创建代理文件
        fs::write(
            agent_dir.join("mod.rs"),
            r#"use lumosai::tools::calculator;
use lumosai::llm::openai;

pub fn run() {
    println!("Hello from test agent!");
}

fn process_input(input: &str) -> String {
    format!("Processed: {}", input)
}
"#,
        ).unwrap();
        
        // 测试解析代理代码
        let agent_code = fs::read_to_string(agent_dir.join("mod.rs")).unwrap();
        let (tools, llms, functions) = parse_agent_code(&agent_code);
        
        assert_eq!(tools, vec!["calculator"]);
        assert_eq!(llms, vec!["openai"]);
        assert_eq!(functions, vec!["run", "process_input"]);
        
        // 清理
        fs::remove_dir_all(&temp_dir).unwrap();
        
        Ok(())
    }
} 