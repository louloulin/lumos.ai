# LumosAI Dockerfile - 多阶段构建
# 优化镜像大小和构建速度

# ============================================================================
# Stage 1: Builder - 构建 Rust 应用
# ============================================================================
FROM rust:1.75-slim as builder

LABEL maintainer="LumosAI Team <team@lumosai.dev>"
LABEL description="LumosAI - Enterprise-grade AI Agent Framework"

WORKDIR /app

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 复制 Cargo 配置文件（用于依赖缓存）
COPY Cargo.toml Cargo.lock ./
COPY bunfig.toml ./

# 复制所有包的 Cargo.toml（用于依赖缓存）
COPY lumosai_core/Cargo.toml lumosai_core/Cargo.toml
COPY lumosai_auth/Cargo.toml lumosai_auth/Cargo.toml
COPY lumosai_rag/Cargo.toml lumosai_rag/Cargo.toml
COPY lumosai_vector/Cargo.toml lumosai_vector/Cargo.toml
COPY lumosai_vector/core/Cargo.toml lumosai_vector/core/Cargo.toml
COPY lumosai_vector/qdrant/Cargo.toml lumosai_vector/qdrant/Cargo.toml
COPY lumosai_vector/memory/Cargo.toml lumosai_vector/memory/Cargo.toml
COPY lumosai_cli/Cargo.toml lumosai_cli/Cargo.toml
COPY lumos_macro/Cargo.toml lumos_macro/Cargo.toml

# 复制所有源代码
COPY lumosai_core lumosai_core
COPY lumosai_auth lumosai_auth
COPY lumosai_rag lumosai_rag
COPY lumosai_vector lumosai_vector
COPY lumosai_cli lumosai_cli
COPY lumosai_examples lumosai_examples
COPY lumos_macro lumos_macro
COPY src src

# 构建 release 版本（优化编译）
RUN cargo build --release --bin lumosai-cli

# ============================================================================
# Stage 2: Runtime - 最小运行时镜像
# ============================================================================
FROM debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 -s /bin/bash lumosai && \
    mkdir -p /app/data /app/logs && \
    chown -R lumosai:lumosai /app

# 复制编译好的二进制文件
COPY --from=builder /app/target/release/lumosai-cli /usr/local/bin/lumosai

# 切换到非 root 用户
USER lumosai

# 暴露端口（HTTP API）
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 默认启动命令
CMD ["lumosai", "serve", "--host", "0.0.0.0", "--port", "8080"]

