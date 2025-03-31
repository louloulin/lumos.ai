/**
 * Configuration options for the LumosAI client
 */
export interface LumosAIClientConfig {
  /**
   * API key for authentication
   */
  apiKey: string;
  
  /**
   * Base URL for the API (defaults to https://api.lumosai.com)
   */
  baseUrl?: string;
  
  /**
   * Additional headers to include in requests
   */
  headers?: Record<string, string>;
}

/**
 * Message in a conversation
 */
export interface Message {
  /**
   * Role of the message sender
   */
  role: 'user' | 'assistant' | 'system';
  
  /**
   * Content of the message
   */
  content: string;
  
  /**
   * Optional message ID
   */
  id?: string;
  
  /**
   * Thread ID the message belongs to
   */
  threadId?: string;
  
  /**
   * Creation timestamp
   */
  createdAt?: Date;
  
  /**
   * Message type
   */
  type?: string;
}

/**
 * Response from an agent generation
 */
export interface AgentResponse {
  /**
   * Response message
   */
  message: Message;
  
  /**
   * Parsed object if output schema was provided
   */
  object?: Record<string, any>;
}

/**
 * Parameters for agent generation
 */
export interface GenerateParams {
  /**
   * Messages to include in the conversation
   */
  messages: Message[];
  
  /**
   * Thread ID for conversation context
   */
  threadId?: string;
  
  /**
   * Resource ID
   */
  resourceId?: string;
  
  /**
   * Output schema or configuration
   */
  output?: any;
}

/**
 * Tool definition
 */
export interface Tool {
  /**
   * Tool ID
   */
  id: string;
  
  /**
   * Tool description
   */
  description: string;
  
  /**
   * Input schema for the tool
   */
  inputSchema?: any;
  
  /**
   * Output schema for the tool
   */
  outputSchema?: any;
}

/**
 * Stream response handler callbacks
 */
export interface StreamHandlers {
  /**
   * Callback for text parts
   */
  onTextPart?: (text: string) => void;
  
  /**
   * Callback for file parts
   */
  onFilePart?: (file: any) => void;
  
  /**
   * Callback for data parts
   */
  onDataPart?: (data: any) => void;
  
  /**
   * Callback for error parts
   */
  onErrorPart?: (error: Error) => void;
}

/**
 * Workflow run parameters
 */
export interface WorkflowRunParams {
  /**
   * Trigger data for the workflow
   */
  triggerData: Record<string, any>;
}

/**
 * Memory thread details
 */
export interface MemoryThreadDetails {
  /**
   * Thread ID
   */
  id: string;
  
  /**
   * Thread title
   */
  title: string;
  
  /**
   * Thread metadata
   */
  metadata?: Record<string, any>;
  
  /**
   * Resource ID
   */
  resourceId?: string;
  
  /**
   * Creation timestamp
   */
  createdAt?: Date;
  
  /**
   * Last update timestamp
   */
  updatedAt?: Date;
}

/**
 * Parameters for updating a memory thread
 */
export interface UpdateThreadParams {
  /**
   * New thread title
   */
  title?: string;
  
  /**
   * New thread metadata
   */
  metadata?: Record<string, any>;
  
  /**
   * New resource ID
   */
  resourceId?: string;
} 