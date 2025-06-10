@echo off
REM Lumos AI 框架功能演示运行脚本 (Windows 版本)
REM 
REM 此脚本用于运行所有已实现的演示示例
REM 确保在运行前已经正确配置了开发环境

setlocal enabledelayedexpansion

echo 🚀 Lumos AI 框架功能演示
echo =========================
echo.

REM 检查 Rust 环境
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ 错误: 未找到 Cargo。请先安装 Rust 开发环境。
    echo    安装命令: 访问 https://rustup.rs/ 下载安装程序
    pause
    exit /b 1
)

echo ✅ Rust 环境检查通过
for /f "tokens=*" %%i in ('cargo --version') do echo    Cargo 版本: %%i
echo.

REM 检查项目结构
if not exist "Cargo.toml" (
    echo ❌ 错误: 未找到 Cargo.toml 文件。请确保在项目根目录运行此脚本。
    pause
    exit /b 1
)

echo ✅ 项目结构检查通过
echo.

REM 编译项目
echo 🔨 编译项目...
cargo build --examples
if %errorlevel% neq 0 (
    echo ❌ 编译失败。请检查代码错误。
    pause
    exit /b 1
)

echo ✅ 编译成功
echo.

REM 阶段一：基础功能演示
echo 🎯 阶段一：基础功能演示
echo ======================
echo.

call :run_demo "基础 Agent 创建" "basic_agent" "展示如何创建和配置基础的 AI Agent"
call :run_demo "工具集成" "tool_integration" "演示自定义工具创建和 Agent 工具集成"
call :run_demo "记忆系统" "memory_system" "展示对话记忆和记忆管理功能"
call :run_demo "流式响应" "streaming_response" "演示实时流式输出和事件处理"

echo 🎉 阶段一演示完成！
echo.

REM 阶段二：高级功能演示
echo 🎯 阶段二：高级功能演示
echo ======================
echo.

call :run_demo "RAG 系统" "rag_system" "展示检索增强生成系统和知识库构建"
call :run_demo "向量存储" "vector_storage" "演示多种向量存储后端和性能测试"
call :run_demo "多代理工作流" "multi_agent_workflow" "展示复杂的多代理协作和工作流编排"
call :run_demo "事件驱动架构" "event_driven_architecture" "演示事件驱动的代理协作系统"

echo 🎉 阶段二演示完成！
echo.

REM 总结
echo 🏆 所有演示完成！
echo ==================
echo.
echo 已成功运行的功能演示：
echo ✅ 基础 Agent 创建和配置
echo ✅ 工具系统集成
echo ✅ 记忆系统管理
echo ✅ 流式响应处理
echo ✅ RAG 系统构建
echo ✅ 向量存储操作
echo ✅ 多代理工作流编排
echo ✅ 事件驱动架构
echo.
echo 🎯 下一步建议：
echo 1. 查看各个演示的源代码了解实现细节
echo 2. 根据需要修改配置和参数
echo 3. 集成到您的实际项目中
echo 4. 参考 demo.md 文档了解更多功能
echo.
echo 📚 相关文档：
echo - demo.md - 完整功能演示文档
echo - examples/ - 所有演示源代码
echo - lumosai_core/ - 核心框架代码
echo.
echo 感谢使用 Lumos AI 框架！🚀
pause
exit /b 0

REM 运行演示函数
:run_demo
set demo_name=%~1
set demo_file=%~2
set description=%~3

echo 📋 运行演示: %demo_name%
echo    描述: %description%
echo    文件: examples\%demo_file%
echo    ----------------------------------------

cargo run --example %demo_file%
if %errorlevel% neq 0 (
    echo    ❌ %demo_name% 演示失败
    pause
    exit /b 1
)

echo    ✅ %demo_name% 演示完成
echo.
echo    按任意键继续下一个演示...
pause >nul
echo.
goto :eof
