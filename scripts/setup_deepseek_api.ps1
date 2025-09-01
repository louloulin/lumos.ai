# DeepSeek API 设置脚本 (PowerShell)
# 用于快速设置 DeepSeek API Key 环境变量

param(
    [Parameter(Mandatory=$false)]
    [string]$ApiKey,
    
    [Parameter(Mandatory=$false)]
    [switch]$Permanent,
    
    [Parameter(Mandatory=$false)]
    [switch]$Verify,
    
    [Parameter(Mandatory=$false)]
    [switch]$Help
)

function Show-Help {
    Write-Host "🔑 DeepSeek API 设置脚本" -ForegroundColor Cyan
    Write-Host "========================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "用法:" -ForegroundColor Yellow
    Write-Host "  .\setup_deepseek_api.ps1 -ApiKey <your-api-key> [-Permanent] [-Verify]" -ForegroundColor White
    Write-Host ""
    Write-Host "参数:" -ForegroundColor Yellow
    Write-Host "  -ApiKey <key>    设置 DeepSeek API Key" -ForegroundColor White
    Write-Host "  -Permanent       永久设置环境变量（需要管理员权限）" -ForegroundColor White
    Write-Host "  -Verify          验证 API Key 设置" -ForegroundColor White
    Write-Host "  -Help            显示此帮助信息" -ForegroundColor White
    Write-Host ""
    Write-Host "示例:" -ForegroundColor Yellow
    Write-Host "  # 临时设置 API Key" -ForegroundColor Green
    Write-Host "  .\setup_deepseek_api.ps1 -ApiKey sk-your-api-key-here" -ForegroundColor White
    Write-Host ""
    Write-Host "  # 永久设置 API Key" -ForegroundColor Green
    Write-Host "  .\setup_deepseek_api.ps1 -ApiKey sk-your-api-key-here -Permanent" -ForegroundColor White
    Write-Host ""
    Write-Host "  # 验证当前设置" -ForegroundColor Green
    Write-Host "  .\setup_deepseek_api.ps1 -Verify" -ForegroundColor White
    Write-Host ""
    Write-Host "获取 API Key:" -ForegroundColor Yellow
    Write-Host "  1. 访问 https://platform.deepseek.com/" -ForegroundColor White
    Write-Host "  2. 注册并登录账户" -ForegroundColor White
    Write-Host "  3. 在 API 管理页面创建新的 API Key" -ForegroundColor White
    Write-Host ""
}

function Test-ApiKeyFormat {
    param([string]$Key)
    
    if ([string]::IsNullOrEmpty($Key)) {
        return $false
    }
    
    # 检查 API Key 格式
    if ($Key -match "^sk-[a-zA-Z0-9]{32,64}$") {
        return $true
    }
    
    return $false
}

function Set-ApiKey {
    param(
        [string]$Key,
        [bool]$IsPermanent
    )
    
    Write-Host "🔧 设置 DeepSeek API Key..." -ForegroundColor Yellow
    
    # 验证 API Key 格式
    if (-not (Test-ApiKeyFormat -Key $Key)) {
        Write-Host "❌ API Key 格式无效！" -ForegroundColor Red
        Write-Host "   API Key 应该以 'sk-' 开头，后跟 32-64 个字符" -ForegroundColor Red
        return $false
    }
    
    try {
        if ($IsPermanent) {
            Write-Host "⚠️  正在设置永久环境变量（需要管理员权限）..." -ForegroundColor Yellow
            [Environment]::SetEnvironmentVariable("DEEPSEEK_API_KEY", $Key, "User")
            Write-Host "✅ 永久环境变量设置成功！" -ForegroundColor Green
            Write-Host "   重启 PowerShell 后生效" -ForegroundColor Yellow
        } else {
            $env:DEEPSEEK_API_KEY = $Key
            Write-Host "✅ 临时环境变量设置成功！" -ForegroundColor Green
            Write-Host "   仅在当前会话中有效" -ForegroundColor Yellow
        }
        
        # 显示设置的 API Key（部分隐藏）
        $maskedKey = $Key.Substring(0, 8) + "..." + $Key.Substring($Key.Length - 8)
        Write-Host "   API Key: $maskedKey" -ForegroundColor Cyan
        
        return $true
    }
    catch {
        Write-Host "❌ 设置环境变量失败: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

function Test-ApiKeySetup {
    Write-Host "🧪 验证 API Key 设置..." -ForegroundColor Yellow
    
    # 检查环境变量
    $currentKey = $env:DEEPSEEK_API_KEY
    if ([string]::IsNullOrEmpty($currentKey)) {
        Write-Host "❌ DEEPSEEK_API_KEY 环境变量未设置" -ForegroundColor Red
        return $false
    }
    
    # 验证格式
    if (-not (Test-ApiKeyFormat -Key $currentKey)) {
        Write-Host "❌ API Key 格式无效" -ForegroundColor Red
        return $false
    }
    
    # 显示当前设置
    $maskedKey = $currentKey.Substring(0, 8) + "..." + $currentKey.Substring($currentKey.Length - 8)
    Write-Host "✅ API Key 已正确设置: $maskedKey" -ForegroundColor Green
    
    # 检查是否为永久设置
    $userKey = [Environment]::GetEnvironmentVariable("DEEPSEEK_API_KEY", "User")
    if ($userKey -eq $currentKey) {
        Write-Host "✅ 永久环境变量已设置" -ForegroundColor Green
    } else {
        Write-Host "⚠️  仅设置了临时环境变量" -ForegroundColor Yellow
    }
    
    return $true
}

function Test-LumosAIExample {
    Write-Host "🚀 测试 LumosAI 示例..." -ForegroundColor Yellow
    
    # 检查是否在正确的目录
    if (-not (Test-Path "Cargo.toml")) {
        Write-Host "❌ 请在 LumosAI 项目根目录运行此脚本" -ForegroundColor Red
        return $false
    }
    
    # 检查示例文件是否存在
    if (-not (Test-Path "examples/real_deepseek_api_validation.rs")) {
        Write-Host "❌ 找不到真实 API 验证示例文件" -ForegroundColor Red
        return $false
    }
    
    Write-Host "✅ 可以运行以下命令测试 API:" -ForegroundColor Green
    Write-Host "   cargo run --example real_deepseek_api_validation" -ForegroundColor Cyan
    
    return $true
}

# 主逻辑
if ($Help) {
    Show-Help
    exit 0
}

Write-Host "🔑 DeepSeek API 设置脚本" -ForegroundColor Cyan
Write-Host "========================" -ForegroundColor Cyan
Write-Host ""

if ($Verify) {
    $success = Test-ApiKeySetup
    if ($success) {
        Test-LumosAIExample
    }
    exit 0
}

if ([string]::IsNullOrEmpty($ApiKey)) {
    Write-Host "❌ 请提供 API Key" -ForegroundColor Red
    Write-Host "   使用 -Help 参数查看使用说明" -ForegroundColor Yellow
    exit 1
}

# 设置 API Key
$success = Set-ApiKey -Key $ApiKey -IsPermanent $Permanent

if ($success) {
    Write-Host ""
    Write-Host "🎉 设置完成！" -ForegroundColor Green
    Write-Host ""
    
    # 验证设置
    Test-ApiKeySetup | Out-Null
    
    # 提供下一步指导
    Write-Host "📋 下一步:" -ForegroundColor Yellow
    Write-Host "  1. 运行验证示例:" -ForegroundColor White
    Write-Host "     cargo run --example real_deepseek_api_validation" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  2. 查看更多示例:" -ForegroundColor White
    Write-Host "     cargo run --example simple_api_validation" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  3. 阅读文档:" -ForegroundColor White
    Write-Host "     docs/DEEPSEEK_API_SETUP.md" -ForegroundColor Cyan
    Write-Host ""
    
    if (-not $Permanent) {
        Write-Host "💡 提示: 使用 -Permanent 参数可以永久设置环境变量" -ForegroundColor Blue
    }
} else {
    Write-Host ""
    Write-Host "❌ 设置失败，请检查 API Key 格式或权限" -ForegroundColor Red
    exit 1
}
