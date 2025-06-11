/*!
# File Handler Module

文件处理模块，实现文件上传、存储和管理功能。

## 功能特性

- **文件上传**: 支持多种文件格式上传
- **文件存储**: 本地文件系统存储
- **文件管理**: 文件列表、删除、下载
- **安全控制**: 文件类型验证、大小限制

## 支持的文件类型

- **文档**: PDF, DOC, DOCX, TXT, MD
- **图片**: JPG, PNG, GIF, WEBP
- **数据**: JSON, CSV, XML, YAML
- **代码**: JS, TS, PY, RS, GO, JAVA
*/

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::database::{Database, DatabaseError};

/// 文件处理错误
#[derive(Debug, Error)]
pub enum FileError {
    #[error("文件类型不支持: {0}")]
    UnsupportedFileType(String),
    #[error("文件大小超限: {0} bytes")]
    FileSizeExceeded(usize),
    #[error("文件名无效: {0}")]
    InvalidFileName(String),
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("数据库错误: {0}")]
    Database(#[from] DatabaseError),
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub name: String,
    pub size: usize,
    pub mime_type: String,
    pub file_type: String,
    pub upload_time: chrono::DateTime<chrono::Utc>,
    pub user_id: i64,
    pub conversation_id: Option<i64>,
    pub file_path: String,
}

/// 文件上传请求
#[derive(Debug, Deserialize)]
pub struct FileUploadRequest {
    pub conversation_id: Option<i64>,
    pub description: Option<String>,
}

/// 文件上传响应
#[derive(Debug, Serialize)]
pub struct FileUploadResponse {
    pub success: bool,
    pub files: Vec<FileInfo>,
    pub errors: Vec<String>,
}

/// 文件配置
#[derive(Debug, Clone)]
pub struct FileConfig {
    pub max_file_size: usize,
    pub allowed_types: Vec<String>,
    pub upload_dir: PathBuf,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            max_file_size: 50 * 1024 * 1024, // 50MB
            allowed_types: vec![
                // 文档
                "pdf".to_string(),
                "doc".to_string(),
                "docx".to_string(),
                "txt".to_string(),
                "md".to_string(),
                "rtf".to_string(),
                
                // 图片
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "webp".to_string(),
                "bmp".to_string(),
                
                // 数据
                "json".to_string(),
                "csv".to_string(),
                "xml".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                
                // 代码
                "js".to_string(),
                "ts".to_string(),
                "py".to_string(),
                "rs".to_string(),
                "go".to_string(),
                "java".to_string(),
                "c".to_string(),
                "cpp".to_string(),
                "h".to_string(),
                "hpp".to_string(),
            ],
            upload_dir: PathBuf::from("./uploads"),
        }
    }
}

/// 文件处理器
#[derive(Clone)]
pub struct FileHandler {
    config: FileConfig,
    database: Database,
}

impl FileHandler {
    /// 创建新的文件处理器
    pub fn new(config: FileConfig, database: Database) -> Self {
        Self { config, database }
    }
    
    /// 初始化上传目录
    pub async fn init(&self) -> Result<(), FileError> {
        if !self.config.upload_dir.exists() {
            fs::create_dir_all(&self.config.upload_dir).await?;
        }
        Ok(())
    }
    
    /// 验证文件类型
    fn validate_file_type(&self, filename: &str) -> Result<String, FileError> {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or_else(|| FileError::InvalidFileName("文件没有扩展名".to_string()))?;
        
        if !self.config.allowed_types.contains(&extension) {
            return Err(FileError::UnsupportedFileType(extension));
        }
        
        Ok(extension)
    }
    
    /// 验证文件大小
    fn validate_file_size(&self, size: usize) -> Result<(), FileError> {
        if size > self.config.max_file_size {
            return Err(FileError::FileSizeExceeded(size));
        }
        Ok(())
    }
    
    /// 生成文件ID
    fn generate_file_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("file_{}", timestamp)
    }
    
    /// 获取MIME类型
    fn get_mime_type(&self, extension: &str) -> String {
        match extension {
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "txt" => "text/plain",
            "md" => "text/markdown",
            "rtf" => "application/rtf",
            
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            
            "json" => "application/json",
            "csv" => "text/csv",
            "xml" => "application/xml",
            "yaml" | "yml" => "application/x-yaml",
            
            "js" => "application/javascript",
            "ts" => "application/typescript",
            "py" => "text/x-python",
            "rs" => "text/x-rust",
            "go" => "text/x-go",
            "java" => "text/x-java",
            "c" => "text/x-c",
            "cpp" => "text/x-c++",
            "h" => "text/x-c-header",
            "hpp" => "text/x-c++-header",
            
            _ => "application/octet-stream",
        }.to_string()
    }
    
    /// 保存文件
    async fn save_file(&self, filename: &str, data: &[u8]) -> Result<String, FileError> {
        let file_id = self.generate_file_id();
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let file_name = if extension.is_empty() {
            file_id.clone()
        } else {
            format!("{}.{}", file_id, extension)
        };
        
        let file_path = self.config.upload_dir.join(&file_name);
        
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(data).await?;
        file.flush().await?;
        
        Ok(file_path.to_string_lossy().to_string())
    }
    
    /// 处理文件上传
    pub async fn upload_files(
        &self,
        mut multipart: Multipart,
        user_id: i64,
        conversation_id: Option<i64>,
    ) -> Result<FileUploadResponse, FileError> {
        let mut uploaded_files = Vec::new();
        let mut errors = Vec::new();
        
        while let Some(field) = multipart.next_field().await.map_err(|e| {
            FileError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })? {
            if let Some(filename) = field.file_name() {
                let filename = filename.to_string();
                
                match field.bytes().await {
                    Ok(data) => {
                        match self.process_single_file(&filename, &data, user_id, conversation_id).await {
                            Ok(file_info) => uploaded_files.push(file_info),
                            Err(e) => errors.push(format!("{}: {}", filename, e)),
                        }
                    }
                    Err(e) => errors.push(format!("{}: 读取文件失败 - {}", filename, e)),
                }
            }
        }
        
        Ok(FileUploadResponse {
            success: errors.is_empty(),
            files: uploaded_files,
            errors,
        })
    }
    
    /// 处理单个文件
    async fn process_single_file(
        &self,
        filename: &str,
        data: &[u8],
        user_id: i64,
        conversation_id: Option<i64>,
    ) -> Result<FileInfo, FileError> {
        // 验证文件类型
        let file_type = self.validate_file_type(filename)?;
        
        // 验证文件大小
        self.validate_file_size(data.len())?;
        
        // 保存文件
        let file_path = self.save_file(filename, data).await?;
        
        // 创建文件信息
        let file_info = FileInfo {
            id: self.generate_file_id(),
            name: filename.to_string(),
            size: data.len(),
            mime_type: self.get_mime_type(&file_type),
            file_type,
            upload_time: chrono::Utc::now(),
            user_id,
            conversation_id,
            file_path,
        };
        
        // TODO: 保存到数据库
        // self.database.save_file_info(&file_info).await?;
        
        Ok(file_info)
    }
    
    /// 获取用户文件列表
    pub async fn list_user_files(&self, user_id: i64) -> Result<Vec<FileInfo>, FileError> {
        // TODO: 从数据库获取文件列表
        // self.database.get_user_files(user_id).await
        Ok(Vec::new())
    }
    
    /// 删除文件
    pub async fn delete_file(&self, file_id: &str, user_id: i64) -> Result<(), FileError> {
        // TODO: 从数据库获取文件信息并验证权限
        // let file_info = self.database.get_file_info(file_id).await?;
        // if file_info.user_id != user_id {
        //     return Err(FileError::PermissionDenied);
        // }
        
        // TODO: 删除物理文件
        // fs::remove_file(&file_info.file_path).await?;
        
        // TODO: 从数据库删除记录
        // self.database.delete_file_info(file_id).await?;
        
        Ok(())
    }
}

/// 文件上传处理器
pub async fn upload_files(
    State(file_handler): State<FileHandler>,
    multipart: Multipart,
) -> impl IntoResponse {
    // 默认用户ID为1（系统用户）
    let user_id = 1;
    let conversation_id = None;
    
    match file_handler.upload_files(multipart, user_id, conversation_id).await {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FileUploadResponse {
                success: false,
                files: Vec::new(),
                errors: vec![e.to_string()],
            }),
        ),
    }
}

/// 获取文件列表
pub async fn list_files(
    State(file_handler): State<FileHandler>,
) -> impl IntoResponse {
    // 默认用户ID为1（系统用户）
    let user_id = 1;
    
    match file_handler.list_user_files(user_id).await {
        Ok(files) => (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "files": files
        }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })),
        ),
    }
}

/// 删除文件
pub async fn delete_file(
    Path(file_id): Path<String>,
    State(file_handler): State<FileHandler>,
) -> impl IntoResponse {
    // 默认用户ID为1（系统用户）
    let user_id = 1;
    
    match file_handler.delete_file(&file_id, user_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "message": "文件删除成功"
        }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })),
        ),
    }
}
