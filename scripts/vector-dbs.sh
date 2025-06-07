#!/bin/bash

# Lumos Vector Databases Management Script
# 用于管理向量数据库的便利脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
COMPOSE_FILE="docker-compose.vector-dbs.yml"
ENV_FILE=".env.vector-dbs"

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 检查Docker和Docker Compose
check_dependencies() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
}

# 创建环境变量文件
create_env_file() {
    if [ ! -f "$ENV_FILE" ]; then
        print_info "Creating environment file: $ENV_FILE"
        cat > "$ENV_FILE" << EOF
# Lumos Vector Databases Environment Variables

# Qdrant
QDRANT_URL=http://localhost:6334

# Weaviate
WEAVIATE_URL=http://localhost:8080

# PostgreSQL
DATABASE_URL=postgresql://postgres:password@localhost:5432/lumos
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=lumos
POSTGRES_USER=postgres
POSTGRES_PASSWORD=password

# Redis
REDIS_URL=redis://localhost:6379

# Elasticsearch
ELASTICSEARCH_URL=http://localhost:9200
EOF
        print_success "Environment file created: $ENV_FILE"
        print_info "You can source this file: source $ENV_FILE"
    fi
}

# 启动所有服务
start_all() {
    print_info "Starting all vector databases..."
    docker-compose -f "$COMPOSE_FILE" up -d
    print_success "All services started!"
    show_status
}

# 启动特定服务
start_service() {
    local service=$1
    if [ -z "$service" ]; then
        print_error "Please specify a service name"
        show_help
        exit 1
    fi
    
    print_info "Starting $service..."
    docker-compose -f "$COMPOSE_FILE" up -d "$service"
    print_success "$service started!"
}

# 停止所有服务
stop_all() {
    print_info "Stopping all vector databases..."
    docker-compose -f "$COMPOSE_FILE" down
    print_success "All services stopped!"
}

# 停止特定服务
stop_service() {
    local service=$1
    if [ -z "$service" ]; then
        print_error "Please specify a service name"
        show_help
        exit 1
    fi
    
    print_info "Stopping $service..."
    docker-compose -f "$COMPOSE_FILE" stop "$service"
    print_success "$service stopped!"
}

# 重启所有服务
restart_all() {
    print_info "Restarting all vector databases..."
    docker-compose -f "$COMPOSE_FILE" restart
    print_success "All services restarted!"
}

# 显示服务状态
show_status() {
    print_info "Vector databases status:"
    docker-compose -f "$COMPOSE_FILE" ps
    
    echo ""
    print_info "Health checks:"
    
    # Qdrant
    if curl -s http://localhost:6333/health > /dev/null 2>&1; then
        print_success "Qdrant: http://localhost:6333 (REST) / http://localhost:6334 (gRPC)"
    else
        print_warning "Qdrant: Not responding"
    fi
    
    # Weaviate
    if curl -s http://localhost:8080/v1/.well-known/ready > /dev/null 2>&1; then
        print_success "Weaviate: http://localhost:8080"
    else
        print_warning "Weaviate: Not responding"
    fi
    
    # PostgreSQL
    if docker-compose -f "$COMPOSE_FILE" exec -T postgres pg_isready -U postgres -d lumos > /dev/null 2>&1; then
        print_success "PostgreSQL: localhost:5432/lumos"
    else
        print_warning "PostgreSQL: Not responding"
    fi
    
    # Redis
    if docker-compose -f "$COMPOSE_FILE" exec -T redis redis-cli ping > /dev/null 2>&1; then
        print_success "Redis: localhost:6379"
    else
        print_warning "Redis: Not responding"
    fi
    
    # Elasticsearch
    if curl -s http://localhost:9200/_cluster/health > /dev/null 2>&1; then
        print_success "Elasticsearch: http://localhost:9200"
    else
        print_warning "Elasticsearch: Not responding"
    fi
}

# 显示日志
show_logs() {
    local service=$1
    if [ -z "$service" ]; then
        docker-compose -f "$COMPOSE_FILE" logs -f
    else
        docker-compose -f "$COMPOSE_FILE" logs -f "$service"
    fi
}

# 清理数据
clean_data() {
    print_warning "This will remove all data from vector databases!"
    read -p "Are you sure? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Stopping services and removing volumes..."
        docker-compose -f "$COMPOSE_FILE" down -v
        print_success "All data cleaned!"
    else
        print_info "Operation cancelled."
    fi
}

# 运行测试
run_tests() {
    print_info "Running vector database integration tests..."
    
    # 确保环境变量已设置
    if [ -f "$ENV_FILE" ]; then
        source "$ENV_FILE"
    fi
    
    # 运行Rust测试
    cargo test vector_integration_test --features vector-all -- --nocapture
}

# 显示帮助
show_help() {
    echo "Lumos Vector Databases Management Script"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  start [service]     Start all services or a specific service"
    echo "  stop [service]      Stop all services or a specific service"
    echo "  restart             Restart all services"
    echo "  status              Show status of all services"
    echo "  logs [service]      Show logs for all services or a specific service"
    echo "  clean               Stop services and remove all data"
    echo "  test                Run integration tests"
    echo "  env                 Create environment variables file"
    echo "  help                Show this help message"
    echo ""
    echo "Services:"
    echo "  qdrant              Qdrant vector database"
    echo "  weaviate            Weaviate vector database"
    echo "  postgres            PostgreSQL with pgvector"
    echo "  redis               Redis cache"
    echo "  elasticsearch       Elasticsearch"
    echo ""
    echo "Examples:"
    echo "  $0 start            # Start all services"
    echo "  $0 start qdrant     # Start only Qdrant"
    echo "  $0 status           # Show status"
    echo "  $0 logs weaviate    # Show Weaviate logs"
    echo "  $0 test             # Run tests"
}

# 主函数
main() {
    check_dependencies
    
    case "${1:-help}" in
        start)
            create_env_file
            if [ -n "$2" ]; then
                start_service "$2"
            else
                start_all
            fi
            ;;
        stop)
            if [ -n "$2" ]; then
                stop_service "$2"
            else
                stop_all
            fi
            ;;
        restart)
            restart_all
            ;;
        status)
            show_status
            ;;
        logs)
            show_logs "$2"
            ;;
        clean)
            clean_data
            ;;
        test)
            run_tests
            ;;
        env)
            create_env_file
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"
