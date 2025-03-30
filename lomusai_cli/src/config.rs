use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{CliError, CliResult};
use dirs::home_dir;

const CONFIG_DIR_NAME: &str = ".lomusai";
const CONFIG_FILE_NAME: &str = "config.toml";

/// CLI工具的配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// 项目配置
    pub project: ProjectConfig,
    
    /// 开发配置
    pub dev: DevConfig,
    
    /// 构建配置
    pub build: BuildConfig,
    
    /// 部署配置
    pub deploy: DeployConfig,
}

/// 项目配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// 默认项目模板
    pub default_template: Option<String>,
    
    /// 默认输出目录
    pub default_output_dir: Option<String>,
}

/// 开发配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DevConfig {
    /// 默认端口
    pub default_port: Option<u16>,
    
    /// 是否启用热重载
    pub hot_reload: Option<bool>,
}

/// 构建配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BuildConfig {
    /// 默认输出目录
    pub default_output_dir: Option<String>,
    
    /// 构建前要运行的脚本
    pub pre_build_script: Option<String>,
    
    /// 构建后要运行的脚本
    pub post_build_script: Option<String>,
}

/// 部署配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeployConfig {
    /// 默认部署目标
    pub default_target: Option<String>,
    
    /// AWS相关配置
    pub aws: Option<AwsConfig>,
    
    /// Azure相关配置
    pub azure: Option<AzureConfig>,
    
    /// GCP相关配置
    pub gcp: Option<GcpConfig>,
    
    /// Docker相关配置
    pub docker: Option<DockerConfig>,
}

/// AWS配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AwsConfig {
    /// 区域
    pub region: Option<String>,
    
    /// 服务类型 (lambda, ec2, ecs)
    pub service: Option<String>,
}

/// Azure配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AzureConfig {
    /// 区域
    pub region: Option<String>,
    
    /// 服务类型 (functions, app-service)
    pub service: Option<String>,
}

/// GCP配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GcpConfig {
    /// 区域
    pub region: Option<String>,
    
    /// 服务类型 (cloud-functions, cloud-run)
    pub service: Option<String>,
}

/// Docker配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DockerConfig {
    /// 基础镜像
    pub base_image: Option<String>,
    
    /// 目标镜像名称
    pub image_name: Option<String>,
    
    /// 镜像标签
    pub tag: Option<String>,
}

impl Config {
    /// 加载配置文件
    pub fn load() -> CliResult<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            return Ok(Config::default());
        }
        
        let content = fs::read_to_string(&config_path)
            .map_err(|e| CliError::Io(e))?;
            
        let config = toml::from_str(&content)
            .map_err(|e| CliError::config(format!("解析配置文件失败: {}", e)))?;
            
        Ok(config)
    }
    
    /// 获取配置文件路径
    pub fn get_config_path() -> CliResult<PathBuf> {
        get_config_file_path()
    }
    
    /// 保存配置文件
    pub fn save(&self) -> CliResult<()> {
        let config_path = get_config_file_path()?;
        
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CliError::Io(e))?;
        }
        
        let content = toml::to_string(self)
            .map_err(|e| CliError::config(format!("序列化配置失败: {}", e)))?;
            
        fs::write(&config_path, content)
            .map_err(|e| CliError::Io(e))?;
            
        Ok(())
    }
    
    /// 更新配置
    pub fn update(&mut self) -> CliResult<()> {
        self.save()
    }
}

/// 获取配置目录路径
pub fn get_config_dir() -> CliResult<PathBuf> {
    let home = home_dir().ok_or_else(|| 
        CliError::config("无法找到用户主目录"))?;
        
    Ok(home.join(CONFIG_DIR_NAME))
}

/// 获取配置文件路径
pub fn get_config_file_path() -> CliResult<PathBuf> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join(CONFIG_FILE_NAME))
}

/// 获取缓存目录路径
pub fn get_cache_dir() -> CliResult<PathBuf> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("cache"))
}

/// 获取模板目录路径
pub fn get_templates_dir() -> CliResult<PathBuf> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("templates"))
}

/// 确保目录存在
pub fn ensure_dir_exists(path: &Path) -> CliResult<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| CliError::Io(e))?;
    }
    Ok(())
} 