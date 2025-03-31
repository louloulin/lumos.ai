import { LumosAIClient } from './client';
import { AgentResponse, GenerateParams, Message, StreamHandlers, Tool } from './types';

/**
 * Class for interacting with an AI agent
 */
export class Agent {
  private readonly agentId: string;
  private readonly client: LumosAIClient;
  private agentDetails?: Record<string, any>;

  /**
   * Create a new Agent instance
   */
  constructor(agentId: string, client: LumosAIClient, agentDetails?: Record<string, any>) {
    this.agentId = agentId;
    this.client = client;
    this.agentDetails = agentDetails;
  }

  /**
   * Get detailed information about the agent
   */
  async details(): Promise<Record<string, any>> {
    if (this.agentDetails) {
      return this.agentDetails;
    }

    const response = await this.client.request('GET', `/agents/${this.agentId}`);
    
    if (!response.ok) {
      throw new Error(`Failed to get agent details: ${response.statusText}`);
    }

    const details = await response.json();
    this.agentDetails = details;
    return details;
  }

  /**
   * Generate a response from the agent
   */
  async generate(
    input: string | GenerateParams,
    options?: Partial<GenerateParams>
  ): Promise<AgentResponse> {
    let params: GenerateParams;

    if (typeof input === 'string') {
      params = {
        messages: [{ role: 'user', content: input }],
        ...options,
      };
    } else {
      params = { ...input, ...options };
    }

    const response = await this.client.request(
      'POST',
      `/agents/${this.agentId}/generate`,
      params
    );

    if (!response.ok) {
      throw new Error(`Failed to generate response: ${response.statusText}`);
    }

    return await response.json();
  }

  /**
   * Stream a response from the agent for real-time interactions
   */
  async stream(
    input: string | GenerateParams,
    options?: Partial<GenerateParams>
  ): Promise<StreamResponse> {
    let params: GenerateParams;

    if (typeof input === 'string') {
      params = {
        messages: [{ role: 'user', content: input }],
        ...options,
      };
    } else {
      params = { ...input, ...options };
    }

    const response = await this.client.request(
      'POST',
      `/agents/${this.agentId}/stream`,
      params
    );

    if (!response.ok) {
      throw new Error(`Failed to stream response: ${response.statusText}`);
    }

    return new StreamResponse(response);
  }

  /**
   * Get information about a specific tool available to the agent
   */
  async getTool(toolId: string): Promise<Tool> {
    const response = await this.client.request(
      'GET',
      `/agents/${this.agentId}/tools/${toolId}`
    );

    if (!response.ok) {
      throw new Error(`Failed to get tool: ${response.statusText}`);
    }

    return await response.json();
  }

  /**
   * Get evaluation results for the agent
   */
  async evals(): Promise<any> {
    const response = await this.client.request(
      'GET',
      `/agents/${this.agentId}/evals`
    );

    if (!response.ok) {
      throw new Error(`Failed to get evaluations: ${response.statusText}`);
    }

    return await response.json();
  }

  /**
   * Get live evaluation results for the agent
   */
  async liveEvals(): Promise<any> {
    const response = await this.client.request(
      'GET',
      `/agents/${this.agentId}/evals/live`
    );

    if (!response.ok) {
      throw new Error(`Failed to get live evaluations: ${response.statusText}`);
    }

    return await response.json();
  }
}

/**
 * Class for handling streamed responses from an agent
 */
export class StreamResponse {
  readonly body: ReadableStream<Uint8Array>;
  private readonly response: Response;

  constructor(response: Response) {
    this.response = response;
    this.body = response.body!;
  }

  /**
   * Process the data stream with customizable handlers
   */
  async processDataStream(handlers: StreamHandlers): Promise<void> {
    const reader = this.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          if (!line.trim()) continue;

          try {
            if (line.startsWith('data: ')) {
              const data = JSON.parse(line.substring(6));

              if (data.type === 'text' && handlers.onTextPart) {
                handlers.onTextPart(data.content);
              } else if (data.type === 'file' && handlers.onFilePart) {
                handlers.onFilePart(data.content);
              } else if (data.type === 'data' && handlers.onDataPart) {
                handlers.onDataPart(data.content);
              } else if (data.type === 'error' && handlers.onErrorPart) {
                handlers.onErrorPart(new Error(data.content));
              }
            }
          } catch (err) {
            if (handlers.onErrorPart) {
              handlers.onErrorPart(err instanceof Error ? err : new Error(String(err)));
            }
          }
        }
      }
      
      // Decode any remaining bytes
      buffer += decoder.decode();
      
      if (buffer.trim() && buffer.startsWith('data: ')) {
        try {
          const data = JSON.parse(buffer.substring(6));
          
          if (data.type === 'text' && handlers.onTextPart) {
            handlers.onTextPart(data.content);
          } else if (data.type === 'file' && handlers.onFilePart) {
            handlers.onFilePart(data.content);
          } else if (data.type === 'data' && handlers.onDataPart) {
            handlers.onDataPart(data.content);
          } else if (data.type === 'error' && handlers.onErrorPart) {
            handlers.onErrorPart(new Error(data.content));
          }
        } catch (err) {
          if (handlers.onErrorPart) {
            handlers.onErrorPart(err instanceof Error ? err : new Error(String(err)));
          }
        }
      }
    } finally {
      reader.releaseLock();
    }
  }
} 