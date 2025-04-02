import { invoke } from '@tauri-apps/api/tauri';

// 资源类型定义
export type ResourceType = 'model' | 'tool' | 'agent' | 'workflow';

// 基础资源接口
export interface Resource {
  id: string;
  name: string;
  description: string;
  type: ResourceType;
  createdAt: string;
  updatedAt: string;
  tags: string[];
}

// 模型资源
export interface Model extends Resource {
  type: 'model';
  provider: string;
  contextWindow: number;
  capabilities: string[];
  parameters?: Record<string, any>;
}

// 工具资源
export interface Tool extends Resource {
  type: 'tool';
  category: string;
  inputs: {
    name: string;
    type: string;
    required: boolean;
    description: string;
  }[];
  outputs: {
    name: string;
    type: string;
    description: string;
  }[];
  implementation: string; // 'api' | 'function' | 'workflow'
}

// 代理资源
export interface Agent extends Resource {
  type: 'agent';
  model: string;
  tools: string[];
  memory?: string;
  prompt?: string;
  status: 'active' | 'draft' | 'archived';
}

// 工作流资源
export interface Workflow extends Resource {
  type: 'workflow';
  nodes: {
    id: string;
    type: string;
    position: { x: number; y: number };
    data: any;
  }[];
  edges: {
    id: string;
    source: string;
    target: string;
    label?: string;
  }[];
  status: 'active' | 'draft' | 'archived';
}

// 模拟数据
const MOCK_MODELS: Model[] = [
  {
    id: 'gpt-4',
    name: 'GPT-4',
    description: 'OpenAI\'s most advanced large language model',
    type: 'model',
    provider: 'OpenAI',
    contextWindow: 8192,
    capabilities: ['text-generation', 'code-generation', 'tool-use'],
    createdAt: '2023-01-01T00:00:00Z',
    updatedAt: '2023-03-15T00:00:00Z',
    tags: ['language-model', 'premium']
  },
  {
    id: 'claude-3-opus',
    name: 'Claude 3 Opus',
    description: 'Anthropic\'s most capable model for complex tasks',
    type: 'model',
    provider: 'Anthropic',
    contextWindow: 100000,
    capabilities: ['text-generation', 'code-generation', 'reasoning'],
    createdAt: '2023-02-15T00:00:00Z',
    updatedAt: '2023-04-01T00:00:00Z',
    tags: ['language-model', 'premium']
  },
  {
    id: 'gemini-pro',
    name: 'Gemini Pro',
    description: 'Google\'s multimodal AI model',
    type: 'model',
    provider: 'Google',
    contextWindow: 32768,
    capabilities: ['text-generation', 'image-understanding', 'tool-use'],
    createdAt: '2023-03-10T00:00:00Z',
    updatedAt: '2023-03-30T00:00:00Z',
    tags: ['language-model', 'multimodal']
  }
];

const MOCK_TOOLS: Tool[] = [
  {
    id: 'weather-api',
    name: 'Weather API',
    description: 'Fetch current weather and forecasts for any location',
    type: 'tool',
    category: 'API',
    inputs: [
      {
        name: 'location',
        type: 'string',
        required: true,
        description: 'City name or coordinates'
      },
      {
        name: 'units',
        type: 'string',
        required: false,
        description: 'Units of measurement (metric/imperial)'
      }
    ],
    outputs: [
      {
        name: 'current',
        type: 'object',
        description: 'Current weather conditions'
      },
      {
        name: 'forecast',
        type: 'array',
        description: 'Weather forecast for next days'
      }
    ],
    implementation: 'api',
    createdAt: '2023-01-20T00:00:00Z',
    updatedAt: '2023-02-15T00:00:00Z',
    tags: ['weather', 'api']
  },
  {
    id: 'pdf-reader',
    name: 'PDF Reader',
    description: 'Extract and parse text from PDF documents',
    type: 'tool',
    category: 'Document Processing',
    inputs: [
      {
        name: 'file',
        type: 'file',
        required: true,
        description: 'PDF file to process'
      }
    ],
    outputs: [
      {
        name: 'text',
        type: 'string',
        description: 'Extracted text content'
      },
      {
        name: 'metadata',
        type: 'object',
        description: 'Document metadata'
      }
    ],
    implementation: 'function',
    createdAt: '2023-02-05T00:00:00Z',
    updatedAt: '2023-03-10T00:00:00Z',
    tags: ['document', 'pdf', 'extraction']
  },
  {
    id: 'web-search',
    name: 'Web Search',
    description: 'Search the web for information',
    type: 'tool',
    category: 'Search',
    inputs: [
      {
        name: 'query',
        type: 'string',
        required: true,
        description: 'Search query'
      },
      {
        name: 'limit',
        type: 'number',
        required: false,
        description: 'Maximum number of results'
      }
    ],
    outputs: [
      {
        name: 'results',
        type: 'array',
        description: 'Search results'
      }
    ],
    implementation: 'api',
    createdAt: '2023-01-15T00:00:00Z',
    updatedAt: '2023-03-20T00:00:00Z',
    tags: ['search', 'web']
  }
];

const MOCK_AGENTS: Agent[] = [
  {
    id: 'research-assistant',
    name: 'Research Assistant',
    description: 'Agent for conducting research on any topic',
    type: 'agent',
    model: 'gpt-4',
    tools: ['web-search', 'pdf-reader'],
    memory: 'conversation-buffer',
    status: 'active',
    createdAt: '2023-02-10T00:00:00Z',
    updatedAt: '2023-04-05T00:00:00Z',
    tags: ['research', 'assistant']
  },
  {
    id: 'customer-support',
    name: 'Customer Support Bot',
    description: 'Agent for handling customer inquiries',
    type: 'agent',
    model: 'claude-3-opus',
    tools: [],
    status: 'active',
    createdAt: '2023-03-01T00:00:00Z',
    updatedAt: '2023-04-02T00:00:00Z',
    tags: ['support', 'customer-service']
  },
  {
    id: 'weather-bot',
    name: 'Weather Assistant',
    description: 'Agent for providing weather forecasts',
    type: 'agent',
    model: 'gemini-pro',
    tools: ['weather-api'],
    status: 'draft',
    createdAt: '2023-03-15T00:00:00Z',
    updatedAt: '2023-03-20T00:00:00Z',
    tags: ['weather', 'assistant']
  }
];

const MOCK_WORKFLOWS: Workflow[] = [
  {
    id: 'document-processing',
    name: 'Document Processing Pipeline',
    description: 'Extract, analyze, and summarize document content',
    type: 'workflow',
    nodes: [
      {
        id: 'node-1',
        type: 'input',
        position: { x: 250, y: 100 },
        data: { label: 'Input Document' }
      },
      {
        id: 'node-2',
        type: 'tool',
        position: { x: 250, y: 200 },
        data: { tool: 'pdf-reader', label: 'Extract Text' }
      },
      {
        id: 'node-3',
        type: 'agent',
        position: { x: 250, y: 300 },
        data: { agent: 'research-assistant', label: 'Analyze Content' }
      },
      {
        id: 'node-4',
        type: 'output',
        position: { x: 250, y: 400 },
        data: { label: 'Output Summary' }
      }
    ],
    edges: [
      {
        id: 'edge-1-2',
        source: 'node-1',
        target: 'node-2',
        label: 'PDF Document'
      },
      {
        id: 'edge-2-3',
        source: 'node-2',
        target: 'node-3',
        label: 'Extracted Text'
      },
      {
        id: 'edge-3-4',
        source: 'node-3',
        target: 'node-4',
        label: 'Summary'
      }
    ],
    status: 'active',
    createdAt: '2023-03-05T00:00:00Z',
    updatedAt: '2023-04-01T00:00:00Z',
    tags: ['document', 'processing', 'analysis']
  },
  {
    id: 'research-workflow',
    name: 'Automated Research Workflow',
    description: 'Search, analyze, and compile research findings',
    type: 'workflow',
    nodes: [
      {
        id: 'node-1',
        type: 'input',
        position: { x: 250, y: 100 },
        data: { label: 'Research Topic' }
      },
      {
        id: 'node-2',
        type: 'tool',
        position: { x: 250, y: 200 },
        data: { tool: 'web-search', label: 'Web Search' }
      },
      {
        id: 'node-3',
        type: 'agent',
        position: { x: 250, y: 300 },
        data: { agent: 'research-assistant', label: 'Analyze Results' }
      },
      {
        id: 'node-4',
        type: 'output',
        position: { x: 250, y: 400 },
        data: { label: 'Research Report' }
      }
    ],
    edges: [
      {
        id: 'edge-1-2',
        source: 'node-1',
        target: 'node-2',
        label: 'Query'
      },
      {
        id: 'edge-2-3',
        source: 'node-2',
        target: 'node-3',
        label: 'Search Results'
      },
      {
        id: 'edge-3-4',
        source: 'node-3',
        target: 'node-4',
        label: 'Report'
      }
    ],
    status: 'draft',
    createdAt: '2023-03-10T00:00:00Z',
    updatedAt: '2023-03-25T00:00:00Z',
    tags: ['research', 'automation']
  }
];

// 资源查询函数

/**
 * 获取所有模型
 */
export async function getModels(): Promise<Model[]> {
  await new Promise(resolve => setTimeout(resolve, 500));
  
  try {
    // 实际应用中使用invoke调用Tauri API
    // return await invoke('get_models');
    return MOCK_MODELS;
  } catch (error) {
    console.error('Failed to fetch models:', error);
    return [];
  }
}

/**
 * 获取所有工具
 */
export async function getTools(): Promise<Tool[]> {
  await new Promise(resolve => setTimeout(resolve, 500));
  
  try {
    // return await invoke('get_tools');
    return MOCK_TOOLS;
  } catch (error) {
    console.error('Failed to fetch tools:', error);
    return [];
  }
}

/**
 * 获取所有代理
 */
export async function getAgents(): Promise<Agent[]> {
  await new Promise(resolve => setTimeout(resolve, 500));
  
  try {
    // return await invoke('get_agents');
    return MOCK_AGENTS;
  } catch (error) {
    console.error('Failed to fetch agents:', error);
    return [];
  }
}

/**
 * 获取所有工作流
 */
export async function getWorkflows(): Promise<Workflow[]> {
  await new Promise(resolve => setTimeout(resolve, 500));
  
  try {
    // return await invoke('get_workflows');
    return MOCK_WORKFLOWS;
  } catch (error) {
    console.error('Failed to fetch workflows:', error);
    return [];
  }
}

/**
 * 获取所有资源
 */
export async function getAllResources(): Promise<Resource[]> {
  await new Promise(resolve => setTimeout(resolve, 500));
  
  try {
    const [models, tools, agents, workflows] = await Promise.all([
      getModels(),
      getTools(),
      getAgents(),
      getWorkflows()
    ]);
    
    return [...models, ...tools, ...agents, ...workflows];
  } catch (error) {
    console.error('Failed to fetch all resources:', error);
    return [];
  }
}

/**
 * 根据ID和类型获取特定资源
 */
export async function getResourceById(id: string, type: ResourceType): Promise<Resource | null> {
  await new Promise(resolve => setTimeout(resolve, 300));
  
  try {
    // 根据类型从不同的集合中查找
    let resource: Resource | undefined;
    
    switch (type) {
      case 'model':
        resource = MOCK_MODELS.find(m => m.id === id);
        break;
      case 'tool':
        resource = MOCK_TOOLS.find(t => t.id === id);
        break;
      case 'agent':
        resource = MOCK_AGENTS.find(a => a.id === id);
        break;
      case 'workflow':
        resource = MOCK_WORKFLOWS.find(w => w.id === id);
        break;
    }
    
    return resource || null;
  } catch (error) {
    console.error(`Failed to fetch ${type} with id ${id}:`, error);
    return null;
  }
}

/**
 * 搜索资源
 */
export async function searchResources(
  query: string, 
  types?: ResourceType[], 
  tags?: string[]
): Promise<Resource[]> {
  await new Promise(resolve => setTimeout(resolve, 600));
  
  try {
    // 获取所有资源
    const resources = await getAllResources();
    
    // 过滤资源
    return resources.filter(resource => {
      // 根据类型过滤
      if (types && types.length > 0 && !types.includes(resource.type)) {
        return false;
      }
      
      // 根据标签过滤
      if (tags && tags.length > 0 && !tags.some(tag => resource.tags.includes(tag))) {
        return false;
      }
      
      // 根据查询字符串过滤（在名称和描述中搜索）
      if (query && query.trim() !== '') {
        const normalizedQuery = query.toLowerCase().trim();
        return (
          resource.name.toLowerCase().includes(normalizedQuery) ||
          resource.description.toLowerCase().includes(normalizedQuery)
        );
      }
      
      return true;
    });
  } catch (error) {
    console.error('Failed to search resources:', error);
    return [];
  }
} 