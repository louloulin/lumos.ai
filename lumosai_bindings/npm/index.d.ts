/**
 * Lumos.ai Node.js/TypeScript绑定
 * 
 * 高性能AI Agent框架的JavaScript/TypeScript接口
 */

export interface Config {
  /** 模型配置 */
  model: ModelConfig;
  /** 工具列表 */
  tools: string[];
  /** 运行时配置 */
  runtime: RuntimeConfig;
}

export interface ModelConfig {
  /** 模型名称 */
  name: string;
  /** API密钥 */
  apiKey?: string;
  /** 基础URL */
  baseUrl?: string;
}

export interface RuntimeConfig {
  /** 超时时间（秒） */
  timeoutSeconds: number;
  /** 最大重试次数 */
  maxRetries: number;
  /** 并发限制 */
  concurrencyLimit: number;
  /** 启用日志 */
  enableLogging: boolean;
  /** 日志级别 */
  logLevel: string;
}

export interface Response {
  /** 响应内容 */
  content: string;
  /** 响应类型 */
  responseType: string;
  /** 元数据 */
  metadata: Record<string, any>;
  /** 工具调用结果 */
  toolCalls: ToolCallResult[];
  /** 错误信息 */
  error?: string;
}

export interface ToolCallResult {
  /** 工具名称 */
  toolName: string;
  /** 调用参数 */
  parameters: any;
  /** 调用结果 */
  result: any;
  /** 执行时间（毫秒） */
  executionTimeMs: number;
  /** 是否成功 */
  success: boolean;
  /** 错误信息 */
  error?: string;
}

export interface ToolMetadata {
  /** 工具名称 */
  name: string;
  /** 工具描述 */
  description: string;
  /** 工具类型 */
  toolType: string;
  /** 是否异步 */
  isAsync: boolean;
  /** 参数模式 */
  parameters: any;
}

/**
 * Lumos.ai Agent
 * 
 * 高性能的AI Agent，支持工具调用、异步执行和流式响应
 * 
 * @example
 * ```typescript
 * import { Agent, tools } from '@lumosai/core';
 * 
 * const agent = Agent.quick('assistant', '你是一个AI助手')
 *   .model('deepseek-chat')
 *   .tools([tools.webSearch(), tools.calculator()])
 *   .build();
 * 
 * const response = await agent.generateAsync('帮我搜索最新的AI新闻');
 * console.log(response.content);
 * ```
 */
export declare class Agent {
  /**
   * 生成响应（同步）
   * @param input 输入文本
   * @returns 生成的响应
   */
  generate(input: string): Response;

  /**
   * 生成响应（异步）
   * @param input 输入文本
   * @returns 生成的响应
   */
  generateAsync(input: string): Promise<Response>;

  /**
   * 获取Agent配置
   * @returns 配置信息
   */
  getConfig(): Config;

  /**
   * 快速创建Agent构建器
   * @param name Agent名称
   * @param instructions 指令
   * @returns Agent构建器
   */
  static quick(name: string, instructions: string): AgentBuilder;

  /**
   * 创建Agent构建器
   * @returns Agent构建器
   */
  static builder(): AgentBuilder;
}

/**
 * Agent构建器
 * 
 * 使用构建器模式创建和配置Agent
 * 
 * @example
 * ```typescript
 * const builder = Agent.builder()
 *   .name('assistant')
 *   .instructions('你是一个AI助手')
 *   .model('gpt-4')
 *   .tools([tools.webSearch()]);
 * 
 * const agent = builder.build();
 * ```
 */
export declare class AgentBuilder {
  /**
   * 设置Agent名称
   * @param name Agent名称
   * @returns 返回自身以支持链式调用
   */
  name(name: string): AgentBuilder;

  /**
   * 设置Agent指令
   * @param instructions 指令文本
   * @returns 返回自身以支持链式调用
   */
  instructions(instructions: string): AgentBuilder;

  /**
   * 设置模型
   * @param model 模型名称
   * @returns 返回自身以支持链式调用
   */
  model(model: string): AgentBuilder;

  /**
   * 添加工具
   * @param tool 工具实例
   * @returns 返回自身以支持链式调用
   */
  tool(tool: Tool): AgentBuilder;

  /**
   * 添加多个工具
   * @param tools 工具列表
   * @returns 返回自身以支持链式调用
   */
  tools(tools: Tool[]): AgentBuilder;

  /**
   * 构建Agent
   * @returns 构建的Agent实例
   */
  build(): Agent;

  /**
   * 异步构建Agent
   * @returns 构建的Agent实例
   */
  buildAsync(): Promise<Agent>;
}

/**
 * Lumos.ai工具
 * 
 * 封装各种功能的工具，如网络搜索、文件操作、数学计算等
 * 
 * @example
 * ```typescript
 * const tool = tools.webSearch();
 * const metadata = tool.metadata();
 * console.log(metadata.name); // 'web_search'
 * 
 * const result = tool.execute({ query: 'Python教程' });
 * console.log(result.success); // true
 * ```
 */
export declare class Tool {
  /**
   * 获取工具元数据
   * @returns 工具元数据
   */
  metadata(): ToolMetadata;

  /**
   * 执行工具
   * @param parameters 工具参数
   * @returns 执行结果
   */
  execute(parameters: Record<string, any>): ToolCallResult;
}

/**
 * 工具模块
 * 
 * 提供各种预定义工具
 */
export declare namespace tools {
  /**
   * Web搜索工具
   * @returns Web搜索工具实例
   */
  function webSearch(): Tool;

  /**
   * HTTP请求工具
   * @returns HTTP请求工具实例
   */
  function httpRequest(): Tool;

  /**
   * 计算器工具
   * @returns 计算器工具实例
   */
  function calculator(): Tool;

  /**
   * 文件读取工具
   * @returns 文件读取工具实例
   */
  function fileReader(): Tool;

  /**
   * 文件写入工具
   * @returns 文件写入工具实例
   */
  function fileWriter(): Tool;

  /**
   * JSON处理工具
   * @returns JSON处理工具实例
   */
  function jsonProcessor(): Tool;

  /**
   * CSV处理工具
   * @returns CSV处理工具实例
   */
  function csvProcessor(): Tool;

  /**
   * XML处理工具
   * @returns XML处理工具实例
   */
  function xmlProcessor(): Tool;

  /**
   * 数学表达式求值工具
   * @returns 数学求值工具实例
   */
  function mathEvaluator(): Tool;

  /**
   * Shell执行工具
   * @returns Shell执行工具实例
   */
  function shellExecutor(): Tool;

  /**
   * 环境变量读取工具
   * @returns 环境变量读取工具实例
   */
  function environmentReader(): Tool;

  /**
   * Ping工具
   * @returns Ping工具实例
   */
  function pingTool(): Tool;

  /**
   * DNS解析工具
   * @returns DNS解析工具实例
   */
  function dnsResolver(): Tool;

  /**
   * 日期时间格式化工具
   * @returns 日期时间格式化工具实例
   */
  function datetimeFormatter(): Tool;

  /**
   * 时区转换工具
   * @returns 时区转换工具实例
   */
  function timezoneConverter(): Tool;
}

/**
 * 便利函数：快速创建Agent
 * @param name Agent名称
 * @param instructions 指令
 * @returns Agent构建器
 */
export declare function quickAgent(name: string, instructions: string): AgentBuilder;

/**
 * 便利函数：创建AgentBuilder
 * @returns Agent构建器
 */
export declare function createAgentBuilder(): AgentBuilder;

/**
 * Lumos.ai错误类
 */
export declare class LumosError extends Error {
  /** 错误代码 */
  readonly code: string;
  /** 是否可重试 */
  readonly retryable: boolean;
  
  constructor(message: string, code?: string, retryable?: boolean);
}

/**
 * 版本信息
 */
export declare const version: string;

/**
 * 默认导出
 */
declare const _default: {
  Agent: typeof Agent;
  AgentBuilder: typeof AgentBuilder;
  Tool: typeof Tool;
  tools: typeof tools;
  quickAgent: typeof quickAgent;
  createAgentBuilder: typeof createAgentBuilder;
  LumosError: typeof LumosError;
  version: typeof version;
};

export default _default;
