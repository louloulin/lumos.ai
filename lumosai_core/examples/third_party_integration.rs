//! 第三方集成综合示例
//! 
//! 展示LumosAI的第三方集成能力，包括：
//! - 多个LLM提供商
//! - 云服务适配器
//! - 统一API使用
//! - 部署配置

use lumosai_core::{
    llm::*,
    cloud::*,
    unified_api,
    error::Result,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI 第三方集成综合示例");
    println!("=====================================\n");

    // 1. 展示所有LLM提供商
    demonstrate_llm_providers().await?;
    
    // 2. 展示云服务适配器
    demonstrate_cloud_adapters().await?;
    
    // 3. 展示统一API
    demonstrate_unified_api().await?;
    
    // 4. 展示部署配置
    demonstrate_deployment_config().await?;

    println!("\n🎉 所有第三方集成示例运行完成!");
    Ok(())
}

/// 展示所有LLM提供商
async fn demonstrate_llm_providers() -> Result<()> {
    println!("📱 LLM提供商展示");
    println!("------------------");

    // 创建所有提供商实例
    let providers = vec![
        ("OpenAI", Box::new(OpenAiProvider::new("demo-key".to_string(), "gpt-4".to_string())) as Box<dyn LlmProvider>),
        ("Anthropic", Box::new(AnthropicProvider::new("demo-key".to_string(), "claude-3-sonnet".to_string()))),
        ("Claude", Box::new(ClaudeProvider::new("demo-key".to_string(), "claude-3-5-sonnet".to_string()))),
        ("Qwen", Box::new(QwenProvider::new("demo-key".to_string(), "qwen-turbo".to_string(), "https://api.qwen.com".to_string()))),
        ("DeepSeek", Box::new(DeepSeekProvider::new("demo-key".to_string(), Some("deepseek-chat".to_string())))),
        ("Cohere", Box::new(CohereProvider::new("demo-key".to_string(), "command-r".to_string()))),
        ("Gemini", Box::new(GeminiProvider::new("demo-key".to_string(), "gemini-pro".to_string()))),
        ("Ollama", Box::new(OllamaProvider::localhost("llama2".to_string()))),
        ("Together", Box::new(TogetherProvider::new("demo-key".to_string(), "meta-llama/Llama-2-7b-chat-hf".to_string()))),
    ];

    for (name, provider) in &providers {
        println!("🔵 {}: {}", name, provider.name());
        println!("   - 函数调用支持: {}", if provider.supports_function_calling() { "✅" } else { "❌" });
        println!("   - 函数调用支持: {}", if provider.supports_function_calling() { "✅" } else { "❌" });
    }

    // 特别展示Claude的模型变体
    println!("\n🎭 Claude模型变体:");
    let claude_sonnet = ClaudeProvider::sonnet("demo-key".to_string());
    let claude_opus = ClaudeProvider::opus("demo-key".to_string());
    let claude_haiku = ClaudeProvider::haiku("demo-key".to_string());
    
    println!("   - Sonnet: {}", claude_sonnet.model());
    println!("   - Opus: {}", claude_opus.model());
    println!("   - Haiku: {}", claude_haiku.model());

    println!("✅ LLM提供商展示完成 (9个提供商)\n");
    Ok(())
}

/// 展示云服务适配器
async fn demonstrate_cloud_adapters() -> Result<()> {
    println!("☁️  云服务适配器展示");
    println!("--------------------");

    // 创建所有云适配器
    let aws = AwsAdapter::new(
        "us-east-1".to_string(),
        "demo-key".to_string(),
        "demo-secret".to_string(),
    ).with_ecs_cluster("my-cluster".to_string());

    let azure = AzureAdapter::new(
        "subscription-123".to_string(),
        "resource-group".to_string(),
        "tenant-123".to_string(),
        "client-123".to_string(),
        "client-secret".to_string(),
        "East US".to_string(),
    );

    let gcp = GcpAdapter::new(
        "my-project".to_string(),
        "service-account.json".to_string(),
        "us-central1".to_string(),
    ).with_zone("us-central1-a".to_string());

    let adapters: Vec<(&str, Box<dyn CloudAdapter>)> = vec![
        ("AWS", Box::new(aws)),
        ("Azure", Box::new(azure)),
        ("GCP", Box::new(gcp)),
    ];

    for (name, adapter) in &adapters {
        println!("🟦 {}: {}", name, adapter.name());
        println!("   - 支持的服务:");
        for service in adapter.supported_services() {
            let service_name = match service {
                CloudService::Container => "容器服务",
                CloudService::Function => "函数计算",
                CloudService::Database => "数据库",
                CloudService::Storage => "对象存储",
                CloudService::MessageQueue => "消息队列",
                CloudService::Cache => "缓存服务",
                CloudService::Monitoring => "监控服务",
                CloudService::Logging => "日志服务",
            };
            println!("     • {}", service_name);
        }
    }

    println!("✅ 云服务适配器展示完成 (3个适配器)\n");
    Ok(())
}

/// 展示统一API
async fn demonstrate_unified_api() -> Result<()> {
    println!("🔧 统一API展示");
    println!("---------------");

    // 展示LLM便利函数
    println!("📱 LLM便利函数:");
    let ollama_provider = unified_api::llm::ollama("llama2");
    println!("   - Ollama提供商: {}", ollama_provider.name());

    // 展示向量存储便利函数
    println!("🗄️  向量存储便利函数:");
    let memory_storage = unified_api::vector::memory().await?;
    println!("   - 内存存储: {:?}", memory_storage);

    // 展示快速功能
    println!("⚡ 快速功能:");
    println!("   - 快速聊天机器人: 可用");
    println!("   - 快速RAG系统: 可用");
    println!("   - 快速云部署: 可用");
    println!("   - 一键AI应用: 可用");

    println!("✅ 统一API展示完成\n");
    Ok(())
}

/// 展示部署配置
async fn demonstrate_deployment_config() -> Result<()> {
    println!("📦 部署配置展示");
    println!("----------------");

    // 创建完整的部署配置
    let mut environment = HashMap::new();
    environment.insert("NODE_ENV".to_string(), "production".to_string());
    environment.insert("PORT".to_string(), "8080".to_string());

    let config = DeploymentConfig {
        name: "lumosai-demo-app".to_string(),
        version: "1.0.0".to_string(),
        image: "lumosai/demo:latest".to_string(),
        environment,
        resources: ResourceConfig {
            cpu: 2.0,
            memory: 4096,
            storage: Some(20),
            gpu: Some(1),
        },
        network: NetworkConfig {
            ports: vec![
                PortMapping {
                    container_port: 8080,
                    host_port: Some(80),
                    protocol: "HTTP".to_string(),
                },
                PortMapping {
                    container_port: 8443,
                    host_port: Some(443),
                    protocol: "HTTPS".to_string(),
                },
            ],
            public: true,
            domain: Some("demo.lumosai.com".to_string()),
            ssl: Some(SslConfig {
                certificate_id: "cert-123456".to_string(),
                force_https: true,
            }),
        },
        storage: Some(StorageConfig {
            storage_type: StorageType::Persistent,
            size: 100,
            mount_path: "/data".to_string(),
        }),
        autoscaling: Some(AutoscalingConfig {
            min_instances: 2,
            max_instances: 20,
            cpu_threshold: 70.0,
            memory_threshold: 80.0,
            scale_up_cooldown: 300,
            scale_down_cooldown: 600,
        }),
        health_check: Some(HealthCheckConfig {
            path: "/health".to_string(),
            interval: 30,
            timeout: 10,
            healthy_threshold: 2,
            unhealthy_threshold: 3,
        }),
    };

    println!("🏗️  应用配置:");
    println!("   - 名称: {}", config.name);
    println!("   - 版本: {}", config.version);
    println!("   - 镜像: {}", config.image);
    println!("   - 环境变量: {} 个", config.environment.len());

    println!("💻 资源配置:");
    println!("   - CPU: {} 核", config.resources.cpu);
    println!("   - 内存: {} MB", config.resources.memory);
    println!("   - 存储: {} GB", config.resources.storage.unwrap_or(0));
    println!("   - GPU: {} 个", config.resources.gpu.unwrap_or(0));

    println!("🌐 网络配置:");
    println!("   - 端口数量: {}", config.network.ports.len());
    println!("   - 公开访问: {}", config.network.public);
    println!("   - 域名: {}", config.network.domain.as_ref().unwrap_or(&"无".to_string()));

    if let Some(autoscaling) = &config.autoscaling {
        println!("📈 自动扩缩容:");
        println!("   - 最小实例: {}", autoscaling.min_instances);
        println!("   - 最大实例: {}", autoscaling.max_instances);
        println!("   - CPU阈值: {}%", autoscaling.cpu_threshold);
        println!("   - 内存阈值: {}%", autoscaling.memory_threshold);
    }

    if let Some(health_check) = &config.health_check {
        println!("🏥 健康检查:");
        println!("   - 检查路径: {}", health_check.path);
        println!("   - 检查间隔: {} 秒", health_check.interval);
        println!("   - 超时时间: {} 秒", health_check.timeout);
    }

    // 模拟部署到不同云平台
    println!("\n🚀 模拟部署:");
    
    let aws_adapter = AwsAdapter::new(
        "us-east-1".to_string(),
        "demo-key".to_string(),
        "demo-secret".to_string(),
    );
    
    match aws_adapter.deploy_application(&config).await {
        Ok(result) => {
            println!("   - AWS部署: ✅ 成功");
            println!("     • 部署ID: {}", result.deployment_id);
            println!("     • 状态: {:?}", result.status);
            println!("     • URL: {}", result.url.unwrap_or("无".to_string()));
        }
        Err(e) => {
            println!("   - AWS部署: ❌ 失败 ({})", e);
        }
    }

    println!("✅ 部署配置展示完成\n");
    Ok(())
}

/// 展示功能统计
#[allow(dead_code)]
fn show_feature_statistics() {
    println!("📊 功能统计");
    println!("------------");
    println!("🔵 LLM提供商: 9个");
    println!("   • OpenAI, Anthropic, Claude, Qwen, DeepSeek");
    println!("   • Cohere, Gemini, Ollama, Together");
    println!();
    println!("☁️  云服务适配器: 3个");
    println!("   • AWS (ECS, Lambda, CloudWatch)");
    println!("   • Azure (Container Instances, Functions, Monitor)");
    println!("   • GCP (Cloud Run, Functions, Monitoring)");
    println!();
    println!("🔧 统一API模块: 4个");
    println!("   • llm (LLM便利函数)");
    println!("   • vector (向量存储便利函数)");
    println!("   • agent (Agent便利函数)");
    println!("   • cloud (云服务便利函数)");
    println!("   • quick (一站式便利函数)");
    println!();
    println!("📦 部署功能:");
    println!("   • 完整的部署配置支持");
    println!("   • 自动扩缩容配置");
    println!("   • 负载均衡配置");
    println!("   • 健康检查配置");
    println!("   • SSL/TLS支持");
}
