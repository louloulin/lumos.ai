import { useState, useEffect, useRef, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import ReactFlow, {
  Controls,
  Background,
  applyNodeChanges,
  applyEdgeChanges,
  addEdge,
  Node,
  Edge,
  Connection,
  NodeChange,
  EdgeChange,
  Panel,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { LomusaiClient } from '@lomusai/client-js';
import { useToast } from '@/components/ui/use-toast';
import { SaveIcon, PlayIcon, ArrowLeftIcon, Plus } from 'lucide-react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Save, Play, Settings, ChevronRight, X, Trash2 } from 'lucide-react';
import { Textarea } from '@/components/ui/textarea';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Switch } from '@/components/ui/switch';
import { Separator } from '@/components/ui/separator';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';

// Custom node types
import { WorkflowDefaultNode } from '@/domains/workflows/workflow/workflow-default-node';
import { WorkflowConditionNode } from '@/domains/workflows/workflow/workflow-condition-node';
import { WorkflowAfterNode } from '@/domains/workflows/workflow/workflow-after-node';
import { WorkflowLoopResultNode } from '@/domains/workflows/workflow/workflow-loop-result-node';

// Node types mapping
const nodeTypes = {
  default: WorkflowDefaultNode,
  condition: WorkflowConditionNode,
  after: WorkflowAfterNode,
  loopResult: WorkflowLoopResultNode,
};

// Node template for new nodes
const getNodeTemplate = (type: string, position: { x: number, y: number }) => {
  const id = `node-${Date.now()}`;
  
  const baseNode = {
    id,
    position,
    data: { label: type.charAt(0).toUpperCase() + type.slice(1), nodeId: id },
  };

  switch (type) {
    case 'agent':
      return {
        ...baseNode,
        type: 'default',
        data: { 
          ...baseNode.data, 
          type: 'agent',
          config: { 
            agentId: '',
            input: '' 
          } 
        },
      };
    case 'condition':
      return {
        ...baseNode,
        type: 'condition',
        data: { 
          ...baseNode.data, 
          type: 'condition',
          config: { 
            conditions: [] 
          } 
        },
      };
    case 'tool':
      return {
        ...baseNode,
        type: 'default',
        data: { 
          ...baseNode.data, 
          type: 'tool',
          config: { 
            toolName: '',
            params: {} 
          } 
        },
      };
    default:
      return {
        ...baseNode,
        type: 'default',
        data: { 
          ...baseNode.data, 
          type: 'default',
          config: {} 
        },
      };
  }
};

// 工作流步骤类型接口
interface Step {
  id: string;
  name: string;
  type: 'agent' | 'condition' | 'loop' | 'output';
  description?: string;
  agentId?: string;
  condition?: string;
  outputFormat?: string;
  iterations?: number;
  nextSteps?: string[];
  isActive: boolean;
}

// 工作流接口
interface Workflow {
  id: string;
  name: string;
  description: string;
  isActive: boolean;
  firstStepId?: string;
  steps: Step[];
  tags: string[];
  createdAt: string;
  updatedAt: string;
}

// 代理接口简化版
interface Agent {
  id: string;
  name: string;
  description: string;
}

// 模拟的代理数据
const demoAgents: Agent[] = [
  { id: 'assistant', name: '通用助手', description: '一个通用的AI助手' },
  { id: 'researcher', name: '研究员', description: '专注于帮助用户进行研究' },
  { id: 'customer-service', name: '客服代理', description: '专门处理客户服务查询' },
  { id: 'data-analyst', name: '数据分析师', description: '分析数据并提供洞察' },
];

// 示例工作流
const exampleWorkflow: Workflow = {
  id: 'customer-support',
  name: '客户服务流程',
  description: '处理客户请求的自动化流程',
  isActive: true,
  firstStepId: 'step1',
  steps: [
    {
      id: 'step1',
      name: '问题分类',
      type: 'agent',
      description: '识别客户问题的类型',
      agentId: 'assistant',
      nextSteps: ['step2'],
      isActive: true,
    },
    {
      id: 'step2',
      name: '技术问题判断',
      type: 'condition',
      description: '判断是否为技术问题',
      condition: 'type === "technical"',
      nextSteps: ['step3', 'step4'],
      isActive: true,
    },
    {
      id: 'step3',
      name: '技术支持',
      type: 'agent',
      description: '提供技术支持解决方案',
      agentId: 'customer-service',
      nextSteps: ['step5'],
      isActive: true,
    },
    {
      id: 'step4',
      name: '一般帮助',
      type: 'agent',
      description: '提供一般服务支持',
      agentId: 'assistant',
      nextSteps: ['step5'],
      isActive: true,
    },
    {
      id: 'step5',
      name: '客户满意度',
      type: 'output',
      description: '记录结果和客户满意度',
      outputFormat: 'json',
      nextSteps: [],
      isActive: true,
    },
  ],
  tags: ['客服', '自动化'],
  createdAt: '2023-03-15T08:30:00Z',
  updatedAt: '2023-04-01T10:15:00Z',
};

export default function WorkflowEditor() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { toast } = useToast();
  
  const [workflow, setWorkflow] = useState<Workflow | null>(null);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isDragging, setIsDragging] = useState(false);
  const [isAddNodeDialogOpen, setIsAddNodeDialogOpen] = useState(false);
  const [newNodeType, setNewNodeType] = useState('agent');
  const [nodeName, setNodeName] = useState('');
  const [selectedStep, setSelectedStep] = useState<string | null>(null);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const [reactFlowInstance, setReactFlowInstance] = useState<any>(null);

  const client = new LomusaiClient({
    baseUrl: import.meta.env.VITE_API_BASE_URL || '',
  });

  // Load workflow data
  const fetchWorkflow = async () => {
    if (!id) return;
    
    setIsLoading(true);
    try {
      const workflowData = await client.getWorkflow(id);
      setWorkflow(workflowData);
      
      // Convert workflow steps to nodes and edges
      const workflowNodes: Node[] = [];
      const workflowEdges: Edge[] = [];
      
      // Process nodes
      workflowData.steps.forEach((step: any, index: number) => {
        const node: Node = {
          id: step.id,
          type: step.type === 'condition' ? 'condition' : 'default',
          position: { x: 250, y: index * 150 }, // Default position if not specified
          data: { 
            label: step.name || step.type,
            nodeId: step.id,
            type: step.type,
            config: step.config || {}
          },
        };
        
        workflowNodes.push(node);
        
        // Process connections to create edges
        if (step.next) {
          if (typeof step.next === 'string') {
            // Simple connection
            workflowEdges.push({
              id: `e-${step.id}-${step.next}`,
              source: step.id,
              target: step.next,
              type: 'default',
            });
          } else if (Array.isArray(step.next)) {
            // Multiple connections (for condition nodes)
            step.next.forEach((nextStep: any) => {
              workflowEdges.push({
                id: `e-${step.id}-${nextStep.id}`,
                source: step.id,
                target: nextStep.id,
                type: 'default',
                label: nextStep.condition || '',
              });
            });
          }
        }
      });
      
      setNodes(workflowNodes);
      setEdges(workflowEdges);
    } catch (error) {
      console.error('Error fetching workflow:', error);
      toast({
        title: 'Error loading workflow',
        description: 'Could not load the workflow details',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  // Initialize
  useEffect(() => {
    fetchWorkflow();
  }, [id]);

  // Node changes handler
  const onNodesChange = useCallback((changes: NodeChange[]) => {
    setNodes((nds) => applyNodeChanges(changes, nds));
  }, []);

  // Edge changes handler
  const onEdgesChange = useCallback((changes: EdgeChange[]) => {
    setEdges((eds) => applyEdgeChanges(changes, eds));
  }, []);

  // Connection handler
  const onConnect = useCallback((connection: Connection) => {
    setEdges((eds) => addEdge(connection, eds));
  }, []);

  // Save workflow
  const saveWorkflow = async () => {
    if (!workflow) return;
    
    try {
      // Convert nodes and edges back to workflow format
      const steps = nodes.map(node => {
        const connections = edges.filter(edge => edge.source === node.id);
        
        let next;
        if (node.data.type === 'condition' && connections.length > 0) {
          next = connections.map(edge => ({
            id: edge.target,
            condition: edge.label || '',
          }));
        } else if (connections.length > 0) {
          next = connections[0].target;
        } else {
          next = null;
        }
        
        return {
          id: node.id,
          name: node.data.label,
          type: node.data.type,
          config: node.data.config,
          next,
        };
      });
      
      await client.updateWorkflow(workflow.id, {
        ...workflow,
        steps,
      });
      
      toast({
        title: 'Workflow saved',
        description: 'Workflow has been updated successfully',
      });
    } catch (error) {
      console.error('Error saving workflow:', error);
      toast({
        title: 'Error saving workflow',
        description: 'Could not save the workflow changes',
        variant: 'destructive',
      });
    }
  };

  // Run workflow
  const runWorkflow = async () => {
    if (!workflow) return;
    
    try {
      await client.runWorkflow(workflow.id);
      
      toast({
        title: 'Workflow started',
        description: 'The workflow has been started successfully',
      });
    } catch (error) {
      console.error('Error running workflow:', error);
      toast({
        title: 'Error running workflow',
        description: 'Could not run the workflow',
        variant: 'destructive',
      });
    }
  };

  // Add new node
  const addNode = () => {
    if (!reactFlowInstance || !nodeName) return;
    
    const position = reactFlowInstance.project({
      x: reactFlowWrapper.current?.clientWidth ? reactFlowWrapper.current.clientWidth / 2 : 250,
      y: reactFlowWrapper.current?.clientHeight ? reactFlowWrapper.current.clientHeight / 2 : 200,
    });
    
    const newNode = getNodeTemplate(newNodeType, position);
    newNode.data.label = nodeName;
    
    setNodes((nds) => [...nds, newNode]);
    setNodeName('');
    setIsAddNodeDialogOpen(false);
  };

  // Drag and drop handlers
  const onDragOver = useCallback((event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent<HTMLDivElement>) => {
      event.preventDefault();

      if (!reactFlowWrapper.current || !reactFlowInstance) return;

      const type = event.dataTransfer.getData('application/reactflow/type');
      if (!type) return;

      const position = reactFlowInstance.screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });
      
      const newNode = getNodeTemplate(type, position);
      setNodes((nds) => [...nds, newNode]);
    },
    [reactFlowInstance]
  );

  const handleSave = async () => {
    if (!workflow) return;
    
    setIsSaving(true);
    try {
      // 在实际应用中，这里会是API调用
      await new Promise(resolve => setTimeout(resolve, 800)); // 模拟网络延迟
      
      toast({
        title: "保存成功",
        description: "工作流已成功保存",
      });
      
      if (id === 'new') {
        // 重定向到已保存的工作流
        navigate(`/workflows/${workflow.id}`);
      }
    } catch (error) {
      toast({
        title: "保存失败",
        description: "无法保存工作流",
        variant: "destructive",
      });
    } finally {
      setIsSaving(false);
    }
  };

  const handleAddStep = () => {
    if (!workflow) return;
    
    const newStep: Step = {
      id: `step-${Date.now()}`,
      name: '新步骤',
      type: 'agent',
      description: '',
      agentId: 'assistant',
      nextSteps: [],
      isActive: true,
    };
    
    setWorkflow({
      ...workflow,
      steps: [...workflow.steps, newStep],
      updatedAt: new Date().toISOString(),
    });
    
    setSelectedStep(newStep.id);
  };

  const handleDeleteStep = (stepId: string) => {
    if (!workflow) return;
    
    // 过滤掉要删除的步骤
    const newSteps = workflow.steps.filter(step => step.id !== stepId);
    
    // 更新其他步骤的nextSteps链接
    const updatedSteps = newSteps.map(step => ({
      ...step,
      nextSteps: step.nextSteps?.filter(nextId => nextId !== stepId) || [],
    }));
    
    // 如果删除的是第一个步骤，更新firstStepId
    let newFirstStepId = workflow.firstStepId;
    if (workflow.firstStepId === stepId) {
      newFirstStepId = updatedSteps.length > 0 ? updatedSteps[0].id : undefined;
    }
    
    setWorkflow({
      ...workflow,
      steps: updatedSteps,
      firstStepId: newFirstStepId,
      updatedAt: new Date().toISOString(),
    });
    
    setSelectedStep(newFirstStepId || null);
    setIsDeleteDialogOpen(false);
  };

  const handleUpdateStep = (stepId: string, updates: Partial<Step>) => {
    if (!workflow) return;
    
    const updatedSteps = workflow.steps.map(step => 
      step.id === stepId ? { ...step, ...updates } : step
    );
    
    setWorkflow({
      ...workflow,
      steps: updatedSteps,
      updatedAt: new Date().toISOString(),
    });
  };

  const handleUpdateWorkflow = (updates: Partial<Workflow>) => {
    if (!workflow) return;
    
    setWorkflow({
      ...workflow,
      ...updates,
      updatedAt: new Date().toISOString(),
    });
  };

  const getStepById = (stepId: string | null) => {
    if (!stepId || !workflow) return null;
    return workflow.steps.find(step => step.id === stepId) || null;
  };

  const currentStep = getStepById(selectedStep);

  if (isLoading || !workflow) {
    return (
      <div className="container py-8">
        <div className="flex justify-between items-center mb-6">
          <div className="h-8 w-48 bg-muted rounded animate-pulse"></div>
          <div className="h-10 w-24 bg-muted rounded animate-pulse"></div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="md:col-span-1">
            <div className="h-80 bg-muted rounded animate-pulse"></div>
          </div>
          <div className="md:col-span-2">
            <div className="h-80 bg-muted rounded animate-pulse"></div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="container py-8">
      <div className="flex justify-between items-center mb-6">
        <div>
          <Input
            className="text-2xl font-bold border-none bg-transparent h-auto p-0 focus-visible:ring-0 w-full max-w-md"
            value={workflow.name}
            onChange={(e) => handleUpdateWorkflow({ name: e.target.value })}
            placeholder="工作流名称"
          />
          <Textarea
            className="text-muted-foreground border-none bg-transparent resize-none p-0 focus-visible:ring-0 w-full max-w-md"
            value={workflow.description}
            onChange={(e) => handleUpdateWorkflow({ description: e.target.value })}
            placeholder="添加描述..."
            rows={2}
          />
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => navigate('/workflows')}>
            取消
          </Button>
          <Button variant="default" onClick={handleSave} disabled={isSaving}>
            <Save className="mr-2 h-4 w-4" />
            {isSaving ? '保存中...' : '保存'}
          </Button>
        </div>
      </div>
      
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* 左侧步骤列表 */}
        <div className="md:col-span-1 space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex justify-between items-center">
                <span>工作流步骤</span>
                <Button size="sm" variant="ghost" onClick={handleAddStep}>
                  <Plus className="h-4 w-4" />
                </Button>
              </CardTitle>
            </CardHeader>
            <CardContent>
              {workflow.steps.length === 0 ? (
                <div className="text-center p-4 border border-dashed rounded-md">
                  <p className="text-muted-foreground">还没有步骤</p>
                  <Button className="mt-2" size="sm" variant="outline" onClick={handleAddStep}>
                    添加第一个步骤
                  </Button>
                </div>
              ) : (
                <div className="space-y-2">
                  {workflow.steps.map((step) => (
                    <div
                      key={step.id}
                      className={`p-3 rounded-md cursor-pointer border ${
                        selectedStep === step.id ? 'border-primary bg-primary/5' : 'border-border'
                      }`}
                      onClick={() => setSelectedStep(step.id)}
                    >
                      <div className="flex justify-between items-center">
                        <div className="font-medium truncate">{step.name}</div>
                        <div className="flex items-center">
                          <span className="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded">
                            {step.type === 'agent' ? '代理' : 
                             step.type === 'condition' ? '条件' : 
                             step.type === 'loop' ? '循环' : '输出'}
                          </span>
                          {workflow.firstStepId === step.id && (
                            <span className="ml-2 text-xs bg-primary/20 text-primary px-2 py-0.5 rounded">
                              起始
                            </span>
                          )}
                        </div>
                      </div>
                      {step.description && (
                        <p className="text-xs text-muted-foreground mt-1 truncate">
                          {step.description}
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader>
              <CardTitle>工作流设置</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label htmlFor="workflow-active">启用工作流</Label>
                  <Switch
                    id="workflow-active"
                    checked={workflow.isActive}
                    onCheckedChange={(checked) => handleUpdateWorkflow({ isActive: checked })}
                  />
                </div>
                
                <div className="space-y-2">
                  <Label>起始步骤</Label>
                  {workflow.steps.length === 0 ? (
                    <p className="text-sm text-muted-foreground">先添加步骤</p>
                  ) : (
                    <Select 
                      value={workflow.firstStepId} 
                      onValueChange={(value) => handleUpdateWorkflow({ firstStepId: value })}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="选择起始步骤" />
                      </SelectTrigger>
                      <SelectContent>
                        {workflow.steps.map((step) => (
                          <SelectItem key={step.id} value={step.id}>
                            {step.name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  )}
                </div>
                
                <div className="space-y-2">
                  <Label>标签</Label>
                  <div className="flex flex-wrap gap-1">
                    {workflow.tags.map((tag, index) => (
                      <span 
                        key={index} 
                        className="text-xs bg-muted px-2 py-1 rounded-full flex items-center"
                      >
                        {tag}
                        <X
                          className="ml-1 h-3 w-3 cursor-pointer hover:text-destructive" 
                          onClick={() => handleUpdateWorkflow({
                            tags: workflow.tags.filter((_, i) => i !== index)
                          })}
                        />
                      </span>
                    ))}
                    <Input 
                      className="w-20 h-6 text-xs"
                      placeholder="添加..."
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' && e.currentTarget.value) {
                          handleUpdateWorkflow({
                            tags: [...workflow.tags, e.currentTarget.value]
                          });
                          e.currentTarget.value = '';
                        }
                      }}
                    />
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
        
        {/* 右侧步骤编辑器 */}
        <div className="md:col-span-2">
          {currentStep ? (
            <Card>
              <CardHeader className="flex flex-row items-start justify-between space-y-0">
                <div>
                  <CardTitle>编辑步骤</CardTitle>
                  <CardDescription>设置步骤属性和连接</CardDescription>
                </div>
                <Button 
                  variant="destructive" 
                  size="sm" 
                  onClick={() => setIsDeleteDialogOpen(true)}
                >
                  <Trash2 className="h-4 w-4 mr-1" />
                  删除
                </Button>
              </CardHeader>
              <CardContent>
                <Tabs defaultValue="basic">
                  <TabsList className="mb-4">
                    <TabsTrigger value="basic">基本信息</TabsTrigger>
                    <TabsTrigger value="config">配置</TabsTrigger>
                    <TabsTrigger value="connections">连接</TabsTrigger>
                  </TabsList>
                  
                  <TabsContent value="basic" className="space-y-4">
                    <div className="space-y-2">
                      <Label htmlFor="step-name">步骤名称</Label>
                      <Input
                        id="step-name"
                        value={currentStep.name}
                        onChange={(e) => handleUpdateStep(currentStep.id, { name: e.target.value })}
                        placeholder="步骤名称"
                      />
                    </div>
                    
                    <div className="space-y-2">
                      <Label htmlFor="step-description">描述（可选）</Label>
                      <Textarea
                        id="step-description"
                        value={currentStep.description || ''}
                        onChange={(e) => handleUpdateStep(currentStep.id, { description: e.target.value })}
                        placeholder="描述这个步骤的用途..."
                        rows={3}
                      />
                    </div>
                    
                    <div className="space-y-2">
                      <Label htmlFor="step-type">步骤类型</Label>
                      <Select 
                        value={currentStep.type} 
                        onValueChange={(value) => handleUpdateStep(
                          currentStep.id, 
                          { 
                            type: value as Step['type'],
                            // 重置不相关的字段
                            ...(value === 'agent' ? { agentId: demoAgents[0].id } : {}),
                            ...(value === 'condition' ? { condition: '' } : {}),
                            ...(value === 'loop' ? { iterations: 5 } : {}),
                            ...(value === 'output' ? { outputFormat: 'json' } : {})
                          }
                        )}
                      >
                        <SelectTrigger id="step-type">
                          <SelectValue placeholder="选择步骤类型" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="agent">代理步骤</SelectItem>
                          <SelectItem value="condition">条件步骤</SelectItem>
                          <SelectItem value="loop">循环步骤</SelectItem>
                          <SelectItem value="output">输出步骤</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    
                    <div className="flex items-center space-x-2 pt-2">
                      <Switch 
                        id="step-active" 
                        checked={currentStep.isActive}
                        onCheckedChange={(checked) => handleUpdateStep(currentStep.id, { isActive: checked })}
                      />
                      <Label htmlFor="step-active">启用此步骤</Label>
                    </div>
                  </TabsContent>
                  
                  <TabsContent value="config" className="space-y-4">
                    {currentStep.type === 'agent' && (
                      <div className="space-y-2">
                        <Label htmlFor="agent-id">选择代理</Label>
                        <Select 
                          value={currentStep.agentId} 
                          onValueChange={(value) => handleUpdateStep(currentStep.id, { agentId: value })}
                        >
                          <SelectTrigger id="agent-id">
                            <SelectValue placeholder="选择代理" />
                          </SelectTrigger>
                          <SelectContent>
                            {demoAgents.map((agent) => (
                              <SelectItem key={agent.id} value={agent.id}>
                                {agent.name}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                        <p className="text-xs text-muted-foreground">
                          {demoAgents.find(a => a.id === currentStep.agentId)?.description || '选择代理'}
                        </p>
                      </div>
                    )}
                    
                    {currentStep.type === 'condition' && (
                      <div className="space-y-2">
                        <Label htmlFor="condition">条件表达式</Label>
                        <Textarea
                          id="condition"
                          value={currentStep.condition || ''}
                          onChange={(e) => handleUpdateStep(currentStep.id, { condition: e.target.value })}
                          placeholder="输入条件表达式，如：data.score > 0.5"
                          rows={3}
                        />
                        <p className="text-xs text-muted-foreground">
                          条件为true时将执行第一个后续步骤，为false时执行第二个后续步骤（如果有）
                        </p>
                      </div>
                    )}
                    
                    {currentStep.type === 'loop' && (
                      <div className="space-y-2">
                        <Label htmlFor="iterations">最大迭代次数</Label>
                        <Input
                          id="iterations"
                          type="number"
                          min={1}
                          max={100}
                          value={currentStep.iterations || 5}
                          onChange={(e) => handleUpdateStep(currentStep.id, { 
                            iterations: parseInt(e.target.value) || 5 
                          })}
                        />
                        <p className="text-xs text-muted-foreground">
                          循环将执行指定次数或直到满足退出条件
                        </p>
                      </div>
                    )}
                    
                    {currentStep.type === 'output' && (
                      <div className="space-y-2">
                        <Label htmlFor="output-format">输出格式</Label>
                        <Select 
                          value={currentStep.outputFormat || 'json'} 
                          onValueChange={(value) => handleUpdateStep(currentStep.id, { outputFormat: value })}
                        >
                          <SelectTrigger id="output-format">
                            <SelectValue placeholder="选择输出格式" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="json">JSON</SelectItem>
                            <SelectItem value="text">纯文本</SelectItem>
                            <SelectItem value="markdown">Markdown</SelectItem>
                            <SelectItem value="html">HTML</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    )}
                  </TabsContent>
                  
                  <TabsContent value="connections" className="space-y-4">
                    <div className="space-y-2">
                      <Label>后续步骤</Label>
                      {workflow.steps.length <= 1 ? (
                        <p className="text-sm text-muted-foreground">
                          需要添加更多步骤才能创建连接
                        </p>
                      ) : (
                        <div className="space-y-2">
                          {currentStep.type === 'condition' ? (
                            <>
                              <div className="space-y-1">
                                <p className="text-xs text-muted-foreground">条件为真时执行:</p>
                                <Select 
                                  value={currentStep.nextSteps?.[0] || ''} 
                                  onValueChange={(value) => {
                                    const nextSteps = [...(currentStep.nextSteps || [])];
                                    nextSteps[0] = value;
                                    handleUpdateStep(currentStep.id, { nextSteps });
                                  }}
                                >
                                  <SelectTrigger>
                                    <SelectValue placeholder="选择步骤" />
                                  </SelectTrigger>
                                  <SelectContent>
                                    <SelectItem value="">无</SelectItem>
                                    {workflow.steps
                                      .filter(s => s.id !== currentStep.id)
                                      .map((step) => (
                                        <SelectItem key={step.id} value={step.id}>
                                          {step.name}
                                        </SelectItem>
                                      ))}
                                  </SelectContent>
                                </Select>
                              </div>
                              <div className="space-y-1">
                                <p className="text-xs text-muted-foreground">条件为假时执行:</p>
                                <Select 
                                  value={currentStep.nextSteps?.[1] || ''} 
                                  onValueChange={(value) => {
                                    const nextSteps = [...(currentStep.nextSteps || []), ''];
                                    nextSteps[1] = value;
                                    handleUpdateStep(currentStep.id, { nextSteps });
                                  }}
                                >
                                  <SelectTrigger>
                                    <SelectValue placeholder="选择步骤" />
                                  </SelectTrigger>
                                  <SelectContent>
                                    <SelectItem value="">无</SelectItem>
                                    {workflow.steps
                                      .filter(s => s.id !== currentStep.id)
                                      .map((step) => (
                                        <SelectItem key={step.id} value={step.id}>
                                          {step.name}
                                        </SelectItem>
                                      ))}
                                  </SelectContent>
                                </Select>
                              </div>
                            </>
                          ) : (
                            <Select 
                              value={currentStep.nextSteps?.[0] || ''} 
                              onValueChange={(value) => {
                                handleUpdateStep(currentStep.id, { 
                                  nextSteps: value ? [value] : [] 
                                });
                              }}
                            >
                              <SelectTrigger>
                                <SelectValue placeholder="选择步骤" />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="">无</SelectItem>
                                {workflow.steps
                                  .filter(s => s.id !== currentStep.id)
                                  .map((step) => (
                                    <SelectItem key={step.id} value={step.id}>
                                      {step.name}
                                    </SelectItem>
                                  ))}
                              </SelectContent>
                            </Select>
                          )}
                        </div>
                      )}
                    </div>
                    
                    <div className="space-y-2">
                      <Label>流程预览</Label>
                      <div className="border rounded-md p-4 space-y-2">
                        {workflow.steps.filter(s => s.nextSteps?.includes(currentStep.id)).length > 0 ? (
                          <>
                            <p className="text-sm">前置步骤:</p>
                            <div className="flex flex-wrap gap-2 mb-2">
                              {workflow.steps
                                .filter(s => s.nextSteps?.includes(currentStep.id))
                                .map(step => (
                                  <span 
                                    key={step.id}
                                    className="text-xs bg-muted px-2 py-1 rounded cursor-pointer"
                                    onClick={() => setSelectedStep(step.id)}
                                  >
                                    {step.name}
                                  </span>
                                ))}
                            </div>
                          </>
                        ) : workflow.firstStepId === currentStep.id ? (
                          <p className="text-sm text-muted-foreground">这是起始步骤</p>
                        ) : (
                          <p className="text-sm text-muted-foreground">没有前置步骤</p>
                        )}
                        
                        <Separator />
                        
                        <p className="text-sm mt-2">当前步骤: <span className="font-medium">{currentStep.name}</span></p>
                        
                        <Separator />
                        
                        {currentStep.nextSteps && currentStep.nextSteps.length > 0 ? (
                          <>
                            <p className="text-sm mt-2">后续步骤:</p>
                            <div className="flex flex-wrap gap-2">
                              {currentStep.nextSteps.map((nextId, index) => {
                                const nextStep = workflow.steps.find(s => s.id === nextId);
                                if (!nextStep) return null;
                                return (
                                  <div 
                                    key={nextId}
                                    className="text-xs bg-muted px-2 py-1 rounded flex items-center cursor-pointer"
                                    onClick={() => setSelectedStep(nextId)}
                                  >
                                    {currentStep.type === 'condition' && (
                                      <span className="mr-1 text-muted-foreground">
                                        {index === 0 ? '真:' : '假:'}
                                      </span>
                                    )}
                                    {nextStep.name}
                                  </div>
                                );
                              })}
                            </div>
                          </>
                        ) : (
                          <p className="text-sm text-muted-foreground mt-2">没有后续步骤</p>
                        )}
                      </div>
                    </div>
                  </TabsContent>
                </Tabs>
              </CardContent>
            </Card>
          ) : (
            <Card>
              <CardContent className="flex flex-col items-center justify-center p-12 text-center">
                <Settings className="h-12 w-12 text-muted-foreground mb-4" />
                <h3 className="text-lg font-medium mb-2">工作流编辑器</h3>
                <p className="text-muted-foreground mb-4">
                  {workflow.steps.length === 0 
                    ? "开始创建工作流。添加第一个步骤开始构建流程。" 
                    : "选择左侧的步骤进行编辑。"}
                </p>
                {workflow.steps.length === 0 && (
                  <Button onClick={handleAddStep}>
                    <Plus className="mr-2 h-4 w-4" /> 添加步骤
                  </Button>
                )}
              </CardContent>
            </Card>
          )}
        </div>
      </div>
      
      {/* 删除确认对话框 */}
      <AlertDialog open={isDeleteDialogOpen} onOpenChange={setIsDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              您确定要删除这个步骤吗？这个操作无法撤销，并可能影响工作流连接。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction 
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              onClick={() => currentStep && handleDeleteStep(currentStep.id)}
            >
              删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
} 