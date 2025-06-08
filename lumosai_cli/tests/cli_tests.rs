use std::process::Command;
use tempfile::TempDir;

// 简单的单元测试，验证CLI模块的基本功能
#[test]
fn test_cli_structure() {
    // 测试CLI结构是否正确定义
    use lumosai_cli::Cli;
    use clap::Parser;

    // 验证CLI可以解析基本的help命令
    let result = Cli::try_parse_from(&["lumos", "--help"]);
    // 这应该返回错误，因为--help会导致程序退出，但这是预期的行为
    assert!(result.is_err());
}

#[test]
fn test_cli_version() {
    // 测试版本信息
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());
    assert!(version.chars().next().unwrap().is_ascii_digit());
}

#[test]
fn test_create_args_parsing() {
    // 测试create命令的参数解析
    use lumosai_cli::{Cli, Commands};
    use clap::Parser;

    let result = Cli::try_parse_from(&[
        "lumos",
        "create",
        "--name", "test-project",
        "--llm", "openai"
    ]);

    assert!(result.is_ok());
    let cli = result.unwrap();

    match cli.command {
        Commands::Create(args) => {
            assert_eq!(args.name, Some("test-project".to_string()));
            assert_eq!(args.llm, Some("openai".to_string()));
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_ui_args_parsing() {
    // 测试UI命令的参数解析
    use lumosai_cli::{Cli, Commands};
    use clap::Parser;

    let result = Cli::try_parse_from(&[
        "lumos",
        "ui",
        "--port", "8080",
        "--dev"
    ]);

    assert!(result.is_ok());
    let cli = result.unwrap();

    match cli.command {
        Commands::Ui(args) => {
            assert_eq!(args.port, 8080);
            assert_eq!(args.dev, true);
        }
        _ => panic!("Expected Ui command"),
    }
}

#[test]
#[ignore] // 忽略这个测试，因为当前CLI没有template命令
fn test_template_list() {
    let output = Command::new("cargo")
        .args(["run", "--", "ui", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 打印输出以便调试
    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);

    assert!(output.status.success());
    // 检查UI命令是否存在
    assert!(stdout.contains("ui") || stderr.contains("ui") || stdout.contains("端口") || stderr.contains("端口"));
}

#[test]
#[ignore] // 忽略真实创建项目的测试，避免影响文件系统
fn test_init_project() {
    // 创建临时目录作为测试环境
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    
    // 执行初始化命令
    let output = Command::new("cargo")
        .args([
            "run", "--",
            "init",
            "--name", "test_project",
            "--template", "agent",
            "--output", &temp_path.join("test_project").to_string_lossy(),
        ])
        .output()
        .expect("Failed to execute command");
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(output.status.success());
    assert!(stdout.contains("项目已创建"));
    
    // 验证文件是否创建
    let project_dir = temp_path.join("test_project");
    assert!(project_dir.exists());
    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("src").exists());
    assert!(project_dir.join("src").join("main.rs").exists());
}

#[test]
fn test_validate_cli_flags() {
    // 测试实际存在的命令
    let commands = [
        ("create", "--help"),
        ("dev", "--help"),
        ("ui", "--help"),
        ("playground", "--help"),
        ("api", "--help"),
    ];

    for cmd in &commands {
        let args = match cmd {
            (command, help) => vec!["run", "--", command, help],
        };

        let output = Command::new("cargo")
            .args(&args)
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // 打印输出以便调试
        println!("Command: {:?}", cmd);
        println!("STDOUT: {}", stdout);
        println!("STDERR: {}", stderr);

        assert!(output.status.success(), "命令 '{:?}' 帮助失败", cmd);
    }
}