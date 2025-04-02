import { useState, useEffect, useCallback } from 'react';
import { XYPosition } from 'reactflow';
import { User } from './auth';

// 工作流节点类型
export type NodeType = 'input' | 'output' | 'llm' | 'tool' | 'condition' | 'function' | 'data' | 'process';

// 工作流节点接口
export interface WorkflowNode {
  id: string;
  type: NodeType;
  position: XYPosition;
  data: {
    name: string;
    description?: string;
    inputs?: Array<{ id: string; name: string; type: string }>;
    outputs?: Array<{ id: string; name: string; type: string }>;
    config?: Record<string, any>;
    [key: string]: any;
  };
}

// 工作流边接口
export interface WorkflowEdge {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string;
  targetHandle?: string;
  label?: React.ReactNode;
}

// 工作流接口
export interface Workflow {
  id: string;
  name: string;
  description: string;
  createdAt: string;
  updatedAt: string;
  createdBy: string;
  isPublic: boolean;
  status: 'draft' | 'active' | 'archived';
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  tags: string[];
  metadata?: Record<string, any>;
}

// 新建工作流表单
export interface NewWorkflowForm {
  name: string;
  description: string;
  isPublic: boolean;
  tags: string[];
}

// 工作流执行记录
export interface WorkflowRun {
  id: string;
  workflowId: string;
  startTime: string;
  endTime?: string;
  status: 'running' | 'completed' | 'failed';
  inputs: Record<string, any>;
  outputs?: Record<string, any>;
  error?: string;
  logs: Array<{
    timestamp: string;
    nodeId: string;
    message: string;
    level: 'info' | 'warn' | 'error';
  }>;
}

// 工作流执行输入
export interface WorkflowExecuteInput {
  [key: string]: any;
}

// 模拟工作流数据
const MOCK_WORKFLOWS: Workflow[] = [
  {
    id: 'wf-1',
    name: '基础对话工作流',
    description: '简单的对话式AI处理工作流',
    createdAt: '2023-04-21T08:00:00Z',
    updatedAt: '2023-04-21T09:30:00Z',
    createdBy: 'user-1',
    isPublic: true,
    status: 'active',
    nodes: [
      {
        id: 'node-1',
        type: 'input',
        position: { x: 250, y: 100 },
        data: {
          name: '用户输入',
          outputs: [{ id: 'out-1', name: '输出', type: 'string' }]
        }
      },
      {
        id: 'node-2',
        type: 'llm',
        position: { x: 250, y: 250 },
        data: {
          name: 'GPT-4处理',
          description: '使用GPT-4处理用户输入',
          config: {
            model: 'gpt-4',
            maxTokens: 1000, 
            temperature: 0.7,
          },
          inputs: [{ id: 'in-1', name: '输入', type: 'string' }],
          outputs: [{ id: 'out-1', name: '输出', type: 'string' }]
        }
      },
      {
        id: 'node-3',
        type: 'output',
        position: { x: 250, y: 400 },
        data: {
          name: 'AI响应',
          inputs: [{ id: 'in-1', name: '输入', type: 'string' }]
        }
      }
    ],
    edges: [
      {
        id: 'edge-1',
        source: 'node-1',
        target: 'node-2',
        sourceHandle: 'out-1',
        targetHandle: 'in-1'
      },
      {
        id: 'edge-2',
        source: 'node-2',
        target: 'node-3',
        sourceHandle: 'out-1',
        targetHandle: 'in-1'
      }
    ],
    tags: ['对话', '基础', 'GPT-4']
  },
  {
    id: 'wf-2',
    name: '检索增强生成工作流',
    description: '结合知识库的RAG对话工作流',
    createdAt: '2023-04-22T10:00:00Z',
    updatedAt: '2023-04-22T14:30:00Z',
    createdBy: 'user-1',
    isPublic: true,
    status: 'active',
    nodes: [
      {
        id: 'node-1',
        type: 'input',
        position: { x: 100, y: 100 },
        data: {
          name: '用户问题',
          outputs: [{ id: 'out-1', name: '输出', type: 'string' }]
        }
      },
      {
        id: 'node-2',
        type: 'tool',
        position: { x: 100, y: 250 },
        data: {
          name: '知识库检索',
          description: '检索相关文档片段',
          config: {
            toolName: 'vector-search',
            params: {
              collectionName: 'documents',
              maxResults: 5
            }
          },
          inputs: [{ id: 'in-1', name: '查询', type: 'string' }],
          outputs: [{ id: 'out-1', name: '检索结果', type: 'array' }]
        }
      },
      {
        id: 'node-3',
        type: 'llm',
        position: { x: 100, y: 400 },
        data: {
          name: 'LLM回答生成',
          description: '根据检索结果生成回答',
          config: {
            model: 'gpt-4',
            maxTokens: 1500,
            temperature: 0.5
          },
          inputs: [
            { id: 'in-1', name: '用户问题', type: 'string' },
            { id: 'in-2', name: '检索结果', type: 'array' }
          ],
          outputs: [{ id: 'out-1', name: '回答', type: 'string' }]
        }
      },
      {
        id: 'node-4',
        type: 'output',
        position: { x: 100, y: 550 },
        data: {
          name: '最终回答',
          inputs: [{ id: 'in-1', name: '回答', type: 'string' }]
        }
      }
    ],
    edges: [
      {
        id: 'edge-1',
        source: 'node-1',
        target: 'node-2',
        sourceHandle: 'out-1',
        targetHandle: 'in-1'
      },
      {
        id: 'edge-2',
        source: 'node-1',
        target: 'node-3',
        sourceHandle: 'out-1',
        targetHandle: 'in-1'
      },
      {
        id: 'edge-3',
        source: 'node-2',
        target: 'node-3',
        sourceHandle: 'out-1',
        targetHandle: 'in-2'
      },
      {
        id: 'edge-4',
        source: 'node-3',
        target: 'node-4',
        sourceHandle: 'out-1',
        targetHandle: 'in-1'
      }
    ],
    tags: ['RAG', '知识库', '高级']
  }
];

// 模拟工作流执行记录
const MOCK_WORKFLOW_RUNS: Record<string, WorkflowRun[]> = {
  'wf-1': [
    {
      id: 'run-1',
      workflowId: 'wf-1',
      startTime: '2023-04-21T10:00:00Z',
      endTime: '2023-04-21T10:00:05Z',
      status: 'completed',
      inputs: { input: '你好，请介绍一下自己' },
      outputs: { output: '你好！我是一个AI助手，基于大型语言模型训练而成。我可以回答问题、提供信息、协助创作内容等。有什么我可以帮助你的吗？' },
      logs: [
        { timestamp: '2023-04-21T10:00:01Z', nodeId: 'node-1', message: '接收到用户输入', level: 'info' },
        { timestamp: '2023-04-21T10:00:03Z', nodeId: 'node-2', message: '处理完成', level: 'info' },
        { timestamp: '2023-04-21T10:00:05Z', nodeId: 'node-3', message: '输出生成完毕', level: 'info' }
      ]
    }
  ]
};

// 工作流服务类
class WorkflowsService {
  // 获取所有工作流
  async getAllWorkflows(userId?: string): Promise<Workflow[]> {
    await this.delay(500);
    
    if (userId) {
      return MOCK_WORKFLOWS.filter(wf => wf.createdBy === userId || wf.isPublic);
    }
    
    return MOCK_WORKFLOWS.filter(wf => wf.isPublic);
  }
  
  // 通过ID获取工作流
  async getWorkflowById(id: string): Promise<Workflow | null> {
    await this.delay(300);
    
    const workflow = MOCK_WORKFLOWS.find(wf => wf.id === id);
    return workflow || null;
  }
  
  // 创建工作流
  async createWorkflow(workflowData: NewWorkflowForm, userId: string): Promise<Workflow> {
    await this.delay(600);
    
    const now = new Date().toISOString();
    const newWorkflow: Workflow = {
      id: `wf-${Date.now()}`,
      ...workflowData,
      createdAt: now,
      updatedAt: now,
      createdBy: userId,
      status: 'draft',
      nodes: [],
      edges: [],
      tags: workflowData.tags || []
    };
    
    // 在实际应用中这里会将工作流保存到数据库
    // 这里我们只是模拟
    MOCK_WORKFLOWS.push(newWorkflow);
    
    return newWorkflow;
  }
  
  // 更新工作流
  async updateWorkflow(
    id: string, 
    updates: Partial<Omit<Workflow, 'id' | 'createdAt' | 'createdBy'>>
  ): Promise<Workflow> {
    await this.delay(500);
    
    const workflowIndex = MOCK_WORKFLOWS.findIndex(wf => wf.id === id);
    
    if (workflowIndex === -1) {
      throw new Error(`Workflow with id ${id} not found`);
    }
    
    const updatedWorkflow = {
      ...MOCK_WORKFLOWS[workflowIndex],
      ...updates,
      updatedAt: new Date().toISOString()
    };
    
    // 更新模拟数据
    MOCK_WORKFLOWS[workflowIndex] = updatedWorkflow;
    
    return updatedWorkflow;
  }
  
  // 删除工作流
  async deleteWorkflow(id: string): Promise<boolean> {
    await this.delay(400);
    
    const workflowIndex = MOCK_WORKFLOWS.findIndex(wf => wf.id === id);
    
    if (workflowIndex === -1) {
      throw new Error(`Workflow with id ${id} not found`);
    }
    
    // 从数组中移除
    MOCK_WORKFLOWS.splice(workflowIndex, 1);
    
    return true;
  }
  
  // 执行工作流
  async executeWorkflow(id: string, inputs: WorkflowExecuteInput): Promise<WorkflowRun> {
    await this.delay(1000); // 模拟执行时间
    
    const workflow = await this.getWorkflowById(id);
    
    if (!workflow) {
      throw new Error(`Workflow with id ${id} not found`);
    }
    
    const now = new Date().toISOString();
    const runId = `run-${Date.now()}`;
    
    // 创建新的运行记录
    const newRun: WorkflowRun = {
      id: runId,
      workflowId: id,
      startTime: now,
      status: 'running',
      inputs,
      logs: [
        { timestamp: now, nodeId: '', message: '工作流开始执行', level: 'info' }
      ]
    };
    
    // 模拟工作流执行
    setTimeout(() => {
      const endTime = new Date().toISOString();
      
      // 更新运行状态
      const updatedRun: WorkflowRun = {
        ...newRun,
        status: 'completed',
        endTime,
        outputs: { result: '模拟的工作流执行结果' },
        logs: [
          ...newRun.logs,
          { timestamp: endTime, nodeId: '', message: '工作流执行完成', level: 'info' }
        ]
      };
      
      // 存储运行记录
      if (!MOCK_WORKFLOW_RUNS[id]) {
        MOCK_WORKFLOW_RUNS[id] = [];
      }
      
      MOCK_WORKFLOW_RUNS[id].push(updatedRun);
    }, 2000);
    
    return newRun;
  }
  
  // 获取工作流执行历史
  async getWorkflowRuns(workflowId: string): Promise<WorkflowRun[]> {
    await this.delay(400);
    
    return MOCK_WORKFLOW_RUNS[workflowId] || [];
  }
  
  // 获取单个执行记录详情
  async getWorkflowRunById(workflowId: string, runId: string): Promise<WorkflowRun | null> {
    await this.delay(300);
    
    const runs = MOCK_WORKFLOW_RUNS[workflowId] || [];
    return runs.find(run => run.id === runId) || null;
  }
  
  // 辅助方法: 延迟函数
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  // 辅助方法: 保存工作流到本地存储
  saveWorkflowsToStorage() {
    try {
      localStorage.setItem('workflows', JSON.stringify(MOCK_WORKFLOWS));
    } catch (error) {
      console.error('Error saving workflows to storage:', error);
    }
  }
  
  // 辅助方法: 从本地存储加载工作流
  loadWorkflowsFromStorage() {
    try {
      const storedWorkflows = localStorage.getItem('workflows');
      if (storedWorkflows) {
        const parsed = JSON.parse(storedWorkflows);
        // 清空现有数组并填充加载的数据
        MOCK_WORKFLOWS.length = 0;
        MOCK_WORKFLOWS.push(...parsed);
      }
    } catch (error) {
      console.error('Error loading workflows from storage:', error);
    }
  }
}

// 导出工作流服务实例
export const workflowsService = new WorkflowsService();

// Hook: 使用工作流列表
export function useWorkflows(userId?: string) {
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  
  const fetchWorkflows = useCallback(async () => {
    setLoading(true);
    setError(null);
    
    try {
      const data = await workflowsService.getAllWorkflows(userId);
      setWorkflows(data);
    } catch (err) {
      console.error('Error fetching workflows:', err);
      setError(err instanceof Error ? err : new Error('Failed to fetch workflows'));
    } finally {
      setLoading(false);
    }
  }, [userId]);
  
  useEffect(() => {
    fetchWorkflows();
  }, [fetchWorkflows]);
  
  return { workflows, loading, error, refetch: fetchWorkflows };
}

// Hook: 使用单个工作流详情
export function useWorkflow(id: string | undefined) {
  const [workflow, setWorkflow] = useState<Workflow | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  
  const fetchWorkflow = useCallback(async () => {
    if (!id) return;
    
    setLoading(true);
    setError(null);
    
    try {
      const data = await workflowsService.getWorkflowById(id);
      setWorkflow(data);
    } catch (err) {
      console.error('Error fetching workflow:', err);
      setError(err instanceof Error ? err : new Error('Failed to fetch workflow'));
    } finally {
      setLoading(false);
    }
  }, [id]);
  
  useEffect(() => {
    fetchWorkflow();
  }, [fetchWorkflow]);
  
  return { workflow, loading, error, refetch: fetchWorkflow };
} 