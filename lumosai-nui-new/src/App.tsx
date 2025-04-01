import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "./components/ui/button";
import { Input } from "./components/ui/input";
import { ThemeSwitch } from "./components/theme-switch";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "./components/ui/card";
import { Checkbox } from "./components/ui/checkbox";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./components/ui/select";
import { Switch } from "./components/ui/switch";
import { Label } from "./components/ui/label";
import { Container } from "./components/ui/container";
import { Flex } from "./components/ui/flex";
import { Separator } from "./components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./components/ui/tabs";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "./components/ui/dialog";
import { Popover, PopoverContent, PopoverTrigger } from "./components/ui/popover";
import { useToast } from "./components/ui/use-toast";
import { Toaster } from "./components/ui/toaster";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [checked, setChecked] = useState(false);
  const [switchOn, setSwitchOn] = useState(false);
  const { toast } = useToast();

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main style={{ backgroundColor: "rgb(var(--background))" }} className="min-h-screen py-8">
      <Toaster />
      <Container>
        <Flex justify="between" align="center" className="mb-8">
          <h1 className="text-3xl font-bold">LumosAI UI</h1>
          <ThemeSwitch />
        </Flex>

        <Tabs defaultValue="基础组件" className="mb-8">
          <TabsList className="mb-4">
            <TabsTrigger value="基础组件">基础组件</TabsTrigger>
            <TabsTrigger value="布局组件">布局组件</TabsTrigger>
            <TabsTrigger value="导航组件">导航组件</TabsTrigger>
            <TabsTrigger value="反馈组件">反馈组件</TabsTrigger>
          </TabsList>
          
          <TabsContent value="基础组件" className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>基础组件</CardTitle>
                <CardDescription>展示 Lumosai UI 的基础组件</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-4">
                  <h3 className="text-lg font-medium">按钮</h3>
                  <Flex gap="sm" wrap="wrap">
                    <Button>默认</Button>
                    <Button variant="secondary">次要</Button>
                    <Button variant="outline">轮廓</Button>
                    <Button variant="destructive">警告</Button>
                    <Button variant="ghost">幽灵</Button>
                    <Button variant="link">链接</Button>
                  </Flex>
                </div>
                
                <Separator />
                
                <div className="space-y-4">
                  <h3 className="text-lg font-medium">输入框</h3>
                  <Input placeholder="请输入文本..." />
                </div>
                
                <Separator />
                
                <div className="space-y-4">
                  <h3 className="text-lg font-medium">复选框</h3>
                  <Flex align="center" gap="sm">
                    <Checkbox 
                      id="terms" 
                      checked={checked} 
                      onCheckedChange={(value) => setChecked(value === true)} 
                    />
                    <Label htmlFor="terms">接受条款</Label>
                  </Flex>
                </div>
                
                <Separator />
                
                <div className="space-y-4">
                  <h3 className="text-lg font-medium">开关</h3>
                  <Flex align="center" gap="sm">
                    <Switch id="airplane-mode" checked={switchOn} onCheckedChange={setSwitchOn} />
                    <Label htmlFor="airplane-mode">飞行模式</Label>
                  </Flex>
                </div>
                
                <Separator />
                
                <div className="space-y-4">
                  <h3 className="text-lg font-medium">选择框</h3>
                  <Select>
                    <SelectTrigger className="w-full">
                      <SelectValue placeholder="选择一个选项" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="option1">选项 1</SelectItem>
                      <SelectItem value="option2">选项 2</SelectItem>
                      <SelectItem value="option3">选项 3</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>
            
            <Card>
              <CardHeader>
                <CardTitle>Tauri 集成</CardTitle>
                <CardDescription>演示与 Tauri 后端的集成</CardDescription>
              </CardHeader>
              <CardContent>
                <form
                  className="space-y-4"
                  onSubmit={(e) => {
                    e.preventDefault();
                    greet();
                  }}
                >
                  <Flex direction="column" gap="sm">
                    <Label htmlFor="name-input">您的名字</Label>
                    <Input
                      id="name-input"
                      value={name}
                      onChange={(e) => setName(e.currentTarget.value)}
                      placeholder="输入名称..."
                    />
                  </Flex>
                  <Button type="submit">问候</Button>
                  {greetMsg && (
                    <div className="p-4 rounded-md" style={{ backgroundColor: "rgb(var(--muted))" }}>
                      {greetMsg}
                    </div>
                  )}
                </form>
              </CardContent>
              <CardFooter>
                <p className="text-sm text-muted-foreground">
                  使用 Tauri 命令与 Rust 后端交互
                </p>
              </CardFooter>
            </Card>
          </TabsContent>
          
          <TabsContent value="布局组件">
            <Card className="mb-8">
              <CardHeader>
                <CardTitle>布局组件</CardTitle>
                <CardDescription>展示 Container、Flex 等布局组件</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-6">
                  <div>
                    <h3 className="text-lg font-medium mb-2">Flex 布局</h3>
                    <div className="border rounded-md p-4">
                      <Flex gap="md" wrap="wrap">
                        {[1, 2, 3, 4].map((i) => (
                          <div key={i} className="h-16 w-16 rounded-md flex items-center justify-center" style={{ backgroundColor: "rgb(var(--primary))", color: "rgb(var(--primary-foreground))" }}>
                            {i}
                          </div>
                        ))}
                      </Flex>
                    </div>
                  </div>
                  
                  <div>
                    <h3 className="text-lg font-medium mb-2">Flex 方向</h3>
                    <div className="border rounded-md p-4">
                      <Flex direction="column" gap="sm">
                        {[1, 2, 3].map((i) => (
                          <div key={i} className="h-10 w-full rounded-md flex items-center justify-center" style={{ backgroundColor: "rgb(var(--secondary))", color: "rgb(var(--secondary-foreground))" }}>
                            项目 {i}
                          </div>
                        ))}
                      </Flex>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
          
          <TabsContent value="导航组件">
            <Card>
              <CardHeader>
                <CardTitle>导航组件</CardTitle>
                <CardDescription>展示 Tabs 等导航组件</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-6">
                  <div>
                    <h3 className="text-lg font-medium mb-4">选项卡 (Tabs)</h3>
                    <Tabs defaultValue="tab1" className="w-full">
                      <TabsList>
                        <TabsTrigger value="tab1">选项卡 1</TabsTrigger>
                        <TabsTrigger value="tab2">选项卡 2</TabsTrigger>
                        <TabsTrigger value="tab3">选项卡 3</TabsTrigger>
                      </TabsList>
                      <TabsContent value="tab1" className="p-4 border rounded-md mt-2">
                        选项卡 1 的内容。这是一个简单的选项卡示例。
                      </TabsContent>
                      <TabsContent value="tab2" className="p-4 border rounded-md mt-2">
                        选项卡 2 的内容。您可以在不同选项卡之间切换查看内容。
                      </TabsContent>
                      <TabsContent value="tab3" className="p-4 border rounded-md mt-2">
                        选项卡 3 的内容。这个组件使用 Radix UI 构建。
                      </TabsContent>
                    </Tabs>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
          
          <TabsContent value="反馈组件">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <Card>
                <CardHeader>
                  <CardTitle>对话框 (Dialog)</CardTitle>
                  <CardDescription>用于显示重要信息或需要用户确认的内容</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <Dialog>
                    <DialogTrigger asChild>
                      <Button>打开对话框</Button>
                    </DialogTrigger>
                    <DialogContent className="sm:max-w-[425px]">
                      <DialogHeader>
                        <DialogTitle>应用设置</DialogTitle>
                        <DialogDescription>
                          在这里调整您的应用设置。完成后点击保存。
                        </DialogDescription>
                      </DialogHeader>
                      <div className="py-4">
                        <div className="space-y-4">
                          <div className="space-y-2">
                            <Label htmlFor="name">用户名</Label>
                            <Input id="name" placeholder="输入用户名" />
                          </div>
                          <Flex align="center" gap="sm">
                            <Checkbox id="notifications" />
                            <Label htmlFor="notifications">启用通知</Label>
                          </Flex>
                        </div>
                      </div>
                      <DialogFooter>
                        <Button type="submit">保存更改</Button>
                      </DialogFooter>
                    </DialogContent>
                  </Dialog>
                </CardContent>
              </Card>
              
              <Card>
                <CardHeader>
                  <CardTitle>悬浮框 (Popover)</CardTitle>
                  <CardDescription>用于显示附加信息或控件</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <Popover>
                    <PopoverTrigger asChild>
                      <Button variant="outline">打开悬浮框</Button>
                    </PopoverTrigger>
                    <PopoverContent className="p-4">
                      <div className="space-y-2">
                        <h4 className="font-medium">快速设置</h4>
                        <p className="text-sm text-muted-foreground">
                          调整常用设置项
                        </p>
                        <Separator />
                        <div className="space-y-2">
                          <Flex justify="between" align="center">
                            <Label htmlFor="width">不透明度</Label>
                            <span className="text-sm text-muted-foreground">70%</span>
                          </Flex>
                          <Input id="width" type="range" />
                        </div>
                      </div>
                    </PopoverContent>
                  </Popover>
                </CardContent>
              </Card>
              
              <Card>
                <CardHeader>
                  <CardTitle>提示 (Toast)</CardTitle>
                  <CardDescription>用于显示简短的通知信息</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <Flex gap="sm" wrap="wrap">
                    <Button 
                      onClick={() => {
                        toast({
                          title: "操作成功",
                          description: "您的更改已成功保存。",
                        })
                      }}
                    >
                      显示成功提示
                    </Button>
                    <Button 
                      variant="destructive"
                      onClick={() => {
                        toast({
                          variant: "destructive",
                          title: "出现错误",
                          description: "无法保存您的更改。",
                        })
                      }}
                    >
                      显示错误提示
                    </Button>
                  </Flex>
                </CardContent>
              </Card>
            </div>
          </TabsContent>
        </Tabs>
      </Container>
      
      <footer className="mt-16 py-6 border-t" style={{ borderColor: "rgb(var(--border))" }}>
        <Container>
          <Flex justify="between" align="center">
            <div className="text-sm" style={{ color: "rgb(var(--muted-foreground))" }}>
              LumosAI UI Framework - 基于 Tauri、React 和 Tailwind CSS
            </div>
            <div className="text-sm" style={{ color: "rgb(var(--muted-foreground))" }}>
              版本 0.1.0
            </div>
          </Flex>
        </Container>
      </footer>
    </main>
  );
}

export default App;
