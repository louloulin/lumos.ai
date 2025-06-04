//! CLI functionality tests
//! 
//! Tests for the enhanced CLI tools including project creation,
//! development server, and tool management.

use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_project_config_creation() {
    // Test creating a project configuration
    let config = lumosai_core::cli::ProjectConfig {
        name: "test-project".to_string(),
        version: "0.1.0".to_string(),
        description: Some("A test project".to_string()),
        authors: vec!["Test Author <test@example.com>".to_string()],
        license: Some("MIT".to_string()),
        dependencies: {
            let mut deps = HashMap::new();
            deps.insert("lumosai".to_string(), "0.1.0".to_string());
            deps
        },
        dev_dependencies: HashMap::new(),
        tools: vec![],
        build: lumosai_core::cli::BuildConfig {
            target: "release".to_string(),
            optimization: "speed".to_string(),
            features: vec!["default".to_string()],
            exclude: vec!["tests".to_string()],
        },
        deployment: lumosai_core::cli::DeploymentConfig {
            platforms: HashMap::new(),
            environment: HashMap::new(),
        },
    };
    
    assert_eq!(config.name, "test-project");
    assert_eq!(config.version, "0.1.0");
    assert_eq!(config.description, Some("A test project".to_string()));
    assert_eq!(config.authors.len(), 1);
    assert_eq!(config.license, Some("MIT".to_string()));
    assert!(!config.dependencies.is_empty());
}

#[tokio::test]
async fn test_default_project_config() {
    // Test default project configuration
    let config = lumosai_core::cli::ProjectConfig::default();
    
    assert_eq!(config.name, "my-lumos-project");
    assert_eq!(config.version, "0.1.0");
    assert_eq!(config.license, Some("MIT".to_string()));
    assert!(config.authors.is_empty());
    assert!(config.dependencies.is_empty());
    assert!(config.tools.is_empty());
}

#[tokio::test]
async fn test_tool_config_creation() {
    // Test creating tool configurations
    let tool_config = lumosai_core::cli::ToolConfig {
        name: "web-search".to_string(),
        version: "1.0.0".to_string(),
        source: lumosai_core::cli::ToolSource::Marketplace { 
            package: "web-search".to_string() 
        },
        config: {
            let mut config = HashMap::new();
            config.insert("api_key".to_string(), serde_json::json!("test-key"));
            config
        },
    };
    
    assert_eq!(tool_config.name, "web-search");
    assert_eq!(tool_config.version, "1.0.0");
    assert!(!tool_config.config.is_empty());
    
    match tool_config.source {
        lumosai_core::cli::ToolSource::Marketplace { package } => {
            assert_eq!(package, "web-search");
        }
        _ => panic!("Expected Marketplace source"),
    }
}

#[tokio::test]
async fn test_tool_source_variants() {
    // Test different tool source variants
    let marketplace_source = lumosai_core::cli::ToolSource::Marketplace { 
        package: "test-package".to_string() 
    };
    
    let git_source = lumosai_core::cli::ToolSource::Git { 
        url: "https://github.com/user/repo.git".to_string(),
        branch: Some("main".to_string()),
    };
    
    let local_source = lumosai_core::cli::ToolSource::Local { 
        path: PathBuf::from("/path/to/tool") 
    };
    
    let mcp_source = lumosai_core::cli::ToolSource::MCP { 
        server: "mcp://localhost:8080".to_string() 
    };
    
    // Test serialization/deserialization
    let sources = vec![marketplace_source, git_source, local_source, mcp_source];
    
    for source in sources {
        let serialized = serde_json::to_string(&source);
        assert!(serialized.is_ok());
        
        let deserialized: Result<lumosai_core::cli::ToolSource, _> = 
            serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }
}

#[tokio::test]
async fn test_build_config() {
    // Test build configuration
    let build_config = lumosai_core::cli::BuildConfig {
        target: "wasm".to_string(),
        optimization: "size".to_string(),
        features: vec!["web".to_string(), "async".to_string()],
        exclude: vec!["examples".to_string(), "benches".to_string()],
    };
    
    assert_eq!(build_config.target, "wasm");
    assert_eq!(build_config.optimization, "size");
    assert_eq!(build_config.features.len(), 2);
    assert_eq!(build_config.exclude.len(), 2);
}

#[tokio::test]
async fn test_deployment_config() {
    // Test deployment configuration
    let mut platforms = HashMap::new();
    platforms.insert("production".to_string(), lumosai_core::cli::PlatformConfig {
        provider: "aws".to_string(),
        region: Some("us-west-2".to_string()),
        instance_type: Some("t3.micro".to_string()),
        scaling: Some(lumosai_core::cli::ScalingConfig {
            min_instances: 1,
            max_instances: 10,
            target_cpu: 70.0,
            target_memory: 80.0,
        }),
        environment: {
            let mut env = HashMap::new();
            env.insert("NODE_ENV".to_string(), "production".to_string());
            env
        },
    });
    
    let deployment_config = lumosai_core::cli::DeploymentConfig {
        platforms,
        environment: {
            let mut env = HashMap::new();
            env.insert("LOG_LEVEL".to_string(), "info".to_string());
            env
        },
    };
    
    assert!(!deployment_config.platforms.is_empty());
    assert!(!deployment_config.environment.is_empty());
    
    let platform = deployment_config.platforms.get("production").unwrap();
    assert_eq!(platform.provider, "aws");
    assert_eq!(platform.region, Some("us-west-2".to_string()));
    assert!(platform.scaling.is_some());
}

#[tokio::test]
async fn test_scaling_config() {
    // Test scaling configuration
    let scaling_config = lumosai_core::cli::ScalingConfig {
        min_instances: 2,
        max_instances: 20,
        target_cpu: 75.0,
        target_memory: 85.0,
    };
    
    assert_eq!(scaling_config.min_instances, 2);
    assert_eq!(scaling_config.max_instances, 20);
    assert_eq!(scaling_config.target_cpu, 75.0);
    assert_eq!(scaling_config.target_memory, 85.0);
}

#[tokio::test]
async fn test_cli_utils_project_detection() {
    // Test project detection utilities
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Initially should not be a Lumos project
    assert!(!lumosai_core::cli::CliUtils::is_lumos_project(project_path));
    
    // Create lumos.toml file
    let lumos_toml = project_path.join("lumos.toml");
    std::fs::write(&lumos_toml, "[package]\nname = \"test\"").unwrap();
    
    // Now should be detected as a Lumos project
    assert!(lumosai_core::cli::CliUtils::is_lumos_project(project_path));
}

#[tokio::test]
async fn test_cli_utils_project_root_finding() {
    // Test finding project root
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    let sub_dir = project_path.join("src").join("deep").join("nested");
    std::fs::create_dir_all(&sub_dir).unwrap();
    
    // Create lumos.toml in project root
    let lumos_toml = project_path.join("lumos.toml");
    std::fs::write(&lumos_toml, "[package]\nname = \"test\"").unwrap();
    
    // Should find project root from nested directory
    let found_root = lumosai_core::cli::CliUtils::find_project_root(&sub_dir);
    assert!(found_root.is_some());
    assert_eq!(found_root.unwrap(), project_path);
}

#[tokio::test]
async fn test_cli_utils_config_serialization() {
    // Test configuration save/load
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test-config.toml");
    
    let original_config = lumosai_core::cli::ProjectConfig {
        name: "test-serialization".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test config serialization".to_string()),
        ..Default::default()
    };
    
    // Save configuration
    let save_result = lumosai_core::cli::CliUtils::save_config(&config_path, &original_config);
    assert!(save_result.is_ok());
    
    // Load configuration
    let load_result = lumosai_core::cli::CliUtils::load_config(&config_path);
    assert!(load_result.is_ok());
    
    let loaded_config = load_result.unwrap();
    assert_eq!(loaded_config.name, original_config.name);
    assert_eq!(loaded_config.version, original_config.version);
    assert_eq!(loaded_config.description, original_config.description);
}

#[tokio::test]
async fn test_dev_server_config() {
    // Test development server configuration
    let dev_config = lumosai_core::cli::dev_server::DevServerConfig {
        port: 3000,
        hot_reload: true,
        debug: true,
        project_root: PathBuf::from("/test/project"),
        watch_paths: vec![
            PathBuf::from("/test/project/src"),
            PathBuf::from("/test/project/tests"),
        ],
        ignore_patterns: vec![
            "target".to_string(),
            ".git".to_string(),
            "node_modules".to_string(),
        ],
    };
    
    assert_eq!(dev_config.port, 3000);
    assert!(dev_config.hot_reload);
    assert!(dev_config.debug);
    assert_eq!(dev_config.watch_paths.len(), 2);
    assert_eq!(dev_config.ignore_patterns.len(), 3);
}

#[tokio::test]
async fn test_dev_server_utils() {
    // Test development server utilities
    let test_path = PathBuf::from("test.rs");
    assert!(lumosai_core::cli::dev_server::DevServerUtils::is_source_file(&test_path));
    
    let non_source_path = PathBuf::from("test.txt");
    assert!(!lumosai_core::cli::dev_server::DevServerUtils::is_source_file(&non_source_path));
    
    // Test file extension detection
    let extension = lumosai_core::cli::dev_server::DevServerUtils::get_file_extension(&test_path);
    assert_eq!(extension, Some("rs".to_string()));
    
    // Test ignore pattern checking
    let ignore_patterns = vec!["target".to_string(), ".git".to_string()];
    let target_path = PathBuf::from("target/debug/test");
    assert!(lumosai_core::cli::dev_server::DevServerUtils::should_ignore_path(&target_path, &ignore_patterns));
    
    let src_path = PathBuf::from("src/main.rs");
    assert!(!lumosai_core::cli::dev_server::DevServerUtils::should_ignore_path(&src_path, &ignore_patterns));
}

#[tokio::test]
async fn test_project_stats() {
    // Test project statistics
    let mut stats = lumosai_core::cli::dev_server::ProjectStats::default();
    
    stats.source_files = 10;
    stats.test_files = 5;
    stats.total_size = 1024 * 1024; // 1MB
    stats.last_modified = Some(std::time::SystemTime::now());
    
    assert_eq!(stats.source_files, 10);
    assert_eq!(stats.test_files, 5);
    assert_eq!(stats.total_size, 1024 * 1024);
    assert!(stats.last_modified.is_some());
    
    // Test file size formatting
    let formatted_size = lumosai_core::cli::dev_server::DevServerUtils::format_file_size(stats.total_size);
    assert!(formatted_size.contains("MB"));
}

#[tokio::test]
async fn test_template_registry() {
    // Test template registry
    let registry = lumosai_core::cli::templates::TemplateRegistry::new();
    
    // Test getting templates
    let basic_template = registry.get_template("basic");
    assert!(basic_template.is_some());
    assert_eq!(basic_template.unwrap().name, "basic");
    
    let web_agent_template = registry.get_template("web-agent");
    assert!(web_agent_template.is_some());
    assert_eq!(web_agent_template.unwrap().name, "web-agent");
    
    // Test listing all templates
    let all_templates = registry.list_templates();
    assert!(!all_templates.is_empty());
    assert!(all_templates.len() >= 4); // basic, web-agent, data-agent, chat-bot
    
    // Test getting templates by category
    let basic_templates = registry.get_templates_by_category(
        lumosai_core::cli::templates::TemplateCategory::Basic
    );
    assert!(!basic_templates.is_empty());
}

#[tokio::test]
async fn test_deployment_manager() {
    // Test deployment manager creation
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_path_buf();
    let project_config = lumosai_core::cli::ProjectConfig::default();
    
    let deployment_manager = lumosai_core::cli::deployment::DeploymentManager::new(
        project_root,
        project_config,
    );
    
    // Test deployment utilities
    let docker_available = lumosai_core::cli::deployment::DeploymentUtils::check_docker().await;
    // Note: This might be false in test environment, which is expected
    
    let k8s_available = lumosai_core::cli::deployment::DeploymentUtils::check_kubernetes().await;
    // Note: This might be false in test environment, which is expected
    
    // These are just availability checks, not assertions since tools might not be installed
    println!("Docker available: {}", docker_available);
    println!("Kubernetes available: {}", k8s_available);
}
