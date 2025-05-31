@echo off
REM DeepSeek Agent 测试脚本 (Windows)
REM 用于验证 DeepSeek 集成是否正常工作

echo 🧪 DeepSeek Agent 测试脚本
echo ==========================

REM 检查是否设置了 API 密钥
if "%DEEPSEEK_API_KEY%"=="" (
    echo ❌ 错误：未设置 DEEPSEEK_API_KEY 环境变量
    echo 请设置您的 DeepSeek API 密钥：
    echo set DEEPSEEK_API_KEY=your-api-key
    echo 或者：
    echo $env:DEEPSEEK_API_KEY="your-api-key"
    pause
    exit /b 1
)

echo ✅ 找到 DeepSeek API 密钥

REM 检查 Rust 环境
cargo --version >nul 2>&1
if errorlevel 1 (
    echo ❌ 错误：未找到 Cargo (Rust 包管理器)
    echo 请安装 Rust: https://rustup.rs/
    pause
    exit /b 1
)

echo ✅ Rust 环境检查通过

REM 进入示例目录
cd /d "%~dp0"

echo 🔧 编译 DeepSeek Agent 示例...

REM 编译示例
cargo build --example deepseek_agent_demo
if errorlevel 1 (
    echo ❌ 编译失败
    pause
    exit /b 1
)

echo ✅ 编译成功

echo 🚀 运行 DeepSeek Agent 演示...
echo 注意：这将调用 DeepSeek API，可能产生费用
echo 按 Ctrl+C 可以随时停止
echo.

REM 运行示例
cargo run --example deepseek_agent_demo

echo.
echo 🎉 测试完成！
pause
