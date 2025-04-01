#!/usr/bin/env node

/**
 * 该脚本用于验证工作区设置是否正确，
 * 并确保client-js和UI包可以正确相互引用
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// 彩色日志输出
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m'
};

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

function success(message) {
  log(`✓ ${message}`, 'green');
}

function error(message) {
  log(`✗ ${message}`, 'red');
  process.exitCode = 1;
}

function info(message) {
  log(`ℹ ${message}`, 'blue');
}

// 验证工作区结构
function verifyWorkspaceStructure() {
  info('验证工作区结构...');
  
  // 检查pnpm-workspace.yaml
  if (!fs.existsSync(path.join(process.cwd(), 'pnpm-workspace.yaml'))) {
    error('找不到pnpm-workspace.yaml配置文件');
    return false;
  }
  
  // 检查package.json中的工作区配置
  try {
    const packageJson = JSON.parse(fs.readFileSync(path.join(process.cwd(), 'package.json'), 'utf8'));
    if (!packageJson.workspaces || !Array.isArray(packageJson.workspaces)) {
      error('package.json中没有正确配置workspaces字段');
      return false;
    }
    success('工作区配置正确');
  } catch (err) {
    error(`读取package.json失败: ${err.message}`);
    return false;
  }
  
  return true;
}

// 验证客户端包
function verifyClientJs() {
  info('验证client-js包...');
  
  const clientJsDir = path.join(process.cwd(), 'packages', 'client-js');
  const rootDir = process.cwd();
  
  if (!fs.existsSync(clientJsDir)) {
    error('找不到client-js包目录');
    return false;
  }
  
  // 检查package.json
  try {
    const packageJson = JSON.parse(fs.readFileSync(path.join(clientJsDir, 'package.json'), 'utf8'));
    if (packageJson.name !== '@lumosai/client-js') {
      error(`client-js包名称错误: ${packageJson.name} (应为 @lumosai/client-js)`);
      return false;
    }
    success('client-js包配置正确');
  } catch (err) {
    error(`读取client-js的package.json失败: ${err.message}`);
    return false;
  }
  
  // 检查是否可以构建
  try {
    info('尝试构建client-js包...');
    process.chdir(clientJsDir);
    execSync('bun run build', { stdio: 'inherit' });
    success('client-js构建成功');
    // 切回根目录
    process.chdir(rootDir);
  } catch (err) {
    error(`构建client-js失败: ${err.message}`);
    // 确保出错时也切回根目录
    process.chdir(rootDir);
    return false;
  }
  
  return true;
}

// 验证UI包
function verifyUi() {
  info('验证UI包...');
  
  const uiDir = path.join(process.cwd(), 'lumosai_ui');
  
  info(`检查UI目录: ${uiDir}`);
  if (!fs.existsSync(uiDir)) {
    error(`找不到lumosai_ui目录: ${uiDir}`);
    // 列出当前目录内容帮助调试
    try {
      const files = fs.readdirSync(process.cwd());
      info('当前目录内容:');
      files.forEach(file => log(` - ${file}`, 'yellow'));
    } catch (err) {
      error(`无法列出目录内容: ${err.message}`);
    }
    return false;
  }
  
  // 检查package.json
  try {
    const packageJson = JSON.parse(fs.readFileSync(path.join(uiDir, 'package.json'), 'utf8'));
    if (packageJson.name !== '@lumosai/playground-ui') {
      error(`UI包名称错误: ${packageJson.name} (应为 @lumosai/playground-ui)`);
      return false;
    }
    
    // 验证依赖项引用
    const clientJsDep = packageJson.dependencies['@lumosai/client-js'];
    if (!clientJsDep) {
      error('UI包中没有依赖@lumosai/client-js');
      return false;
    }
    
    if (clientJsDep !== 'workspace:*') {
      error(`UI包中@lumosai/client-js引用不正确: ${clientJsDep} (应为 workspace:*)`);
      return false;
    }
    
    success('UI包配置正确');
  } catch (err) {
    error(`读取UI的package.json失败: ${err.message}`);
    return false;
  }
  
  return true;
}

// 主函数
async function main() {
  log('LumosAI Workspace 验证', 'cyan');
  log('======================', 'cyan');
  
  const structureOk = verifyWorkspaceStructure();
  if (!structureOk) {
    error('工作区结构验证失败，请修复上述问题');
    return;
  }
  
  const clientJsOk = verifyClientJs();
  const uiOk = verifyUi();
  
  if (clientJsOk && uiOk) {
    success('\n工作区配置验证通过！\n');
    info('你可以使用以下命令开发项目:');
    log('  - pnpm dev:ui         # 开发UI');
    log('  - pnpm dev:client     # 开发客户端库');
    log('  - pnpm build:all      # 构建所有包');
  } else {
    error('\n工作区配置验证失败，请修复上述问题\n');
  }
}

// 运行主函数
main().catch(err => {
  error(`出现未预期的错误: ${err.message}`);
  process.exit(1);
}); 