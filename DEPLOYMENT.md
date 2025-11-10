# 🚀 LumosAI 部署指南

> **文档版本**: v1.0  
> **适用版本**: LumosAI 0.2.0+  
> **最后更新**: 2025-11-10

---

## 📋 目录

1. [快速开始](#快速开始)
2. [Docker 部署](#docker-部署)
3. [环境配置](#环境配置)
4. [健康检查](#健康检查)
5. [故障排除](#故障排除)
6. [生产部署](#生产部署)

---

## 快速开始

### 前置要求

- Docker 20.10+
- Docker Compose 2.0+
- 至少 4GB RAM
- 至少 10GB 磁盘空间

### 一键启动

```bash
# 1. 克隆项目
git clone https://github.com/your-org/lumosai.git
cd lumosai

# 2. 快速启动
./scripts/quick-start.sh
```

启动后访问：
- **API**: http://localhost:8080
- **Health**: http://localhost:8080/health
- **Qdrant**: http://localhost:6333/dashboard

---

## Docker 部署

### 手动部署

```bash
# 1. 构建镜像
docker build -t lumosai:latest .

# 2. 启动所有服务
docker-compose up -d

# 3. 查看日志
docker-compose logs -f lumosai

# 4. 健康检查
curl http://localhost:8080/health
```

### 服务说明

| 服务 | 端口 | 用途 |
|------|------|------|
| **lumosai** | 8080 | 主服务 API |
| **postgres** | 5432 | 主数据库 (pgvector) |
| **redis** | 6379 | 缓存和会话 |
| **qdrant** | 6333 | 向量数据库 |
| **weaviate** | 8081 | 向量数据库（备选） |

### 仅启动核心服务

```bash
# 启动最小化服务（lumosai + postgres + redis + qdrant）
docker-compose up -d lumosai postgres redis qdrant
```

---

## 环境配置

### 环境变量

创建 `.env` 文件：

```bash
# JWT 配置（必需）
JWT_SECRET=your-super-secret-key-at-least-32-bytes-long
JWT_EXPIRATION=3600

# 数据库（必需）
DATABASE_URL=postgres://postgres:lumosai_pass@postgres:5432/lumosai
REDIS_URL=redis://redis:6379

# 向量数据库
QDRANT_URL=http://qdrant:6333
WEAVIATE_URL=http://weaviate:8081

# LLM API Keys（可选）
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
ZHIPUAI_API_KEY=...

# 日志级别
RUST_LOG=info
RUST_BACKTRACE=1
```

### 生产环境配置

**重要**: 生产环境必须修改以下配置：

1. **JWT_SECRET**: 使用强随机密钥
   ```bash
   # 生成随机密钥
   openssl rand -base64 32
   ```

2. **数据库密码**: 修改默认密码
   ```yaml
   # docker-compose.yml
   environment:
     - POSTGRES_PASSWORD=your-strong-password
   ```

3. **Redis 密码**: 启用认证
   ```yaml
   command: redis-server --requirepass your-redis-password
   ```

---

## 健康检查

### 服务健康检查

```bash
# LumosAI
curl http://localhost:8080/health

# PostgreSQL
docker-compose exec postgres pg_isready -U postgres -d lumosai

# Redis
docker-compose exec redis redis-cli ping

# Qdrant
curl http://localhost:6333/health
```

### 健康检查端点

LumosAI 提供以下端点：

- `/health` - 基础健康检查
- `/ready` - 就绪检查（所有依赖正常）
- `/metrics` - Prometheus 指标（可选）

**响应格式**:
```json
{
  "status": "healthy",
  "version": "0.2.0",
  "uptime": 3600,
  "dependencies": {
    "postgres": "ok",
    "redis": "ok",
    "qdrant": "ok"
  }
}
```

---

## 故障排除

### 常见问题

#### 1. 端口被占用

**症状**: `Error: port is already allocated`

**解决**:
```bash
# 查找占用端口的进程
lsof -i :8080

# 或修改端口
# docker-compose.yml
ports:
  - "8081:8080"  # 改为 8081
```

#### 2. 内存不足

**症状**: 容器启动失败或 OOM

**解决**:
```bash
# 增加 Docker 内存限制
# docker-compose.yml
deploy:
  resources:
    limits:
      memory: 4G
```

#### 3. 健康检查失败

**症状**: `curl: (7) Failed to connect`

**解决**:
```bash
# 查看日志
docker-compose logs lumosai

# 重启服务
docker-compose restart lumosai

# 检查依赖服务
docker-compose ps
```

#### 4. 构建失败

**症状**: `error: could not compile`

**解决**:
```bash
# 清理缓存重新构建
docker-compose build --no-cache

# 或本地测试编译
cargo build --release
```

### 查看日志

```bash
# 所有服务日志
docker-compose logs

# 特定服务日志
docker-compose logs lumosai
docker-compose logs postgres

# 实时日志
docker-compose logs -f lumosai

# 最近 100 行
docker-compose logs --tail=100 lumosai
```

---

## 生产部署

### 性能优化

1. **使用 release 构建**
   ```dockerfile
   RUN cargo build --release
   ```

2. **启用缓存**
   ```yaml
   redis:
     command: redis-server --appendonly yes --maxmemory 2gb
   ```

3. **配置连接池**
   ```bash
   DATABASE_POOL_SIZE=20
   REDIS_POOL_SIZE=10
   ```

### 安全加固

1. **更改默认密码**
   ```yaml
   environment:
     - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
   ```

2. **启用 TLS**
   ```yaml
   environment:
     - TLS_ENABLED=true
     - TLS_CERT_PATH=/certs/server.crt
     - TLS_KEY_PATH=/certs/server.key
   ```

3. **限制网络访问**
   ```yaml
   networks:
     - internal  # 内部网络
   ```

### 高可用部署

```yaml
# docker-compose.prod.yml
services:
  lumosai:
    deploy:
      replicas: 3
      update_config:
        parallelism: 1
        delay: 10s
      restart_policy:
        condition: on-failure
```

### 监控和告警

```yaml
services:
  lumosai:
    labels:
      - "prometheus.io/scrape=true"
      - "prometheus.io/port=8080"
      - "prometheus.io/path=/metrics"
```

---

## 常用命令

### 服务管理

```bash
# 启动
docker-compose up -d

# 停止
docker-compose down

# 重启
docker-compose restart

# 查看状态
docker-compose ps

# 查看资源使用
docker stats
```

### 数据管理

```bash
# 备份数据库
docker-compose exec postgres pg_dump -U postgres lumosai > backup.sql

# 恢复数据库
cat backup.sql | docker-compose exec -T postgres psql -U postgres -d lumosai

# 清理数据卷
docker-compose down -v  # ⚠️ 会删除所有数据
```

### 调试

```bash
# 进入容器
docker-compose exec lumosai /bin/bash

# 查看环境变量
docker-compose exec lumosai env

# 测试网络连接
docker-compose exec lumosai curl http://postgres:5432
```

---

## 性能基准

### 资源使用（默认配置）

| 服务 | CPU | Memory | Disk |
|------|-----|--------|------|
| LumosAI | ~10% | ~500MB | ~100MB |
| PostgreSQL | ~5% | ~200MB | ~1GB |
| Redis | ~2% | ~50MB | ~100MB |
| Qdrant | ~5% | ~300MB | ~500MB |
| **总计** | ~22% | ~1GB | ~2GB |

### 性能指标

```
Docker 镜像大小: ~450MB (优化后)
首次构建时间: 3-5 分钟
重新构建时间: 1-2 分钟（缓存）
启动时间: ~20 秒
健康检查: 第 10 秒开始
```

---

## 更多信息

- **GitHub**: https://github.com/your-org/lumosai
- **文档**: https://docs.lumosai.dev
- **Issues**: https://github.com/your-org/lumosai/issues

---

**最后更新**: 2025-11-10  
**贡献者**: LumosAI Development Team

