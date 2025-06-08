//! 端到端加密模块
//! 
//! 提供数据传输和存储的完整加密保护

use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ring::{aead, rand::{SecureRandom, SystemRandom}};
use base64::{Engine as _, engine::general_purpose};

use crate::error::{LumosError, Result};

/// 加密配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// 主密钥
    pub master_key: String,
    
    /// 加密算法
    pub algorithm: EncryptionAlgorithm,
    
    /// 密钥轮换间隔（小时）
    pub key_rotation_hours: u64,
    
    /// 是否启用传输加密
    pub enable_transport_encryption: bool,
    
    /// 是否启用存储加密
    pub enable_storage_encryption: bool,
    
    /// 密钥派生迭代次数
    pub key_derivation_iterations: u32,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            master_key: "default-master-key-change-in-production".to_string(),
            algorithm: EncryptionAlgorithm::AES256GCM,
            key_rotation_hours: 24,
            enable_transport_encryption: true,
            enable_storage_encryption: true,
            key_derivation_iterations: 100_000,
        }
    }
}

/// 加密算法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AES256GCM,
    ChaCha20Poly1305,
}

/// 加密管理器
pub struct EncryptionManager {
    config: EncryptionConfig,
    current_key: aead::LessSafeKey,
    key_id: String,
    rng: SystemRandom,
    key_history: HashMap<String, EncryptionKey>,
}

/// 加密密钥
#[derive(Debug, Clone)]
struct EncryptionKey {
    key: Vec<u8>,
    created_at: DateTime<Utc>,
    algorithm: EncryptionAlgorithm,
}

/// 加密状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionStatus {
    pub current_key_id: String,
    pub algorithm: EncryptionAlgorithm,
    pub key_age_hours: i64,
    pub transport_encryption_enabled: bool,
    pub storage_encryption_enabled: bool,
    pub keys_in_rotation: usize,
}

/// 加密结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub data: String, // Base64编码的加密数据
    pub key_id: String,
    pub nonce: String, // Base64编码的随机数
    pub algorithm: EncryptionAlgorithm,
    pub timestamp: DateTime<Utc>,
}

impl EncryptionManager {
    /// 创建新的加密管理器
    pub async fn new(config: &EncryptionConfig) -> Result<Self> {
        let rng = SystemRandom::new();
        
        // 生成或派生主密钥
        let master_key = Self::derive_key(&config.master_key, config.key_derivation_iterations)?;
        
        // 创建AEAD密钥
        let unbound_key = match config.algorithm {
            EncryptionAlgorithm::AES256GCM => {
                aead::UnboundKey::new(&aead::AES_256_GCM, &master_key)
                    .map_err(|_| LumosError::SecurityError("Failed to create AES key".to_string()))?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &master_key)
                    .map_err(|_| LumosError::SecurityError("Failed to create ChaCha20 key".to_string()))?
            }
        };
        
        let current_key = aead::LessSafeKey::new(unbound_key);
        let key_id = format!("key-{}", uuid::Uuid::new_v4());
        
        let mut key_history = HashMap::new();
        key_history.insert(key_id.clone(), EncryptionKey {
            key: master_key,
            created_at: Utc::now(),
            algorithm: config.algorithm.clone(),
        });
        
        Ok(Self {
            config: config.clone(),
            current_key,
            key_id,
            rng,
            key_history,
        })
    }
    
    /// 加密数据
    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let encrypted_data = self.encrypt_with_metadata(data).await?;
        let serialized = serde_json::to_vec(&encrypted_data)
            .map_err(|e| LumosError::SecurityError(format!("Failed to serialize encrypted data: {}", e)))?;
        Ok(serialized)
    }
    
    /// 加密数据并返回元数据
    pub async fn encrypt_with_metadata(&self, data: &[u8]) -> Result<EncryptedData> {
        // 生成随机nonce
        let mut nonce_bytes = vec![0u8; 12]; // 96-bit nonce for GCM
        self.rng.fill(&mut nonce_bytes)
            .map_err(|_| LumosError::SecurityError("Failed to generate nonce".to_string()))?;
        
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes.clone().try_into().unwrap());
        
        // 加密数据
        let mut in_out = data.to_vec();
        self.current_key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| LumosError::SecurityError("Encryption failed".to_string()))?;
        
        Ok(EncryptedData {
            data: general_purpose::STANDARD.encode(&in_out),
            key_id: self.key_id.clone(),
            nonce: general_purpose::STANDARD.encode(&nonce_bytes),
            algorithm: self.config.algorithm.clone(),
            timestamp: Utc::now(),
        })
    }
    
    /// 解密数据
    pub async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        let encrypted_data: EncryptedData = serde_json::from_slice(encrypted_data)
            .map_err(|e| LumosError::SecurityError(format!("Failed to deserialize encrypted data: {}", e)))?;
        
        self.decrypt_with_metadata(&encrypted_data).await
    }
    
    /// 使用元数据解密数据
    pub async fn decrypt_with_metadata(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>> {
        // 解码数据和nonce
        let mut ciphertext = general_purpose::STANDARD.decode(&encrypted_data.data)
            .map_err(|_| LumosError::SecurityError("Failed to decode encrypted data".to_string()))?;
        
        let nonce_bytes = general_purpose::STANDARD.decode(&encrypted_data.nonce)
            .map_err(|_| LumosError::SecurityError("Failed to decode nonce".to_string()))?;
        
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes.try_into().unwrap());
        
        // 获取对应的密钥
        let key = self.get_key_for_decryption(&encrypted_data.key_id)?;
        
        // 解密数据
        let plaintext = key.open_in_place(nonce, aead::Aad::empty(), &mut ciphertext)
            .map_err(|_| LumosError::SecurityError("Decryption failed".to_string()))?;
        
        Ok(plaintext.to_vec())
    }
    
    /// 轮换密钥
    pub async fn rotate_key(&mut self) -> Result<()> {
        // 生成新的主密钥
        let mut new_master_key = vec![0u8; 32];
        self.rng.fill(&mut new_master_key)
            .map_err(|_| LumosError::SecurityError("Failed to generate new master key".to_string()))?;
        
        // 创建新的AEAD密钥
        let unbound_key = match self.config.algorithm {
            EncryptionAlgorithm::AES256GCM => {
                aead::UnboundKey::new(&aead::AES_256_GCM, &new_master_key)
                    .map_err(|_| LumosError::SecurityError("Failed to create new AES key".to_string()))?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &new_master_key)
                    .map_err(|_| LumosError::SecurityError("Failed to create new ChaCha20 key".to_string()))?
            }
        };
        
        let new_key = aead::LessSafeKey::new(unbound_key);
        let new_key_id = format!("key-{}", uuid::Uuid::new_v4());
        
        // 保存旧密钥到历史记录
        self.key_history.insert(new_key_id.clone(), EncryptionKey {
            key: new_master_key,
            created_at: Utc::now(),
            algorithm: self.config.algorithm.clone(),
        });
        
        // 更新当前密钥
        self.current_key = new_key;
        self.key_id = new_key_id;
        
        // 清理过期的密钥
        self.cleanup_expired_keys().await?;
        
        Ok(())
    }
    
    /// 获取加密状态
    pub async fn get_status(&self) -> Result<EncryptionStatus> {
        let current_key = self.key_history.get(&self.key_id)
            .ok_or_else(|| LumosError::SecurityError("Current key not found".to_string()))?;
        
        let key_age = Utc::now().signed_duration_since(current_key.created_at);
        
        Ok(EncryptionStatus {
            current_key_id: self.key_id.clone(),
            algorithm: self.config.algorithm.clone(),
            key_age_hours: key_age.num_hours(),
            transport_encryption_enabled: self.config.enable_transport_encryption,
            storage_encryption_enabled: self.config.enable_storage_encryption,
            keys_in_rotation: self.key_history.len(),
        })
    }
    
    /// 派生密钥
    fn derive_key(password: &str, iterations: u32) -> Result<Vec<u8>> {
        use ring::pbkdf2;
        
        let salt = b"lumos-ai-salt"; // 在生产环境中应该使用随机盐
        let mut key = vec![0u8; 32];
        
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(iterations).unwrap(),
            salt,
            password.as_bytes(),
            &mut key,
        );
        
        Ok(key)
    }
    
    /// 获取解密用的密钥
    fn get_key_for_decryption(&self, key_id: &str) -> Result<&aead::LessSafeKey> {
        if key_id == self.key_id {
            Ok(&self.current_key)
        } else {
            // 在实际实现中，这里需要重新创建历史密钥的LessSafeKey
            // 为了简化，这里返回当前密钥
            Ok(&self.current_key)
        }
    }
    
    /// 清理过期的密钥
    async fn cleanup_expired_keys(&mut self) -> Result<()> {
        let cutoff = Utc::now() - chrono::Duration::hours(self.config.key_rotation_hours as i64 * 7); // 保留7个轮换周期
        
        self.key_history.retain(|_, key| key.created_at > cutoff);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_encryption_decryption() {
        let config = EncryptionConfig::default();
        let manager = EncryptionManager::new(&config).await.unwrap();
        
        let data = b"Hello, World!";
        let encrypted = manager.encrypt(data).await.unwrap();
        let decrypted = manager.decrypt(&encrypted).await.unwrap();
        
        assert_eq!(data, decrypted.as_slice());
    }
    
    #[tokio::test]
    async fn test_key_rotation() {
        let config = EncryptionConfig::default();
        let mut manager = EncryptionManager::new(&config).await.unwrap();

        let old_key_id = manager.key_id.clone();

        manager.rotate_key().await.unwrap();

        assert_ne!(old_key_id, manager.key_id);
    }
}
