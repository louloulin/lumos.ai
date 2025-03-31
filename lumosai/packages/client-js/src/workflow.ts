import { LumosAIClient } from './client';
import { WorkflowRunParams } from './types';

/**
 * Class for interacting with workflows
 */
export class Workflow {
  private readonly workflowId: string;
  private readonly client: LumosAIClient;
  private workflowDetails?: Record<string, any>;

  /**
   * Create a new Workflow instance
   */
  constructor(
    workflowId: string,
    client: LumosAIClient,
    workflowDetails?: Record<string, any>
  ) {
    this.workflowId = workflowId;
    this.client = client;
    this.workflowDetails = workflowDetails;
  }

  /**
   * Get detailed information about the workflow
   */
  async details(): Promise<Record<string, any>> {
    if (this.workflowDetails) {
      return this.workflowDetails;
    }

    const response = await this.client.request('GET', `/workflows/${this.workflowId}`);
    
    if (!response.ok) {
      throw new Error(`Failed to get workflow details: ${response.statusText}`);
    }

    const details = await response.json();
    this.workflowDetails = details;
    return details;
  }

  /**
   * Create a new run of the workflow
   */
  createRun(): WorkflowRun {
    return new WorkflowRun(this.workflowId, this.client);
  }

  /**
   * Get all runs of this workflow
   */
  async getRuns(params?: {
    limit?: number;
    cursor?: string;
  }): Promise<{
    runs: Array<any>;
    nextCursor?: string;
  }> {
    const url = new URL(`/workflows/${this.workflowId}/runs`, 'https://placeholder');
    
    if (params?.limit) {
      url.searchParams.append('limit', params.limit.toString());
    }

    if (params?.cursor) {
      url.searchParams.append('cursor', params.cursor);
    }

    const response = await this.client.request('GET', url.pathname + url.search);
    
    if (!response.ok) {
      throw new Error(`Failed to get workflow runs: ${response.statusText}`);
    }

    return await response.json();
  }
}

/**
 * Class for interacting with a specific workflow run
 */
export class WorkflowRun {
  private readonly workflowId: string;
  private readonly client: LumosAIClient;
  private runId?: string;

  constructor(workflowId: string, client: LumosAIClient) {
    this.workflowId = workflowId;
    this.client = client;
  }

  /**
   * Start the workflow run with trigger data
   */
  async start(params: WorkflowRunParams): Promise<any> {
    const response = await this.client.request(
      'POST',
      `/workflows/${this.workflowId}/runs`,
      params
    );
    
    if (!response.ok) {
      throw new Error(`Failed to start workflow run: ${response.statusText}`);
    }

    const data = await response.json();
    this.runId = data.id;
    return data;
  }

  /**
   * Get details of the workflow run
   */
  async details(): Promise<any> {
    if (!this.runId) {
      throw new Error('Run ID is not available. Make sure to start the run first.');
    }

    const response = await this.client.request(
      'GET',
      `/workflows/${this.workflowId}/runs/${this.runId}`
    );
    
    if (!response.ok) {
      throw new Error(`Failed to get run details: ${response.statusText}`);
    }

    return await response.json();
  }

  /**
   * Cancel the workflow run
   */
  async cancel(): Promise<any> {
    if (!this.runId) {
      throw new Error('Run ID is not available. Make sure to start the run first.');
    }

    const response = await this.client.request(
      'POST',
      `/workflows/${this.workflowId}/runs/${this.runId}/cancel`
    );
    
    if (!response.ok) {
      throw new Error(`Failed to cancel run: ${response.statusText}`);
    }

    return await response.json();
  }
} 