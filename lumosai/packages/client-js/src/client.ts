import { Agent } from './agent';
import { MemoryThread } from './memory';
import { Workflow } from './workflow';
import { LumosAIClientConfig } from './types';

/**
 * The main LumosAI client for interacting with the API
 */
export class LumosAIClient {
  private readonly apiKey: string;
  private readonly baseUrl: string;
  private readonly headers: Record<string, string>;

  /**
   * Create a new LumosAI client
   */
  constructor(config: LumosAIClientConfig) {
    this.apiKey = config.apiKey;
    this.baseUrl = config.baseUrl || 'https://api.lumosai.com';
    this.headers = {
      'Authorization': `Bearer ${this.apiKey}`,
      'Content-Type': 'application/json',
      'Accept': 'application/json',
      ...config.headers,
    };
  }

  /**
   * Get an agent by ID
   */
  getAgent(agentId: string): Agent {
    return new Agent(agentId, this);
  }

  /**
   * Get all available agents
   */
  async getAgents(): Promise<Agent[]> {
    const response = await fetch(`${this.baseUrl}/agents`, {
      method: 'GET',
      headers: this.headers,
    });

    if (!response.ok) {
      throw new Error(`Failed to get agents: ${response.statusText}`);
    }

    const data = await response.json();
    return data.agents.map((agentData: any) => new Agent(agentData.id, this, agentData));
  }

  /**
   * Get a workflow by ID
   */
  getWorkflow(workflowId: string): Workflow {
    return new Workflow(workflowId, this);
  }

  /**
   * Get all available workflows
   */
  async getWorkflows(): Promise<Workflow[]> {
    const response = await fetch(`${this.baseUrl}/workflows`, {
      method: 'GET',
      headers: this.headers,
    });

    if (!response.ok) {
      throw new Error(`Failed to get workflows: ${response.statusText}`);
    }

    const data = await response.json();
    return data.workflows.map((workflowData: any) => new Workflow(workflowData.id, this, workflowData));
  }

  /**
   * Get a memory thread by ID
   */
  getMemoryThread(threadId: string, agentId?: string): MemoryThread {
    return new MemoryThread(threadId, this, agentId);
  }

  /**
   * Create a new memory thread
   */
  async createMemoryThread(params: {
    title: string;
    metadata?: Record<string, any>;
    resourceId?: string;
    agentId?: string;
  }): Promise<MemoryThread> {
    const response = await fetch(`${this.baseUrl}/memory/threads`, {
      method: 'POST',
      headers: this.headers,
      body: JSON.stringify(params),
    });

    if (!response.ok) {
      throw new Error(`Failed to create memory thread: ${response.statusText}`);
    }

    const data = await response.json();
    return new MemoryThread(data.id, this, params.agentId);
  }

  /**
   * Get all memory threads for a resource
   */
  async getMemoryThreads(params: {
    resourceId?: string;
    agentId?: string;
  }): Promise<MemoryThread[]> {
    const url = new URL(`${this.baseUrl}/memory/threads`);
    
    if (params.resourceId) {
      url.searchParams.append('resourceId', params.resourceId);
    }

    if (params.agentId) {
      url.searchParams.append('agentId', params.agentId);
    }

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers: this.headers,
    });

    if (!response.ok) {
      throw new Error(`Failed to get memory threads: ${response.statusText}`);
    }

    const data = await response.json();
    return data.threads.map((threadData: any) => 
      new MemoryThread(threadData.id, this, params.agentId, threadData)
    );
  }

  /**
   * Save messages to memory
   */
  async saveMessageToMemory(params: {
    messages: Array<{
      role: 'user' | 'assistant' | 'system';
      content: string;
      id?: string;
      threadId: string;
      createdAt?: Date;
      type?: string;
    }>;
    agentId?: string;
  }): Promise<any> {
    const response = await fetch(`${this.baseUrl}/memory/messages`, {
      method: 'POST',
      headers: this.headers,
      body: JSON.stringify(params),
    });

    if (!response.ok) {
      throw new Error(`Failed to save messages to memory: ${response.statusText}`);
    }

    return await response.json();
  }

  /**
   * Get memory status
   */
  async getMemoryStatus(agentId?: string): Promise<any> {
    const url = new URL(`${this.baseUrl}/memory/status`);
    
    if (agentId) {
      url.searchParams.append('agentId', agentId);
    }

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers: this.headers,
    });

    if (!response.ok) {
      throw new Error(`Failed to get memory status: ${response.statusText}`);
    }

    return await response.json();
  }

  /**
   * Make a raw API request to the LumosAI API
   */
  async request(
    method: string,
    path: string,
    body?: any,
    additionalHeaders?: Record<string, string>
  ): Promise<Response> {
    const url = `${this.baseUrl}${path}`;
    const headers = { ...this.headers, ...additionalHeaders };
    
    const options: RequestInit = {
      method,
      headers,
    };

    if (body) {
      options.body = JSON.stringify(body);
    }

    return fetch(url, options);
  }
} 