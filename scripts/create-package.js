#!/usr/bin/env node

/**
 * 该脚本用于快速创建新的JavaScript包
 * 用法: node scripts/create-package.js <包名>
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
  process.exit(1);
}

function info(message) {
  log(`ℹ ${message}`, 'blue');
}

// 检查参数
if (process.argv.length < 3) {
  error('请提供包名');
  console.log('用法: node scripts/create-package.js <包名>');
  process.exit(1);
}

// 获取包名参数
const packageName = process.argv[2];
const fullPackageName = packageName.startsWith('@lumosai/') 
  ? packageName 
  : `@lumosai/${packageName}`;

// 包目录
const packageDir = path.join(
  process.cwd(), 
  'packages', 
  packageName.startsWith('@lumosai/') 
    ? packageName.split('/')[1] 
    : packageName
);

// 检查包是否已存在
if (fs.existsSync(packageDir)) {
  error(`包 ${fullPackageName} 已存在，请使用其他名称`);
  process.exit(1);
}

// 创建包目录结构
info(`创建包目录: ${packageDir}`);
fs.mkdirSync(packageDir, { recursive: true });
fs.mkdirSync(path.join(packageDir, 'src'), { recursive: true });
fs.mkdirSync(path.join(packageDir, 'dist'), { recursive: true });
fs.mkdirSync(path.join(packageDir, 'tests'), { recursive: true });

// 创建package.json
const packageJson = {
  name: fullPackageName,
  version: '0.1.0',
  description: `LumosAI ${packageName} package`,
  main: 'dist/index.js',
  module: 'dist/index.mjs',
  types: 'dist/index.d.ts',
  files: ['dist'],
  scripts: {
    build: 'bun build ./src/index.ts --outdir dist --target node',
    dev: 'bun build ./src/index.ts --outdir dist --target node --watch',
    lint: 'bun x eslint src --ext .ts',
    test: 'bun test',
    'test:watch': 'bun test --watch'
  },
  keywords: ['lumosai', 'ai', packageName],
  author: 'LumosAI',
  license: 'MIT',
  devDependencies: {
    'bun-types': 'latest',
    'eslint': '^8.56.0',
    'typescript': '^5.3.3'
  },
  dependencies: {}
};

info('创建package.json');
fs.writeFileSync(
  path.join(packageDir, 'package.json'),
  JSON.stringify(packageJson, null, 2)
);

// 创建tsconfig.json
const tsConfig = {
  compilerOptions: {
    target: 'ES2020',
    module: 'ESNext',
    moduleResolution: 'node',
    esModuleInterop: true,
    declaration: true,
    outDir: './dist',
    strict: true
  },
  include: ['src/**/*'],
  exclude: ['node_modules', 'dist', 'tests']
};

info('创建tsconfig.json');
fs.writeFileSync(
  path.join(packageDir, 'tsconfig.json'),
  JSON.stringify(tsConfig, null, 2)
);

// 创建入口文件
const indexContent = `/**
 * LumosAI ${fullPackageName}
 */

export function hello(name: string): string {
  return \`Hello, \${name}! Welcome to LumosAI ${fullPackageName}\`;
}

export default {
  hello
};
`;

info('创建入口文件');
fs.writeFileSync(path.join(packageDir, 'src', 'index.ts'), indexContent);

// 创建测试文件
const testContent = `import { test, expect } from 'bun:test';
import { hello } from '../src/index';

test('基本功能', () => {
  expect(hello('world')).toContain('Hello, world!');
});
`;

info('创建测试文件');
fs.writeFileSync(path.join(packageDir, 'tests', 'index.test.ts'), testContent);

// 创建README
const readmeContent = `# ${fullPackageName}

${packageJson.description}

## 安装

\`\`\`bash
pnpm add ${fullPackageName}
\`\`\`

## 使用

\`\`\`typescript
import { hello } from '${fullPackageName}';

console.log(hello('world'));
\`\`\`

## 开发

\`\`\`bash
# 构建
pnpm build:${packageName.startsWith('@lumosai/') ? packageName.split('/')[1] : packageName}

# 开发模式 (监视文件变化)
pnpm dev:${packageName.startsWith('@lumosai/') ? packageName.split('/')[1] : packageName}

# 测试
pnpm --filter ${fullPackageName} test
\`\`\`
`;

info('创建README.md');
fs.writeFileSync(path.join(packageDir, 'README.md'), readmeContent);

// 创建.gitignore
const gitignoreContent = `node_modules/
dist/
*.log
.DS_Store
`;

fs.writeFileSync(path.join(packageDir, '.gitignore'), gitignoreContent);

// 更新root package.json的scripts
try {
  const rootPackageJsonPath = path.join(process.cwd(), 'package.json');
  const rootPackageJson = JSON.parse(fs.readFileSync(rootPackageJsonPath, 'utf8'));
  
  const shortName = packageName.startsWith('@lumosai/') ? packageName.split('/')[1] : packageName;
  
  if (!rootPackageJson.scripts) {
    rootPackageJson.scripts = {};
  }
  
  rootPackageJson.scripts[`dev:${shortName}`] = `pnpm --filter ${fullPackageName} dev`;
  rootPackageJson.scripts[`build:${shortName}`] = `pnpm --filter ${fullPackageName} build`;
  
  fs.writeFileSync(rootPackageJsonPath, JSON.stringify(rootPackageJson, null, 2));
  info('已更新根package.json的scripts');
  
} catch (err) {
  error(`更新根package.json失败: ${err.message}`);
}

// 安装依赖
info('安装依赖');
try {
  execSync('pnpm install', { stdio: 'inherit' });
  success('依赖安装成功');
} catch (err) {
  error(`依赖安装失败: ${err.message}`);
}

success(`包 ${fullPackageName} 创建成功!`);
info(`目录: ${packageDir}`);
info(`可以使用以下命令开始开发:`);
log(`  pnpm dev:${packageName.startsWith('@lumosai/') ? packageName.split('/')[1] : packageName}`);
log(`  pnpm build:${packageName.startsWith('@lumosai/') ? packageName.split('/')[1] : packageName}`); 