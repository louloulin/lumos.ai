import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Plus, Edit, Trash2, Play, PauseCircle, MoreVertical } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { useToast } from '@/components/ui/use-toast';
import { Dialog, DialogContent, DialogTrigger } from '@/components/ui/dialog';

// 工作流定义
interface Workflow {
  id: string;
  name: string;
  description: string;
  isActive: boolean;
  agentCount: number;
  lastRun: string | null;
  tags: string[];
}

// 示例数据
const demoWorkflows: Workflow[] = [
  {
    id: 'customer-support',
    name: '客户服务流程',
    description: '一个完整的客户服务流程，包括初步问题诊断、分类和解决方案推荐。',
    isActive: true,
    agentCount: 3,
    lastRun: '2023-04-01T08:30:00Z',
    tags: ['客服', '自动化'],
  },
  {
    id: 'data-analysis',
    name: '数据分析流程',
    description: '分析用户提供的数据，生成报告并提供洞察。',
    isActive: true,
    agentCount: 2,
    lastRun: '2023-04-02T15:20:00Z',
    tags: ['分析', '报告'],
  },
  {
    id: 'content-creation',
    name: '内容创作流程',
    description: '根据主题和风格要求，自动生成文章、社交媒体内容等。',
    isActive: false,
    agentCount: 4,
    lastRun: null,
    tags: ['创作', '内容'],
  },
];

export default function WorkflowsPage() {
  const { toast } = useToast();
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  
  useEffect(() => {
    // 模拟从API加载工作流
    const loadWorkflows = async () => {
      setIsLoading(true);
      try {
        // 在实际应用中，这里会是API调用
        await new Promise(resolve => setTimeout(resolve, 800)); // 模拟网络延迟
        setWorkflows(demoWorkflows);
      } catch (error) {
        toast({
          title: "加载失败",
          description: "无法加载工作流列表",
          variant: "destructive",
        });
      } finally {
        setIsLoading(false);
      }
    };

    loadWorkflows();
  }, [toast]);

  const handleCreateWorkflow = () => {
    // 在实际应用中，跳转到工作流创建页面或显示创建对话框
    setIsCreateDialogOpen(true);
  };

  const handleToggleStatus = (workflowId: string) => {
    setWorkflows(
      workflows.map(workflow => 
        workflow.id === workflowId 
          ? { ...workflow, isActive: !workflow.isActive } 
          : workflow
      )
    );
    
    // 获取工作流名称
    const workflowName = workflows.find(w => w.id === workflowId)?.name || workflowId;
    
    toast({
      title: `工作流状态已更新`,
      description: `${workflowName} 已${workflows.find(w => w.id === workflowId)?.isActive ? '暂停' : '激活'}`,
    });
  };

  const handleDeleteWorkflow = (workflowId: string) => {
    if (confirm(`确定要删除工作流 "${workflows.find(w => w.id === workflowId)?.name}" 吗？`)) {
      setWorkflows(workflows.filter(w => w.id !== workflowId));
      toast({
        title: "工作流已删除",
        description: "工作流已成功删除",
      });
    }
  };

  const formatDate = (dateString: string | null) => {
    if (!dateString) return '从未运行';
    
    const date = new Date(dateString);
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: 'numeric',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="container py-8">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h1 className="text-3xl font-bold">工作流</h1>
          <p className="text-muted-foreground">创建和管理AI工作流程</p>
        </div>
        <Button onClick={handleCreateWorkflow}>
          <Plus className="mr-2 h-4 w-4" /> 创建工作流
        </Button>
      </div>

      {isLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {[1, 2].map(i => (
            <Card key={i} className="animate-pulse">
              <CardHeader className="pb-2">
                <div className="h-7 bg-muted rounded mb-2"></div>
                <div className="h-4 bg-muted rounded w-3/4"></div>
              </CardHeader>
              <CardContent>
                <div className="h-16 bg-muted rounded"></div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : workflows.length === 0 ? (
        <Card className="text-center p-12">
          <CardContent>
            <p className="text-muted-foreground mb-4">还没有创建任何工作流</p>
            <Button onClick={handleCreateWorkflow}>创建第一个工作流</Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {workflows.map(workflow => (
            <Card key={workflow.id}>
              <CardHeader className="pb-2">
                <div className="flex justify-between items-start">
                  <div>
                    <CardTitle className="text-xl">{workflow.name}</CardTitle>
                    <CardDescription>ID: {workflow.id}</CardDescription>
                  </div>
                  <Badge variant={workflow.isActive ? "default" : "outline"}>
                    {workflow.isActive ? '活跃' : '暂停'}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground line-clamp-2 mb-4">
                  {workflow.description || '没有描述'}
                </p>
                
                <div className="flex gap-2 flex-wrap mb-3">
                  {workflow.tags.map(tag => (
                    <Badge key={tag} variant="secondary" className="text-xs">
                      {tag}
                    </Badge>
                  ))}
                </div>
                
                <div className="grid grid-cols-2 gap-2 text-xs text-muted-foreground mb-4">
                  <div>包含代理: {workflow.agentCount}个</div>
                  <div>上次运行: {formatDate(workflow.lastRun)}</div>
                </div>
                
                <div className="flex justify-between pt-1">
                  <Button variant="outline" size="sm" asChild>
                    <Link to={`/workflows/${workflow.id}`}>
                      <Play className="mr-2 h-4 w-4" /> 查看
                    </Link>
                  </Button>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm">
                        <MoreVertical className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem asChild>
                        <Link to={`/workflows/${workflow.id}/edit`}>
                          <Edit className="mr-2 h-4 w-4" />
                          编辑
                        </Link>
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => handleToggleStatus(workflow.id)}>
                        <PauseCircle className="mr-2 h-4 w-4" />
                        {workflow.isActive ? '暂停' : '激活'}
                      </DropdownMenuItem>
                      <DropdownMenuItem 
                        className="text-destructive focus:text-destructive"
                        onClick={() => handleDeleteWorkflow(workflow.id)}
                      >
                        <Trash2 className="mr-2 h-4 w-4" />
                        删除
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* 工作流创建对话框占位符 - 实际实现中会链接到工作流编辑器 */}
      <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
        <DialogContent className="max-w-md">
          <div className="p-4 text-center">
            <h2 className="text-xl font-semibold mb-4">创建新工作流</h2>
            <p className="text-muted-foreground mb-4">
              在实际应用中，这里将跳转到工作流编辑器或显示工作流创建表单。
            </p>
            <Link to="/workflows/new">
              <Button className="mr-2">前往工作流编辑器</Button>
            </Link>
            <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
              取消
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
} 