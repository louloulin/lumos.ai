# Lomus AI CLI

Lomus AI命令行工具，用于创建、开发、构建和部署Lomus AI应用。

## 安装

```bash
cargo install --path .
```

## 使用方法

### 初始化项目

```bash
# 交互式创建项目
lomus init

# 指定项目名称和模板
lomus init --name my_project --template agent

# 从URL下载模板并初始化
lomus init --template_url https://github.com/example/template
```

### 开发模式

```bash
# 启动开发服务器
lomus dev

# 指定端口和启用热重载
lomus dev --port 8080 -r
```

### 运行应用

```bash
# 运行当前目录的应用
lomus run

# 指定项目目录
lomus run --project_dir /path/to/project
```

### 构建应用

```bash
# 构建当前目录的应用
lomus build

# 指定输出目录
lomus build --output /path/to/output
```

### 部署应用

```bash
# 本地部署
lomus deploy --target local

# Docker部署
lomus deploy --target docker

# 云平台部署
lomus deploy --target aws
lomus deploy --target azure
lomus deploy --target gcp
```

### 模板管理

```bash
# 列出可用模板
lomus template list

# 下载模板
lomus template download --url https://github.com/example/template

# 删除模板
lomus template remove --name template_name
```

## 命令参考

- `init`: 初始化一个新的Lomus AI项目
- `dev`: 启动开发服务器
- `run`: 运行Lomus AI应用
- `build`: 构建Lomus AI应用
- `deploy`: 部署Lomus AI应用
- `template`: 模板管理

## 项目结构

典型的Lomus AI项目结构如下：

```
my_project/
├── Cargo.toml        # Rust项目配置
├── lomusai.toml      # Lomus AI项目配置
├── src/              # 源代码目录
│   └── main.rs       # 入口文件
├── assets/           # 静态资源目录
└── README.md         # 项目说明
```

## 模板类型

Lomus AI CLI支持以下预设模板类型：

- `agent`: Agent模板，适用于构建智能代理应用
- `workflow`: Workflow模板，适用于构建工作流应用
- `rag`: RAG模板，适用于检索增强生成应用
- `custom`: 自定义模板，可以指定自己的模板类型 