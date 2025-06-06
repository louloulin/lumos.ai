//! Docker集成模块
//! 
//! 提供Docker容器化和部署功能

use bollard::{Docker, API_DEFAULT_VERSION};
use bollard::container::{CreateContainerOptions, Config, StartContainerOptions};
use bollard::image::CreateImageOptions;
use bollard::models::{ContainerCreateResponse, HostConfig, PortBinding};
use std::collections::HashMap;
use futures::stream::StreamExt;
use crate::{DeploymentConfig, DeploymentResult, DeploymentStatus, ResourceInfo, Result, CloudError};
use chrono::Utc;

/// Docker管理器
pub struct DockerManager {
    /// Docker客户端
    client: Docker,
}

impl DockerManager {
    /// 创建新的Docker管理器
    pub async fn new() -> Result<Self> {
        let client = Docker::connect_with_socket_defaults()
            .map_err(|e| CloudError::DockerConnection(e.to_string()))?;
        
        // 测试连接
        client.ping().await
            .map_err(|e| CloudError::DockerConnection(e.to_string()))?;
        
        Ok(Self { client })
    }
    
    /// 部署Agent到Docker
    pub async fn deploy_agent(&self, config: DeploymentConfig) -> Result<DeploymentResult> {
        let image_name = format!("{}:{}", config.agent_config.image, config.agent_config.tag);
        
        // 拉取镜像
        self.pull_image(&image_name).await?;
        
        // 创建容器
        let container_id = self.create_container(&config, &image_name).await?;
        
        // 启动容器
        self.start_container(&container_id).await?;
        
        // 获取容器信息
        let container_info = self.client.inspect_container(&container_id, None).await
            .map_err(|e| CloudError::DockerDeployment(e.to_string()))?;
        
        // 构建端点列表
        let mut endpoints = Vec::new();
        if let Some(network_settings) = container_info.network_settings {
            if let Some(ports) = network_settings.ports {
                for (container_port, host_ports) in ports {
                    if let Some(host_ports) = host_ports {
                        for host_port in host_ports {
                            if let Some(host_port_num) = host_port.host_port {
                                endpoints.push(format!("http://localhost:{}", host_port_num));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(DeploymentResult {
            deployment_id: container_id.clone(),
            status: DeploymentStatus::Running,
            deployed_at: Utc::now(),
            endpoints,
            resources: vec![
                ResourceInfo {
                    resource_type: "Container".to_string(),
                    name: container_id,
                    status: "Running".to_string(),
                    created_at: Utc::now(),
                }
            ],
            logs: vec!["Container deployed successfully".to_string()],
        })
    }
    
    /// 拉取镜像
    async fn pull_image(&self, image_name: &str) -> Result<()> {
        let options = Some(CreateImageOptions {
            from_image: image_name,
            ..Default::default()
        });
        
        let mut stream = self.client.create_image(options, None, None);
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(_) => continue,
                Err(e) => return Err(CloudError::DockerDeployment(e.to_string())),
            }
        }
        
        Ok(())
    }
    
    /// 创建容器
    async fn create_container(&self, config: &DeploymentConfig, image_name: &str) -> Result<String> {
        let mut env_vars = Vec::new();
        for (key, value) in &config.agent_config.environment {
            env_vars.push(format!("{}={}", key, value));
        }
        
        // 端口映射
        let mut port_bindings = HashMap::new();
        let mut exposed_ports = HashMap::new();
        
        for port_config in &config.networking.ports {
            let container_port = format!("{}/tcp", port_config.target_port);
            let host_port = port_config.port.to_string();
            
            exposed_ports.insert(container_port.clone(), HashMap::new());
            port_bindings.insert(
                container_port,
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(host_port),
                }]),
            );
        }
        
        let container_config = Config {
            image: Some(image_name.to_string()),
            env: Some(env_vars),
            exposed_ports: Some(exposed_ports),
            host_config: Some(HostConfig {
                port_bindings: Some(port_bindings),
                memory: Some(self.parse_memory(&config.resources.memory_limit)?),
                nano_cpus: Some(self.parse_cpu(&config.resources.cpu_limit)?),
                ..Default::default()
            }),
            ..Default::default()
        };
        
        let options = CreateContainerOptions {
            name: config.name.clone(),
            platform: None,
        };
        
        let response: ContainerCreateResponse = self.client
            .create_container(Some(options), container_config)
            .await
            .map_err(|e| CloudError::DockerDeployment(e.to_string()))?;
        
        Ok(response.id)
    }
    
    /// 启动容器
    async fn start_container(&self, container_id: &str) -> Result<()> {
        self.client
            .start_container(container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| CloudError::DockerDeployment(e.to_string()))?;
        
        Ok(())
    }
    
    /// 解析内存配置
    fn parse_memory(&self, memory_str: &str) -> Result<i64> {
        let memory_str = memory_str.to_lowercase();
        
        if memory_str.ends_with("gi") {
            let value = memory_str.trim_end_matches("gi").parse::<f64>()
                .map_err(|_| CloudError::Configuration("Invalid memory format".to_string()))?;
            Ok((value * 1024.0 * 1024.0 * 1024.0) as i64)
        } else if memory_str.ends_with("mi") {
            let value = memory_str.trim_end_matches("mi").parse::<f64>()
                .map_err(|_| CloudError::Configuration("Invalid memory format".to_string()))?;
            Ok((value * 1024.0 * 1024.0) as i64)
        } else {
            Err(CloudError::Configuration("Unsupported memory format".to_string()))
        }
    }
    
    /// 解析CPU配置
    fn parse_cpu(&self, cpu_str: &str) -> Result<i64> {
        let cpu_str = cpu_str.to_lowercase();
        
        if cpu_str.ends_with("m") {
            let value = cpu_str.trim_end_matches("m").parse::<f64>()
                .map_err(|_| CloudError::Configuration("Invalid CPU format".to_string()))?;
            Ok((value * 1_000_000.0) as i64) // 转换为纳秒
        } else {
            let value = cpu_str.parse::<f64>()
                .map_err(|_| CloudError::Configuration("Invalid CPU format".to_string()))?;
            Ok((value * 1_000_000_000.0) as i64) // 转换为纳秒
        }
    }
    
    /// 停止容器
    pub async fn stop_container(&self, container_id: &str) -> Result<()> {
        self.client
            .stop_container(container_id, None)
            .await
            .map_err(|e| CloudError::DockerDeployment(e.to_string()))?;
        
        Ok(())
    }
    
    /// 删除容器
    pub async fn remove_container(&self, container_id: &str) -> Result<()> {
        self.client
            .remove_container(container_id, None)
            .await
            .map_err(|e| CloudError::DockerDeployment(e.to_string()))?;
        
        Ok(())
    }
    
    /// 获取容器日志
    pub async fn get_container_logs(&self, container_id: &str) -> Result<Vec<String>> {
        use bollard::container::LogsOptions;
        
        let options = Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: "100".to_string(),
            ..Default::default()
        });
        
        let mut stream = self.client.logs(container_id, options);
        let mut logs = Vec::new();
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(log_output) => {
                    logs.push(log_output.to_string());
                }
                Err(e) => return Err(CloudError::DockerDeployment(e.to_string())),
            }
        }
        
        Ok(logs)
    }
}
