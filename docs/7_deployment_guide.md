# 7. 部署指南

本章节介绍Lumos-X的各种部署选项，从本地开发环境到生产环境的完整部署指南。

## 7.1 部署架构概述

Lumos-X提供三种主要部署模式：

1. **单机部署** - 适用于个人使用和小型团队
2. **服务器部署** - 适用于团队和企业内部使用
3. **分布式部署** - 适用于大规模生产环境和去中心化应用

### 7.1.1 部署组件

一个完整的Lumos-X部署通常包含以下组件：

- **Frontend UI** - React前端界面
- **Agent Server** - Rust后端服务器
- **Database** - 数据存储（PostgreSQL, SQLite等）
- **P2P Network** - 去中心化网络节点
- **Storage** - 内容存储系统

### 7.1.2 部署拓扑

**基本拓扑**:
```
客户端 → 服务器 → 数据库
   ↓
P2P网络 ←→ 其他节点
```

**扩展拓扑**:
```
       负载均衡器
          ↓
前端集群 → API服务器集群 → 数据库集群
             ↓
       P2P网络网关 ←→ 去中心化网络
```

## 7.2 环境需求

### 7.2.1 最低系统要求

**单机部署**:
- CPU: 2核心
- 内存: 4GB RAM
- 存储: 10GB可用空间
- 操作系统: Linux, macOS, Windows

**服务器部署**:
- CPU: 4核心
- 内存: 8GB RAM
- 存储: 50GB SSD
- 操作系统: Linux (推荐Ubuntu 20.04或更高版本)
- 网络: 稳定的互联网连接，开放必要端口

**分布式部署**:
- 多台服务器，每台至少4核心8GB RAM
- 高速网络连接
- 负载均衡器
- 分布式数据库支持

### 7.2.2 软件依赖

- **Docker** (19.03或更高版本)
- **Docker Compose** (1.25或更高版本)
- **Node.js** (v16或更高，仅开发环境)
- **PostgreSQL** (13或更高，可选，使用外部数据库时)
- **Nginx** (用于反向代理，可选)

## 7.3 Docker部署

Docker是部署Lumos-X最简单的方式，适用于大多数使用场景。

### 7.3.1 使用预构建镜像

```bash
# 拉取最新版本镜像
docker pull lumosai/lumos-server:latest
docker pull lumosai/lumos-ui:latest

# 创建网络
docker network create lumos-network

# 启动服务器
docker run -d --name lumos-server \
  --network lumos-network \
  -p 8080:8080 \
  -v lumos-data:/data \
  -e "DATABASE_URL=sqlite:///data/lumos.db" \
  -e "P2P_ENABLED=true" \
  lumosai/lumos-server:latest

# 启动UI
docker run -d --name lumos-ui \
  --network lumos-network \
  -p 3000:80 \
  -e "API_ENDPOINT=http://lumos-server:8080" \
  lumosai/lumos-ui:latest
```

现在可以通过 `http://localhost:3000` 访问Lumos-X界面。

### 7.3.2 使用Docker Compose

创建 `docker-compose.yml` 文件:

```yaml
version: '3.8'

services:
  lumos-server:
    image: lumosai/lumos-server:latest
    restart: always
    volumes:
      - lumos-data:/data
    environment:
      - DATABASE_URL=sqlite:///data/lumos.db
      - P2P_ENABLED=true
      - P2P_LISTEN_ADDRESSES=/ip4/0.0.0.0/tcp/9090
      - LOG_LEVEL=info
    ports:
      - "8080:8080"
      - "9090:9090"

  lumos-ui:
    image: lumosai/lumos-ui:latest
    restart: always
    environment:
      - API_ENDPOINT=http://lumos-server:8080
    ports:
      - "3000:80"
    depends_on:
      - lumos-server

volumes:
  lumos-data:
```

启动服务:

```bash
docker-compose up -d
```

### 7.3.3 高级Docker配置

**使用PostgreSQL数据库**:

```yaml
version: '3.8'

services:
  lumos-db:
    image: postgres:14
    restart: always
    environment:
      - POSTGRES_USER=lumos
      - POSTGRES_PASSWORD=securepassword
      - POSTGRES_DB=lumosdb
    volumes:
      - pg-data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  lumos-server:
    image: lumosai/lumos-server:latest
    restart: always
    environment:
      - DATABASE_URL=postgres://lumos:securepassword@lumos-db:5432/lumosdb
      - P2P_ENABLED=true
    ports:
      - "8080:8080"
    depends_on:
      - lumos-db

  lumos-ui:
    image: lumosai/lumos-ui:latest
    # ... 配置同上

volumes:
  pg-data:
  lumos-data:
```

## 7.4 Kubernetes部署

对于大规模生产部署，Kubernetes提供了更好的可扩展性和管理能力。

### 7.4.1 前提条件

- 运行中的Kubernetes集群
- 已安装kubectl工具
- Helm (可选，用于简化部署)

### 7.4.2 基本Kubernetes部署

创建命名空间:

```bash
kubectl create namespace lumos
```

**部署PostgreSQL**:

```yaml
# postgres.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: lumos
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: lumos
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:14
        env:
        - name: POSTGRES_USER
          value: lumos
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: lumos-secrets
              key: db-password
        - name: POSTGRES_DB
          value: lumosdb
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: lumos
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
```

**创建密钥**:

```bash
kubectl create secret generic lumos-secrets \
  --namespace lumos \
  --from-literal=db-password=securepassword
```

**部署Lumos服务器**:

```yaml
# lumos-server.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: lumos-server
  namespace: lumos
spec:
  replicas: 2
  selector:
    matchLabels:
      app: lumos-server
  template:
    metadata:
      labels:
        app: lumos-server
    spec:
      containers:
      - name: lumos-server
        image: lumosai/lumos-server:latest
        env:
        - name: DATABASE_URL
          value: postgres://lumos:$(DB_PASSWORD)@postgres:5432/lumosdb
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: lumos-secrets
              key: db-password
        - name: P2P_ENABLED
          value: "true"
        - name: P2P_LISTEN_ADDRESSES
          value: "/ip4/0.0.0.0/tcp/9090"
        - name: LOG_LEVEL
          value: "info"
        ports:
        - containerPort: 8080
        - containerPort: 9090
        resources:
          limits:
            cpu: "1"
            memory: "1Gi"
          requests:
            cpu: "500m"
            memory: "512Mi"
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: lumos-server
  namespace: lumos
spec:
  selector:
    app: lumos-server
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  - name: p2p
    port: 9090
    targetPort: 9090
```

**部署Lumos UI**:

```yaml
# lumos-ui.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: lumos-ui
  namespace: lumos
spec:
  replicas: 2
  selector:
    matchLabels:
      app: lumos-ui
  template:
    metadata:
      labels:
        app: lumos-ui
    spec:
      containers:
      - name: lumos-ui
        image: lumosai/lumos-ui:latest
        env:
        - name: API_ENDPOINT
          value: http://lumos-server:8080
        ports:
        - containerPort: 80
        resources:
          limits:
            cpu: "500m"
            memory: "512Mi"
          requests:
            cpu: "200m"
            memory: "256Mi"
---
apiVersion: v1
kind: Service
metadata:
  name: lumos-ui
  namespace: lumos
spec:
  selector:
    app: lumos-ui
  ports:
  - port: 80
    targetPort: 80
```

**创建Ingress**:

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: lumos-ingress
  namespace: lumos
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: lumos.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: lumos-ui
            port:
              number: 80
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: lumos-server
            port:
              number: 8080
```

应用配置:

```bash
kubectl apply -f postgres.yaml
kubectl apply -f lumos-server.yaml
kubectl apply -f lumos-ui.yaml
kubectl apply -f ingress.yaml
```

### 7.4.3 使用Helm部署

如果您更喜欢使用Helm，可以创建一个Lumos-X的Helm Chart，或者使用我们提供的官方Chart:

```bash
# 添加Lumos Helm仓库
helm repo add lumosai https://charts.lumosai.com

# 更新仓库
helm repo update

# 部署Lumos-X
helm install lumos lumosai/lumos-x \
  --namespace lumos \
  --create-namespace \
  --set postgresql.auth.password=securepassword \
  --set ingress.hosts[0].host=lumos.example.com \
  --set server.p2p.enabled=true
```

## 7.5 裸机/VM部署

对于不使用容器化的环境，可以直接在服务器上部署Lumos-X。

### 7.5.1 系统准备

**安装依赖 (Ubuntu):**

```bash
# 更新系统
sudo apt update
sudo apt upgrade -y

# 安装必要依赖
sudo apt install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  curl \
  git \
  nginx \
  postgresql \
  postgresql-contrib
```

### 7.5.2 数据库设置

```bash
# 创建数据库用户和数据库
sudo -u postgres psql -c "CREATE USER lumos WITH PASSWORD 'securepassword';"
sudo -u postgres psql -c "CREATE DATABASE lumosdb OWNER lumos;"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE lumosdb TO lumos;"
```

### 7.5.3 部署服务器

```bash
# 创建部署目录
sudo mkdir -p /opt/lumos/server
sudo mkdir -p /opt/lumos/ui

# 下载最新发布版本
curl -L https://github.com/lumosai/lumos-x/releases/latest/download/lumos-server-linux-x86_64.tar.gz | sudo tar xz -C /opt/lumos/server
curl -L https://github.com/lumosai/lumos-x/releases/latest/download/lumos-ui.tar.gz | sudo tar xz -C /opt/lumos/ui

# 创建配置文件
cat << EOF | sudo tee /opt/lumos/server/config.toml
[database]
url = "postgres://lumos:securepassword@localhost/lumosdb"

[server]
host = "0.0.0.0"
port = 8080

[p2p]
enabled = true
listen_addresses = ["/ip4/0.0.0.0/tcp/9090"]

[log]
level = "info"
EOF

# 创建系统服务
cat << EOF | sudo tee /etc/systemd/system/lumos-server.service
[Unit]
Description=Lumos-X Server
After=network.target postgresql.service

[Service]
Type=simple
User=nobody
WorkingDirectory=/opt/lumos/server
ExecStart=/opt/lumos/server/lumos-server --config config.toml
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# 启用并启动服务
sudo systemctl daemon-reload
sudo systemctl enable lumos-server
sudo systemctl start lumos-server
```

### 7.5.4 配置Nginx

```bash
# 创建Nginx配置
cat << EOF | sudo tee /etc/nginx/sites-available/lumos
server {
    listen 80;
    server_name lumos.example.com;

    location / {
        root /opt/lumos/ui;
        index index.html;
        try_files \$uri \$uri/ /index.html;
    }

    location /api {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
    }
}
EOF

# 启用站点
sudo ln -s /etc/nginx/sites-available/lumos /etc/nginx/sites-enabled/
sudo systemctl reload nginx
```

## 7.6 配置参考

### 7.6.1 服务器配置参数

| 参数 | 描述 | 默认值 | 示例 |
|------|------|--------|------|
| `SERVER_HOST` | 服务器监听地址 | 0.0.0.0 | 127.0.0.1 |
| `SERVER_PORT` | 服务器监听端口 | 8080 | 9000 |
| `DATABASE_URL` | 数据库连接URL | sqlite://lumos.db | postgres://user:pass@host/db |
| `LOG_LEVEL` | 日志级别 | info | debug, info, warn, error |
| `P2P_ENABLED` | 是否启用P2P网络 | false | true |
| `P2P_LISTEN_ADDRESSES` | P2P监听地址 | /ip4/0.0.0.0/tcp/9090 | /ip4/0.0.0.0/tcp/9091 |
| `P2P_BOOTSTRAP_PEERS` | 引导节点列表 | [] | ["/ip4/1.2.3.4/tcp/9090/p2p/QmYyQS..."] |
| `AUTH_SECRET` | JWT验证密钥 | | 随机字符串(必须设置) |
| `AUTH_EXPIRY` | 令牌过期时间 | 24h | 1h, 7d |

### 7.6.2 UI配置参数

| 参数 | 描述 | 默认值 | 示例 |
|------|------|--------|------|
| `API_ENDPOINT` | 服务器API端点 | http://localhost:8080 | https://api.example.com |
| `WASM_PATH` | WebAssembly文件路径 | /wasm/lumos_core_bg.wasm | /assets/core.wasm |
| `DEFAULT_MODEL` | 默认模型 | gpt-4 | anthropic/claude-3-opus |
| `FEATURES_EXPERIMENTAL` | 启用实验性功能 | false | true |

### 7.6.3 环境特定配置

**开发环境**:

```bash
LOG_LEVEL=debug
DATABASE_URL=sqlite://lumos_dev.db
P2P_ENABLED=true
AUTH_EXPIRY=7d
FEATURES_EXPERIMENTAL=true
```

**生产环境**:

```bash
LOG_LEVEL=info
DATABASE_URL=postgres://lumos:password@db.example.com/lumosdb
P2P_ENABLED=true
P2P_BOOTSTRAP_PEERS=["/ip4/bootstrap.lumosai.com/tcp/9090/p2p/QmYyQS..."]
AUTH_SECRET=xxxxxxxxxxxxxxxxxxxx
AUTH_EXPIRY=24h
FEATURES_EXPERIMENTAL=false
```

## 7.7 监控与维护

### 7.7.1 日志管理

**Docker日志**:

```bash
# 查看服务器日志
docker logs -f lumos-server

# 查看UI日志
docker logs -f lumos-ui
```

**Kubernetes日志**:

```bash
# 查看服务器日志
kubectl logs -f -l app=lumos-server -n lumos

# 查看UI日志
kubectl logs -f -l app=lumos-ui -n lumos
```

### 7.7.2 监控

可以使用Prometheus和Grafana建立监控系统:

```yaml
# prometheus.yaml (Kubernetes示例)
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: monitoring
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
      - job_name: 'lumos-server'
        kubernetes_sd_configs:
          - role: endpoints
            namespaces:
              names:
                - lumos
        relabel_configs:
          - source_labels: [__meta_kubernetes_service_name]
            regex: lumos-server
            action: keep
          - source_labels: [__meta_kubernetes_endpoint_port_name]
            regex: metrics
            action: keep
```

为Lumos服务器启用Prometheus指标:

```yaml
# 在lumos-server.yaml中添加
spec:
  template:
    metadata:
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
```

### 7.7.3 备份与恢复

**数据库备份**:

```bash
# PostgreSQL备份
pg_dump -U lumos -h localhost lumosdb > lumos_backup_$(date +%Y%m%d).sql

# 自动备份脚本
cat << EOF > /usr/local/bin/backup-lumos.sh
#!/bin/bash
BACKUP_DIR="/backups/lumos"
mkdir -p \$BACKUP_DIR
TIMESTAMP=\$(date +%Y%m%d%H%M%S)
pg_dump -U lumos -h localhost lumosdb > \$BACKUP_DIR/lumos_\$TIMESTAMP.sql
find \$BACKUP_DIR -name "lumos_*.sql" -mtime +7 -delete
EOF
chmod +x /usr/local/bin/backup-lumos.sh

# 添加到crontab
(crontab -l ; echo "0 2 * * * /usr/local/bin/backup-lumos.sh") | crontab -
```

**数据库恢复**:

```bash
# PostgreSQL恢复
psql -U lumos -h localhost lumosdb < lumos_backup_20231101.sql
```

## 7.8 扩展与高可用性

### 7.8.1 负载均衡

使用Nginx作为负载均衡器:

```nginx
upstream lumos_servers {
    server 192.168.1.101:8080;
    server 192.168.1.102:8080;
    server 192.168.1.103:8080;
}

server {
    listen 80;
    server_name lumos.example.com;

    location /api {
        proxy_pass http://lumos_servers;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
    
    # UI静态文件
    location / {
        root /var/www/lumos-ui;
        index index.html;
        try_files $uri $uri/ /index.html;
    }
}
```

### 7.8.2 数据库高可用

使用PostgreSQL复制实现高可用:

**主服务器配置** (`postgresql.conf`):

```
listen_addresses = '*'
wal_level = replica
max_wal_senders = 10
wal_keep_size = 2GB
```

**主服务器访问控制** (`pg_hba.conf`):

```
# 允许复制连接
host replication replicator 192.168.1.0/24 md5
```

**从服务器设置**:

```bash
# 停止PostgreSQL
sudo systemctl stop postgresql

# 清空数据目录
sudo rm -rf /var/lib/postgresql/14/main/*

# 使用pg_basebackup创建初始备份
sudo -u postgres pg_basebackup -h 192.168.1.101 -D /var/lib/postgresql/14/main -U replicator -P -v

# 创建恢复配置
sudo -u postgres touch /var/lib/postgresql/14/main/standby.signal

# 配置复制
cat << EOF | sudo tee /var/lib/postgresql/14/main/postgresql.auto.conf
primary_conninfo = 'host=192.168.1.101 port=5432 user=replicator password=securepassword'
EOF

# 启动PostgreSQL
sudo systemctl start postgresql
```

### 7.8.3 容灾与恢复

实施跨区域备份和故障转移:

1. **备份策略**:
   - 每日数据库完整备份
   - 持续WAL归档
   - 备份存储在远程位置（如S3）

2. **故障转移流程**:
   - 监控系统检测主站点故障
   - 触发DNS变更，指向备用站点
   - 在备用站点恢复最新备份
   - 启动备用服务

## 7.9 安全最佳实践

### 7.9.1 网络安全

- 使用防火墙限制访问
- 仅开放必要端口
- 使用HTTPS加密传输
- 使用VPN或私有网络隔离服务器

### 7.9.2 认证安全

- 使用强密码
- 实施两因素认证
- 定期轮换密钥和证书
- 限制失败登录尝试

### 7.9.3 数据安全

- 加密敏感数据
- 实施最小权限原则
- 定期安全审计
- 数据备份加密

示例数据加密配置:

```toml
[security]
encrypt_sensitive_data = true
encryption_key_path = "/secure/keys/lumos.key"
```

### 7.9.4 Docker安全

- 使用非root用户
- 限制容器资源
- 定期更新基础镜像
- 扫描容器漏洞

示例Dockerfile安全配置:

```dockerfile
# 使用特定版本，避免使用latest标签
FROM rust:1.60-slim AS builder

# ... 构建阶段 ...

# 使用最小化基础镜像
FROM debian:bullseye-slim

# 创建非root用户
RUN groupadd -r lumos && useradd -r -g lumos lumos

# 设置工作目录和权限
WORKDIR /app
COPY --from=builder /build/target/release/lumos-server /app/
RUN chown -R lumos:lumos /app

# 使用非root用户运行
USER lumos

# 暴露端口
EXPOSE 8080 9090

CMD ["/app/lumos-server"]
```