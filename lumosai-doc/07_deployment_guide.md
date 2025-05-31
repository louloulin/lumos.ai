# 7. 部署指南

本章节提供 Lumosai 的部署配置和操作指南，包括本地部署、云服务部署和去中心化 P2P 网络部署模式。

## 7.1 部署准备

在部署 Lumosai 之前，请确保满足以下条件：

### 7.1.1 系统要求

| 组件 | 开发环境 | 生产环境 (最小) | 生产环境 (推荐) |
|------|---------|----------------|----------------|
| CPU  | 双核处理器 | 四核处理器 | 八核处理器 |
| 内存 | 4GB RAM | 8GB RAM | 16GB RAM |
| 存储 | 10GB 可用空间 | 20GB SSD | 50GB+ SSD |
| 网络 | 宽带连接 | 10Mbps+ | 100Mbps+ |
| 操作系统 | Windows/macOS/Linux | Ubuntu 20.04+ / Debian 11+ | Ubuntu 22.04 LTS |

### 7.1.2 依赖检查清单

1. **基础依赖**:
   - Rust 1.70.0+ (`rustc --version`)
   - Node.js 18.0.0+ (`node --version`)
   - pnpm 7.0.0+ (`pnpm --version`)
   - Docker 20.10.0+ (可选，用于容器化部署)

2. **LLM 访问** (至少选择一项):
   - OpenAI API 密钥
   - Anthropic API 密钥
   - 本地模型（需要 GPU 支持）
   - 自托管开源模型

3. **存储选项** (至少选择一项):
   - 本地文件系统
   - SQLite（小型部署）
   - PostgreSQL（生产环境）
   - 向量数据库（Qdrant/Milvus）

4. **（可选）P2P 网络**:
   - 可公开访问的 IP 或域名
   - 开放的网络端口（默认 4001/tcp）
   - NAT 穿透/端口转发配置

## 7.2 本地部署（桌面应用模式）

本节介绍如何将 Lumosai 作为桌面应用部署在本地机器上。

### 7.2.1 构建桌面应用

```bash
# 克隆仓库
git clone https://github.com/your-org/lumosai.git
cd lumosai

# 安装依赖
pnpm install

# 构建所有组件
pnpm build

# 构建桌面应用
cd lumosai_ui
pnpm desktop:build
```

构建完成后，桌面应用安装包将位于 `lumosai_ui/dist-desktop/` 目录中，支持以下平台：
- Windows: `.exe` 或 `.msi` 安装程序
- macOS: `.dmg` 安装镜像
- Linux: `.AppImage`, `.deb` 或 `.rpm` 包

### 7.2.2 配置桌面应用

1. **安装应用**:
   - 双击安装包并按照向导完成安装

2. **初始设置**:
   - 首次启动时，会显示配置向导
   - 设置 API 密钥（若使用云服务 LLM）
   - 配置本地模型路径（若使用本地模型）
   - 设置数据存储位置
   - 配置 P2P 网络参数（可选）

3. **配置文件**:
   桌面应用的配置存储在以下位置：
   - Windows: `%APPDATA%\Lumosai\config.json`
   - macOS: `~/Library/Application Support/Lumosai/config.json`
   - Linux: `~/.config/lumosai/config.json`

   配置文件示例：
   ```json
   {
     "app": {
       "theme": "system",
       "language": "zh-CN"
     },
     "llm": {
       "provider": "openai", // 支持 "openai", "anthropic", "deepseek", "local"
       "apiKey": "sk-xxxxxxxxxxxx",
       "defaultModel": "gpt-4"
     },
     "storage": {
       "type": "local",
       "path": "/home/user/lumosai-data"
     },
     "p2p": {
       "enabled": true,
       "peerId": "QmXxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxYY",
       "bootstrapPeers": [
         "/ip4/bootstrap.lumosai.org/tcp/4001/p2p/QmZxxxxxxxxxxxxx"
       ]
     }
   }
   ```

### 7.2.3 使用本地模型

要在桌面应用中使用本地模型，请按照以下步骤操作：

1. **下载模型**:
   从 Hugging Face 或其他来源下载兼容的模型

2. **配置本地模型**:
   在应用中选择 "设置 > LLM > 本地模型"，然后配置以下选项：
   - 模型路径
   - 量化选项（如 4-bit、8-bit）
   - GPU 加速设置
   - 上下文大小
   - 推理参数（温度、top-p 等）

3. **资源管理**:
   本地模型需要大量内存，可以通过以下配置优化资源使用：
   - 在 `settings.json` 中设置 `"llm.local.maxRam": "8GB"`
   - 降低 `llm.local.contextSize` 以减少内存使用
   - 启用 `llm.local.useGpu` 以使用 GPU 加速

## 7.3 服务器部署（云服务模式）

本节介绍如何将 Lumosai 部署到服务器，以提供多用户访问和更高的计算能力。

### 7.3.1 手动部署

#### 准备服务器

1. **创建服务器实例**:
   - 推荐使用 Ubuntu 22.04 LTS
   - 最小配置：4 核 CPU，8GB RAM，20GB SSD
   - 推荐配置：8 核 CPU，16GB RAM，50GB+ SSD

2. **安装依赖**:

```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装基本依赖
sudo apt install -y curl build-essential pkg-config libssl-dev git

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 安装 Node.js 和 pnpm
curl -fsSL https://fnm.vercel.app/install | bash
source ~/.bashrc
fnm install 20
fnm use 20
npm install -g pnpm

# 安装 PostgreSQL（如需使用）
sudo apt install -y postgresql postgresql-contrib
sudo systemctl enable postgresql
sudo systemctl start postgresql
```

#### 部署应用

```bash
# 克隆仓库
git clone https://github.com/your-org/lumosai.git
cd lumosai

# 安装依赖
pnpm install

# 构建所有组件
pnpm build

# 配置服务器参数
cp config.example.json config.json
nano config.json  # 编辑配置文件

# 启动服务器
pnpm start
```

#### 使用 Systemd 管理服务

创建 systemd 服务文件：

```bash
sudo nano /etc/systemd/system/lumosai.service
```

服务文件内容：

```ini
[Unit]
Description=Lumosai AI Agent Server
After=network.target postgresql.service

[Service]
Type=simple
User=lumosai
WorkingDirectory=/opt/lumosai
ExecStart=/usr/bin/env pnpm start
Restart=on-failure
Environment="NODE_ENV=production"
Environment="PORT=3000"

[Install]
WantedBy=multi-user.target
```

启用和管理服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable lumosai
sudo systemctl start lumosai
sudo systemctl status lumosai
```

### 7.3.2 Docker 部署

Lumosai 提供了 Docker 和 Docker Compose 支持，简化部署流程。

#### 使用 Docker Compose

创建 `docker-compose.yml` 文件：

```yaml
version: '3.8'

services:
  lumosai-server:
    image: lumosai/server:latest
    container_name: lumosai-server
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - DB_HOST=postgres
      - DB_PORT=5432
      - DB_USER=lumosai
      - DB_PASSWORD=your_secure_password
      - DB_NAME=lumosai
      - OPENAI_API_KEY=your_openai_api_key
    volumes:
      - ./data:/app/data
    depends_on:
      - postgres
      - qdrant
    restart: unless-stopped

  postgres:
    image: postgres:15-alpine
    container_name: lumosai-postgres
    environment:
      - POSTGRES_USER=lumosai
      - POSTGRES_PASSWORD=your_secure_password
      - POSTGRES_DB=lumosai
    volumes:
      - postgres-data:/var/lib/postgresql/data
    restart: unless-stopped

  qdrant:
    image: qdrant/qdrant:latest
    container_name: lumosai-qdrant
    ports:
      - "6333:6333"
    volumes:
      - qdrant-data:/qdrant/storage
    restart: unless-stopped

volumes:
  postgres-data:
  qdrant-data:
```

启动服务：

```bash
docker-compose up -d
```

#### 使用自定义 Dockerfile

如果需要自定义镜像，可以使用项目中的 Dockerfile：

```bash
# 构建自定义镜像
docker build -t my-lumosai:latest .

# 运行容器
docker run -d \
  --name lumosai \
  -p 3000:3000 \
  -e NODE_ENV=production \
  -e OPENAI_API_KEY=your_openai_api_key \
  -v $(pwd)/data:/app/data \
  my-lumosai:latest
```

### 7.3.3 Kubernetes 部署

对于大规模部署，Lumosai 支持 Kubernetes 集群部署。

1. **准备 Helm Chart**:
   Lumosai 提供官方 Helm Chart，位于 `deploy/charts/lumosai`

2. **安装 Chart**:

```bash
# 添加 Lumosai Helm 仓库
helm repo add lumosai https://charts.lumosai.org
helm repo update

# 安装 Chart (使用 OpenAI)
helm install lumosai lumosai/lumosai \
  --namespace lumosai \
  --create-namespace \
  --set llm.provider=openai \
  --set llm.apiKey=your_openai_api_key \
  --set storage.postgres.enabled=true \
  --set storage.postgres.password=your_secure_password

# 或者使用 DeepSeek
helm install lumosai lumosai/lumosai \
  --namespace lumosai \
  --create-namespace \
  --set llm.provider=deepseek \
  --set llm.apiKey=your_deepseek_api_key \
  --set storage.postgres.enabled=true \
  --set storage.postgres.password=your_secure_password
```

3. **自定义 values.yaml**:

```yaml
# values.yaml
replicaCount: 3

llm:
  provider: openai  # 或 "deepseek", "anthropic", "local"
  apiKey: your_openai_api_key
  defaultModel: gpt-4

storage:
  postgres:
    enabled: true
    host: lumosai-postgres
    user: lumosai
    password: your_secure_password
    database: lumosai
  
  qdrant:
    enabled: true
    url: http://lumosai-qdrant:6333

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: lumosai.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: lumosai-tls
      hosts:
        - lumosai.example.com

resources:
  limits:
    cpu: 2000m
    memory: 2Gi
  requests:
    cpu: 500m
    memory: 1Gi
```

应用自定义配置：

```bash
helm install lumosai lumosai/lumosai -f values.yaml --namespace lumosai
```

## 7.4 去中心化部署（P2P 网络模式）

Lumosai 支持去中心化的 P2P 网络部署，适合分布式团队和隐私敏感场景。

### 7.4.1 P2P 网络配置

1. **配置节点**:
   编辑配置文件，启用 P2P 功能：

```json
{
  "p2p": {
    "enabled": true,
    "listenAddresses": ["/ip4/0.0.0.0/tcp/4001", "/ip6/::/tcp/4001"],
    "bootstrapPeers": [
      "/ip4/bootstrap1.lumosai.org/tcp/4001/p2p/QmXxxx...",
      "/ip4/bootstrap2.lumosai.org/tcp/4001/p2p/QmYxxx..."
    ],
    "announceAddresses": [
      "/ip4/your_public_ip/tcp/4001"
    ],
    "discover": true,
    "enableMDNS": true,
    "enableRelay": true
  }
}
```

2. **网络设置**:
   - 确保节点间可以相互通信
   - 配置防火墙允许 TCP 端口 4001 入站连接
   - 对于 NAT 后的节点，配置端口转发或使用中继

3. **引导节点**:
   P2P 网络需要一些稳定的引导节点来帮助新节点发现网络：

```bash
# 运行专用引导节点
lumosai-node bootstrap \
  --listen-addr=/ip4/0.0.0.0/tcp/4001 \
  --peer-id-file=peer-id.json
```

引导节点配置示例：

```json
{
  "node": {
    "type": "bootstrap",
    "peerId": "QmXxxx...",
    "privateKey": "CAESxxx...",
    "listenAddresses": ["/ip4/0.0.0.0/tcp/4001"],
    "enableRelay": true,
    "enableDHT": true
  }
}
```

### 7.4.2 资源共享配置

在 P2P 网络中配置资源共享：

1. **共享策略**:
   
```json
{
  "sharing": {
    "enabled": true,
    "resources": {
      "models": {
        "enabled": true,
        "maxSizeBytes": 4294967296
      },
      "datasets": {
        "enabled": true,
        "maxSizeBytes": 2147483648
      },
      "agents": {
        "enabled": true
      }
    },
    "replication": {
      "minReplicas": 3,
      "maxReplicas": 10
    }
  }
}
```

2. **内容寻址存储**:
   
```json
{
  "cas": {
    "enabled": true,
    "backend": "ipfs",
    "ipfs": {
      "path": "./ipfs",
      "api": "/ip4/127.0.0.1/tcp/5001"
    }
  }
}
```

### 7.4.3 安全配置

P2P 网络需要特别注意安全配置：

1. **身份和认证**:
   
```json
{
  "security": {
    "identity": {
      "type": "did",
      "method": "key"
    },
    "authentication": {
      "enabled": true,
      "methods": ["signature"]
    },
    "encryption": {
      "enabled": true,
      "peerToPeer": true,
      "dataAtRest": true
    }
  }
}
```

2. **权限控制**:
   
```json
{
  "permissions": {
    "defaultPolicy": "deny",
    "rules": [
      {
        "resource": "storage/public/*",
        "operations": ["read"],
        "subject": "*",
        "effect": "allow"
      },
      {
        "resource": "storage/private/*",
        "operations": ["read", "write"],
        "subject": "self",
        "effect": "allow"
      },
      {
        "resource": "agent/*",
        "operations": ["execute"],
        "subject": "group:collaborators",
        "effect": "allow"
      }
    ]
  }
}
```

## 7.5 多环境配置

Lumosai 支持多环境配置，实现不同部署环境的统一管理。

### 7.5.1 环境配置文件

使用环境特定的配置文件：

```
config/
├── default.json     # 默认配置
├── development.json # 开发环境特定配置
├── staging.json     # 预发布环境特定配置
├── production.json  # 生产环境特定配置
└── local.json       # 本地覆盖（不提交到版本控制）
```

配置加载优先级（从低到高）：
1. `default.json`
2. 环境特定配置（`{NODE_ENV}.json`）
3. `local.json`

### 7.5.2 环境变量配置

Lumosai 支持通过环境变量覆盖配置：

```bash
# 基本配置
export NODE_ENV=production
export PORT=3000
export LOG_LEVEL=info

# LLM 配置
export LLM_PROVIDER=openai
export LLM_API_KEY=sk-xxxx
export LLM_DEFAULT_MODEL=gpt-4

# 数据库配置
export DB_HOST=localhost
export DB_PORT=5432
export DB_USER=lumosai
export DB_PASSWORD=xxxx
export DB_NAME=lumosai

# P2P 网络配置
export P2P_ENABLED=true
export P2P_LISTEN_ADDR=/ip4/0.0.0.0/tcp/4001
```

环境变量会覆盖配置文件中的对应值。

## 7.6 监控和日志

### 7.6.1 日志配置

Lumosai 使用结构化日志，可以配置如下：

```json
{
  "logging": {
    "level": "info",
    "format": "json",
    "destination": "file",
    "filePath": "/var/log/lumosai/server.log",
    "rotation": {
      "maxSize": "100m",
      "maxFiles": 10,
      "compress": true
    }
  }
}
```

### 7.6.2 监控设置

Lumosai 提供多种监控选项：

1. **Prometheus 指标**:
   
```json
{
  "monitoring": {
    "prometheus": {
      "enabled": true,
      "endpoint": "/metrics",
      "port": 9090
    }
  }
}
```

2. **健康检查**:
   
```json
{
  "health": {
    "enabled": true,
    "endpoint": "/health",
    "checks": ["db", "llm", "storage"]
  }
}
```

3. **遥测**:
   
```json
{
  "telemetry": {
    "enabled": true,
    "anonymize": true,
    "endpoint": "https://telemetry.lumosai.org/collect"
  }
}
```

### 7.6.3 通知和警报

配置系统警报和通知：

```json
{
  "alerts": {
    "enabled": true,
    "channels": [
      {
        "type": "email",
        "recipients": ["admin@example.com"],
        "events": ["error", "critical"]
      },
      {
        "type": "slack",
        "webhook": "https://hooks.slack.com/services/xxx",
        "events": ["critical"]
      }
    ]
  }
}
```

## 7.7 扩展和升级

### 7.7.1 插件安装

Lumosai 支持通过插件系统扩展功能：

```bash
# 安装插件
lumosai plugin install @lumosai/plugin-name

# 列出已安装插件
lumosai plugin list

# 更新插件
lumosai plugin update @lumosai/plugin-name

# 移除插件
lumosai plugin remove @lumosai/plugin-name
```

### 7.7.2 版本升级

升级 Lumosai 版本：

```bash
# 检查当前版本
lumosai --version

# 检查可用更新
lumosai update check

# 执行升级
lumosai update

# 特定版本升级
lumosai update --version 1.2.3
```

使用 Docker 升级：

```bash
# 拉取最新镜像
docker pull lumosai/server:latest

# 重启容器
docker-compose down
docker-compose up -d
```

### 7.7.3 数据迁移

执行数据库迁移：

```bash
# 查看迁移状态
lumosai db status

# 生成迁移脚本
lumosai db migrate generate --name add_new_field

# 应用迁移
lumosai db migrate up

# 回滚迁移
lumosai db migrate down
```

## 7.8 故障排查

### 7.8.1 常见问题诊断

1. **服务无法启动**:
   - 检查日志文件
   - 验证配置是否正确
   - 确认必需的端口未被占用
   - 检查系统资源使用情况

2. **性能问题**:
   - 检查 CPU/内存/磁盘使用率
   - 分析数据库查询性能
   - 优化缓存配置
   - 考虑扩展资源

3. **网络连接问题**:
   - 验证防火墙和网络配置
   - 检查 DNS 解析
   - 测试 P2P 连接状态
   - 使用 traceroute 分析网络路径

### 7.8.2 诊断工具

Lumosai 提供内置诊断工具：

```bash
# 运行系统自检
lumosai doctor

# 检查特定组件
lumosai doctor --component p2p

# 性能基准测试
lumosai benchmark

# 检查配置有效性
lumosai config validate
```

### 7.8.3 恢复和备份

配置自动备份：

```json
{
  "backup": {
    "enabled": true,
    "schedule": "0 2 * * *",  // 每天凌晨 2 点
    "retention": {
      "count": 7,
      "days": 30
    },
    "storage": {
      "type": "s3",
      "bucket": "lumosai-backups",
      "prefix": "prod/"
    }
  }
}
```

手动创建备份：

```bash
# 创建完整备份
lumosai backup create --name full-backup

# 仅备份数据库
lumosai backup create --type database

# 恢复备份
lumosai backup restore --id backup_20230615_020101
``` 