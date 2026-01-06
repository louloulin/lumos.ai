#!/bin/bash
# LumosAI 快速启动脚本
# 一键启动完整的 LumosAI 环境

set -e

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              🚀 LumosAI 快速启动                               ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# 检查 Docker 是否安装
if ! command -v docker &> /dev/null; then
    echo "❌ Error: Docker is not installed"
    echo "Please install Docker: https://docs.docker.com/get-docker/"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo "❌ Error: docker-compose is not installed"
    echo "Please install docker-compose: https://docs.docker.com/compose/install/"
    exit 1
fi

# 检查 .env 文件（可选）
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cat > .env << 'EOF'
# LumosAI Environment Configuration

# JWT Configuration
JWT_SECRET=change-this-secret-in-production-must-be-at-least-32-bytes
JWT_EXPIRATION=3600

# LLM API Keys (optional)
# OPENAI_API_KEY=your_openai_api_key
# ANTHROPIC_API_KEY=your_anthropic_api_key
# ZHIPUAI_API_KEY=your_zhipu_api_key

# Database
DATABASE_URL=postgres://postgres:lumosai_pass@postgres:5432/lumosai
REDIS_URL=redis://redis:6379

# Vector Databases
QDRANT_URL=http://qdrant:6333
WEAVIATE_URL=http://weaviate:8081

# Logging
RUST_LOG=info
RUST_BACKTRACE=1
EOF
    echo "✅ Created .env file"
fi

# 停止现有容器
echo ""
echo "🛑 Stopping existing containers..."
docker-compose down 2>/dev/null || true

# 构建镜像
echo ""
echo "🔨 Building LumosAI Docker image..."
echo "   (This may take 3-5 minutes on first run)"
docker-compose build

# 启动所有服务
echo ""
echo "🚀 Starting all services..."
docker-compose up -d

# 等待服务启动
echo ""
echo "⏳ Waiting for services to be ready..."
echo "   - PostgreSQL..."
sleep 5

echo "   - Redis..."
sleep 2

echo "   - Qdrant..."
sleep 3

echo "   - LumosAI Server..."
sleep 5

# 健康检查
echo ""
echo "🏥 Performing health checks..."

# 检查 PostgreSQL
if docker-compose exec -T postgres pg_isready -U postgres -d lumosai &> /dev/null; then
    echo "   ✅ PostgreSQL is ready"
else
    echo "   ⚠️  PostgreSQL is not ready yet"
fi

# 检查 Redis  
if docker-compose exec -T redis redis-cli ping &> /dev/null; then
    echo "   ✅ Redis is ready"
else
    echo "   ⚠️  Redis is not ready yet"
fi

# 检查 Qdrant
if curl -f http://localhost:6333/health &> /dev/null; then
    echo "   ✅ Qdrant is ready"
else
    echo "   ⚠️  Qdrant is not ready yet"
fi

# 检查 LumosAI
echo ""
if curl -f http://localhost:8080/health &> /dev/null; then
    echo "✅ LumosAI is running successfully!"
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "🎉 LumosAI Started Successfully!"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    echo "📖 Access Points:"
    echo "   - LumosAI API:      http://localhost:8080"
    echo "   - Health Check:     http://localhost:8080/health"
    echo "   - Metrics:          http://localhost:8080/metrics"
    echo "   - Qdrant Dashboard: http://localhost:6333/dashboard"
    echo "   - PostgreSQL:       localhost:5432"
    echo "   - Redis:            localhost:6379"
    echo ""
    echo "📚 Next Steps:"
    echo "   - View logs:        docker-compose logs -f lumosai"
    echo "   - Stop services:    docker-compose down"
    echo "   - Restart:          docker-compose restart lumosai"
    echo ""
    echo "🔧 Useful Commands:"
    echo "   - Test Agent:       cargo run --example mvp_01_simple_agent"
    echo "   - Test Auth:        cargo run -p lumosai_auth --example jwt_auth_demo"
    echo ""
else
    echo "❌ LumosAI failed to start"
    echo ""
    echo "Checking logs..."
    docker-compose logs lumosai | tail -20
    echo ""
    echo "Try:"
    echo "   docker-compose logs lumosai    # View full logs"
    echo "   docker-compose restart lumosai # Restart service"
    exit 1
fi

