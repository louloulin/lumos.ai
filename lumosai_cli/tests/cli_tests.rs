use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(output.status.success());
    assert!(stdout.contains("Lumos AI"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("dev"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("build"));
    assert!(stdout.contains("deploy"));
    assert!(stdout.contains("template"));
}

#[test]
fn test_init_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "init", "--help"])
        .output()
        .expect("Failed to execute command");
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(output.status.success());
    assert!(stdout.contains("项目名称"));
    assert!(stdout.contains("模板类型"));
    assert!(stdout.contains("输出目录"));
}

#[test]
fn test_dev_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "dev", "--help"])
        .output()
        .expect("Failed to execute command");
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(output.status.success());
    assert!(stdout.contains("项目目录"));
    assert!(stdout.contains("端口号"));
    assert!(stdout.contains("热重载"));
}

#[test]
fn test_template_list() {
    let output = Command::new("cargo")
        .args(["run", "--", "template", "list"])
        .output()
        .expect("Failed to execute command");
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(output.status.success());
    // 因为初始没有模板，所以应该提示下载
    assert!(stdout.contains("下载模板") || stdout.contains("可用模板"));
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
    // 测试多个命令的短选项是否冲突
    let commands = [
        ("init", "--help"),
        ("dev", "--help"),
        ("run", "--help"),
        ("build", "--help"),
        ("deploy", "--help"),
        ("template", "--help"),
    ];
    
    for cmd in &commands {
        let args = match cmd {
            (command, help) => vec!["run", "--", command, help],
        };
        
        let output = Command::new("cargo")
            .args(&args)
            .output()
            .expect("Failed to execute command");
            
        assert!(output.status.success(), "命令 '{:?}' 帮助失败", cmd);
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        if args.contains(&"dev") {
            // 特别验证dev命令的热重载选项使用-r而不是-h
            assert!(stdout.contains("-r, --hot-reload") || stdout.contains("-r, --hot_reload"));
        }
    }
} 