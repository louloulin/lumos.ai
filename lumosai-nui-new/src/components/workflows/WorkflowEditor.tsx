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
import { Loader2, Save, Play, Plus, Trash2 } from 'lucide-react';
import { Badge } from '../ui/badge';
import { Alert, AlertDescription } from '../ui/alert';

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

  // 加载状态
  if (loading) {
    return (
      <div className="flex items-center justify-center h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
        <span className="ml-2">加载工作流...</span>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="editor">编辑器</TabsTrigger>
          <TabsTrigger value="details">工作流详情</TabsTrigger>
          <TabsTrigger value="runs">执行历史</TabsTrigger>
        </TabsList>
        
        <TabsContent value="editor" className="flex-1">
          <div className="flex flex-col h-[700px]">
            <div className="flex justify-between p-2 bg-muted/20 rounded-t-md">
              <div className="flex items-center">
                <h2 className="text-lg font-semibold mr-2">
                  {workflowDetails.name || '未命名工作流'}
                </h2>
                {workflowDetails.tags.map(tag => (
                  <Badge key={tag} variant="secondary" className="mr-1">
                    {tag}
                  </Badge>
                ))}
              </div>
              
              <div className="flex items-center gap-2">
                <Button 
                  variant="outline" 
                  size="sm" 
                  disabled={executing}
                  onClick={executeWorkflow}
                >
                  {executing ? (
                    <>
                      <Loader2 className="mr-1 h-3 w-3 animate-spin" />
                      执行中
                    </>
                  ) : (
                    <>
                      <Play className="mr-1 h-3 w-3" />
                      执行
                    </>
                  )}
                </Button>
                
                <Button 
                  size="sm" 
                  disabled={saving}
                  onClick={saveWorkflow}
                >
                  {saving ? (
                    <>
                      <Loader2 className="mr-1 h-3 w-3 animate-spin" />
                      保存中
                    </>
                  ) : (
                    <>
                      <Save className="mr-1 h-3 w-3" />
                      保存
                    </>
                  )}
                </Button>
              </div>
            </div>
            
            {error && (
              <Alert variant="destructive" className="mt-2">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            
            {success && (
              <Alert className="mt-2 bg-green-50 border-green-200 text-green-800">
                <AlertDescription>{success}</AlertDescription>
              </Alert>
            )}
            
            <div className="flex-1 relative border rounded-b-md overflow-hidden" ref={reactFlowWrapper}>
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
                >
                  <Background />
                  <Controls />
                  <MiniMap />
                  
                  <Panel position="top-left" className="flex flex-col gap-2 p-2 bg-background rounded shadow">
                    <h3 className="text-sm font-medium mb-1">节点类型</h3>
                    <div className="flex flex-wrap gap-2">
                      <div
                        className="px-2 py-1 text-xs bg-blue-50 border border-blue-200 rounded cursor-move hover:bg-blue-100"
                        onDragStart={(e) => onDragStart(e, 'input')}
                        draggable
                      >
                        输入
                      </div>
                      <div
                        className="px-2 py-1 text-xs bg-green-50 border border-green-200 rounded cursor-move hover:bg-green-100"
                        onDragStart={(e) => onDragStart(e, 'output')}
                        draggable
                      >
                        输出
                      </div>
                      <div
                        className="px-2 py-1 text-xs bg-purple-50 border border-purple-200 rounded cursor-move hover:bg-purple-100"
                        onDragStart={(e) => onDragStart(e, 'llm')}
                        draggable
                      >
                        LLM
                      </div>
                      <div
                        className="px-2 py-1 text-xs bg-orange-50 border border-orange-200 rounded cursor-move hover:bg-orange-100"
                        onDragStart={(e) => onDragStart(e, 'tool')}
                        draggable
                      >
                        工具
                      </div>
                      <div
                        className="px-2 py-1 text-xs bg-yellow-50 border border-yellow-200 rounded cursor-move hover:bg-yellow-100"
                        onDragStart={(e) => onDragStart(e, 'condition')}
                        draggable
                      >
                        条件
                      </div>
                    </div>
                  </Panel>
                  
                  {selectedNode && (
                    <Panel position="top-right" className="w-60 p-3 bg-background rounded shadow">
                      <h3 className="text-sm font-medium mb-2">节点属性</h3>
                      <div className="space-y-2">
                        <div>
                          <Label htmlFor="nodeName" className="text-xs">名称</Label>
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
                            className="h-7 text-xs"
                          />
                        </div>
                        
                        <div className="flex justify-end">
                          <Button
                            variant="destructive"
                            size="sm"
                            onClick={deleteSelectedNode}
                            className="h-7 text-xs"
                          >
                            <Trash2 className="h-3 w-3 mr-1" />
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
        
        <TabsContent value="details">
          <Card>
            <CardContent className="pt-6">
              <div className="space-y-4">
                <div className="grid gap-2">
                  <Label htmlFor="name">工作流名称</Label>
                  <Input
                    id="name"
                    name="name"
                    value={workflowDetails.name}
                    onChange={handleWorkflowDetailsChange}
                    placeholder="输入工作流名称"
                  />
                </div>
                
                <div className="grid gap-2">
                  <Label htmlFor="description">工作流描述</Label>
                  <Input
                    id="description"
                    name="description"
                    value={workflowDetails.description}
                    onChange={handleWorkflowDetailsChange}
                    placeholder="描述此工作流的功能和用途"
                  />
                </div>
                
                <div className="grid gap-2">
                  <Label htmlFor="tags">标签</Label>
                  <div className="flex items-center gap-2">
                    <Input
                      id="tags"
                      value={newTag}
                      onChange={(e) => setNewTag(e.target.value)}
                      placeholder="添加标签"
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
                    >
                      <Plus className="h-4 w-4" />
                    </Button>
                  </div>
                  
                  {workflowDetails.tags.length > 0 && (
                    <div className="flex flex-wrap gap-2 mt-2">
                      {workflowDetails.tags.map(tag => (
                        <Badge key={tag} variant="secondary" className="flex items-center gap-1 px-2 py-1">
                          {tag}
                          <button
                            type="button"
                            className="text-muted-foreground hover:text-foreground"
                            onClick={() => removeTag(tag)}
                          >
                            <Trash2 className="h-3 w-3" />
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
        
        <TabsContent value="runs">
          <div className="flex items-center justify-center h-[400px] border rounded-lg bg-slate-50">
            <p className="text-gray-500">工作流执行历史记录即将推出</p>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
} 