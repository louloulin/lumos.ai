import React, { useState, useCallback, useRef, useEffect } from 'react';
import ReactFlow, {
  ReactFlowProvider,
  Background,
  Controls,
  MiniMap,
  addEdge,
  Panel,
  useNodesState,
  useEdgesState,
  Connection,
  Edge,
  Node,
  NodeChange,
  EdgeChange,
  useReactFlow,
} from 'reactflow';
import 'reactflow/dist/style.css';

import { workflowsService, Workflow, WorkflowNode as WorkflowNodeType, WorkflowEdge, NodeType } from '../../services/workflows';
import { useAuth } from '../../services/auth';
import { Card, CardContent } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Button } from '../ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { Loader2, Save, Play, Plus, Trash2, Settings, MoreHorizontal, XCircle, Code, Search, Info } from 'lucide-react';
import { Badge } from '../ui/badge';
import { Alert, AlertDescription } from '../ui/alert';
import { Separator } from '../ui/separator';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '../ui/tooltip';

// 导入自定义节点
import { InputNode } from './nodes/InputNode';
import { OutputNode } from './nodes/OutputNode';
import { LLMNode } from './nodes/LLMNode';
import { ToolNode } from './nodes/ToolNode';
import { ConditionNode } from './nodes/ConditionNode';

// 节点类型映射
const nodeTypes = {
  input: InputNode,
  output: OutputNode,
  llm: LLMNode,
  tool: ToolNode,
  condition: ConditionNode,
};

// 自定义样式
const flowStyles = {
  background: '#121212',
  height: '100%',
};

interface WorkflowEditorProps {
  workflowId?: string;
  onSave?: (workflow: Workflow) => void;
}

export function WorkflowEditor({ workflowId, onSave }: WorkflowEditorProps) {
  const { user } = useAuth();
  const reactFlowWrapper = useRef<HTMLDivElement | null>(null);
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [workflowDetails, setWorkflowDetails] = useState<{
    id: string;
    name: string;
    description: string;
    isPublic: boolean;
    tags: string[];
    status: 'draft' | 'active' | 'archived';
  }>({
    id: '',
    name: '',
    description: '',
    isPublic: false,
    tags: [],
    status: 'draft',
  });
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [executing, setExecuting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);
  const [newTag, setNewTag] = useState('');
  const [activeTab, setActiveTab] = useState('editor');

  const reactFlowInstance = useReactFlow();

  // 加载工作流数据
  useEffect(() => {
    const loadWorkflow = async () => {
      if (workflowId) {
        setLoading(true);
        setError(null);
        
        try {
          const workflow = await workflowsService.getWorkflowById(workflowId);
          
          if (workflow) {
            // 转换工作流节点为ReactFlow节点
            const flowNodes = workflow.nodes.map((node: WorkflowNodeType) => ({
              id: node.id,
              type: node.type,
              position: node.position,
              data: node.data,
            }));
            
            // 转换工作流边为ReactFlow边
            const flowEdges = workflow.edges.map((edge: WorkflowEdge) => ({
              id: edge.id,
              source: edge.source,
              target: edge.target,
              sourceHandle: edge.sourceHandle,
              targetHandle: edge.targetHandle,
              label: edge.label,
            }));
            
            setNodes(flowNodes);
            setEdges(flowEdges);
            
            setWorkflowDetails({
              id: workflow.id,
              name: workflow.name,
              description: workflow.description,
              isPublic: workflow.isPublic,
              tags: workflow.tags,
              status: workflow.status,
            });
          }
        } catch (err) {
          console.error("Error loading workflow:", err);
          setError("加载工作流时出错");
        } finally {
          setLoading(false);
        }
      }
    };
    
    loadWorkflow();
  }, [workflowId]);

  // 保存工作流
  const saveWorkflow = async () => {
    if (!user) {
      setError("请先登录");
      return;
    }
    
    setSaving(true);
    setError(null);
    setSuccess(null);
    
    try {
      const workflowNodes: WorkflowNodeType[] = nodes.map(node => ({
        id: node.id,
        type: node.type as NodeType,
        position: node.position,
        data: node.data,
      }));
      
      const workflowEdges: WorkflowEdge[] = edges.map(edge => ({
        id: edge.id,
        source: edge.source,
        target: edge.target,
        sourceHandle: edge.sourceHandle,
        targetHandle: edge.targetHandle,
        label: edge.label,
      }));
      
      let savedWorkflow: Workflow;
      
      if (workflowId) {
        // 更新现有工作流
        savedWorkflow = await workflowsService.updateWorkflow(workflowId, {
          name: workflowDetails.name,
          description: workflowDetails.description,
          isPublic: workflowDetails.isPublic,
          nodes: workflowNodes,
          edges: workflowEdges,
          tags: workflowDetails.tags,
          status: workflowDetails.status,
        });
        
        setSuccess("工作流已成功更新");
      } else {
        // 创建新工作流
        savedWorkflow = await workflowsService.createWorkflow(
          {
            name: workflowDetails.name,
            description: workflowDetails.description,
            isPublic: workflowDetails.isPublic,
            tags: workflowDetails.tags,
          },
          user.id
        );
        
        // 更新新创建的工作流，添加节点和边
        savedWorkflow = await workflowsService.updateWorkflow(savedWorkflow.id, {
          nodes: workflowNodes,
          edges: workflowEdges,
        });
        
        setWorkflowDetails(prev => ({ ...prev, id: savedWorkflow.id }));
        setSuccess("工作流已成功创建");
      }
      
      if (onSave) {
        onSave(savedWorkflow);
      }
    } catch (err) {
      console.error("Error saving workflow:", err);
      setError(err instanceof Error ? err.message : "保存工作流时出错");
    } finally {
      setSaving(false);
      
      // 3秒后清除成功消息
      if (success) {
        setTimeout(() => setSuccess(null), 3000);
      }
    }
  };

  // 执行工作流
  const executeWorkflow = async () => {
    if (!workflowId) {
      setError("请先保存工作流");
      return;
    }
    
    setExecuting(true);
    setError(null);
    
    try {
      const inputs = { input: "示例输入" }; // 实际情况可能需要从UI收集输入
      await workflowsService.executeWorkflow(workflowId, inputs);
      setActiveTab('runs');
    } catch (err) {
      console.error("Error executing workflow:", err);
      setError(err instanceof Error ? err.message : "执行工作流时出错");
    } finally {
      setExecuting(false);
    }
  };

  // 处理边连接
  const onConnect = useCallback((params: Connection) => {
    setEdges(eds => addEdge({ ...params, id: `edge-${Date.now()}` }, eds));
  }, [setEdges]);

  // 处理拖放节点
  const onDragOver = useCallback((event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  // 处理节点放置
  const onDrop = useCallback(
    (event: React.DragEvent<HTMLDivElement>) => {
      event.preventDefault();

      if (!reactFlowWrapper.current || !reactFlowInstance) return;

      const reactFlowBounds = reactFlowWrapper.current.getBoundingClientRect();
      const type = event.dataTransfer.getData('application/reactflow') as NodeType;
      
      // 检查拖放类型是否有效
      if (!type) return;

      const position = reactFlowInstance.project({
        x: event.clientX - reactFlowBounds.left,
        y: event.clientY - reactFlowBounds.top,
      });

      // 创建不同类型的节点
      let newNode: Node = {
        id: `node-${Date.now()}`,
        type,
        position,
        data: { name: `新${getNodeTypeLabel(type)}节点` },
      };

      // 根据节点类型设置特定数据
      switch (type) {
        case 'input':
          newNode.data = {
            ...newNode.data,
            outputs: [{ id: 'out-1', name: '输出', type: 'string' }],
          };
          break;
        case 'output':
          newNode.data = {
            ...newNode.data,
            inputs: [{ id: 'in-1', name: '输入', type: 'string' }],
          };
          break;
        case 'llm':
          newNode.data = {
            ...newNode.data,
            description: 'LLM处理节点',
            config: {
              model: 'gpt-4',
              maxTokens: 1000,
              temperature: 0.7,
            },
            inputs: [{ id: 'in-1', name: '输入', type: 'string' }],
            outputs: [{ id: 'out-1', name: '输出', type: 'string' }],
          };
          break;
        case 'tool':
          newNode.data = {
            ...newNode.data,
            description: '工具调用节点',
            config: {
              toolName: '',
              params: {},
            },
            inputs: [{ id: 'in-1', name: '输入', type: 'string' }],
            outputs: [{ id: 'out-1', name: '输出', type: 'string' }],
          };
          break;
        case 'condition':
          newNode.data = {
            ...newNode.data,
            description: '条件分支节点',
            inputs: [{ id: 'in-1', name: '条件', type: 'string' }],
            outputs: [
              { id: 'out-1', name: '真', type: 'boolean' },
              { id: 'out-2', name: '假', type: 'boolean' },
            ],
          };
          break;
      }

      setNodes(nds => nds.concat(newNode));
    },
    [reactFlowInstance, setNodes]
  );

  // 删除选中节点
  const deleteSelectedNode = useCallback(() => {
    if (selectedNode) {
      setNodes(nodes => nodes.filter(node => node.id !== selectedNode.id));
      setEdges(edges => edges.filter(edge => 
        edge.source !== selectedNode.id && edge.target !== selectedNode.id
      ));
      setSelectedNode(null);
    }
  }, [selectedNode, setNodes, setEdges]);

  // 节点选择处理
  const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    setSelectedNode(node);
  }, []);

  // 背景点击处理
  const onPaneClick = useCallback(() => {
    setSelectedNode(null);
  }, []);

  // 处理工作流详情变化
  const handleWorkflowDetailsChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    const { name, value } = e.target;
    setWorkflowDetails(prev => ({ ...prev, [name]: value }));
  };

  // 添加标签
  const addTag = () => {
    if (newTag.trim() && !workflowDetails.tags.includes(newTag.trim())) {
      setWorkflowDetails(prev => ({
        ...prev,
        tags: [...prev.tags, newTag.trim()]
      }));
      setNewTag('');
    }
  };

  // 删除标签
  const removeTag = (tagToRemove: string) => {
    setWorkflowDetails(prev => ({
      ...prev,
      tags: prev.tags.filter(tag => tag !== tagToRemove)
    }));
  };

  // 获取节点类型标签
  const getNodeTypeLabel = (type: string): string => {
    const typeMap: Record<string, string> = {
      input: '输入',
      output: '输出',
      llm: 'LLM',
      tool: '工具',
      condition: '条件',
      function: '函数',
      data: '数据',
      process: '处理'
    };
    
    return typeMap[type] || type;
  };

  // 拖拽开始处理
  const onDragStart = (event: React.DragEvent<HTMLDivElement>, nodeType: NodeType) => {
    event.dataTransfer.setData('application/reactflow', nodeType);
    event.dataTransfer.effectAllowed = 'move';
  };

  // 获取节点类型颜色
  const getNodeTypeColor = (type: string): { bg: string, border: string, text: string, icon: JSX.Element } => {
    const typeColors: Record<string, { bg: string, border: string, text: string, icon: JSX.Element }> = {
      input: { 
        bg: 'bg-[#1E293B]', 
        border: 'border-[#334155]', 
        text: 'text-blue-400',
        icon: <Code size={12} className="text-blue-400" />
      },
      output: { 
        bg: 'bg-[#1A2E1A]', 
        border: 'border-[#294429]', 
        text: 'text-emerald-400',
        icon: <Code size={12} className="text-emerald-400" />
      },
      llm: { 
        bg: 'bg-[#2D1B3A]', 
        border: 'border-[#4A2D62]', 
        text: 'text-purple-400',
        icon: <Settings size={12} className="text-purple-400" />
      },
      tool: { 
        bg: 'bg-[#2E1C11]', 
        border: 'border-[#4F301D]', 
        text: 'text-orange-400',
        icon: <Settings size={12} className="text-orange-400" />
      },
      condition: { 
        bg: 'bg-[#2E2911]', 
        border: 'border-[#4F461D]', 
        text: 'text-yellow-400',
        icon: <Search size={12} className="text-yellow-400" />
      }
    };
    
    return typeColors[type] || { 
      bg: 'bg-[#1C1C1C]', 
      border: 'border-[#2E2E2E]', 
      text: 'text-gray-400',
      icon: <Info size={12} className="text-gray-400" />
    };
  };

  // 加载状态
  if (loading) {
    return (
      <div className="flex items-center justify-center h-[400px] bg-[#121212] text-white">
        <Loader2 className="h-8 w-8 animate-spin text-emerald-500" />
        <span className="ml-2 text-gray-300">加载工作流...</span>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-[#0A0A0A]">
      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
        <TabsList className="bg-[#1C1C1C] border border-[#2E2E2E] p-0.5 h-9 mb-4">
          <TabsTrigger value="editor" className="px-4 text-xs h-8 data-[state=active]:bg-[#2E2E2E] data-[state=active]:text-white">
            编辑器
          </TabsTrigger>
          <TabsTrigger value="details" className="px-4 text-xs h-8 data-[state=active]:bg-[#2E2E2E] data-[state=active]:text-white">
            工作流详情
          </TabsTrigger>
          <TabsTrigger value="runs" className="px-4 text-xs h-8 data-[state=active]:bg-[#2E2E2E] data-[state=active]:text-white">
            执行历史
          </TabsTrigger>
        </TabsList>
        
        <TabsContent value="editor" className="flex-1 mt-0">
          <div className="flex flex-col h-[700px]">
            <div className="flex justify-between items-center p-3 bg-[#1C1C1C] border border-[#2E2E2E] rounded-t-md">
              <div className="flex items-center">
                <h2 className="text-sm font-medium text-white mr-3">
                  {workflowDetails.name || '未命名工作流'}
                </h2>
                <div className="flex gap-1">
                  {workflowDetails.tags.slice(0, 3).map(tag => (
                    <Badge key={tag} className="px-1.5 py-0.5 bg-[#2E2E2E] text-[10px] font-normal text-gray-300 hover:bg-[#3E3E3E]">
                      {tag}
                    </Badge>
                  ))}
                  {workflowDetails.tags.length > 3 && (
                    <Badge className="px-1.5 py-0.5 bg-[#2E2E2E] text-[10px] font-normal text-gray-300 hover:bg-[#3E3E3E]">
                      +{workflowDetails.tags.length - 3}
                    </Badge>
                  )}
                </div>
                <Badge 
                  className={`ml-3 px-2 py-0.5 text-[10px] font-normal ${
                    workflowDetails.status === 'active' 
                      ? 'bg-emerald-900/30 text-emerald-400 border-emerald-900/50' 
                      : workflowDetails.status === 'draft' 
                      ? 'bg-amber-900/30 text-amber-400 border-amber-900/50'
                      : 'bg-gray-900/30 text-gray-400 border-gray-900/50'
                  }`}
                >
                  {workflowDetails.status === 'active' ? '已激活' : workflowDetails.status === 'draft' ? '草稿' : '已归档'}
                </Badge>
              </div>
              
              <div className="flex items-center gap-2">
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button 
                        variant="outline" 
                        size="sm" 
                        disabled={executing}
                        onClick={executeWorkflow}
                        className="h-7 text-xs border-[#2E2E2E] bg-[#1C1C1C] text-gray-300 hover:bg-[#2E2E2E] hover:text-white"
                      >
                        {executing ? (
                          <Loader2 className="h-3 w-3 animate-spin" />
                        ) : (
                          <Play className="h-3 w-3" />
                        )}
                        <span className="ml-1.5">执行</span>
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p className="text-xs">执行当前工作流</p>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
                
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button 
                        size="sm" 
                        disabled={saving}
                        onClick={saveWorkflow}
                        className="h-7 text-xs bg-emerald-600 hover:bg-emerald-700 text-white"
                      >
                        {saving ? (
                          <Loader2 className="h-3 w-3 animate-spin" />
                        ) : (
                          <Save className="h-3 w-3" />
                        )}
                        <span className="ml-1.5">保存</span>
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p className="text-xs">保存工作流</p>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              </div>
            </div>
            
            {error && (
              <Alert variant="destructive" className="mt-2 bg-red-900/20 border-red-900/50 text-red-300">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            
            {success && (
              <Alert className="mt-2 bg-emerald-900/20 border-emerald-900/50 text-emerald-300">
                <AlertDescription>{success}</AlertDescription>
              </Alert>
            )}
            
            <div className="flex-1 relative border border-[#2E2E2E] border-t-0 rounded-b-md overflow-hidden" ref={reactFlowWrapper}>
              <ReactFlowProvider>
                <ReactFlow
                  nodes={nodes}
                  edges={edges}
                  onNodesChange={onNodesChange}
                  onEdgesChange={onEdgesChange}
                  onConnect={onConnect}
                  onDrop={onDrop}
                  onDragOver={onDragOver}
                  onNodeClick={onNodeClick}
                  onPaneClick={onPaneClick}
                  nodeTypes={nodeTypes}
                  fitView
                  style={flowStyles}
                  defaultEdgeOptions={{
                    style: { stroke: '#3E3E3E', strokeWidth: 1.5 },
                    animated: true,
                  }}
                >
                  <Background 
                    color="#2E2E2E" 
                    gap={16} 
                    size={1}
                    variant="dots"
                  />
                  <Controls 
                    className="bg-[#1C1C1C] border border-[#2E2E2E] rounded-md p-1"
                    style={{ 
                      button: { 
                        backgroundColor: '#2E2E2E', 
                        color: '#9CA3AF', 
                        border: 'none',
                        width: '24px',
                        height: '24px',
                        margin: '2px'
                      } 
                    }}
                  />
                  <MiniMap 
                    style={{ 
                      backgroundColor: '#1C1C1C',
                      border: '1px solid #2E2E2E',
                      borderRadius: '4px'
                    }}
                    nodeColor={(node) => {
                      const type = node.type as NodeType || 'default';
                      switch (type) {
                        case 'input': return '#3B82F6';
                        case 'output': return '#10B981';
                        case 'llm': return '#8B5CF6';
                        case 'tool': return '#F97316';
                        case 'condition': return '#EAB308';
                        default: return '#9CA3AF';
                      }
                    }}
                    maskColor="rgba(12, 12, 12, 0.5)"
                  />
                  
                  <Panel position="top-left" className="p-3 bg-[#1C1C1C] border border-[#2E2E2E] rounded-md shadow-md">
                    <h3 className="text-xs font-medium text-white mb-2">节点类型</h3>
                    <Separator className="bg-[#2E2E2E] mb-2" />
                    <div className="grid grid-cols-2 gap-2">
                      {['input', 'output', 'llm', 'tool', 'condition'].map(type => {
                        const { bg, border, text, icon } = getNodeTypeColor(type);
                        return (
                          <div
                            key={type}
                            className={`px-2 py-1.5 text-xs ${bg} border ${border} rounded-md cursor-move hover:opacity-80 transition-opacity flex items-center gap-1.5`}
                            onDragStart={(e) => onDragStart(e, type as NodeType)}
                            draggable
                          >
                            {icon}
                            <span className={`${text}`}>{getNodeTypeLabel(type)}</span>
                          </div>
                        );
                      })}
                    </div>
                  </Panel>
                  
                  {selectedNode && (
                    <Panel position="top-right" className="w-64 p-3 bg-[#1C1C1C] border border-[#2E2E2E] rounded-md shadow-md">
                      <div className="flex items-center justify-between">
                        <h3 className="text-xs font-medium text-white">节点属性</h3>
                        <Button 
                          variant="ghost" 
                          size="sm" 
                          className="h-6 w-6 p-0" 
                          onClick={() => setSelectedNode(null)}>
                          <XCircle size={14} className="text-gray-400 hover:text-white" />
                        </Button>
                      </div>
                      <Separator className="bg-[#2E2E2E] my-2" />
                      <div className="space-y-3">
                        <div>
                          <Label htmlFor="nodeName" className="text-xs text-gray-400">名称</Label>
                          <Input
                            id="nodeName"
                            value={selectedNode.data.name || ''}
                            onChange={(e) => {
                              setNodes(nodes.map(n => {
                                if (n.id === selectedNode.id) {
                                  return {
                                    ...n,
                                    data: { ...n.data, name: e.target.value }
                                  };
                                }
                                return n;
                              }));
                              
                              setSelectedNode({
                                ...selectedNode,
                                data: { ...selectedNode.data, name: e.target.value }
                              });
                            }}
                            className="h-7 text-xs mt-1 bg-[#1C1C1C] border-[#2E2E2E] text-white focus:ring-1 focus:ring-emerald-500 focus:border-emerald-500"
                          />
                        </div>
                        
                        <div className="flex justify-end">
                          <Button
                            variant="destructive"
                            size="sm"
                            onClick={deleteSelectedNode}
                            className="h-7 text-xs flex items-center gap-1 bg-red-900/60 hover:bg-red-900 text-white"
                          >
                            <Trash2 className="h-3 w-3" />
                            删除节点
                          </Button>
                        </div>
                      </div>
                    </Panel>
                  )}
                </ReactFlow>
              </ReactFlowProvider>
            </div>
          </div>
        </TabsContent>
        
        <TabsContent value="details" className="mt-0">
          <Card className="bg-[#1C1C1C] border-[#2E2E2E] text-white shadow-md">
            <CardContent className="pt-6">
              <div className="space-y-4">
                <div className="grid gap-2">
                  <Label htmlFor="name" className="text-gray-400">工作流名称</Label>
                  <Input
                    id="name"
                    name="name"
                    value={workflowDetails.name}
                    onChange={handleWorkflowDetailsChange}
                    placeholder="输入工作流名称"
                    className="bg-[#1C1C1C] border-[#2E2E2E] text-white focus:ring-1 focus:ring-emerald-500 focus:border-emerald-500"
                  />
                </div>
                
                <div className="grid gap-2">
                  <Label htmlFor="description" className="text-gray-400">工作流描述</Label>
                  <Input
                    id="description"
                    name="description"
                    value={workflowDetails.description}
                    onChange={handleWorkflowDetailsChange}
                    placeholder="描述此工作流的功能和用途"
                    className="bg-[#1C1C1C] border-[#2E2E2E] text-white focus:ring-1 focus:ring-emerald-500 focus:border-emerald-500"
                  />
                </div>
                
                <div className="grid gap-2">
                  <Label htmlFor="tags" className="text-gray-400">标签</Label>
                  <div className="flex items-center gap-2">
                    <Input
                      id="tags"
                      value={newTag}
                      onChange={(e) => setNewTag(e.target.value)}
                      placeholder="添加标签"
                      className="bg-[#1C1C1C] border-[#2E2E2E] text-white focus:ring-1 focus:ring-emerald-500 focus:border-emerald-500"
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                          e.preventDefault();
                          addTag();
                        }
                      }}
                    />
                    <Button 
                      type="button" 
                      size="sm" 
                      onClick={addTag}
                      disabled={!newTag.trim()}
                      className="h-8 bg-emerald-600 hover:bg-emerald-700 text-white border-0"
                    >
                      <Plus className="h-4 w-4" />
                    </Button>
                  </div>
                  
                  {workflowDetails.tags.length > 0 && (
                    <div className="flex flex-wrap gap-2 mt-2">
                      {workflowDetails.tags.map(tag => (
                        <Badge key={tag} className="px-1.5 py-0.5 bg-[#2E2E2E] text-xs text-gray-300 hover:bg-[#3E3E3E] flex items-center gap-1">
                          {tag}
                          <button
                            type="button"
                            className="text-gray-400 hover:text-white"
                            onClick={() => removeTag(tag)}
                          >
                            <XCircle className="h-3 w-3" />
                          </button>
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>
                
                <div className="flex justify-end">
                  <Button 
                    onClick={saveWorkflow}
                    disabled={saving}
                    className="bg-emerald-600 hover:bg-emerald-700 text-white border-0"
                  >
                    {saving ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        保存中...
                      </>
                    ) : (
                      '保存工作流'
                    )}
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
        
        <TabsContent value="runs" className="mt-0">
          <div className="flex items-center justify-center h-[400px] border border-[#2E2E2E] rounded-lg bg-[#1C1C1C] text-center">
            <div>
              <Loader2 className="h-10 w-10 animate-spin text-gray-500 mx-auto mb-4" />
              <p className="text-gray-400">工作流执行历史记录即将推出</p>
              <p className="text-gray-500 text-sm mt-2">我们正在努力实现这一功能</p>
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
} 