//! 加密解密工具演示程序

use lumosai_core::tool::builtin::crypto::*;
use lumosai_core::tool::{ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("🔐 加密解密工具演示\n");
    println!("{}", "=".repeat(80));
    println!();

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // 测试 1: 哈希计算
    println!("🔑 测试 1: 哈希计算");
    println!("{}", "-".repeat(80));
    
    let tools = get_all_crypto_tools();
    let hash_tool = &tools[0];
    
    println!("工具名称: {}", hash_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", hash_tool.description());
    println!();

    let test_data = "Hello, LumosAI!";
    println!("测试数据: {}", test_data);
    println!();

    let algorithms = vec!["md5", "sha1", "sha256", "sha512"];
    
    for algo in algorithms {
        let params = json!({
            "input": test_data,
            "algorithm": algo,
            "encoding": "hex"
        });

        match hash_tool.execute(params, context.clone(), &options).await {
            Ok(result) => {
                println!("✅ {} 哈希:", result["algorithm"]);
                println!("  哈希值: {}", result["hash"]);
                println!("  长度: {} 字符", result["hash_length"]);
            }
            Err(e) => println!("❌ 计算失败: {}", e),
        }
        println!();
    }

    // 测试 2: 对称加密
    println!("🔒 测试 2: 对称加密");
    println!("{}", "-".repeat(80));
    
    let encrypt_tool = &tools[1];
    
    println!("工具名称: {}", encrypt_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", encrypt_tool.description());
    println!();

    let plaintext = "This is a secret message!";
    let key = "my-super-secret-key-32-bytes!";
    
    println!("明文: {}", plaintext);
    println!("密钥: {}", key);
    println!();

    let encryption_scenarios = vec![
        ("AES-256-GCM", "aes-256-gcm"),
        ("AES-256-CBC", "aes-256-cbc"),
        ("AES-128-GCM", "aes-128-gcm"),
    ];

    for (desc, algo) in encryption_scenarios {
        println!("加密算法: {}", desc);
        
        let params = json!({
            "plaintext": plaintext,
            "key": key,
            "algorithm": algo,
            "encoding": "base64"
        });

        match encrypt_tool.execute(params, context.clone(), &options).await {
            Ok(result) => {
                if result["success"] == true {
                    println!("  ✅ 加密成功:");
                    println!("    密文: {}", result["ciphertext"]);
                    println!("    IV: {}", result["iv"]);
                    if let Some(tag) = result.get("tag") {
                        println!("    认证标签: {}", tag);
                    }
                    println!("    明文长度: {} 字节", result["plaintext_length"]);
                    println!("    密文长度: {} 字节", result["ciphertext_length"]);
                } else {
                    println!("  ❌ 加密失败: {}", result["error"]);
                }
            }
            Err(e) => println!("  ❌ 加密失败: {}", e),
        }
        println!();
    }

    // 测试 3: 对称解密
    println!("🔓 测试 3: 对称解密");
    println!("{}", "-".repeat(80));
    
    let decrypt_tool = &tools[2];
    
    println!("工具名称: {}", decrypt_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", decrypt_tool.description());
    println!();

    let ciphertext = "U2FsdGVkX1+vupppZksvRf5pq5g5XjFRlipRkwB0K1Y=";
    let iv = "1234567890abcdef";
    let tag = "fedcba0987654321";
    
    println!("密文: {}", ciphertext);
    println!("IV: {}", iv);
    println!("认证标签: {}", tag);
    println!();

    let params = json!({
        "ciphertext": ciphertext,
        "key": key,
        "iv": iv,
        "algorithm": "aes-256-gcm",
        "tag": tag
    });

    match decrypt_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == true {
                println!("✅ 解密成功:");
                println!("  明文: {}", result["plaintext"]);
                println!("  明文长度: {} 字节", result["plaintext_length"]);
                println!("  密文长度: {} 字节", result["ciphertext_length"]);
            } else {
                println!("❌ 解密失败: {}", result["error"]);
            }
        }
        Err(e) => println!("❌ 解密失败: {}", e),
    }
    println!();

    // 测试 4: 密码生成
    println!("🎲 测试 4: 密码生成");
    println!("{}", "-".repeat(80));
    
    let password_tool = &tools[3];
    
    println!("工具名称: {}", password_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", password_tool.description());
    println!();

    let password_scenarios = vec![
        (
            "场景 1: 默认配置（16字符）",
            json!({
                "length": 16
            })
        ),
        (
            "场景 2: 强密码（32字符，包含所有类型）",
            json!({
                "length": 32,
                "include_uppercase": true,
                "include_lowercase": true,
                "include_numbers": true,
                "include_symbols": true,
                "exclude_ambiguous": true
            })
        ),
        (
            "场景 3: 简单密码（12字符，仅字母和数字）",
            json!({
                "length": 12,
                "include_uppercase": true,
                "include_lowercase": true,
                "include_numbers": true,
                "include_symbols": false,
                "exclude_ambiguous": true
            })
        ),
        (
            "场景 4: PIN码（8字符，仅数字）",
            json!({
                "length": 8,
                "include_uppercase": false,
                "include_lowercase": false,
                "include_numbers": true,
                "include_symbols": false
            })
        ),
    ];

    for (desc, params) in password_scenarios {
        println!("{}", desc);
        
        match password_tool.execute(params, context.clone(), &options).await {
            Ok(result) => {
                if result["success"] == true {
                    println!("  ✅ 生成成功:");
                    println!("    密码: {}", result["password"]);
                    println!("    长度: {} 字符", result["length"]);
                    println!("    字符集大小: {}", result["charset_size"]);
                    println!("    熵: {} bits", result["entropy_bits"]);
                    println!("    强度: {}", result["strength"]);
                } else {
                    println!("  ❌ 生成失败: {}", result["error"]);
                }
            }
            Err(e) => println!("  ❌ 生成失败: {}", e),
        }
        println!();
    }

    // 测试 5: 参数验证
    println!("🔍 测试 5: 参数验证");
    println!("{}", "-".repeat(80));
    
    println!("测试 5.1: 无效的哈希算法");
    let params = json!({
        "input": "test",
        "algorithm": "sha999"
    });

    match hash_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    println!("测试 5.2: 密钥长度不足");
    let params = json!({
        "plaintext": "test",
        "key": "short"
    });

    match encrypt_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    println!("测试 5.3: 密码长度超出范围");
    let params = json!({
        "length": 200
    });

    match password_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    // 测试 6: 批量获取工具
    println!("📦 测试 6: 获取所有加密解密工具");
    println!("{}", "-".repeat(80));
    
    let all_tools = get_all_crypto_tools();
    println!("总共 {} 个加密解密工具:\n", all_tools.len());
    
    for (i, tool) in all_tools.iter().enumerate() {
        println!("{}. {} - {}", 
            i + 1, 
            tool.name().unwrap_or("unknown"), 
            tool.description()
        );
    }
    println!();

    // 总结
    println!("{}", "=".repeat(80));
    println!("✅ 加密解密工具演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 工具统计:");
    println!("  - 哈希计算工具: hash");
    println!("  - 对称加密工具: encrypt");
    println!("  - 对称解密工具: decrypt");
    println!("  - 密码生成工具: password_generator");
    println!();
    println!("💡 使用建议:");
    println!("  1. 哈希计算: 用于数据完整性验证和密码存储");
    println!("  2. 对称加密: 用于敏感数据的加密存储和传输");
    println!("  3. 对称解密: 用于解密加密的数据");
    println!("  4. 密码生成: 用于生成安全的随机密码");
    println!();
    println!("🚀 下一步:");
    println!("  - 集成真实的加密库（如 ring, rust-crypto）");
    println!("  - 添加非对称加密支持（RSA, ECC）");
    println!("  - 支持密钥派生和密钥管理");
    println!("{}", "=".repeat(80));
}

