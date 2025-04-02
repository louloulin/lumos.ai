import React from 'react';
import { useAuth } from '../../services/auth';
import { Button } from '../ui/button';
import { Avatar, AvatarFallback, AvatarImage } from '../ui/avatar';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { ProjectDashboard } from '../dashboard/ProjectDashboard';
import { ResourceBrowser } from '../resources/ResourceBrowser';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from '../ui/dropdown-menu';
import { LogOut, User, PlusCircle } from 'lucide-react';
import { CreateProjectForm } from '../projects/CreateProjectForm';
import { Dialog, DialogContent, DialogTrigger } from '../ui/dialog';

interface AppLayoutProps {
  children?: React.ReactNode;
}

export function AppLayout({ children }: AppLayoutProps) {
  const { user, logout } = useAuth();
  const [showCreateProject, setShowCreateProject] = React.useState(false);

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

  return (
    <div className="flex flex-col min-h-screen">
      <header className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container flex h-14 items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-bold">LumosAI Platform</h1>
          </div>
          
          <div className="flex items-center gap-4">
            <Dialog open={showCreateProject} onOpenChange={setShowCreateProject}>
              <DialogTrigger asChild>
                <Button size="sm" className="gap-1">
                  <PlusCircle className="h-4 w-4" />
                  创建项目
                </Button>
              </DialogTrigger>
              <DialogContent className="sm:max-w-[600px]">
                <CreateProjectForm 
                  onSuccess={handleCreateProjectSuccess}
                  onCancel={() => setShowCreateProject(false)}
                />
              </DialogContent>
            </Dialog>
            
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" className="relative h-8 w-8 rounded-full">
                  <Avatar className="h-8 w-8">
                    <AvatarImage src={user?.avatar} alt={user?.username} />
                    <AvatarFallback>{user ? getInitials(user.username) : 'U'}</AvatarFallback>
                  </Avatar>
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent className="w-56" align="end" forceMount>
                <DropdownMenuLabel className="font-normal">
                  <div className="flex flex-col space-y-1">
                    <p className="text-sm font-medium leading-none">{user?.username}</p>
                    <p className="text-xs leading-none text-muted-foreground">
                      {user?.email}
                    </p>
                  </div>
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                <DropdownMenuItem>
                  <User className="mr-2 h-4 w-4" />
                  <span>个人资料</span>
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem onClick={handleLogout}>
                  <LogOut className="mr-2 h-4 w-4" />
                  <span>登出</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </header>
      
      <main className="flex-1 container mx-auto p-4">
        <Tabs defaultValue="dashboard" className="w-full">
          <TabsList className="mb-6">
            <TabsTrigger value="dashboard">仪表盘</TabsTrigger>
            <TabsTrigger value="resources">资源</TabsTrigger>
            <TabsTrigger value="workflows">工作流</TabsTrigger>
            <TabsTrigger value="deployments">部署</TabsTrigger>
          </TabsList>
          
          <TabsContent value="dashboard">
            <ProjectDashboard />
          </TabsContent>
          
          <TabsContent value="resources">
            <ResourceBrowser />
          </TabsContent>
          
          <TabsContent value="workflows">
            <div className="flex items-center justify-center h-[400px] border rounded-lg bg-slate-50">
              <p className="text-gray-500">工作流编辑器即将推出</p>
            </div>
          </TabsContent>
          
          <TabsContent value="deployments">
            <div className="flex items-center justify-center h-[400px] border rounded-lg bg-slate-50">
              <p className="text-gray-500">部署中心即将推出</p>
            </div>
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
} 