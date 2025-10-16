use lumosai_core::prelude::*;
use std::env;
use std::process::Command;

/// CLI 基础功能演示
///
/// 这个示例展示了如何使用 LumosAI CLI 工具的基本功能：
/// 1. 检查 CLI 工具是否可用
/// 2. 显示版本信息
/// 3. 显示帮助信息
/// 4. 演示基本命令结构

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI CLI 基础功能演示");
    println!("{}", "=".repeat(50));

    // 1. 检查 CLI 工具是否可用
    println!("\n📋 1. 检查 CLI 工具可用性");
    match check_cli_availability().await {
        Ok(_) => println!("✅ LumosAI CLI 工具可用"),
        Err(e) => {
            println!("❌ CLI 工具不可用: {}", e);
            println!("💡 请确保已编译 lumosai_cli 包");
            return Ok(());
        }
    }

    // 2. 显示版本信息
    println!("\n📋 2. 显示版本信息");
    if let Ok(version) = get_cli_version().await {
        println!("✅ LumosAI CLI 版本: {}", version);
    } else {
        println!("⚠️  无法获取版本信息");
    }

    // 3. 显示帮助信息
    println!("\n📋 3. 显示帮助信息");
    if let Ok(help) = get_cli_help().await {
        println!("✅ CLI 帮助信息:");
        println!("{}", help);
    } else {
        println!("⚠️  无法获取帮助信息");
    }

    // 4. 演示可用命令
    println!("\n📋 4. 可用的 CLI 命令");
    demonstrate_available_commands();

    // 5. 演示如何在代码中调用 CLI
    println!("\n📋 5. 在代码中调用 CLI 示例");
    demonstrate_cli_integration();

    println!("\n🎉 CLI 基础功能演示完成！");
    println!("\n💡 提示:");
    println!("   - 使用 'cargo run --bin lumosai_cli -- --help' 查看完整帮助");
    println!("   - 使用 'cargo run --bin lumosai_cli -- create --help' 查看创建命令帮助");
    println!("   - 使用 'cargo run --bin lumosai_cli -- dev --help' 查看开发命令帮助");

    Ok(())
}

/// 检查 CLI 工具是否可用
async fn check_cli_availability() -> Result<()> {
    // 尝试运行 CLI 工具的 --version 命令
    let output = Command::new("cargo")
        .args(&["run", "--bin", "lumosai_cli", "--", "--version"])
        .current_dir(env::current_dir()?)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(())
            } else {
                Err(Error::Other("CLI 工具执行失败".to_string()))
            }
        }
        Err(_) => Err(Error::Other("无法执行 CLI 工具".to_string())),
    }
}

/// 获取 CLI 版本信息
async fn get_cli_version() -> Result<String> {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "lumosai_cli", "--", "--version"])
        .current_dir(env::current_dir()?)
        .output()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.trim().to_string())
    } else {
        Err(Error::Other("无法获取版本信息".to_string()))
    }
}

/// 获取 CLI 帮助信息
async fn get_cli_help() -> Result<String> {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "lumosai_cli", "--", "--help"])
        .current_dir(env::current_dir()?)
        .output()?;

    if output.status.success() {
        let help = String::from_utf8_lossy(&output.stdout);
        Ok(help.trim().to_string())
    } else {
        Err(Error::Other("无法获取帮助信息".to_string()))
    }
}

/// 演示可用的命令
fn demonstrate_available_commands() {
    println!("✅ LumosAI CLI 主要命令:");

    let commands = vec![
        (
            "create",
            "创建新的 Lumosai 项目",
            "cargo run --bin lumosai_cli -- create --name my-project",
        ),
        (
            "dev",
            "启动开发服务器",
            "cargo run --bin lumosai_cli -- dev",
        ),
        ("ui", "启动 UI 服务器", "cargo run --bin lumosai_cli -- ui"),
        (
            "playground",
            "启动交互式测试环境",
            "cargo run --bin lumosai_cli -- playground",
        ),
        ("api", "生成 API 端点", "cargo run --bin lumosai_cli -- api"),
        (
            "visualize",
            "生成可视化图表",
            "cargo run --bin lumosai_cli -- visualize",
        ),
        (
            "monitoring",
            "启动监控服务器",
            "cargo run --bin lumosai_cli -- monitoring",
        ),
    ];

    for (cmd, desc, example) in commands {
        println!("   📌 {}: {}", cmd, desc);
        println!("      示例: {}", example);
        println!();
    }
}

/// 演示 CLI 集成
fn demonstrate_cli_integration() {
    println!("✅ 在 Rust 代码中集成 CLI 的方法:");

    println!("   📌 方法 1: 使用 std::process::Command");
    println!("      let output = Command::new(\"cargo\")");
    println!("          .args(&[\"run\", \"--bin\", \"lumosai_cli\", \"--\", \"--version\"])");
    println!("          .output()?;");
    println!();

    println!("   📌 方法 2: 直接调用 CLI 库函数");
    println!("      use lumosai_cli::commands;");
    println!("      commands::create::run(name, components, llm, api_key, project_name, example).await?;");
    println!();

    println!("   📌 方法 3: 使用 CLI 配置结构");
    println!("      use lumosai_cli::{{Cli, Commands, CreateArgs}};");
    println!("      let args = CreateArgs {{ name: Some(\\\"my-project\\\".to_string()), ... }};");
    println!("      // 处理命令...");
    println!();
}
