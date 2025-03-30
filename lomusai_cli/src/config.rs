use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use crate::error::{CliError, CliResult};

/// 项目配置
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// 项目名称
    pub name: String,
    
    /// 项目版本
    pub version: String,
    
    /// 项目描述
    pub description: Option<String>,
    
    /// 项目作者
    pub authors: Option<Vec<String>>,
    
    /// 项目类型
    pub project_type: String,
    
    /// 开发配置
    #[serde(default)]
    pub dev: DevConfig,
    
    /// 部署配置
    #[serde(default)]
    pub deploy: DeployConfig,
}

/// 开发配置
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DevConfig {
    /// 开发服务器端口
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// 是否启用热重载
    #[serde(default)]
    pub hot_reload: bool,
}

/// 部署配置
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DeployConfig {
    /// 部署目标
    #[serde(default = "default_target")]
    pub target: String,
    
    /// Docker配置
    #[serde(default)]
    pub docker: DockerConfig,
    
    /// AWS配置
    #[serde(default)]
    pub aws: CloudConfig,
    
    /// Azure配置
    #[serde(default)]
    pub azure: CloudConfig,
    
    /// GCP配置
    #[serde(default)]
    pub gcp: CloudConfig,
}

/// Docker配置
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DockerConfig {
    /// 容器名称
    #[serde(default)]
    pub container_name: Option<String>,
    
    /// 镜像名称
    #[serde(default)]
    pub image_name: Option<String>,
    
    /// 端口映射
    #[serde(default)]
    pub port_mapping: Option<String>,
}

/// 云平台配置
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CloudConfig {
    /// 区域
    #[serde(default)]
    pub region: Option<String>,
    
    /// 服务名称
    #[serde(default)]
    pub service_name: Option<String>,
    
    /// 其他配置项
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, String>,
}

// 默认端口号
fn default_port() -> u16 {
    3000
}

// 默认部署目标
fn default_target() -> String {
    "local".to_string()
}

impl ProjectConfig {
    /// 从文件加载项目配置
    pub fn load<P: AsRef<Path>>(path: P) -> CliResult<Self> {
        let config_path = path.as_ref().join("lomusai.toml");
        
        // 检查配置文件是否存在
        if !config_path.exists() {
            return Err(CliError::Other(format!(
                "找不到Lomus配置文件: {}",
                config_path.display()
            )));
        }
        
        // 读取配置文件
        let content = fs::read_to_string(&config_path)
            .map_err(|e| CliError::io_error(e, &config_path))?;
            
        // 解析TOML
        toml::from_str(&content)
            .map_err(|e| CliError::Other(format!("解析配置文件错误: {}", e)))
    }
    
    /// 保存项目配置到文件
    pub fn save<P: AsRef<Path>>(&self, path: P) -> CliResult<()> {
        let config_path = path.as_ref().join("lomusai.toml");
        
        // 序列化为TOML
        let content = toml::to_string_pretty(self)
            .map_err(|e| CliError::Other(format!("序列化配置失败: {}", e)))?;
            
        // 写入文件
        fs::write(&config_path, content)
            .map_err(|e| CliError::io_error(e, &config_path))?;
            
        Ok(())
    }
    
    /// 创建默认配置
    pub fn default(name: &str) -> Self {
        ProjectConfig {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: Some(format!("A Lomus AI {} project", name)),
            authors: None,
            project_type: "agent".to_string(),
            dev: DevConfig::default(),
            deploy: DeployConfig::default(),
        }
    }
    
    /// 获取项目根目录下的路径
    pub fn get_path<P: AsRef<Path>>(&self, project_root: P, relative_path: &str) -> PathBuf {
        project_root.as_ref().join(relative_path)
    }
} 