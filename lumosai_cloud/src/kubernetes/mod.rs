//! Kubernetes集成模块
//! 
//! 提供完整的Kubernetes集成，包括Operator、CRD、Helm Charts等

pub mod operator;
pub mod crd;
pub mod helm;
pub mod resources;

use kube::{Client, Api};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Service, ConfigMap, Secret};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::{DeploymentConfig, DeploymentResult, DeploymentStatus, ResourceInfo, Result, CloudError};
use chrono::Utc;

/// Kubernetes管理器
pub struct KubernetesManager {
    /// Kubernetes客户端
    client: Client,
    
    /// 默认命名空间
    default_namespace: String,
    
    /// Operator控制器
    operator: Option<operator::LumosOperator>,
}

/// Lumos Agent CRD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LumosAgent {
    /// API版本
    pub api_version: String,
    
    /// 资源类型
    pub kind: String,
    
    /// 元数据
    pub metadata: AgentMetadata,
    
    /// 规格
    pub spec: AgentSpec,
    
    /// 状态
    pub status: Option<AgentStatus>,
}

/// Agent元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// 名称
    pub name: String,
    
    /// 命名空间
    pub namespace: String,
    
    /// 标签
    pub labels: HashMap<String, String>,
    
    /// 注解
    pub annotations: HashMap<String, String>,
}

/// Agent规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    /// 副本数
    pub replicas: u32,
    
    /// 模型配置
    pub model: String,
    
    /// 工具列表
    pub tools: Vec<String>,
    
    /// 资源配置
    pub resources: KubernetesResources,
    
    /// 自动扩容配置
    pub autoscaling: Option<AutoscalingSpec>,
    
    /// 服务配置
    pub service: Option<ServiceSpec>,
    
    /// Ingress配置
    pub ingress: Option<IngressSpec>,
}

/// Kubernetes资源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesResources {
    /// 资源请求
    pub requests: ResourceRequests,
    
    /// 资源限制
    pub limits: ResourceLimits,
}

/// 资源请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequests {
    /// CPU请求
    pub cpu: String,
    
    /// 内存请求
    pub memory: String,
    
    /// 存储请求
    pub storage: Option<String>,
}

/// 资源限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU限制
    pub cpu: String,
    
    /// 内存限制
    pub memory: String,
    
    /// GPU限制
    pub gpu: Option<u32>,
}

/// 自动扩容规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoscalingSpec {
    /// 是否启用
    pub enabled: bool,
    
    /// 最小副本数
    pub min_replicas: u32,
    
    /// 最大副本数
    pub max_replicas: u32,
    
    /// 目标CPU使用率
    pub target_cpu_utilization: u32,
    
    /// 目标内存使用率
    pub target_memory_utilization: Option<u32>,
}

/// 服务规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    /// 服务类型
    pub service_type: String,
    
    /// 端口配置
    pub ports: Vec<ServicePort>,
    
    /// 选择器
    pub selector: HashMap<String, String>,
}

/// 服务端口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    /// 端口名称
    pub name: String,
    
    /// 端口号
    pub port: u16,
    
    /// 目标端口
    pub target_port: u16,
    
    /// 协议
    pub protocol: String,
}

/// Ingress规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressSpec {
    /// 主机名
    pub host: String,
    
    /// 路径
    pub path: String,
    
    /// 后端服务
    pub backend_service: String,
    
    /// 后端端口
    pub backend_port: u16,
    
    /// TLS配置
    pub tls: Option<TlsSpec>,
}

/// TLS规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSpec {
    /// 证书名称
    pub secret_name: String,
    
    /// 主机列表
    pub hosts: Vec<String>,
}

/// Agent状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    /// 当前副本数
    pub replicas: u32,
    
    /// 就绪副本数
    pub ready_replicas: u32,
    
    /// 可用副本数
    pub available_replicas: u32,
    
    /// 状态
    pub phase: String,
    
    /// 条件
    pub conditions: Vec<AgentCondition>,
    
    /// 最后更新时间
    pub last_updated: String,
}

/// Agent条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCondition {
    /// 条件类型
    pub condition_type: String,
    
    /// 状态
    pub status: String,
    
    /// 原因
    pub reason: String,
    
    /// 消息
    pub message: String,
    
    /// 最后转换时间
    pub last_transition_time: String,
}

impl KubernetesManager {
    /// 创建新的Kubernetes管理器
    pub async fn new(namespace: Option<String>) -> Result<Self> {
        let client = Client::try_default().await
            .map_err(|e| CloudError::KubernetesConnection(e.to_string()))?;
        
        let default_namespace = namespace.unwrap_or_else(|| "default".to_string());
        
        Ok(Self {
            client,
            default_namespace,
            operator: None,
        })
    }
    
    /// 启动Operator
    pub async fn start_operator(&mut self) -> Result<()> {
        let operator = operator::LumosOperator::new(self.client.clone()).await?;
        self.operator = Some(operator);
        
        if let Some(op) = &mut self.operator {
            op.start().await?;
        }
        
        Ok(())
    }
    
    /// 部署Agent
    pub async fn deploy_agent(&self, config: DeploymentConfig) -> Result<DeploymentResult> {
        let namespace = if let crate::TargetEnvironment::Kubernetes { namespace, .. } = &config.target_environment {
            namespace.clone()
        } else {
            self.default_namespace.clone()
        };
        
        // 创建LumosAgent CRD实例
        let agent_crd = self.create_agent_crd(&config, &namespace)?;
        
        // 应用CRD到集群
        let agent_api: Api<LumosAgent> = Api::namespaced(self.client.clone(), &namespace);
        
        let result = agent_api.create(&Default::default(), &agent_crd).await
            .map_err(|e| CloudError::KubernetesDeployment(e.to_string()))?;
        
        // 等待部署完成
        self.wait_for_deployment(&namespace, &config.name).await?;
        
        // 获取服务端点
        let endpoints = self.get_service_endpoints(&namespace, &config.name).await?;
        
        Ok(DeploymentResult {
            deployment_id: result.metadata.name.unwrap_or_default(),
            status: DeploymentStatus::Succeeded,
            deployed_at: Utc::now(),
            endpoints,
            resources: vec![
                ResourceInfo {
                    resource_type: "LumosAgent".to_string(),
                    name: config.name.clone(),
                    status: "Running".to_string(),
                    created_at: Utc::now(),
                }
            ],
            logs: vec!["Agent deployed successfully".to_string()],
        })
    }
    
    /// 创建Agent CRD
    fn create_agent_crd(&self, config: &DeploymentConfig, namespace: &str) -> Result<LumosAgent> {
        let mut labels = HashMap::new();
        labels.insert("app".to_string(), "lumos-agent".to_string());
        labels.insert("version".to_string(), "v1".to_string());
        
        let mut annotations = HashMap::new();
        annotations.insert("lumos.ai/managed-by".to_string(), "lumos-operator".to_string());
        
        Ok(LumosAgent {
            api_version: "lumosai.io/v1".to_string(),
            kind: "Agent".to_string(),
            metadata: AgentMetadata {
                name: config.name.clone(),
                namespace: namespace.to_string(),
                labels,
                annotations,
            },
            spec: AgentSpec {
                replicas: config.agent_config.replicas,
                model: config.agent_config.models.first()
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| "default".to_string()),
                tools: config.agent_config.tools.iter()
                    .map(|t| t.name.clone())
                    .collect(),
                resources: KubernetesResources {
                    requests: ResourceRequests {
                        cpu: config.resources.cpu_request.clone(),
                        memory: config.resources.memory_request.clone(),
                        storage: config.resources.storage_request.clone(),
                    },
                    limits: ResourceLimits {
                        cpu: config.resources.cpu_limit.clone(),
                        memory: config.resources.memory_limit.clone(),
                        gpu: config.resources.gpu.as_ref().map(|g| g.count),
                    },
                },
                autoscaling: if config.autoscaling.enabled {
                    Some(AutoscalingSpec {
                        enabled: true,
                        min_replicas: config.autoscaling.min_replicas,
                        max_replicas: config.autoscaling.max_replicas,
                        target_cpu_utilization: config.autoscaling.target_cpu_utilization,
                        target_memory_utilization: config.autoscaling.target_memory_utilization,
                    })
                } else {
                    None
                },
                service: Some(ServiceSpec {
                    service_type: match config.networking.service_type {
                        crate::ServiceType::ClusterIP => "ClusterIP".to_string(),
                        crate::ServiceType::NodePort => "NodePort".to_string(),
                        crate::ServiceType::LoadBalancer => "LoadBalancer".to_string(),
                        crate::ServiceType::ExternalName => "ExternalName".to_string(),
                    },
                    ports: config.networking.ports.iter().map(|p| ServicePort {
                        name: p.name.clone(),
                        port: p.port,
                        target_port: p.target_port,
                        protocol: p.protocol.clone(),
                    }).collect(),
                    selector: {
                        let mut selector = HashMap::new();
                        selector.insert("app".to_string(), "lumos-agent".to_string());
                        selector.insert("instance".to_string(), config.name.clone());
                        selector
                    },
                }),
                ingress: config.networking.ingress.as_ref().map(|ingress| {
                    IngressSpec {
                        host: ingress.host.clone(),
                        path: ingress.paths.first()
                            .map(|p| p.path.clone())
                            .unwrap_or_else(|| "/".to_string()),
                        backend_service: config.name.clone(),
                        backend_port: ingress.paths.first()
                            .map(|p| p.backend_port)
                            .unwrap_or(80),
                        tls: ingress.tls.as_ref().map(|tls| TlsSpec {
                            secret_name: tls.secret_name.clone(),
                            hosts: tls.hosts.clone(),
                        }),
                    }
                }),
            },
            status: None,
        })
    }
    
    /// 等待部署完成
    async fn wait_for_deployment(&self, namespace: &str, name: &str) -> Result<()> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        
        // 简化的等待逻辑，实际应该使用watch API
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        
        match deployments.get(name).await {
            Ok(_) => Ok(()),
            Err(e) => Err(CloudError::KubernetesDeployment(e.to_string())),
        }
    }
    
    /// 获取服务端点
    async fn get_service_endpoints(&self, namespace: &str, name: &str) -> Result<Vec<String>> {
        let services: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        
        match services.get(name).await {
            Ok(service) => {
                let mut endpoints = Vec::new();
                
                if let Some(spec) = service.spec {
                    if let Some(ports) = spec.ports {
                        for port in ports {
                            if let Some(port_num) = port.port {
                                endpoints.push(format!("http://{}:{}", name, port_num));
                            }
                        }
                    }
                }
                
                Ok(endpoints)
            }
            Err(_) => Ok(vec![]),
        }
    }
    
    /// 删除Agent
    pub async fn delete_agent(&self, namespace: &str, name: &str) -> Result<()> {
        let agent_api: Api<LumosAgent> = Api::namespaced(self.client.clone(), namespace);
        
        agent_api.delete(name, &Default::default()).await
            .map_err(|e| CloudError::KubernetesDeployment(e.to_string()))?;
        
        Ok(())
    }
    
    /// 列出所有Agent
    pub async fn list_agents(&self, namespace: Option<&str>) -> Result<Vec<LumosAgent>> {
        let ns = namespace.unwrap_or(&self.default_namespace);
        let agent_api: Api<LumosAgent> = Api::namespaced(self.client.clone(), ns);
        
        let agents = agent_api.list(&Default::default()).await
            .map_err(|e| CloudError::KubernetesConnection(e.to_string()))?;
        
        Ok(agents.items)
    }
    
    /// 获取Agent状态
    pub async fn get_agent_status(&self, namespace: &str, name: &str) -> Result<Option<AgentStatus>> {
        let agent_api: Api<LumosAgent> = Api::namespaced(self.client.clone(), namespace);
        
        match agent_api.get(name).await {
            Ok(agent) => Ok(agent.status),
            Err(_) => Ok(None),
        }
    }
}
