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

export default function WorkflowEditor() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { toast } = useToast();
  
  const [workflow, setWorkflow] = useState<any>(null);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isDragging, setIsDragging] = useState(false);
  const [isAddNodeDialogOpen, setIsAddNodeDialogOpen] = useState(false);
  const [newNodeType, setNewNodeType] = useState('agent');
  const [nodeName, setNodeName] = useState('');

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

  if (isLoading) {
    return <div className="flex items-center justify-center h-screen">Loading workflow...</div>;
  }

  return (
    <div className="flex flex-col h-screen">
      <div className="flex justify-between items-center p-4 border-b">
        <div className="flex items-center">
          <Button variant="ghost" onClick={() => navigate('/workflows')}>
            <ArrowLeftIcon className="h-4 w-4 mr-2" />
            Back to Workflows
          </Button>
          <h1 className="text-xl font-bold ml-4">{workflow?.name || 'Workflow Editor'}</h1>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" onClick={runWorkflow}>
            <PlayIcon className="h-4 w-4 mr-2" />
            Run
          </Button>
          <Button onClick={saveWorkflow}>
            <SaveIcon className="h-4 w-4 mr-2" />
            Save
          </Button>
        </div>
      </div>
      
      <div className="flex flex-1 overflow-hidden">
        <div className="w-64 p-4 border-r overflow-y-auto">
          <h2 className="text-lg font-semibold mb-4">Node Types</h2>
          
          <div className="space-y-2">
            <Card 
              draggable 
              onDragStart={(event) => {
                event.dataTransfer.setData('application/reactflow/type', 'agent');
                setIsDragging(true);
              }}
              onDragEnd={() => setIsDragging(false)}
              className="cursor-move"
            >
              <CardContent className="p-4">Agent</CardContent>
            </Card>
            
            <Card 
              draggable 
              onDragStart={(event) => {
                event.dataTransfer.setData('application/reactflow/type', 'condition');
                setIsDragging(true);
              }}
              onDragEnd={() => setIsDragging(false)}
              className="cursor-move"
            >
              <CardContent className="p-4">Condition</CardContent>
            </Card>
            
            <Card 
              draggable 
              onDragStart={(event) => {
                event.dataTransfer.setData('application/reactflow/type', 'tool');
                setIsDragging(true);
              }}
              onDragEnd={() => setIsDragging(false)}
              className="cursor-move"
            >
              <CardContent className="p-4">Tool</CardContent>
            </Card>
          </div>
        </div>
        
        <div className="flex-1" ref={reactFlowWrapper}>
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            nodeTypes={nodeTypes}
            onInit={setReactFlowInstance}
            onDrop={onDrop}
            onDragOver={onDragOver}
            fitView
          >
            <Background color="#f8f9fa" gap={16} />
            <Controls />
            <Panel position="top-right">
              <Dialog open={isAddNodeDialogOpen} onOpenChange={setIsAddNodeDialogOpen}>
                <DialogTrigger asChild>
                  <Button>
                    <Plus className="h-4 w-4 mr-2" />
                    Add Node
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Add New Node</DialogTitle>
                  </DialogHeader>
                  <div className="grid gap-4 py-4">
                    <div className="grid gap-2">
                      <Label htmlFor="node-name">Node Name</Label>
                      <Input 
                        id="node-name" 
                        value={nodeName} 
                        onChange={(e) => setNodeName(e.target.value)}
                        placeholder="Enter node name" 
                      />
                    </div>
                    
                    <div className="grid gap-2">
                      <Label htmlFor="node-type">Node Type</Label>
                      <Select value={newNodeType} onValueChange={setNewNodeType}>
                        <SelectTrigger>
                          <SelectValue placeholder="Select node type" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="agent">Agent</SelectItem>
                          <SelectItem value="condition">Condition</SelectItem>
                          <SelectItem value="tool">Tool</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                  <div className="flex justify-end gap-2">
                    <Button variant="outline" onClick={() => setIsAddNodeDialogOpen(false)}>
                      Cancel
                    </Button>
                    <Button onClick={addNode}>
                      Add
                    </Button>
                  </div>
                </DialogContent>
              </Dialog>
            </Panel>
          </ReactFlow>
        </div>
      </div>
    </div>
  );
} 