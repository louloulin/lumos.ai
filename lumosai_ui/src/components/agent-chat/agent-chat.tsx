import { useState, useRef, useEffect } from 'react';
import { Send, Paperclip, ChevronDown, Clock, Bot, User, RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { 
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import SyntaxHighlighter from '@/components/syntax-highlighter';

// 消息类型
interface Message {
  id: string;
  role: 'user' | 'agent' | 'system';
  content: string;
  timestamp: Date;
  status: 'sending' | 'sent' | 'error';
  attachments?: { name: string; url: string; type: string }[];
}

// 代理接口
interface Agent {
  id: string;
  name: string;
  description: string;
  avatar?: string;
}

interface AgentChatProps {
  agent: Agent;
  initialMessages?: Message[];
  onSendMessage?: (message: string, attachments?: File[]) => Promise<void>;
}

export default function AgentChat({ 
  agent, 
  initialMessages = [],
  onSendMessage 
}: AgentChatProps) {
  const [messages, setMessages] = useState<Message[]>(initialMessages);
  const [input, setInput] = useState('');
  const [isTyping, setIsTyping] = useState(false);
  const [attachments, setAttachments] = useState<File[]>([]);
  const [isSending, setIsSending] = useState(false);
  const endOfMessagesRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  
  // 滚动到最新消息
  useEffect(() => {
    endOfMessagesRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // 处理发送消息
  const handleSendMessage = async () => {
    if (input.trim() === '' && attachments.length === 0) return;
    
    // 创建用户消息
    const userMessage: Message = {
      id: `msg-${Date.now()}`,
      role: 'user',
      content: input,
      timestamp: new Date(),
      status: 'sending',
      attachments: attachments.map(file => ({
        name: file.name,
        url: URL.createObjectURL(file),
        type: file.type
      }))
    };
    
    // 添加消息到列表
    setMessages(prev => [...prev, userMessage]);
    setInput('');
    setAttachments([]);
    setIsSending(true);
    
    try {
      // 更新消息状态为已发送
      setMessages(prev => 
        prev.map(msg => 
          msg.id === userMessage.id ? { ...msg, status: 'sent' } : msg
        )
      );
      
      // 如果有回调函数，调用它
      if (onSendMessage) {
        await onSendMessage(input, attachments);
      } else {
        // 模拟代理响应
        setIsTyping(true);
        await new Promise(resolve => setTimeout(resolve, 1500));
        
        // 生成代理响应
        const response = await generateDemoResponse(input);
        
        // 添加代理响应
        setMessages(prev => [
          ...prev,
          {
            id: `msg-${Date.now()}`,
            role: 'agent',
            content: response,
            timestamp: new Date(),
            status: 'sent',
          }
        ]);
        setIsTyping(false);
      }
    } catch (error) {
      console.error('Failed to send message:', error);
      setMessages(prev => 
        prev.map(msg => 
          msg.id === userMessage.id ? { ...msg, status: 'error' } : msg
        )
      );
    } finally {
      setIsSending(false);
    }
  };

  // 处理文件上传
  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      setAttachments([...attachments, ...Array.from(e.target.files)]);
    }
  };

  // 处理删除附件
  const handleRemoveAttachment = (index: number) => {
    setAttachments(attachments.filter((_, i) => i !== index));
  };

  // 处理输入框回车键
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  // 模拟代理响应生成（实际应用中应替换为真实的API调用）
  const generateDemoResponse = async (userInput: string): Promise<string> => {
    const responses = [
      `我理解您的问题是关于"${userInput.substring(0, 20)}..."。根据我的知识，这个问题的解答是...`,
      "感谢您的问题。让我为您提供一些相关信息:\n\n1. 首先，我们需要理解问题的核心\n2. 其次，让我们分析可能的解决方案\n3. 最后，我建议您可以尝试以下方法...",
      `这是一个很好的问题。根据最新的研究和数据，我可以告诉您：\n\n\`\`\`json\n{\n  "recommendation": "基于您的问题，我推荐...",\n  "confidence": 0.92,\n  "sources": ["最新研究", "行业最佳实践"]\n}\n\`\`\``,
      "我需要更多信息来准确回答您的问题。您能否提供以下细节：\n1. 具体情境或背景？\n2. 您已经尝试过哪些方法？\n3. 您的预期目标是什么？"
    ];
    
    // 随机选择一个响应
    return responses[Math.floor(Math.random() * responses.length)];
  };

  // 格式化消息内容，识别代码块并使用语法高亮
  const formatMessageContent = (content: string) => {
    if (!content) return null;
    
    // 简单的代码块检测，实际应用中可能需要更复杂的解析
    const codeBlockRegex = /```(\w*)\n([\s\S]*?)```/g;
    const parts = [];
    let lastIndex = 0;
    let match;
    
    while ((match = codeBlockRegex.exec(content)) !== null) {
      // 添加代码块之前的文本
      if (match.index > lastIndex) {
        const textBefore = content.substring(lastIndex, match.index);
        parts.push(<p key={`text-${lastIndex}`} className="whitespace-pre-wrap">{textBefore}</p>);
      }
      
      // 添加代码块
      const language = match[1] || 'text';
      const code = match[2];
      parts.push(
        <div key={`code-${match.index}`} className="my-2 rounded overflow-hidden">
          <SyntaxHighlighter code={code} language={language} />
        </div>
      );
      
      lastIndex = match.index + match[0].length;
    }
    
    // 添加剩余的文本
    if (lastIndex < content.length) {
      const textAfter = content.substring(lastIndex);
      parts.push(<p key={`text-${lastIndex}`} className="whitespace-pre-wrap">{textAfter}</p>);
    }
    
    return parts.length > 0 ? parts : <p className="whitespace-pre-wrap">{content}</p>;
  };

  return (
    <Card className="flex flex-col h-full">
      <CardHeader className="border-b p-4">
        <div className="flex items-center">
          <Avatar className="h-8 w-8 mr-2">
            <AvatarImage src={agent.avatar} alt={agent.name} />
            <AvatarFallback>{agent.name.substring(0, 2)}</AvatarFallback>
          </Avatar>
          <div>
            <CardTitle className="text-lg flex items-center">
              {agent.name}
              <Badge variant="outline" className="ml-2 text-xs">
                在线
              </Badge>
            </CardTitle>
          </div>
          <div className="ml-auto">
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon">
                  <ChevronDown className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={() => setMessages([])}>
                  清除对话
                </DropdownMenuItem>
                <DropdownMenuItem>
                  导出对话
                </DropdownMenuItem>
                <DropdownMenuItem>
                  查看详情
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </CardHeader>
      
      <ScrollArea className="flex-1 p-4">
        <div className="space-y-4">
          {messages.length === 0 ? (
            <div className="flex flex-col items-center justify-center text-center py-8">
              <Bot className="h-12 w-12 text-muted-foreground mb-4" />
              <p className="text-muted-foreground mb-1">开始与 {agent.name} 对话</p>
              <p className="text-xs text-muted-foreground max-w-xs">
                {agent.description || "这个智能助手可以回答您的问题，请在下方输入您的问题。"}
              </p>
            </div>
          ) : (
            messages.map((message) => (
              <div 
                key={message.id} 
                className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
              >
                <div className={`flex gap-3 max-w-[85%] ${message.role === 'user' ? 'flex-row-reverse' : ''}`}>
                  <Avatar className="h-8 w-8 mt-1">
                    {message.role === 'user' ? (
                      <>
                        <AvatarImage src="/user-avatar.png" alt="User" />
                        <AvatarFallback><User className="h-4 w-4" /></AvatarFallback>
                      </>
                    ) : (
                      <>
                        <AvatarImage src={agent.avatar} alt={agent.name} />
                        <AvatarFallback><Bot className="h-4 w-4" /></AvatarFallback>
                      </>
                    )}
                  </Avatar>
                  <div>
                    <div 
                      className={`rounded-lg p-3 ${
                        message.role === 'user' 
                          ? 'bg-primary text-primary-foreground' 
                          : 'bg-muted'
                      }`}
                    >
                      {formatMessageContent(message.content)}
                      
                      {message.attachments && message.attachments.length > 0 && (
                        <div className="mt-2 space-y-1">
                          {message.attachments.map((attachment, index) => (
                            <div key={index} className="flex items-center justify-between p-2 bg-background/50 rounded border text-xs">
                              <span className="truncate max-w-[200px]">{attachment.name}</span>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                    
                    <div className={`text-xs text-muted-foreground mt-1 flex items-center gap-1 ${
                      message.role === 'user' ? 'justify-end' : 'justify-start'
                    }`}>
                      <Clock className="h-3 w-3" />
                      {message.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                      
                      {message.status === 'sending' && (
                        <span className="flex items-center">
                          <RefreshCw className="h-3 w-3 animate-spin ml-1" />
                        </span>
                      )}
                      
                      {message.status === 'error' && (
                        <span className="text-red-500">发送失败</span>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            ))
          )}
          
          {/* 代理正在输入的指示器 */}
          {isTyping && (
            <div className="flex justify-start">
              <div className="flex gap-3 max-w-[85%]">
                <Avatar className="h-8 w-8 mt-1">
                  <AvatarImage src={agent.avatar} alt={agent.name} />
                  <AvatarFallback><Bot className="h-4 w-4" /></AvatarFallback>
                </Avatar>
                <div>
                  <div className="rounded-lg p-3 bg-muted flex items-center space-x-1">
                    <div className="w-2 h-2 rounded-full bg-current animate-bounce" style={{ animationDelay: '0ms' }}></div>
                    <div className="w-2 h-2 rounded-full bg-current animate-bounce" style={{ animationDelay: '150ms' }}></div>
                    <div className="w-2 h-2 rounded-full bg-current animate-bounce" style={{ animationDelay: '300ms' }}></div>
                  </div>
                  <div className="text-xs text-muted-foreground mt-1">正在输入...</div>
                </div>
              </div>
            </div>
          )}
          
          <div ref={endOfMessagesRef} />
        </div>
      </ScrollArea>
      
      <CardFooter className="border-t p-4">
        {attachments.length > 0 && (
          <div className="flex flex-wrap gap-2 mb-2">
            {attachments.map((file, index) => (
              <Badge 
                key={index} 
                variant="secondary"
                className="flex items-center gap-1"
              >
                {file.name.length > 20 ? `${file.name.substring(0, 20)}...` : file.name}
                <button 
                  className="ml-1 hover:text-destructive"
                  onClick={() => handleRemoveAttachment(index)}
                >
                  &times;
                </button>
              </Badge>
            ))}
          </div>
        )}
        
        <div className="flex w-full items-end gap-2">
          <Button 
            variant="outline" 
            size="icon" 
            className="shrink-0" 
            onClick={() => fileInputRef.current?.click()}
          >
            <Paperclip className="h-4 w-4" />
            <input 
              type="file" 
              ref={fileInputRef}
              className="hidden" 
              multiple 
              onChange={handleFileSelect}
            />
          </Button>
          
          <Textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="输入您的消息..."
            className="min-h-[80px] flex-1 resize-none"
          />
          
          <Button 
            className="shrink-0" 
            onClick={handleSendMessage}
            disabled={isSending || (input.trim() === '' && attachments.length === 0)}
          >
            <Send className="h-4 w-4" />
          </Button>
        </div>
      </CardFooter>
    </Card>
  );
} 