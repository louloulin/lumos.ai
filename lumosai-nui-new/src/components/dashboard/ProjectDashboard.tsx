import React, { useEffect, useState } from 'react';
import { Card } from "../ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, TooltipProps } from 'recharts';
import { BarChart2, Calendar, Users, Cpu, Plus, ExternalLink, Info } from 'lucide-react';
import { projectsService, getProjectUsage, Project, UsageStats } from '../../services/projects';
import { Button } from '../ui/button';

// 统计卡片组件
const StatCard = ({ title, value, icon, trend }: { title: string; value: string; icon: React.ReactNode; trend?: string }) => (
  <Card className="p-4 flex flex-col space-y-2 border-[#2E2E2E] bg-[#1C1C1C]">
    <div className="flex justify-between items-center">
      <h3 className="text-sm text-gray-400">{title}</h3>
      <div className="p-1.5 rounded-md bg-[#2E2E2E]">{icon}</div>
    </div>
    <div className="flex items-end justify-between">
      <p className="text-2xl font-medium text-white">{value}</p>
      {trend && <span className={trend.startsWith('+') ? "text-emerald-500 text-xs" : "text-red-500 text-xs"}>{trend}</span>}
    </div>
  </Card>
);

// 项目列表组件
const ProjectList = ({ projects }: { projects: Project[] }) => (
  <div className="space-y-0.5">
    {projects.map((project) => (
      <div key={project.id} className="p-3 hover:bg-[#2E2E2E] cursor-pointer rounded-md group transition-colors">
        <div className="flex justify-between items-center">
          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-medium text-white">{project.name}</h3>
              {project.tags.length > 0 && (
                <div className="flex gap-1">
                  {project.tags.slice(0, 2).map(tag => (
                    <span key={tag} className="px-1.5 py-0.5 rounded text-[10px] border border-[#2E2E2E] bg-[#1C1C1C] text-gray-400">
                      {tag}
                    </span>
                  ))}
                  {project.tags.length > 2 && (
                    <span className="px-1.5 py-0.5 rounded text-[10px] border border-[#2E2E2E] bg-[#1C1C1C] text-gray-400">
                      +{project.tags.length - 2}
                    </span>
                  )}
                </div>
              )}
            </div>
            <p className="text-xs text-gray-500 mt-1">更新于 {new Date(project.updatedAt).toLocaleDateString()}</p>
          </div>
          <div className="flex items-center">
            <span className={`h-6 px-2 py-0.5 rounded text-xs flex items-center ${
              project.status === 'active' ? 'bg-emerald-900/30 text-emerald-500' : 'bg-[#2E2E2E] text-gray-400'
            }`}>
              {project.status}
            </span>
            <Button variant="ghost" size="icon" className="h-8 w-8 rounded-md ml-2 text-gray-400 hover:text-white hover:bg-[#2E2E2E] opacity-0 group-hover:opacity-100 transition-opacity">
              <ExternalLink size={14} />
            </Button>
          </div>
        </div>
      </div>
    ))}
  </div>
);

// 自定义Tooltip组件
const CustomTooltip = ({ active, payload, label }: TooltipProps<number, string>) => {
  if (active && payload && payload.length) {
    return (
      <div className="p-2 bg-[#1C1C1C] border border-[#2E2E2E] rounded-md text-xs shadow-md">
        <p className="text-gray-300 mb-1">{label}</p>
        {payload.map((entry, index) => (
          <p key={index} style={{ color: entry.color }}>
            {entry.name}: {entry.value}
          </p>
        ))}
      </div>
    );
  }
  return null;
};

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
        <CartesianGrid strokeDasharray="3 3" stroke="#2E2E2E" vertical={false} />
        <XAxis dataKey="date" tick={{ fill: '#9CA3AF' }} axisLine={{ stroke: '#2E2E2E' }} />
        <YAxis tick={{ fill: '#9CA3AF' }} axisLine={{ stroke: '#2E2E2E' }} />
        <Tooltip content={<CustomTooltip />} />
        <Line 
          type="monotone" 
          dataKey="tokens" 
          name="Token消耗"
          stroke="#10B981" 
          strokeWidth={2}
          dot={false}
          activeDot={{ r: 6, fill: '#10B981', stroke: '#064E3B', strokeWidth: 2 }} 
        />
        <Line 
          type="monotone" 
          dataKey="requests" 
          name="请求次数"
          stroke="#6366F1" 
          strokeWidth={2}
          dot={false}
          activeDot={{ r: 6, fill: '#6366F1', stroke: '#3730A3', strokeWidth: 2 }} 
        />
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
      <div className="flex justify-center items-center h-[60vh]">
        <div className="animate-spin rounded-full h-8 w-8 border-2 border-emerald-500 border-r-transparent"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-medium text-white">项目仪表盘</h1>
          <p className="text-sm text-gray-400 mt-1">管理和监控您的AI项目</p>
        </div>
        <Button className="h-8 gap-1 bg-emerald-600 hover:bg-emerald-700 text-white border-0">
          <Plus size={14} />
          <span className="text-xs font-medium">新建项目</span>
        </Button>
      </div>
      
      {/* 统计概览 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard 
          title="项目总数" 
          value={projects.length.toString()} 
          icon={<BarChart2 className="text-emerald-500" size={16} />}
          trend="+2 本月" 
        />
        <StatCard 
          title="活跃工作流" 
          value={projects.filter(p => p.status === 'active').length.toString()} 
          icon={<Calendar className="text-emerald-500" size={16} />}
          trend="+3 本周" 
        />
        <StatCard 
          title="团队成员" 
          value="5" 
          icon={<Users className="text-emerald-500" size={16} />}
        />
        <StatCard 
          title="资源用量" 
          value="65%" 
          icon={<Cpu className="text-emerald-500" size={16} />}
          trend="-5% 相比上周" 
        />
      </div>
      
      {/* 主内容区 */}
      <Tabs defaultValue="projects" className="space-y-4">
        <TabsList className="bg-[#1C1C1C] border border-[#2E2E2E] p-0.5 h-9">
          <TabsTrigger value="projects" className="px-4 text-xs h-8 data-[state=active]:bg-[#2E2E2E] data-[state=active]:text-white">项目</TabsTrigger>
          <TabsTrigger value="usage" className="px-4 text-xs h-8 data-[state=active]:bg-[#2E2E2E] data-[state=active]:text-white">使用情况</TabsTrigger>
          <TabsTrigger value="team" className="px-4 text-xs h-8 data-[state=active]:bg-[#2E2E2E] data-[state=active]:text-white">团队活动</TabsTrigger>
        </TabsList>
        
        <TabsContent value="projects" className="mt-6">
          <div className="bg-[#1C1C1C] border border-[#2E2E2E] rounded-md">
            <div className="flex justify-between items-center p-3 border-b border-[#2E2E2E]">
              <h2 className="text-sm font-medium text-white">最近项目</h2>
              <Button variant="ghost" size="sm" className="h-7 text-xs text-gray-400 hover:text-white hover:bg-[#2E2E2E]">
                查看全部
              </Button>
            </div>
            <div className="py-2">
              {projects.length > 0 ? (
                <ProjectList projects={projects} />
              ) : (
                <div className="p-6 text-center">
                  <p className="text-gray-500 text-sm">未找到项目，创建您的第一个项目即可开始。</p>
                  <Button className="mt-4 h-8 gap-1 bg-emerald-600 hover:bg-emerald-700 text-white border-0">
                    <Plus size={14} />
                    <span className="text-xs font-medium">新建项目</span>
                  </Button>
                </div>
              )}
            </div>
          </div>
        </TabsContent>
        
        <TabsContent value="usage" className="mt-6">
          <div className="bg-[#1C1C1C] border border-[#2E2E2E] rounded-md">
            <div className="flex justify-between items-center p-3 border-b border-[#2E2E2E]">
              <h2 className="text-sm font-medium text-white">使用统计</h2>
              <div className="flex items-center">
                <span className="mr-3 text-xs text-gray-400">过去30天</span>
                <Button variant="ghost" size="icon" className="h-7 w-7 text-gray-400 hover:text-white hover:bg-[#2E2E2E]">
                  <Info size={14} />
                </Button>
              </div>
            </div>
            <div className="p-4">
              {usageStats.length > 0 ? (
                <UsageChart data={usageStats} />
              ) : (
                <div className="p-6 text-center">
                  <p className="text-gray-500 text-sm">暂无可用的使用数据。</p>
                </div>
              )}
            </div>
          </div>
        </TabsContent>
        
        <TabsContent value="team" className="mt-6">
          <div className="bg-[#1C1C1C] border border-[#2E2E2E] rounded-md">
            <div className="flex justify-between items-center p-3 border-b border-[#2E2E2E]">
              <h2 className="text-sm font-medium text-white">团队活动</h2>
              <Button variant="ghost" size="sm" className="h-7 text-xs text-gray-400 hover:text-white hover:bg-[#2E2E2E]">
                查看详情
              </Button>
            </div>
            <div className="p-6 text-center">
              <p className="text-gray-500 text-sm">活动信息即将推出...</p>
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
} 