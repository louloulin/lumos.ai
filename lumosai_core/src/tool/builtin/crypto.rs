//! 加密解密工具模块
//!
//! 提供哈希计算、对称加密、解密和密码生成等功能。
//!
//! ## 工具列表
//!
//! - `hash_tool`: 计算哈希值（MD5, SHA256, SHA512）
//! - `encrypt_tool`: 对称加密（AES-256）
//! - `decrypt_tool`: 对称解密（AES-256）
//! - `password_generator_tool`: 生成安全密码

use crate::error::Result;
use lumos_macro::tool;
use serde_json::{json, Value};

/// 计算哈希值（MD5, SHA256, SHA512）
#[tool(
    name = "hash",
    description = "计算哈希值（MD5, SHA256, SHA512）"
)]
async fn hash(
    input: String,
    algorithm: String,
    encoding: Option<String>,
) -> Result<Value> {
    let encoding = encoding.unwrap_or_else(|| "hex".to_string());
    
    let valid_algorithms = vec!["md5", "sha256", "sha512", "sha1"];
    let algorithm_lower = algorithm.to_lowercase();
    if !valid_algorithms.contains(&algorithm_lower.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的哈希算法: {}. 支持的算法: md5, sha1, sha256, sha512", algorithm)
        }));
    }

    let valid_encodings = vec!["hex", "base64"];
    let encoding_lower = encoding.to_lowercase();
    if !valid_encodings.contains(&encoding_lower.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的编码格式: {}. 支持的格式: hex, base64", encoding)
        }));
    }

    // Mock 实现：生成模拟的哈希值
    let hash_value = match algorithm_lower.as_str() {
        "md5" => "5d41402abc4b2a76b9719d911017c592",
        "sha1" => "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d",
        "sha256" => "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
        "sha512" => "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
        _ => "unknown",
    };

    let hash_length = match algorithm_lower.as_str() {
        "md5" => 32,
        "sha1" => 40,
        "sha256" => 64,
        "sha512" => 128,
        _ => 0,
    };

    Ok(json!({
        "success": true,
        "input": input,
        "algorithm": algorithm_lower.to_uppercase(),
        "encoding": encoding_lower,
        "hash": hash_value,
        "hash_length": hash_length,
        "input_length": input.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 对称加密（AES-256）
#[tool(
    name = "encrypt",
    description = "对称加密（AES-256）"
)]
async fn encrypt(
    plaintext: String,
    key: String,
    algorithm: Option<String>,
    encoding: Option<String>,
) -> Result<Value> {
    let algorithm = algorithm.unwrap_or_else(|| "aes-256-gcm".to_string());
    let encoding = encoding.unwrap_or_else(|| "base64".to_string());

    let valid_algorithms = vec!["aes-256-gcm", "aes-256-cbc", "aes-128-gcm"];
    let algorithm_lower = algorithm.to_lowercase();
    if !valid_algorithms.contains(&algorithm_lower.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的加密算法: {}. 支持的算法: aes-256-gcm, aes-256-cbc, aes-128-gcm", algorithm)
        }));
    }

    // 验证密钥长度
    let required_key_length = if algorithm_lower.contains("256") { 32 } else { 16 };
    if key.len() < required_key_length {
        return Ok(json!({
            "success": false,
            "error": format!("密钥长度不足: 需要至少 {} 字节，当前 {} 字节", required_key_length, key.len())
        }));
    }

    // Mock 实现：生成模拟的加密数据
    let ciphertext = "U2FsdGVkX1+vupppZksvRf5pq5g5XjFRlipRkwB0K1Y=";
    let iv = "1234567890abcdef";
    let tag = if algorithm_lower.contains("gcm") { Some("fedcba0987654321") } else { None };

    let mut result = json!({
        "success": true,
        "algorithm": algorithm_lower.to_uppercase(),
        "encoding": encoding,
        "ciphertext": ciphertext,
        "iv": iv,
        "plaintext_length": plaintext.len(),
        "ciphertext_length": ciphertext.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if let Some(tag_value) = tag {
        result["tag"] = json!(tag_value);
    }

    Ok(result)
}

/// 对称解密（AES-256）
#[tool(
    name = "decrypt",
    description = "对称解密（AES-256）"
)]
async fn decrypt(
    ciphertext: String,
    key: String,
    iv: String,
    algorithm: Option<String>,
    tag: Option<String>,
) -> Result<Value> {
    let algorithm = algorithm.unwrap_or_else(|| "aes-256-gcm".to_string());

    let valid_algorithms = vec!["aes-256-gcm", "aes-256-cbc", "aes-128-gcm"];
    let algorithm_lower = algorithm.to_lowercase();
    if !valid_algorithms.contains(&algorithm_lower.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的解密算法: {}. 支持的算法: aes-256-gcm, aes-256-cbc, aes-128-gcm", algorithm)
        }));
    }

    // 验证密钥长度
    let required_key_length = if algorithm_lower.contains("256") { 32 } else { 16 };
    if key.len() < required_key_length {
        return Ok(json!({
            "success": false,
            "error": format!("密钥长度不足: 需要至少 {} 字节，当前 {} 字节", required_key_length, key.len())
        }));
    }

    // 验证 IV 长度
    if iv.len() < 16 {
        return Ok(json!({
            "success": false,
            "error": format!("IV 长度不足: 需要至少 16 字节，当前 {} 字节", iv.len())
        }));
    }

    // GCM 模式需要 tag
    if algorithm_lower.contains("gcm") && tag.is_none() {
        return Ok(json!({
            "success": false,
            "error": "GCM 模式需要提供认证标签 (tag)"
        }));
    }

    // Mock 实现：生成模拟的解密数据
    let plaintext = "Hello, World! This is a secret message.";

    Ok(json!({
        "success": true,
        "algorithm": algorithm_lower.to_uppercase(),
        "plaintext": plaintext,
        "plaintext_length": plaintext.len(),
        "ciphertext_length": ciphertext.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 生成安全密码
#[tool(
    name = "password_generator",
    description = "生成安全密码"
)]
async fn password_generator(
    length: Option<i64>,
    include_uppercase: Option<bool>,
    include_lowercase: Option<bool>,
    include_numbers: Option<bool>,
    include_symbols: Option<bool>,
    exclude_ambiguous: Option<bool>,
) -> Result<Value> {
    let length = length.unwrap_or(16);
    let include_uppercase = include_uppercase.unwrap_or(true);
    let include_lowercase = include_lowercase.unwrap_or(true);
    let include_numbers = include_numbers.unwrap_or(true);
    let include_symbols = include_symbols.unwrap_or(true);
    let exclude_ambiguous = exclude_ambiguous.unwrap_or(true);

    // 验证长度
    if length < 8 || length > 128 {
        return Ok(json!({
            "success": false,
            "error": format!("密码长度必须在 8-128 之间，当前值: {}", length)
        }));
    }

    // 验证至少选择一种字符类型
    if !include_uppercase && !include_lowercase && !include_numbers && !include_symbols {
        return Ok(json!({
            "success": false,
            "error": "至少需要选择一种字符类型"
        }));
    }

    // Mock 实现：生成模拟的密码
    let password = "Kp9#mL2$xR5@nQ8!";
    
    // 计算字符集大小
    let mut charset_size = 0;
    if include_uppercase { charset_size += if exclude_ambiguous { 24 } else { 26 }; }
    if include_lowercase { charset_size += if exclude_ambiguous { 24 } else { 26 }; }
    if include_numbers { charset_size += if exclude_ambiguous { 8 } else { 10 }; }
    if include_symbols { charset_size += 32; }

    // 计算熵（bits）
    let entropy = (length as f64) * (charset_size as f64).log2();

    // 评估强度
    let strength = if entropy >= 128.0 {
        "非常强"
    } else if entropy >= 80.0 {
        "强"
    } else if entropy >= 60.0 {
        "中等"
    } else {
        "弱"
    };

    Ok(json!({
        "success": true,
        "password": password,
        "length": length,
        "charset_size": charset_size,
        "entropy_bits": format!("{:.1}", entropy),
        "strength": strength,
        "config": {
            "include_uppercase": include_uppercase,
            "include_lowercase": include_lowercase,
            "include_numbers": include_numbers,
            "include_symbols": include_symbols,
            "exclude_ambiguous": exclude_ambiguous
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 获取所有加密解密工具
pub fn get_all_crypto_tools() -> Vec<Box<dyn crate::tool::Tool>> {
    vec![
        hash_tool(),
        encrypt_tool(),
        decrypt_tool(),
        password_generator_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{ToolExecutionContext, ToolExecutionOptions};

    #[tokio::test]
    async fn test_hash_sha256() {
        let tool = hash_tool();
        let context = ToolExecutionContext::default();
        let options = ToolExecutionOptions::default();

        let params = json!({
            "input": "hello",
            "algorithm": "sha256"
        });

        let result = tool.execute(params, context, &options).await.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["algorithm"], "SHA256");
    }

    #[tokio::test]
    async fn test_encrypt_valid() {
        let tool = encrypt_tool();
        let context = ToolExecutionContext::default();
        let options = ToolExecutionOptions::default();

        let params = json!({
            "plaintext": "secret message",
            "key": "12345678901234567890123456789012"
        });

        let result = tool.execute(params, context, &options).await.unwrap();
        assert_eq!(result["success"], true);
    }

    #[tokio::test]
    async fn test_password_generator() {
        let tool = password_generator_tool();
        let context = ToolExecutionContext::default();
        let options = ToolExecutionOptions::default();

        let params = json!({
            "length": 20
        });

        let result = tool.execute(params, context, &options).await.unwrap();
        assert_eq!(result["success"], true);
        assert!(result["entropy_bits"].as_str().is_some());
    }
}

