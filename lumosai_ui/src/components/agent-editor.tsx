import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useToast } from '@/components/ui/use-toast';
import { Agent } from '@/types';

interface AgentEditorProps {
  agent?: Agent;
  onSave: (agent: Agent) => void;
  onCancel: () => void;
}

export function AgentEditor({ agent, onSave, onCancel }: AgentEditorProps) {
  const { toast } = useToast();
  const isNew = !agent;
  
  const [formData, setFormData] = useState<Agent>(
    agent || {
      id: '',
      name: '',
      description: '',
      model: 'openai/gpt-3.5-turbo',
      systemPrompt: '',
      temperature: 0.7,
      maxTokens: 1000,
      tools: [],
      isActive: true,
    }
  );

  const handleChange = (field: keyof Agent, value: any) => {
    setFormData((prev) => ({
      ...prev,
      [field]: value,
    }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // Validate form
    if (!formData.id || !formData.name) {
      toast({
        title: "验证失败",
        description: "代理ID和名称为必填项",
        variant: "destructive",
      });
      return;
    }
    
    // Save agent
    onSave(formData);
    
    toast({
      title: `代理${isNew ? '创建' : '更新'}成功`,
      description: `${formData.name} 已${isNew ? '创建' : '更新'}`,
    });
  };

  return (
    <form onSubmit={handleSubmit}>
      <Card className="w-full">
        <CardHeader>
          <CardTitle>{isNew ? '创建新代理' : `编辑代理: ${agent?.name}`}</CardTitle>
          <CardDescription>
            配置代理的基本信息、提示词和参数
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Tabs defaultValue="basic">
            <TabsList className="mb-4">
              <TabsTrigger value="basic">基础信息</TabsTrigger>
              <TabsTrigger value="model">模型与参数</TabsTrigger>
              <TabsTrigger value="system">系统提示词</TabsTrigger>
              <TabsTrigger value="tools">工具</TabsTrigger>
            </TabsList>
            
            <TabsContent value="basic">
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="id">代理ID</Label>
                    <Input 
                      id="id" 
                      value={formData.id} 
                      onChange={(e) => handleChange('id', e.target.value)}
                      placeholder="unique_agent_id"
                      disabled={!isNew}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="name">代理名称</Label>
                    <Input 
                      id="name" 
                      value={formData.name} 
                      onChange={(e) => handleChange('name', e.target.value)}
                      placeholder="我的代理"
                    />
                  </div>
                </div>
                
                <div className="space-y-2">
                  <Label htmlFor="description">描述</Label>
                  <Textarea 
                    id="description" 
                    value={formData.description} 
                    onChange={(e) => handleChange('description', e.target.value)}
                    placeholder="描述这个代理的功能和用途..."
                    rows={4}
                  />
                </div>
                
                <div className="flex items-center space-x-2 pt-2">
                  <Switch 
                    id="isActive" 
                    checked={formData.isActive}
                    onCheckedChange={(checked) => handleChange('isActive', checked)}
                  />
                  <Label htmlFor="isActive">启用</Label>
                </div>
              </div>
            </TabsContent>
            
            <TabsContent value="model">
              <div className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="model">模型</Label>
                  <Select 
                    value={formData.model}
                    onValueChange={(value) => handleChange('model', value)}
                  >
                    <SelectTrigger id="model">
                      <SelectValue placeholder="选择模型" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="openai/gpt-3.5-turbo">OpenAI GPT-3.5 Turbo</SelectItem>
                      <SelectItem value="openai/gpt-4">OpenAI GPT-4</SelectItem>
                      <SelectItem value="anthropic/claude-3-haiku">Anthropic Claude 3 Haiku</SelectItem>
                      <SelectItem value="anthropic/claude-3-sonnet">Anthropic Claude 3 Sonnet</SelectItem>
                      <SelectItem value="anthropic/claude-3-opus">Anthropic Claude 3 Opus</SelectItem>
                      <SelectItem value="gemini/gemini-pro">Google Gemini Pro</SelectItem>
                      <SelectItem value="local/ollama">本地 Ollama</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="temperature">温度 ({formData.temperature})</Label>
                    <Input 
                      id="temperature" 
                      type="range" 
                      min="0" 
                      max="1" 
                      step="0.1"
                      value={formData.temperature} 
                      onChange={(e) => handleChange('temperature', parseFloat(e.target.value))}
                    />
                    <div className="flex justify-between text-xs text-muted-foreground">
                      <span>精确</span>
                      <span>创意</span>
                    </div>
                  </div>
                  
                  <div className="space-y-2">
                    <Label htmlFor="maxTokens">最大Token数</Label>
                    <Input 
                      id="maxTokens" 
                      type="number" 
                      min="100" 
                      max="8000"
                      value={formData.maxTokens} 
                      onChange={(e) => handleChange('maxTokens', parseInt(e.target.value))}
                    />
                  </div>
                </div>
              </div>
            </TabsContent>
            
            <TabsContent value="system">
              <div className="space-y-2">
                <Label htmlFor="systemPrompt">系统提示词</Label>
                <Textarea 
                  id="systemPrompt" 
                  value={formData.systemPrompt} 
                  onChange={(e) => handleChange('systemPrompt', e.target.value)}
                  placeholder="你是一个有用的AI助理..."
                  rows={10}
                />
              </div>
            </TabsContent>
            
            <TabsContent value="tools">
              <div className="p-4 text-center text-muted-foreground">
                <p>工具配置功能即将推出</p>
              </div>
            </TabsContent>
          </Tabs>
        </CardContent>
        <CardFooter className="flex justify-between">
          <Button variant="outline" type="button" onClick={onCancel}>取消</Button>
          <Button type="submit">{isNew ? '创建' : '保存'}</Button>
        </CardFooter>
      </Card>
    </form>
  );
} 