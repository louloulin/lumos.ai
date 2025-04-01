import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Plus, Edit, Trash2, MessageSquare, MoreVertical } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useToast } from '@/components/ui/use-toast';
import { Dialog, DialogContent, DialogTrigger } from '@/components/ui/dialog';
import { Badge } from '@/components/ui/badge';
import { Agent } from '@/types';
import { AgentEditor } from '@/components/agent-editor';

// 示例数据
const demoAgents: Agent[] = [
  {
    id: 'assistant',
    name: '通用助手',
    description: '一个通用的AI助手，可以回答问题、提供建议，帮助完成各种任务。',
    model: 'openai/gpt-4',
    systemPrompt: '你是一个有用的AI助手，总是提供有帮助、安全且真实的回答。',
    temperature: 0.7,
    maxTokens: 2000,
    tools: [],
    isActive: true,
  },
  {
    id: 'researcher',
    name: '研究员',
    description: '专注于帮助用户进行研究，提供深入分析和见解的AI代理。',
    model: 'anthropic/claude-3-opus',
    systemPrompt: '你是一个研究助手，专注于提供深入、全面和准确的信息。对于任何问题，你都会尝试从多个角度进行分析，并提供相关的背景信息。',
    temperature: 0.3,
    maxTokens: 4000,
    tools: [],
    isActive: true,
  },
];

export default function AgentsPage() {
  const { toast } = useToast();
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedAgent, setSelectedAgent] = useState<Agent | undefined>(undefined);
  const [isEditorOpen, setIsEditorOpen] = useState(false);

  useEffect(() => {
    // 模拟从API加载代理
    const loadAgents = async () => {
      setIsLoading(true);
      try {
        // 在实际应用中，这里会是API调用
        await new Promise(resolve => setTimeout(resolve, 800)); // 模拟网络延迟
        setAgents(demoAgents);
      } catch (error) {
        toast({
          title: "加载失败",
          description: "无法加载代理列表",
          variant: "destructive",
        });
      } finally {
        setIsLoading(false);
      }
    };

    loadAgents();
  }, [toast]);

  const handleCreateAgent = () => {
    setSelectedAgent(undefined);
    setIsEditorOpen(true);
  };

  const handleEditAgent = (agent: Agent) => {
    setSelectedAgent(agent);
    setIsEditorOpen(true);
  };

  const handleSaveAgent = (agent: Agent) => {
    setIsEditorOpen(false);
    
    if (selectedAgent) {
      // 更新现有代理
      setAgents(agents.map(a => a.id === agent.id ? agent : a));
    } else {
      // 添加新代理
      setAgents([...agents, agent]);
    }
  };

  const handleDeleteAgent = (agentId: string) => {
    if (confirm(`确定要删除代理 "${agents.find(a => a.id === agentId)?.name}" 吗？`)) {
      setAgents(agents.filter(a => a.id !== agentId));
      toast({
        title: "代理已删除",
        description: "代理已成功删除",
      });
    }
  };

  return (
    <div className="container py-8">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h1 className="text-3xl font-bold">代理</h1>
          <p className="text-muted-foreground">创建和管理你的AI代理</p>
        </div>
        <Button onClick={handleCreateAgent}>
          <Plus className="mr-2 h-4 w-4" /> 创建代理
        </Button>
      </div>

      {isLoading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {[1, 2, 3].map(i => (
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
      ) : agents.length === 0 ? (
        <Card className="text-center p-12">
          <CardContent>
            <p className="text-muted-foreground mb-4">还没有创建任何代理</p>
            <Button onClick={handleCreateAgent}>创建第一个代理</Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {agents.map(agent => (
            <Card key={agent.id}>
              <CardHeader className="pb-2">
                <div className="flex justify-between items-start">
                  <div>
                    <CardTitle className="text-xl">{agent.name}</CardTitle>
                    <CardDescription>ID: {agent.id}</CardDescription>
                  </div>
                  <Badge variant={agent.isActive ? "default" : "outline"}>
                    {agent.isActive ? '已启用' : '已禁用'}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground line-clamp-3 mb-4">
                  {agent.description || '没有描述'}
                </p>
                <div className="flex gap-2 items-center text-xs text-muted-foreground mb-4">
                  <span className="bg-primary/10 rounded-full px-2 py-1">{agent.model.split('/')[0]}</span>
                  <span>温度: {agent.temperature}</span>
                </div>
                <div className="flex justify-between pt-1">
                  <Button variant="outline" size="sm" asChild>
                    <Link to={`/agents/${agent.id}/chat`}>
                      <MessageSquare className="mr-2 h-4 w-4" /> 对话
                    </Link>
                  </Button>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm">
                        <MoreVertical className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem onClick={() => handleEditAgent(agent)}>
                        <Edit className="mr-2 h-4 w-4" />
                        编辑
                      </DropdownMenuItem>
                      <DropdownMenuItem 
                        className="text-destructive focus:text-destructive"
                        onClick={() => handleDeleteAgent(agent.id)}
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

      <Dialog open={isEditorOpen} onOpenChange={setIsEditorOpen}>
        <DialogContent className="max-w-3xl">
          <AgentEditor 
            agent={selectedAgent}
            onSave={handleSaveAgent}
            onCancel={() => setIsEditorOpen(false)}
          />
        </DialogContent>
      </Dialog>
    </div>
  );
} 