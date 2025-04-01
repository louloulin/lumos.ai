import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { 
  Card, 
  CardContent, 
  CardDescription, 
  CardHeader, 
  CardTitle 
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { 
  Select, 
  SelectContent, 
  SelectItem, 
  SelectTrigger, 
  SelectValue 
} from '@/components/ui/select';
import { useToast } from '@/components/ui/use-toast';
import { Brain, BookOpen, Info, History, Settings } from 'lucide-react';
import AgentChat from '@/components/agent-chat/agent-chat';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Separator } from '@/components/ui/separator';

// 代理定义
interface Agent {
  id: string;
  name: string;
  description: string;
  capabilities: string[];
  model: string;
  avatar?: string;
}

// 示例代理数据
const demoAgents: Agent[] = [
  {
    id: 'assistant',
    name: '通用助手',
    description: '一个通用的AI助手，可以回答各种问题、提供建议和帮助完成任务。',
    capabilities: ['回答问题', '提供建议', '写作辅助', '信息检索'],
    model: 'GPT-4',
    avatar: '/avatars/assistant.png'
  },
  {
    id: 'researcher',
    name: '研究员',
    description: '专注于帮助用户进行研究、分析数据和提供深入见解的专业代理。',
    capabilities: ['文献研究', '数据分析', '撰写报告', '趋势分析'],
    model: 'Claude-3',
    avatar: '/avatars/researcher.png'
  },
  {
    id: 'customer-service',
    name: '客服代理',
    description: '专门处理客户服务查询，回答产品问题并协助解决客户问题。',
    capabilities: ['问题解答', '故障排除', '退款处理', '账户管理'],
    model: 'GPT-3.5',
    avatar: '/avatars/customer-service.png'
  },
  {
    id: 'code-assistant',
    name: '代码助手',
    description: '帮助开发人员编写、调试和优化代码的专业编程助手。',
    capabilities: ['代码生成', 'Bug修复', '代码解释', '最佳实践建议'],
    model: 'CodeLlama',
    avatar: '/avatars/code-assistant.png'
  }
];

export default function AgentChatPage() {
  const { agentId } = useParams<{ agentId: string }>();
  const navigate = useNavigate();
  const { toast } = useToast();
  
  const [agents, setAgents] = useState<Agent[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState('chat');
  
  // 加载代理数据
  useEffect(() => {
    const loadAgents = async () => {
      setIsLoading(true);
      try {
        // 在实际应用中，这里会是API调用
        await new Promise(resolve => setTimeout(resolve, 600));
        setAgents(demoAgents);
        
        // 如果URL中有agentId，选择对应的代理
        if (agentId) {
          const agent = demoAgents.find(a => a.id === agentId);
          if (agent) {
            setSelectedAgent(agent);
          } else {
            // 代理不存在，选择默认代理
            setSelectedAgent(demoAgents[0]);
            navigate(`/agent-chat/${demoAgents[0].id}`, { replace: true });
          }
        } else if (demoAgents.length > 0) {
          // 如果没有指定代理，选择第一个
          setSelectedAgent(demoAgents[0]);
          navigate(`/agent-chat/${demoAgents[0].id}`, { replace: true });
        }
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
  }, [agentId, navigate, toast]);
  
  // 切换代理
  const handleAgentChange = (agentId: string) => {
    const agent = agents.find(a => a.id === agentId);
    if (agent) {
      setSelectedAgent(agent);
      navigate(`/agent-chat/${agentId}`);
    }
  };
  
  // 处理发送消息
  const handleSendMessage = async (message: string, attachments?: File[]) => {
    // 在实际应用中，这里会调用API发送消息到代理
    console.log(`Sending message to ${selectedAgent?.name}:`, message);
    if (attachments?.length) {
      console.log('With attachments:', attachments);
    }
    
    // 返回成功，让AgentChat组件继续处理响应（这里是模拟行为）
    return Promise.resolve();
  };
  
  return (
    <div className="container py-8">
      <div className="flex flex-col space-y-4 md:flex-row md:space-y-0 md:space-x-4 h-[calc(100vh-180px)]">
        {/* 左侧代理选择和信息面板 */}
        <div className="w-full md:w-80 flex flex-col space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>选择代理</CardTitle>
              <CardDescription>选择要与之交谈的智能代理</CardDescription>
            </CardHeader>
            <CardContent>
              <Select 
                value={selectedAgent?.id} 
                onValueChange={handleAgentChange}
                disabled={isLoading}
              >
                <SelectTrigger>
                  <SelectValue placeholder="选择代理" />
                </SelectTrigger>
                <SelectContent>
                  {agents.map((agent) => (
                    <SelectItem key={agent.id} value={agent.id}>
                      {agent.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </CardContent>
          </Card>
          
          {selectedAgent && (
            <Card className="flex-1">
              <CardHeader>
                <CardTitle>代理信息</CardTitle>
              </CardHeader>
              <CardContent>
                <Tabs value={activeTab} onValueChange={setActiveTab}>
                  <TabsList className="grid w-full grid-cols-3">
                    <TabsTrigger value="about">关于</TabsTrigger>
                    <TabsTrigger value="capabilities">能力</TabsTrigger>
                    <TabsTrigger value="settings">设置</TabsTrigger>
                  </TabsList>
                  
                  <TabsContent value="about" className="space-y-4">
                    <div className="flex items-center gap-2 mt-4">
                      <Info className="h-4 w-4 text-muted-foreground" />
                      <h3 className="text-sm font-medium">基本信息</h3>
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {selectedAgent.description}
                    </p>
                    
                    <Separator />
                    
                    <div className="flex items-center gap-2">
                      <Brain className="h-4 w-4 text-muted-foreground" />
                      <h3 className="text-sm font-medium">模型</h3>
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {selectedAgent.model}
                    </p>
                    
                    <div className="mt-auto pt-4">
                      <Button variant="outline" className="w-full" asChild>
                        <a href={`/agents/${selectedAgent.id}`}>
                          查看完整详情
                        </a>
                      </Button>
                    </div>
                  </TabsContent>
                  
                  <TabsContent value="capabilities" className="space-y-4">
                    <div className="flex items-center gap-2 mt-4">
                      <BookOpen className="h-4 w-4 text-muted-foreground" />
                      <h3 className="text-sm font-medium">代理能力</h3>
                    </div>
                    <ul className="space-y-2">
                      {selectedAgent.capabilities.map((capability, index) => (
                        <li key={index} className="text-sm flex items-center gap-2">
                          <div className="h-1.5 w-1.5 rounded-full bg-primary"></div>
                          {capability}
                        </li>
                      ))}
                    </ul>
                    
                    <Separator />
                    
                    <div className="flex items-center gap-2">
                      <History className="h-4 w-4 text-muted-foreground" />
                      <h3 className="text-sm font-medium">会话历史</h3>
                    </div>
                    <p className="text-sm text-muted-foreground">
                      此代理的会话历史将被保存30天。
                    </p>
                  </TabsContent>
                  
                  <TabsContent value="settings" className="space-y-4">
                    <div className="flex items-center gap-2 mt-4">
                      <Settings className="h-4 w-4 text-muted-foreground" />
                      <h3 className="text-sm font-medium">代理设置</h3>
                    </div>
                    <p className="text-sm text-muted-foreground">
                      更多设置选项将在后续版本中提供。
                    </p>
                  </TabsContent>
                </Tabs>
              </CardContent>
            </Card>
          )}
        </div>
        
        {/* 右侧聊天区域 */}
        <div className="flex-1 h-full">
          {isLoading ? (
            <Card className="h-full">
              <CardContent className="flex items-center justify-center h-full">
                <div className="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
              </CardContent>
            </Card>
          ) : selectedAgent ? (
            <AgentChat 
              agent={selectedAgent}
              onSendMessage={handleSendMessage}
            />
          ) : (
            <Card className="h-full">
              <CardContent className="flex flex-col items-center justify-center h-full text-center">
                <Brain className="h-12 w-12 text-muted-foreground mb-4" />
                <h3 className="font-medium text-lg mb-2">没有选择代理</h3>
                <p className="text-muted-foreground mb-4">
                  请从左侧选择一个代理开始对话
                </p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
} 