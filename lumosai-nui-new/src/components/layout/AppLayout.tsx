import React, { useState, useEffect } from 'react';
import { useAuth } from '../../services/auth';
import { Button } from '../ui/button';
import { Avatar, AvatarFallback, AvatarImage } from '../ui/avatar';
import { ProjectDashboard } from '../dashboard/ProjectDashboard';
import { ResourceBrowser } from '../resources/ResourceBrowser';
import { WorkflowEditor } from '../workflows/WorkflowEditor';
import { LogOut, User, PlusCircle, Settings, Menu, LayoutDashboard, Database, GitBranch, Rocket, BrainCircuit, Layers, Bot, ChevronRight, Terminal, Code, BarChart, FileText, MessageSquare } from 'lucide-react';
import { CreateProjectForm } from '../projects/CreateProjectForm';
import { useMediaQuery } from '../../hooks/use-media-query';
import { cn } from '../../lib/utils';
import { Link, useLocation, Outlet, useNavigate } from 'react-router-dom';
import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import * as Dialog from '@radix-ui/react-dialog';
import * as Tabs from '@radix-ui/react-tabs';

interface AppLayoutProps {
  children?: React.ReactNode;
}

export function AppLayout({ children }: AppLayoutProps) {
  const { user, logout } = useAuth();
  const [showCreateProject, setShowCreateProject] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [currentTime, setCurrentTime] = useState(new Date());
  const [expandedSection, setExpandedSection] = useState<string | null>(null);
  const isMobile = useMediaQuery('(max-width: 768px)');
  const location = useLocation();
  const navigate = useNavigate();

  // 更新当前时间
  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date());
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  const handleLogout = async () => {
    await logout();
  };

  const handleCreateProjectSuccess = () => {
    setShowCreateProject(false);
  };

  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map(part => part.charAt(0))
      .join('')
      .toUpperCase();
  };

  const getCurrentSection = () => {
    const path = location.pathname.split('/')[1];
    return path || 'dashboard';
  };

  const handleNavigation = (path: string) => {
    navigate(path);
    if (isMobile) {
      setSidebarOpen(false);
    }
  };

  const toggleSection = (section: string) => {
    if (expandedSection === section) {
      setExpandedSection(null);
    } else {
      setExpandedSection(section);
    }
  };

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  const navItems = [
    { id: 'dashboard', label: '仪表盘', icon: <LayoutDashboard className="h-4 w-4" />, path: '/dashboard' },
    { id: 'resources', label: '资源', icon: <Database className="h-4 w-4" />, path: '/resources' },
    { id: 'workflows', label: '工作流', icon: <GitBranch className="h-4 w-4" />, path: '/workflows' },
    { id: 'deployments', label: '部署', icon: <Rocket className="h-4 w-4" />, path: '/deployments' },
  ];

  const toolItems = [
    { id: 'agents', label: '代理', icon: <Bot className="h-3.5 w-3.5" />, path: '/agents' },
    { id: 'prompts', label: '提示库', icon: <MessageSquare className="h-3.5 w-3.5" />, path: '/prompts' },
    { id: 'models', label: '模型', icon: <Layers className="h-3.5 w-3.5" />, path: '/models' },
    { id: 'tools', label: '工具库', icon: <Terminal className="h-3.5 w-3.5" />, path: '/tools' },
  ];

  const Sidebar = () => (
    <div className="h-full w-full flex flex-col bg-gradient-to-b from-[#1A1A1A] to-[#121212] border-r border-[#2E2E2E]">
      <div className="px-3 py-3 flex items-center gap-2 border-b border-[#2E2E2E]">
        <div className="flex items-center justify-center h-7 w-7">
          <BrainCircuit className="h-5 w-5 text-emerald-500" />
        </div>
        <span className="font-medium tracking-tight text-lg text-gray-100">LumosAI</span>
      </div>
      
      <div className="mt-6 px-3 py-1 flex justify-between items-center">
        <span className="text-xs text-gray-400 font-medium tracking-wide">MAIN</span>
        <span className="text-xs text-gray-500 font-mono">{formatTime(currentTime)}</span>
      </div>
      
      <div className="mt-2 px-2 space-y-0.5">
        {navItems.map(item => (
          <Button 
            key={item.id}
            variant="ghost" 
            className={cn(
              "w-full justify-start h-9 px-2 text-sm font-medium",
              getCurrentSection() === item.id ? 
                "bg-gradient-to-r from-emerald-950/70 to-emerald-900/20 text-white border-l-2 border-emerald-500" : 
                "text-gray-400 hover:bg-[#2E2E2E] hover:text-white"
            )}
            onClick={() => handleNavigation(item.path)}
          >
            <span className="mr-3">{item.icon}</span>
            {item.label}
          </Button>
        ))}
      </div>
      
      <div className="mt-6 px-3 py-1">
        <button 
          className="flex justify-between items-center w-full text-xs text-gray-400 font-medium tracking-wide hover:text-white"
          onClick={() => toggleSection('tools')}
        >
          <span>AI TOOLS</span>
          <ChevronRight className={cn(
            "h-3.5 w-3.5 transition-transform duration-200 text-gray-500",
            expandedSection === 'tools' ? "rotate-90" : ""
          )} />
        </button>
      </div>
      
      {expandedSection === 'tools' && (
        <div className="mt-1 px-2 space-y-0.5 animate-in slide-in-from-left-3 duration-150">
          {toolItems.map(item => (
            <Button 
              key={item.id}
              variant="ghost" 
              className={cn(
                "w-full justify-start h-8 px-2 text-[13px]",
                getCurrentSection() === item.id ? 
                  "bg-gradient-to-r from-emerald-950/70 to-emerald-900/20 text-white border-l-2 border-emerald-500" : 
                  "text-gray-400 hover:bg-[#2E2E2E] hover:text-white"
              )}
              onClick={() => handleNavigation(item.path)}
            >
              <span className="mr-3 opacity-70">{item.icon}</span>
              {item.label}
            </Button>
          ))}
        </div>
      )}
      
      <div className="mt-auto">
        <div className="px-3 py-1 mt-6">
          <span className="text-xs text-gray-400 font-medium tracking-wide">WORKSPACE</span>
        </div>
        
        <div className="px-3 py-2">
          <div className="flex items-center justify-between text-sm p-2 rounded-md bg-[#2E2E2E] hover:bg-[#363636] transition-colors">
            <div className="flex items-center gap-2">
              <Avatar className="h-6 w-6 rounded-md">
                <AvatarImage src={user?.avatar} alt={user?.username} className="object-cover" />
                <AvatarFallback className="bg-emerald-800 text-emerald-200 rounded-md text-[11px]">
                  {user ? getInitials(user.username) : 'U'}
                </AvatarFallback>
              </Avatar>
              <div className="flex flex-col">
                <span className="text-sm font-medium text-white leading-none">{user?.username}</span>
                <span className="text-xs text-gray-400 mt-0.5 leading-none">{user?.email}</span>
              </div>
            </div>
            <Button variant="ghost" size="icon" className="h-7 w-7 rounded-md hover:bg-[#444444] text-gray-400 hover:text-white" onClick={handleLogout}>
              <LogOut className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <div className="flex h-screen bg-[#121212] text-gray-100">
      {/* 桌面版侧边栏 */}
      {!isMobile && (
        <div className="hidden md:block w-60 h-screen overflow-hidden">
          <Sidebar />
        </div>
      )}
      
      <div className="flex-1 flex flex-col h-screen overflow-hidden">
        {/* 顶部栏 */}
        <header className="h-14 border-b border-[#2E2E2E] px-4 flex items-center justify-between bg-gradient-to-r from-[#1A1A1A] to-[#121212]">
          <div className="flex items-center gap-2">
            {isMobile && (
              <Button variant="ghost" size="icon" className="md:hidden h-8 w-8 text-gray-400 hover:text-white hover:bg-[#2E2E2E]" onClick={() => setSidebarOpen(!sidebarOpen)}>
                <Menu className="h-5 w-5" />
              </Button>
            )}
            <h1 className={cn("text-lg font-medium tracking-tight text-white", isMobile ? "block" : "hidden")}>LumosAI</h1>
          </div>
          
          <div className="flex items-center gap-4">
            <Button 
              size="sm" 
              className="h-8 gap-1 bg-gradient-to-r from-emerald-600 to-emerald-700 hover:from-emerald-700 hover:to-emerald-800 text-white border-0"
              onClick={() => setShowCreateProject(true)}
            >
              <PlusCircle className="h-3.5 w-3.5" />
              <span className="text-xs font-medium">创建项目</span>
            </Button>
            
            {isMobile && (
              <Button variant="ghost" size="icon" className="h-8 w-8 rounded-md text-gray-400 hover:text-white hover:bg-[#2E2E2E]" onClick={handleLogout}>
                <LogOut className="h-4 w-4" />
              </Button>
            )}
          </div>
        </header>
        
        {/* 移动端侧边栏 */}
        {isMobile && sidebarOpen && (
          <div className="fixed inset-0 z-50 bg-black/80">
            <div className="fixed left-0 top-0 h-full w-60 bg-[#1C1C1C] shadow-lg border-r border-[#2E2E2E]">
              <div className="flex justify-end p-3">
                <Button variant="ghost" size="icon" className="h-8 w-8 text-gray-400 hover:text-white hover:bg-[#2E2E2E]" onClick={() => setSidebarOpen(false)}>
                  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="lucide lucide-x"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                </Button>
              </div>
              <Sidebar />
            </div>
          </div>
        )}
        
        {/* 创建项目对话框 */}
        {showCreateProject && (
          <div className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center">
            <div className="bg-[#1C1C1C] rounded-lg p-6 shadow-xl max-w-[600px] w-full mx-4 border border-[#2E2E2E]">
              <div className="flex justify-between items-center mb-6">
                <h2 className="text-lg font-medium text-white">创建新项目</h2>
                <Button variant="ghost" size="icon" className="h-8 w-8 text-gray-400 hover:text-white hover:bg-[#2E2E2E]" onClick={() => setShowCreateProject(false)}>
                  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="lucide lucide-x"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                </Button>
              </div>
              <CreateProjectForm 
                onSuccess={handleCreateProjectSuccess}
                onCancel={() => setShowCreateProject(false)}
              />
            </div>
          </div>
        )}
        
        {/* 主内容区 */}
        <main className="flex-1 overflow-auto bg-gradient-to-b from-[#121212] to-[#0A0A0A]">
          {children || <Outlet />}
        </main>
      </div>
    </div>
  );
} 