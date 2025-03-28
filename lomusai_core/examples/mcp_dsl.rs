use lomusai_core::Result;
use lomusai_core::mcp::{McpClient, McpTool};
use lumos_macro::mcp_client;

#[tokio::main]
async fn main() -> Result<()> {
    println!("MCP客户端DSL示例");
    
    // 使用mcp_client!宏定义一个MCP客户端配置
    let client = mcp_client! {
        discovery: {
            endpoints: ["https://api.mcp.example.com", "https://tools.mcp.run"],
            auto_register: true,
            interval: 60  // 秒
        },
        
        tools: {
            data_analysis: {
                enabled: true,
                auth: {
                    type: "api_key",
                    key_env: "DATA_ANALYSIS_API_KEY"
                }
            },
            image_generation: {
                enabled: true,
                rate_limit: 100,
                models: ["stable-diffusion", "dalle"]
            },
            translate: {
                enabled: false
            }
        },
        
        cache: {
            enabled: true,
            ttl: 3600,  // 秒
            max_size: 100  // MB
        },
        
        defaults: {
            timeout: 30000,  // 毫秒
            retry: {
                count: 3,
                backoff: "exponential"
            }
        }
    };

    // 获取可用的MCP工具
    println!("获取可用的MCP工具...");
    let tools = client.get_available_tools().await?;
    
    println!("找到 {} 个可用工具:", tools.len());
    for tool in &tools {
        println!("- {}: {}", tool.name, tool.description);
        println!("  参数: {}", tool.parameters.len());
    }
    
    // 使用MCP工具执行任务
    println!("\n使用数据分析工具分析数据...");
    let data_analysis_result = client.execute_tool("data_analysis", serde_json::json!({
        "data": [1, 2, 3, 4, 5],
        "operation": "mean"
    })).await?;
    
    println!("数据分析结果: {}", serde_json::to_string_pretty(&data_analysis_result)?);
    
    // 定义一个更复杂的MCP客户端配置
    let advanced_client = mcp_client! {
        discovery: {
            endpoints: ["https://api.mcp.example.com"],
            registry: {
                url: "https://registry.mcp.example.com",
                auth: {
                    type: "bearer",
                    token_env: "MCP_REGISTRY_TOKEN"
                }
            }
        },
        
        tools: {
            document_ocr: {
                enabled: true,
                version: "2.0",
                options: {
                    "enhance": true,
                    "languages": ["zh", "en", "ja"]
                }
            },
            sentiment_analysis: {
                enabled: true,
                provider: "custom",
                endpoint: "https://nlp.company.com/sentiment",
                headers: {
                    "X-Api-Version": "3"
                }
            }
        },
        
        logging: {
            level: "info",
            format: "json",
            destination: "file",
            path: "./logs/mcp.log"
        },
        
        proxy: {
            url: "http://proxy.company.com:8080",
            auth: {
                username_env: "PROXY_USER",
                password_env: "PROXY_PASS"
            },
            bypass: ["localhost", "127.0.0.1"]
        }
    };
    
    println!("\n高级MCP客户端配置已创建，准备连接自定义端点");
    
    // 展示MCP工具链式调用
    let workflow_result = client.pipeline()
        .add_step("extract_data", "data_analysis", serde_json::json!({"operation": "extract"}))
        .add_step("process_image", "image_generation", serde_json::json!({"prompt": "data visualization"}))
        .add_step("translate_result", "translate", serde_json::json!({"target_lang": "zh"}))
        .execute(serde_json::json!({"input": "raw data"}))
        .await?;
    
    println!("\nMCP工具链式调用结果:");
    println!("{}", serde_json::to_string_pretty(&workflow_result)?);
    
    Ok(())
} 