import React, { useEffect, useState } from 'react';
import { Card } from "../ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { LucideBarChart2, LucideCalendar, LucideUsers, LucideCpu } from 'lucide-react';
import { projectsService, getProjectUsage, Project, UsageStats } from '../../services/projects';

// 统计卡片组件
const StatCard = ({ title, value, icon, trend }: { title: string; value: string; icon: React.ReactNode; trend?: string }) => (
  <Card className="p-4 flex flex-col space-y-2">
    <div className="flex justify-between items-center">
      <h3 className="text-sm text-muted-foreground">{title}</h3>
      <div className="p-2 bg-primary/10 rounded-full">{icon}</div>
    </div>
    <div className="flex items-end justify-between">
      <p className="text-2xl font-bold">{value}</p>
      {trend && <span className={trend.startsWith('+') ? "text-green-500 text-sm" : "text-red-500 text-sm"}>{trend}</span>}
    </div>
  </Card>
);

// 项目列表组件
const ProjectList = ({ projects }: { projects: Project[] }) => (
  <div className="space-y-4">
    {projects.map((project) => (
      <Card key={project.id} className="p-4 hover:bg-secondary/50 cursor-pointer">
        <div className="flex justify-between">
          <div>
            <h3 className="font-medium">{project.name}</h3>
            <p className="text-sm text-muted-foreground">Last updated: {new Date(project.updatedAt).toLocaleDateString()}</p>
          </div>
          <div className="flex flex-col items-end">
            <span className={`px-2 py-1 rounded-full text-xs ${
              project.status === 'active' ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
            }`}>
              {project.status}
            </span>
            <span className="text-xs text-muted-foreground mt-1">{project.tags.length > 0 ? project.tags[0] : 'No tags'}</span>
          </div>
        </div>
      </Card>
    ))}
  </div>
);

// 使用情况图表组件
const UsageChart = ({ data }: { data: UsageStats[] }) => (
  <div className="h-80 w-full">
    <ResponsiveContainer width="100%" height="100%">
      <LineChart
        data={data}
        margin={{
          top: 5,
          right: 30,
          left: 20,
          bottom: 5,
        }}
      >
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="date" />
        <YAxis />
        <Tooltip />
        <Line type="monotone" dataKey="tokens" stroke="#8884d8" activeDot={{ r: 8 }} />
        <Line type="monotone" dataKey="requests" stroke="#82ca9d" />
      </LineChart>
    </ResponsiveContainer>
  </div>
);

// 主仪表盘组件
export function ProjectDashboard() {
  const [loading, setLoading] = useState(true);
  const [projects, setProjects] = useState<Project[]>([]);
  const [usageStats, setUsageStats] = useState<UsageStats[]>([]);

  // 加载项目数据
  useEffect(() => {
    async function loadData() {
      setLoading(true);
      try {
        // 并行获取数据
        const [projectsData, usageData] = await Promise.all([
          projectsService.getProjects(),
          getProjectUsage()
        ]);
        
        setProjects(projectsData);
        setUsageStats(usageData);
      } catch (error) {
        console.error('Failed to load dashboard data:', error);
      } finally {
        setLoading(false);
      }
    }
    
    loadData();
  }, []);

  // 加载状态
  if (loading) {
    return (
      <div className="flex justify-center items-center h-[80vh]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Project Dashboard</h1>
        <button className="bg-primary text-primary-foreground px-4 py-2 rounded-md">New Project</button>
      </div>
      
      {/* 统计概览 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard 
          title="Total Projects" 
          value={projects.length.toString()} 
          icon={<LucideBarChart2 className="text-primary" size={20} />}
          trend="+2 this month" 
        />
        <StatCard 
          title="Active Workflows" 
          value={projects.filter(p => p.status === 'active').length.toString()} 
          icon={<LucideCalendar className="text-primary" size={20} />}
          trend="+3 this week" 
        />
        <StatCard 
          title="Team Members" 
          value="5" 
          icon={<LucideUsers className="text-primary" size={20} />}
        />
        <StatCard 
          title="Computing Usage" 
          value="65%" 
          icon={<LucideCpu className="text-primary" size={20} />}
          trend="-5% from last week" 
        />
      </div>
      
      {/* 主内容区 */}
      <Tabs defaultValue="projects" className="space-y-4">
        <TabsList>
          <TabsTrigger value="projects">Projects</TabsTrigger>
          <TabsTrigger value="usage">Usage</TabsTrigger>
          <TabsTrigger value="team">Team Activity</TabsTrigger>
        </TabsList>
        
        <TabsContent value="projects" className="space-y-4">
          <Card className="p-4">
            <h2 className="text-xl font-semibold mb-4">Recent Projects</h2>
            {projects.length > 0 ? (
              <ProjectList projects={projects} />
            ) : (
              <p className="text-muted-foreground">No projects found. Create your first project to get started.</p>
            )}
          </Card>
        </TabsContent>
        
        <TabsContent value="usage">
          <Card className="p-4">
            <h2 className="text-xl font-semibold mb-4">Usage Statistics</h2>
            {usageStats.length > 0 ? (
              <UsageChart data={usageStats} />
            ) : (
              <p className="text-muted-foreground">No usage data available yet.</p>
            )}
          </Card>
        </TabsContent>
        
        <TabsContent value="team">
          <Card className="p-4">
            <h2 className="text-xl font-semibold mb-4">Team Activity</h2>
            <p className="text-muted-foreground">Activity feed coming soon...</p>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
} 