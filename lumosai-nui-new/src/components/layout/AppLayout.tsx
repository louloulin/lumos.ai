import React, { useState, useEffect } from 'react';
import { useAuth } from '../../services/auth';
import { Button } from '../ui/button';
import { Avatar, AvatarFallback, AvatarImage } from '../ui/avatar';
import { ProjectDashboard } from '../dashboard/ProjectDashboard';
import { ResourceBrowser } from '../resources/ResourceBrowser';
import { WorkflowEditor } from '../workflows/WorkflowEditor';
import { LogOut, User, PlusCircle, Settings, Command, Menu, LayoutDashboard, Database, GitBranch, Rocket, BrainCircuit, Layers, Bot, ChevronRight } from 'lucide-react';
import { CreateProjectForm } from '../projects/CreateProjectForm';
import { useMediaQuery } from '../../hooks/use-media-query';
import { cn } from '../../lib/utils';
import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import * as Dialog from '@radix-ui/react-dialog';
import * as Tabs from '@radix-ui/react-tabs';

interface AppLayoutProps {
  children?: React.ReactNode;
}

export function AppLayout({ children }: AppLayoutProps) {
  const { user, logout } = useAuth();
  const [showCreateProject, setShowCreateProject] = useState(false);
  const [activeTab, setActiveTab] = useState('dashboard');
  const isMobile = useMediaQuery('(max-width: 768px)');
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [currentTime, setCurrentTime] = useState(new Date());
  const [expandedSection, setExpandedSection] = useState<string | null>(null);

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

  const handleTabChange = (value: string) => {
    setActiveTab(value);
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

  const Sidebar = () => (
    <div className="h-full w-full flex flex-col bg-gradient-to-b from-background to-background/95 border-r backdrop-blur-sm">
      <div className="p-4 flex items-center gap-2 border-b bg-background/50">
        <div className="h-8 w-8 rounded-md bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-md">
          <BrainCircuit className="h-4 w-4 text-white" />
        </div>
        <span className="font-bold text-lg bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 to-purple-600">LumosAI</span>
      </div>
      
      <div className="px-3 py-4 text-xs text-muted-foreground font-medium">
        <div className="flex justify-between items-center">
          <span>PLATFORM</span>
          <span className="text-xs opacity-70">{formatTime(currentTime)}</span>
        </div>
      </div>
      
      <div className="flex-1 py-2 overflow-auto px-2 space-y-1">
        <Button 
          variant={activeTab === 'dashboard' ? 'secondary' : 'ghost'} 
          className={cn(
            "w-full justify-start transition-all duration-200",
            activeTab === 'dashboard' ? 
              "bg-gradient-to-r from-indigo-500/10 to-purple-600/10 text-primary font-medium" : 
              "hover:bg-gradient-to-r hover:from-indigo-500/5 hover:to-purple-600/5"
          )}
          onClick={() => handleTabChange('dashboard')}
        >
          <LayoutDashboard className="h-4 w-4 mr-2" />
          仪表盘
        </Button>
        <Button 
          variant={activeTab === 'resources' ? 'secondary' : 'ghost'} 
          className={cn(
            "w-full justify-start transition-all duration-200",
            activeTab === 'resources' ? 
              "bg-gradient-to-r from-indigo-500/10 to-purple-600/10 text-primary font-medium" : 
              "hover:bg-gradient-to-r hover:from-indigo-500/5 hover:to-purple-600/5"
          )}
          onClick={() => handleTabChange('resources')}
        >
          <Database className="h-4 w-4 mr-2" />
          资源
        </Button>
        <Button 
          variant={activeTab === 'workflows' ? 'secondary' : 'ghost'} 
          className={cn(
            "w-full justify-start transition-all duration-200",
            activeTab === 'workflows' ? 
              "bg-gradient-to-r from-indigo-500/10 to-purple-600/10 text-primary font-medium" : 
              "hover:bg-gradient-to-r hover:from-indigo-500/5 hover:to-purple-600/5"
          )}
          onClick={() => handleTabChange('workflows')}
        >
          <GitBranch className="h-4 w-4 mr-2" />
          工作流
        </Button>
        <Button 
          variant={activeTab === 'deployments' ? 'secondary' : 'ghost'} 
          className={cn(
            "w-full justify-start transition-all duration-200",
            activeTab === 'deployments' ? 
              "bg-gradient-to-r from-indigo-500/10 to-purple-600/10 text-primary font-medium" : 
              "hover:bg-gradient-to-r hover:from-indigo-500/5 hover:to-purple-600/5"
          )}
          onClick={() => handleTabChange('deployments')}
        >
          <Rocket className="h-4 w-4 mr-2" />
          部署
        </Button>
      </div>
      
      <div className="mt-4 px-3 py-2 text-xs text-muted-foreground font-medium">
        <button 
          className="flex justify-between items-center w-full hover:text-foreground transition-colors"
          onClick={() => toggleSection('tools')}
        >
          <span>AI TOOLS</span>
          <ChevronRight className={cn(
            "h-4 w-4 transition-transform duration-200",
            expandedSection === 'tools' ? "rotate-90" : ""
          )} />
        </button>
      </div>
      
      {expandedSection === 'tools' && (
        <div className="px-2 py-2 space-y-1 animate-in slide-in-from-left-5 duration-200">
          <Button variant="ghost" className="w-full justify-start text-muted-foreground hover:text-foreground transition-colors" size="sm">
            <Bot className="h-3.5 w-3.5 mr-2" />
            <span className="text-sm">代理</span>
          </Button>
          <Button variant="ghost" className="w-full justify-start text-muted-foreground hover:text-foreground transition-colors" size="sm">
            <Layers className="h-3.5 w-3.5 mr-2" />
            <span className="text-sm">模型</span>
          </Button>
        </div>
      )}
      
      <div className="p-4 border-t bg-background/50 backdrop-blur-sm">
        <div className="flex items-center justify-between">
          <div className="flex items-center">
            <div className="relative">
              <Avatar className="h-8 w-8 ring-2 ring-indigo-500/20 ring-offset-1 ring-offset-background">
                <AvatarImage src={user?.avatar} alt={user?.username} className="object-cover" />
                <AvatarFallback className="bg-gradient-to-br from-indigo-500 to-purple-600 text-white">
                  {user ? getInitials(user.username) : 'U'}
                </AvatarFallback>
              </Avatar>
              <span className="absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full bg-green-500 ring-2 ring-background"></span>
            </div>
            <div className="flex flex-col ml-3">
              <span className="text-sm font-medium leading-none">{user?.username}</span>
              <span className="text-xs text-muted-foreground mt-1 leading-none">{user?.email}</span>
            </div>
          </div>
          <Button variant="ghost" size="icon" className="rounded-full h-8 w-8 hover:bg-background" onClick={handleLogout}>
            <LogOut className="h-4 w-4 text-muted-foreground" />
          </Button>
        </div>
      </div>
    </div>
  );

  return (
    <div className="flex h-screen bg-background/95 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-indigo-500/[0.03] via-background to-background">
      {/* 桌面版侧边栏 */}
      {!isMobile && (
        <div className="hidden md:block w-64 h-screen">
          <Sidebar />
        </div>
      )}
      
      <div className="flex-1 flex flex-col h-screen overflow-hidden">
        {/* 顶部栏 */}
        <header className="h-14 border-b bg-background/50 backdrop-blur-md px-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            {isMobile && (
              <Button variant="ghost" size="icon" className="md:hidden" onClick={() => setSidebarOpen(!sidebarOpen)}>
                <Menu className="h-5 w-5" />
              </Button>
            )}
            <h1 className={cn("text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 to-purple-600", isMobile ? "block" : "hidden")}>LumosAI</h1>
          </div>
          
          <div className="flex items-center gap-4">
            <Button 
              size="sm" 
              className="gap-1 bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 text-white border-0 shadow-md hover:shadow-lg transition-shadow"
              onClick={() => setShowCreateProject(true)}
            >
              <PlusCircle className="h-4 w-4" />
              创建项目
            </Button>
            
            {isMobile && (
              <Button variant="ghost" size="icon" onClick={handleLogout}>
                <LogOut className="h-5 w-5" />
              </Button>
            )}
          </div>
        </header>
        
        {/* 移动端侧边栏 */}
        {isMobile && sidebarOpen && (
          <div className="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm">
            <div className="fixed left-0 top-0 h-full w-64 bg-background/95 shadow-lg border-r">
              <div className="flex justify-end p-4">
                <Button variant="ghost" size="icon" onClick={() => setSidebarOpen(false)}>
                  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="lucide lucide-x"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                </Button>
              </div>
              <Sidebar />
            </div>
          </div>
        )}
        
        {/* 创建项目对话框 */}
        {showCreateProject && (
          <div className="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm flex items-center justify-center">
            <div className="bg-background/95 rounded-lg p-6 shadow-xl max-w-[600px] w-full mx-4 border border-indigo-500/10">
              <div className="flex justify-between items-center mb-4">
                <h2 className="text-xl font-semibold bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 to-purple-600">创建新项目</h2>
                <Button variant="ghost" size="icon" className="rounded-full hover:bg-muted" onClick={() => setShowCreateProject(false)}>
                  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="lucide lucide-x h-5 w-5"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
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
        <main className="flex-1 overflow-auto py-6 backdrop-blur-md">
          <div className="container mx-auto px-4">
            {activeTab === 'dashboard' && <ProjectDashboard />}
            {activeTab === 'resources' && <ResourceBrowser />}
            {activeTab === 'workflows' && <WorkflowEditor />}
            {activeTab === 'deployments' && (
              <div className="flex flex-col items-center justify-center h-[400px] border border-indigo-500/10 rounded-lg bg-gradient-to-b from-indigo-500/[0.03] to-background">
                <Rocket className="h-16 w-16 text-indigo-500/40 mb-4" />
                <p className="text-muted-foreground font-medium text-center">部署中心即将推出</p>
                <div className="mt-4">
                  <Button variant="outline" className="text-xs border-indigo-500/20 text-indigo-500">
                    加入公测计划
                  </Button>
                </div>
              </div>
            )}
          </div>
        </main>
      </div>
    </div>
  );
} 