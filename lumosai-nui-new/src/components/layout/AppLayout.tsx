import React, { useState } from 'react';
import { useAuth } from '../../services/auth';
import { Button } from '../ui/button';
import { Avatar, AvatarFallback, AvatarImage } from '../ui/avatar';
import { ProjectDashboard } from '../dashboard/ProjectDashboard';
import { ResourceBrowser } from '../resources/ResourceBrowser';
import { WorkflowEditor } from '../workflows/WorkflowEditor';
import { LogOut, User, PlusCircle, Settings, Command, Menu, LayoutDashboard, Database, GitBranch, Rocket } from 'lucide-react';
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

  const Sidebar = () => (
    <div className="h-full w-full flex flex-col bg-background border-r">
      <div className="p-4 flex items-center gap-2 border-b">
        <div className="h-8 w-8 rounded-md bg-primary flex items-center justify-center">
          <Command className="h-4 w-4 text-primary-foreground" />
        </div>
        <span className="font-semibold text-lg">LumosAI</span>
      </div>
      
      <div className="flex-1 py-4 overflow-auto">
        <div className="px-2 space-y-1">
          <Button 
            variant={activeTab === 'dashboard' ? 'secondary' : 'ghost'} 
            className="w-full justify-start" 
            onClick={() => handleTabChange('dashboard')}
          >
            <LayoutDashboard className="h-4 w-4 mr-2" />
            仪表盘
          </Button>
          <Button 
            variant={activeTab === 'resources' ? 'secondary' : 'ghost'} 
            className="w-full justify-start" 
            onClick={() => handleTabChange('resources')}
          >
            <Database className="h-4 w-4 mr-2" />
            资源
          </Button>
          <Button 
            variant={activeTab === 'workflows' ? 'secondary' : 'ghost'} 
            className="w-full justify-start" 
            onClick={() => handleTabChange('workflows')}
          >
            <GitBranch className="h-4 w-4 mr-2" />
            工作流
          </Button>
          <Button 
            variant={activeTab === 'deployments' ? 'secondary' : 'ghost'} 
            className="w-full justify-start" 
            onClick={() => handleTabChange('deployments')}
          >
            <Rocket className="h-4 w-4 mr-2" />
            部署
          </Button>
        </div>
      </div>
      
      <div className="p-4 border-t">
        <div className="flex items-center justify-between">
          <div className="flex items-center">
            <Avatar className="h-6 w-6 mr-2">
              <AvatarImage src={user?.avatar} alt={user?.username} />
              <AvatarFallback>{user ? getInitials(user.username) : 'U'}</AvatarFallback>
            </Avatar>
            <div className="flex flex-col">
              <span className="text-sm font-medium">{user?.username}</span>
              <span className="text-xs text-muted-foreground">{user?.email}</span>
            </div>
          </div>
          <Button variant="ghost" size="icon" onClick={handleLogout}>
            <LogOut className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );

  return (
    <div className="flex h-screen bg-background">
      {/* 桌面版侧边栏 */}
      {!isMobile && (
        <div className="hidden md:block w-64 h-screen">
          <Sidebar />
        </div>
      )}
      
      <div className="flex-1 flex flex-col h-screen overflow-hidden">
        {/* 顶部栏 */}
        <header className="h-14 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 px-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            {isMobile && (
              <Button variant="ghost" size="icon" className="md:hidden" onClick={() => setSidebarOpen(!sidebarOpen)}>
                <Menu className="h-5 w-5" />
              </Button>
            )}
            <h1 className={cn("text-xl font-bold", isMobile ? "block" : "hidden")}>LumosAI Platform</h1>
          </div>
          
          <div className="flex items-center gap-4">
            <Button size="sm" className="gap-1" onClick={() => setShowCreateProject(true)}>
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
            <div className="fixed left-0 top-0 h-full w-64 bg-background shadow-lg">
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
            <div className="bg-background rounded-lg p-6 shadow-lg max-w-[600px] w-full mx-4">
              <div className="flex justify-between items-center mb-4">
                <h2 className="text-xl font-semibold">创建新项目</h2>
                <Button variant="ghost" size="icon" onClick={() => setShowCreateProject(false)}>
                  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="lucide lucide-x"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
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
        <main className="flex-1 overflow-auto">
          <div className="container mx-auto p-4">
            {activeTab === 'dashboard' && <ProjectDashboard />}
            {activeTab === 'resources' && <ResourceBrowser />}
            {activeTab === 'workflows' && <WorkflowEditor />}
            {activeTab === 'deployments' && (
              <div className="flex items-center justify-center h-[400px] border rounded-lg bg-slate-50">
                <p className="text-gray-500">部署中心即将推出</p>
              </div>
            )}
          </div>
        </main>
      </div>
    </div>
  );
} 