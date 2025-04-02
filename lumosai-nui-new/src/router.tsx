import React from 'react';
import { createBrowserRouter, Navigate } from 'react-router-dom';
import { AppLayout } from './components/layout/AppLayout';
import { ProjectDashboard } from './components/dashboard/ProjectDashboard';
import { ResourceBrowser } from './components/resources/ResourceBrowser';
import { WorkflowEditor } from './components/workflows/WorkflowEditor';
import { PromptList, PromptEditor, PromptHistory } from './components/prompts';
import { AuthPage } from './components/auth/AuthPage';
import App from './App';

// 定义404页面
const NotFound = () => (
  <div className="container mx-auto py-12 px-4 text-center">
    <div className="mb-6">
      <h1 className="text-4xl font-bold mb-2">404</h1>
      <p className="text-xl text-gray-400">页面未找到</p>
    </div>
    <p className="mb-6 text-gray-500">您访问的页面不存在或已被移除。</p>
    <button 
      onClick={() => window.history.back()}
      className="bg-gray-800 hover:bg-gray-700 text-white px-6 py-2 rounded-md text-sm"
    >
      返回上一页
    </button>
  </div>
);

// 定义即将推出的功能页面
const ComingSoon = ({ title, icon }: { title: string, icon: React.ReactNode }) => (
  <div className="container mx-auto py-6 px-6">
    <div className="flex flex-col items-center justify-center h-[400px] border border-[#2E2E2E] rounded-md bg-gradient-to-b from-[#1C1C1C] to-[#121212] backdrop-blur-sm">
      <div className="p-6 flex flex-col items-center">
        <div className="w-16 h-16 rounded-full bg-black/30 flex items-center justify-center mb-4 border border-[#2E2E2E] shadow-lg">
          {icon}
        </div>
        <p className="text-xl font-medium mb-2 text-white">{title}即将推出</p>
        <p className="text-gray-400 text-sm text-center max-w-md mb-8">
          我们正在构建{title}功能，敬请期待。该功能将为LumosAI平台提供强大的AI能力扩展。
        </p>
        <div className="flex gap-4">
          <button className="h-9 px-5 text-sm border border-[#2E2E2E] bg-black/30 text-gray-300 hover:bg-[#2E2E2E] rounded-md transition-colors">
            查看文档
          </button>
          <button className="h-9 px-5 text-sm bg-gradient-to-r from-emerald-600 to-emerald-700 hover:from-emerald-700 hover:to-emerald-800 text-white rounded-md shadow-md transition-colors">
            加入公测计划
          </button>
        </div>
      </div>
    </div>
  </div>
);

// SVG图标组件
const GitBranchIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-emerald-500">
    <line x1="6" x2="6" y1="3" y2="15"></line>
    <circle cx="18" cy="6" r="3"></circle>
    <circle cx="6" cy="18" r="3"></circle>
    <path d="M18 9a9 9 0 0 1-9 9"></path>
  </svg>
);

const BotIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-emerald-500">
    <path d="M12 8V4H8"></path>
    <rect width="16" height="12" x="4" y="8" rx="2"></rect>
    <path d="M2 14h2"></path>
    <path d="M20 14h2"></path>
    <path d="M15 13v2"></path>
    <path d="M9 13v2"></path>
  </svg>
);

const TerminalIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-emerald-500">
    <polyline points="4 17 10 11 4 5"></polyline>
    <line x1="12" x2="20" y1="19" y2="19"></line>
  </svg>
);

const LayersIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-emerald-500">
    <path d="m12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z"></path>
    <path d="m22 17.65-9.17 4.16a2 2 0 0 1-1.66 0L2 17.65"></path>
    <path d="m22 12.65-9.17 4.16a2 2 0 0 1-1.66 0L2 12.65"></path>
  </svg>
);

const RocketIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-emerald-500">
    <path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"></path>
    <path d="m12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"></path>
    <path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"></path>
    <path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"></path>
  </svg>
);

const SettingsIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-emerald-500">
    <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"></path>
    <circle cx="12" cy="12" r="3"></circle>
  </svg>
);

// 应用路由配置
const router = createBrowserRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      { index: true, element: <Navigate to="/dashboard" replace /> },
      { path: 'dashboard', element: <ProjectDashboard /> },
      { path: 'resources', element: <ResourceBrowser /> },
      
      // 提示工程系统路由
      { path: 'prompts', element: <PromptList /> },
      { path: 'prompts/new', element: <PromptEditor /> },
      { path: 'prompts/:id', element: <PromptEditor /> },
      { path: 'prompts/history', element: <PromptHistory /> },
      
      // 工作流路由
      { path: 'workflows', element: <ComingSoon title="工作流管理" icon={<GitBranchIcon />} /> },
      { path: 'workflows/:id', element: <WorkflowEditor /> },
      
      // 其他功能路由 - 即将推出
      { path: 'agents', element: <ComingSoon title="代理构建" icon={<BotIcon />} /> },
      { path: 'tools', element: <ComingSoon title="工具库" icon={<TerminalIcon />} /> },
      { path: 'models', element: <ComingSoon title="模型管理" icon={<LayersIcon />} /> },
      { path: 'deployments', element: <ComingSoon title="部署中心" icon={<RocketIcon />} /> },
      
      // 设置路由
      { path: 'settings', element: <ComingSoon title="账户设置" icon={<SettingsIcon />} /> },
      
      // 找不到页面的回退路由
      { path: '*', element: <NotFound /> }
    ]
  },
  {
    path: '/auth',
    element: <AuthPage />
  }
]);

export default router; 