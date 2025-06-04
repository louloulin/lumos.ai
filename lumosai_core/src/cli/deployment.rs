//! Deployment utilities for Lumos.ai projects
//! 
//! This module provides deployment capabilities for various platforms

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::Result;
use super::{ProjectConfig, CliUtils};

/// Deployment platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentPlatform {
    Local,
    Docker,
    Kubernetes,
    AWS,
    GCP,
    Azure,
    Vercel,
    Netlify,
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub platform: DeploymentPlatform,
    pub environment: HashMap<String, String>,
    pub build_command: Option<String>,
    pub start_command: Option<String>,
    pub health_check: Option<HealthCheck>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub path: String,
    pub interval: u32,
    pub timeout: u32,
    pub retries: u32,
}

/// Deployment manager
pub struct DeploymentManager {
    project_root: PathBuf,
    project_config: ProjectConfig,
}

impl DeploymentManager {
    /// Create a new deployment manager
    pub fn new(project_root: PathBuf, project_config: ProjectConfig) -> Self {
        Self {
            project_root,
            project_config,
        }
    }

    /// Deploy to a platform
    pub async fn deploy(&self, platform: &str, config_path: Option<&Path>) -> Result<()> {
        match platform {
            "local" => self.deploy_local().await,
            "docker" => self.deploy_docker().await,
            "kubernetes" => self.deploy_kubernetes().await,
            _ => Err(crate::Error::Other(format!("Unsupported platform: {}", platform))),
        }
    }

    /// Deploy locally
    async fn deploy_local(&self) -> Result<()> {
        CliUtils::progress("Deploying locally...");
        
        // Build the project
        CliUtils::execute_command("cargo", &["build", "--release"], Some(&self.project_root)).await?;
        
        // Copy binary to local deployment directory
        let target_dir = self.project_root.join("target/release");
        let deploy_dir = self.project_root.join("deploy");
        std::fs::create_dir_all(&deploy_dir)?;
        
        // Find the binary
        let binary_name = &self.project_config.name;
        let binary_path = target_dir.join(binary_name);
        let deploy_path = deploy_dir.join(binary_name);
        
        std::fs::copy(&binary_path, &deploy_path)?;
        
        CliUtils::success("Local deployment completed");
        CliUtils::info(&format!("Binary: {}", deploy_path.display()));
        
        Ok(())
    }

    /// Deploy with Docker
    async fn deploy_docker(&self) -> Result<()> {
        CliUtils::progress("Building Docker image...");
        
        // Generate Dockerfile if it doesn't exist
        let dockerfile_path = self.project_root.join("Dockerfile");
        if !dockerfile_path.exists() {
            self.generate_dockerfile(&dockerfile_path)?;
        }
        
        // Build Docker image
        let image_name = format!("{}:latest", self.project_config.name);
        CliUtils::execute_command("docker", &["build", "-t", &image_name, "."], Some(&self.project_root)).await?;
        
        CliUtils::success(&format!("Docker image built: {}", image_name));
        
        Ok(())
    }

    /// Deploy to Kubernetes
    async fn deploy_kubernetes(&self) -> Result<()> {
        CliUtils::progress("Deploying to Kubernetes...");
        
        // Generate Kubernetes manifests if they don't exist
        let k8s_dir = self.project_root.join("k8s");
        if !k8s_dir.exists() {
            std::fs::create_dir_all(&k8s_dir)?;
            self.generate_k8s_manifests(&k8s_dir)?;
        }
        
        // Apply manifests
        CliUtils::execute_command("kubectl", &["apply", "-f", "k8s/"], Some(&self.project_root)).await?;
        
        CliUtils::success("Kubernetes deployment completed");
        
        Ok(())
    }

    /// Generate Dockerfile
    fn generate_dockerfile(&self, path: &Path) -> Result<()> {
        let dockerfile_content = format!(r#"# Lumos.ai Agent Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/{} /app/{}

EXPOSE 3000
CMD ["./{}"]
"#, self.project_config.name, self.project_config.name, self.project_config.name);
        
        std::fs::write(path, dockerfile_content)?;
        CliUtils::info("Generated Dockerfile");
        
        Ok(())
    }

    /// Generate Kubernetes manifests
    fn generate_k8s_manifests(&self, dir: &Path) -> Result<()> {
        // Deployment manifest
        let deployment_yaml = format!(r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {name}
  labels:
    app: {name}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: {name}
  template:
    metadata:
      labels:
        app: {name}
    spec:
      containers:
      - name: {name}
        image: {name}:latest
        ports:
        - containerPort: 3000
        env:
        - name: RUST_LOG
          value: "info"
---
apiVersion: v1
kind: Service
metadata:
  name: {name}-service
spec:
  selector:
    app: {name}
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
"#, name = self.project_config.name);
        
        std::fs::write(dir.join("deployment.yaml"), deployment_yaml)?;
        CliUtils::info("Generated Kubernetes manifests");
        
        Ok(())
    }
}

/// Deployment utilities
pub struct DeploymentUtils;

impl DeploymentUtils {
    /// Check if Docker is available
    pub async fn check_docker() -> bool {
        CliUtils::execute_command("docker", &["--version"], None).await.is_ok()
    }

    /// Check if Kubernetes is available
    pub async fn check_kubernetes() -> bool {
        CliUtils::execute_command("kubectl", &["version", "--client"], None).await.is_ok()
    }

    /// Get deployment status
    pub async fn get_deployment_status(platform: &str, name: &str) -> Result<String> {
        match platform {
            "docker" => {
                let output = CliUtils::execute_command("docker", &["ps", "--filter", &format!("name={}", name)], None).await?;
                Ok(output)
            }
            "kubernetes" => {
                let output = CliUtils::execute_command("kubectl", &["get", "deployment", name], None).await?;
                Ok(output)
            }
            _ => Err(crate::Error::Other(format!("Unsupported platform: {}", platform))),
        }
    }
}
