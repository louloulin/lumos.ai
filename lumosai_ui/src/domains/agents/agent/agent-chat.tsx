import { useState, useEffect } from 'react';
import { Send, ChevronDown, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui/sheet';
import { Badge } from '@/components/ui/badge';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { useToast } from '@/components/ui/use-toast';
import { LumosAIClient } from '@lumosai/client-js';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

interface AgentChatProps {
  agentId: string;
  agentName: string;
  apiUrl?: string;
}

export function AgentChat({ agentId, agentName, apiUrl }: AgentChatProps) {
  const { toast } = useToast();
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [client, setClient] = useState<LumosAIClient | null>(null);
  
  useEffect(() => {
    // 初始化客户端
    try {
      const apiKey = localStorage.getItem('lumosai_api_key') || 'demo-key';
      const lumosClient = new LumosAIClient({
        apiKey,
        baseUrl: apiUrl || 'https://api.lumosai.com',
      });
      setClient(lumosClient);
    } catch (error) {
      console.error('初始化客户端失败:', error);
      toast({
        title: "连接失败",
        description: "无法连接到LumosAI API",
        variant: "destructive",
      });
    }
  }, [apiUrl, toast]);

  const handleSendMessage = async () => {
    if (!input.trim() || !client) return;
    
    // 创建用户消息
    const userMessage: Message = {
      id: `msg-${Date.now()}-user`,
      role: 'user',
      content: input,
      timestamp: new Date(),
    };
    
    // 更新消息列表
    setMessages(prev => [...prev, userMessage]);
    setInput('');
    setIsLoading(true);
    
    try {
      // 获取代理实例
      const agent = client.getAgent(agentId);
      
      // 发送请求
      try {
        // 使用流式响应
        const stream = await agent.stream(input);
        
        // 创建助理消息
        const assistantMessage: Message = {
          id: `msg-${Date.now()}-assistant`,
          role: 'assistant',
          content: '',
          timestamp: new Date(),
        };
        
        // 添加初始空消息
        setMessages(prev => [...prev, assistantMessage]);
        
        // 处理流式响应
        await stream.processDataStream({
          onTextPart: (text) => {
            // 更新助理消息内容
            setMessages(prev => {
              const newMessages = [...prev];
              const lastMessage = newMessages[newMessages.length - 1];
              if (lastMessage.role === 'assistant') {
                lastMessage.content += text;
              }
              return newMessages;
            });
          },
          onErrorPart: (error) => {
            toast({
              title: "响应错误",
              description: error.message,
              variant: "destructive",
            });
          }
        });
      } catch (error) {
        // 如果流式响应失败，尝试普通响应
        const response = await agent.generate(input);
        
        // 创建助理消息
        const assistantMessage: Message = {
          id: `msg-${Date.now()}-assistant`,
          role: 'assistant',
          content: response.message.content,
          timestamp: new Date(),
        };
        
        setMessages(prev => [...prev, assistantMessage]);
      }
    } catch (error) {
      console.error('发送消息失败:', error);
      
      // 使用演示模式
      setTimeout(() => {
        const assistantMessage: Message = {
          id: `msg-${Date.now()}-assistant`,
          role: 'assistant',
          content: generateDemoResponse(input, agentName),
          timestamp: new Date(),
        };
        
        setMessages(prev => [...prev, assistantMessage]);
      }, 1000);
      
      toast({
        title: "发送失败",
        description: "API连接失败，使用演示模式响应",
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  // 根据输入生成演示回复（当API不可用时使用）
  const generateDemoResponse = (userInput: string, agentName: string): string => {
    const userQuestion = userInput.toLowerCase();
    
    if (userQuestion.includes('你好') || userQuestion.includes('hi') || userQuestion.includes('hello')) {
      return `你好！我是${agentName}，有什么我可以帮助你的吗？`;
    }
    
    if (userQuestion.includes('能做什么') || userQuestion.includes('功能') || userQuestion.includes('帮我')) {
      return `我可以回答问题、提供信息、协助你完成各种任务。请告诉我你需要什么样的帮助！`;
    }
    
    if (userQuestion.includes('天气')) {
      return `我目前无法获取实时天气信息。在实际应用中，我可以通过天气API来获取天气信息。`;
    }
    
    if (userQuestion.includes('时间') || userQuestion.includes('日期')) {
      return `现在的时间是 ${new Date().toLocaleTimeString()}，日期是 ${new Date().toLocaleDateString()}。`;
    }
    
    return `感谢你的问题。在完整版本中，我会通过调用LLM来生成适当的回复。这是一个演示，实际应用中我会连接到强大的语言模型提供更智能的回答。`;
  };

  return (
    <div className="flex flex-col h-full border rounded-lg overflow-hidden bg-background">
      {/* 聊天头部 */}
      <div className="flex items-center justify-between p-4 border-b">
        <div className="flex items-center">
          <Avatar className="h-8 w-8 mr-2">
            <AvatarImage src={`https://avatar.vercel.sh/${agentId}.png`} alt={agentName} />
            <AvatarFallback>{agentName.substring(0, 2)}</AvatarFallback>
          </Avatar>
          <div>
            <h3 className="font-medium">{agentName}</h3>
            <Badge variant="outline" className="text-xs">ID: {agentId}</Badge>
          </div>
        </div>
        
        <Sheet>
          <SheetTrigger asChild>
            <Button variant="ghost" size="icon">
              <ChevronDown className="h-4 w-4" />
            </Button>
          </SheetTrigger>
          <SheetContent>
            <SheetHeader>
              <SheetTitle>代理信息</SheetTitle>
              <SheetDescription>
                {agentName} 的详细信息
              </SheetDescription>
            </SheetHeader>
            <div className="py-4">
              <p className="text-sm mb-2">代理ID: {agentId}</p>
              <p className="text-sm text-muted-foreground">
                这是一个演示界面。在完整版本中，这里会显示代理的详细配置、性能信息和使用统计。
              </p>
            </div>
          </SheetContent>
        </Sheet>
      </div>
      
      {/* 聊天内容区域 */}
      <ScrollArea className="flex-1 p-4">
        <div className="space-y-4">
          {messages.length === 0 ? (
            <div className="flex justify-center items-center h-[calc(100vh-12rem)] text-center">
              <div className="max-w-md">
                <h3 className="text-lg font-medium mb-2">开始与{agentName}对话</h3>
                <p className="text-sm text-muted-foreground">
                  发送消息开始与代理交流。代理将使用定义的系统提示和配置来响应你的问题。
                </p>
              </div>
            </div>
          ) : (
            messages.map(message => (
              <div
                key={message.id}
                className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
              >
                <div
                  className={`max-w-[80%] rounded-lg p-3 ${
                    message.role === 'user'
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted'
                  }`}
                >
                  <p className="whitespace-pre-wrap break-words">{message.content}</p>
                  <p className="text-xs opacity-70 mt-1 text-right">
                    {message.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                  </p>
                </div>
              </div>
            ))
          )}
          
          {isLoading && (
            <div className="flex justify-start">
              <div className="bg-muted rounded-lg p-3 flex items-center">
                <Loader2 className="h-4 w-4 animate-spin mr-2" />
                <span>思考中...</span>
              </div>
            </div>
          )}
        </div>
      </ScrollArea>
      
      {/* 输入区域 */}
      <div className="p-4 border-t">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            handleSendMessage();
          }}
          className="flex space-x-2"
        >
          <Input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="输入消息..."
            disabled={isLoading}
          />
          <Button type="submit" disabled={isLoading || !input.trim()}>
            <Send className="h-4 w-4" />
          </Button>
        </form>
      </div>
    </div>
  );
}
