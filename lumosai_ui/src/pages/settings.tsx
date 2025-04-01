import { useState, useEffect } from 'react';
import { Settings, Save, RefreshCw } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardFooter } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Switch } from '@/components/ui/switch';
import { useToast } from '@/components/ui/use-toast';

export default function SettingsPage() {
  const { toast } = useToast();
  const [apiUrl, setApiUrl] = useState('');
  const [apiKey, setApiKey] = useState('');
  const [theme, setTheme] = useState('light');
  const [developerMode, setDeveloperMode] = useState(false);

  // 从本地存储加载设置
  useEffect(() => {
    const savedApiUrl = localStorage.getItem('lumosai_api_url');
    const savedApiKey = localStorage.getItem('lumosai_api_key');
    const savedTheme = localStorage.getItem('lumosai_theme');
    const savedDevMode = localStorage.getItem('lumosai_dev_mode');

    if (savedApiUrl) setApiUrl(savedApiUrl);
    if (savedApiKey) setApiKey(savedApiKey);
    if (savedTheme) setTheme(savedTheme);
    if (savedDevMode) setDeveloperMode(savedDevMode === 'true');
    
    // 设置默认 API URL 如果没有设置
    if (!savedApiUrl) {
      const defaultApiUrl = 'http://localhost:3000/api';
      setApiUrl(defaultApiUrl);
    }
  }, []);

  // 保存所有设置
  const saveSettings = () => {
    // 保存到本地存储
    localStorage.setItem('lumosai_api_url', apiUrl);
    localStorage.setItem('lumosai_api_key', apiKey);
    localStorage.setItem('lumosai_theme', theme);
    localStorage.setItem('lumosai_dev_mode', developerMode.toString());
    
    // 应用主题
    document.documentElement.classList.toggle('dark', theme === 'dark');
    
    toast({
      title: "设置已保存",
      description: "您的设置已成功保存并应用",
    });
  };

  // 测试 API 连接
  const testApiConnection = async () => {
    toast({
      title: "测试连接",
      description: "正在测试与 API 的连接...",
    });
    
    try {
      const response = await fetch(`${apiUrl}/info`, {
        headers: {
          'Authorization': `Bearer ${apiKey}`,
        },
      });
      
      if (response.ok) {
        const data = await response.json();
        toast({
          title: "连接成功",
          description: `成功连接到 API。版本: ${data.version || 'unknown'}`,
          variant: "default",
        });
      } else {
        toast({
          title: "连接失败",
          description: "无法连接到 API。请检查 URL 和密钥是否正确。",
          variant: "destructive",
        });
      }
    } catch (error) {
      toast({
        title: "连接错误",
        description: "尝试连接 API 时发生错误。请检查 URL 是否正确。",
        variant: "destructive",
      });
    }
  };

  // 重置设置为默认值
  const resetSettings = () => {
    const defaultApiUrl = 'http://localhost:3000/api';
    const defaultTheme = 'light';
    
    setApiUrl(defaultApiUrl);
    setApiKey('');
    setTheme(defaultTheme);
    setDeveloperMode(false);
    
    // 保存默认设置
    localStorage.setItem('lumosai_api_url', defaultApiUrl);
    localStorage.setItem('lumosai_api_key', '');
    localStorage.setItem('lumosai_theme', defaultTheme);
    localStorage.setItem('lumosai_dev_mode', 'false');
    
    // 应用默认主题
    document.documentElement.classList.remove('dark');
    
    toast({
      title: "设置已重置",
      description: "所有设置已恢复为默认值",
    });
  };

  return (
    <div className="container mx-auto py-8 px-4">
      <div className="flex flex-col space-y-4">
        <div className="flex items-center gap-4">
          <Settings className="h-6 w-6" />
          <h1 className="text-3xl font-bold tracking-tight">设置</h1>
        </div>
        
        <p className="text-muted-foreground">
          配置您的 Lumos AI 界面和 API 连接
        </p>
        
        <Tabs defaultValue="general" className="mt-6">
          <TabsList className="grid w-full md:w-auto md:inline-flex grid-cols-2 md:grid-cols-none">
            <TabsTrigger value="general">常规设置</TabsTrigger>
            <TabsTrigger value="api">API 配置</TabsTrigger>
          </TabsList>
          
          <TabsContent value="general" className="mt-4">
            <Card>
              <CardHeader>
                <CardTitle>常规设置</CardTitle>
                <CardDescription>自定义界面和行为</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="theme">深色主题</Label>
                    <Switch 
                      id="theme" 
                      checked={theme === 'dark'}
                      onCheckedChange={(checked) => setTheme(checked ? 'dark' : 'light')}
                    />
                  </div>
                  <p className="text-sm text-muted-foreground">
                    启用深色主题以减少眼睛疲劳
                  </p>
                </div>
                
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="dev-mode">开发者模式</Label>
                    <Switch 
                      id="dev-mode" 
                      checked={developerMode}
                      onCheckedChange={setDeveloperMode}
                    />
                  </div>
                  <p className="text-sm text-muted-foreground">
                    启用额外的开发工具和调试信息
                  </p>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
          
          <TabsContent value="api" className="mt-4">
            <Card>
              <CardHeader>
                <CardTitle>API 配置</CardTitle>
                <CardDescription>配置 Lumos AI API 连接</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-2">
                  <Label htmlFor="api-url">API URL</Label>
                  <Input 
                    id="api-url" 
                    placeholder="例如: http://localhost:3000/api" 
                    value={apiUrl} 
                    onChange={(e) => setApiUrl(e.target.value)}
                  />
                  <p className="text-sm text-muted-foreground">
                    Lumos AI API 的完整 URL
                  </p>
                </div>
                
                <div className="space-y-2">
                  <Label htmlFor="api-key">API 密钥</Label>
                  <Input 
                    id="api-key" 
                    type="password" 
                    placeholder="输入您的 API 密钥" 
                    value={apiKey} 
                    onChange={(e) => setApiKey(e.target.value)}
                  />
                  <p className="text-sm text-muted-foreground">
                    用于认证的 API 密钥。如果不需要认证，可以留空
                  </p>
                </div>
                
                <Button onClick={testApiConnection} variant="outline" className="w-full">
                  <RefreshCw className="mr-2 h-4 w-4" />
                  测试连接
                </Button>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
        
        <div className="flex justify-between mt-6">
          <Button variant="outline" onClick={resetSettings}>
            重置为默认值
          </Button>
          <Button onClick={saveSettings}>
            <Save className="mr-2 h-4 w-4" />
            保存设置
          </Button>
        </div>
      </div>
    </div>
  );
} 