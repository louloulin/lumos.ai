import { LumosAIClient } from './client';
import { MemoryThreadDetails, UpdateThreadParams } from './types';

/**
 * Class for interacting with memory threads
 */
export class MemoryThread {
  private readonly threadId: string;
  private readonly client: LumosAIClient;
  private readonly agentId?: string;
  private threadDetails?: MemoryThreadDetails;

  /**
   * Create a new MemoryThread instance
   */
  constructor(
    threadId: string, 
    client: LumosAIClient, 
    agentId?: string, 
    threadDetails?: MemoryThreadDetails
  ) {
    this.threadId = threadId;
    this.client = client;
    this.agentId = agentId;
    this.threadDetails = threadDetails;
  }

  /**
   * Get thread details
   */
  async get(): Promise<MemoryThreadDetails> {
    if (this.threadDetails) {
      return this.threadDetails;
    }

    const url = new URL(`/memory/threads/${this.threadId}`, 'https://placeholder');
    
    if (this.agentId) {
      url.searchParams.append('agentId', this.agentId);
    }

    const response = await this.client.request('GET', url.pathname + url.search);
    
    if (!response.ok) {
      throw new Error(`Failed to get thread details: ${response.statusText}`);
    }

    const details = await response.json();
    this.threadDetails = details;
    return details;
  }

  /**
   * Update thread properties
   */
  async update(params: UpdateThreadParams): Promise<MemoryThreadDetails> {
    const url = new URL(`/memory/threads/${this.threadId}`, 'https://placeholder');
    
    if (this.agentId) {
      url.searchParams.append('agentId', this.agentId);
    }

    const response = await this.client.request(
      'PUT',
      url.pathname + url.search,
      params
    );
    
    if (!response.ok) {
      throw new Error(`Failed to update thread: ${response.statusText}`);
    }

    const details = await response.json();
    this.threadDetails = details;
    return details;
  }

  /**
   * Delete a thread and its messages
   */
  async delete(): Promise<void> {
    const url = new URL(`/memory/threads/${this.threadId}`, 'https://placeholder');
    
    if (this.agentId) {
      url.searchParams.append('agentId', this.agentId);
    }

    const response = await this.client.request('DELETE', url.pathname + url.search);
    
    if (!response.ok) {
      throw new Error(`Failed to delete thread: ${response.statusText}`);
    }
  }

  /**
   * Get messages from the thread
   */
  async getMessages(params?: {
    limit?: number;
    cursor?: string;
  }): Promise<{
    messages: Array<any>;
    nextCursor?: string;
  }> {
    const url = new URL(`/memory/threads/${this.threadId}/messages`, 'https://placeholder');
    
    if (this.agentId) {
      url.searchParams.append('agentId', this.agentId);
    }

    if (params?.limit) {
      url.searchParams.append('limit', params.limit.toString());
    }

    if (params?.cursor) {
      url.searchParams.append('cursor', params.cursor);
    }

    const response = await this.client.request('GET', url.pathname + url.search);
    
    if (!response.ok) {
      throw new Error(`Failed to get messages: ${response.statusText}`);
    }

    return await response.json();
  }

  /**
   * Add a message to the thread
   */
  async addMessage(message: {
    role: 'user' | 'assistant' | 'system';
    content: string;
    id?: string;
    createdAt?: Date;
    type?: string;
  }): Promise<any> {
    const url = new URL(`/memory/threads/${this.threadId}/messages`, 'https://placeholder');
    
    if (this.agentId) {
      url.searchParams.append('agentId', this.agentId);
    }

    const messageWithThreadId = {
      ...message,
      threadId: this.threadId,
    };

    const response = await this.client.request(
      'POST',
      url.pathname + url.search,
      { messages: [messageWithThreadId] }
    );
    
    if (!response.ok) {
      throw new Error(`Failed to add message: ${response.statusText}`);
    }

    return await response.json();
  }
} 