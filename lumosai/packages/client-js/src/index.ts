/**
 * @lomusai/client-js
 * 
 * JavaScript client for the LumosAI API
 */

import { LumosAIClient } from './client';
export { LumosAIClient } from './client';
export { Agent, StreamResponse } from './agent';
export { MemoryThread } from './memory';
export { Workflow, WorkflowRun } from './workflow';
export * from './types';

/**
 * Create a new LumosAI client
 */
export function createClient(config: { apiKey: string; baseUrl?: string; headers?: Record<string, string> }) {
  return new LumosAIClient(config);
} 