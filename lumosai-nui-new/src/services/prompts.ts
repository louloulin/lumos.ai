// prompts.ts - LumosAI平台的提示模板服务

// 提示模板类型
export type PromptVariable = {
  name: string;
  description: string;
  required: boolean;
  defaultValue?: string;
  type: 'string' | 'number' | 'boolean' | 'array';
};

export type PromptTemplate = {
  id: string;
  name: string;
  description: string;
  content: string;
  variables: PromptVariable[];
  tags: string[];
  createdAt: string;
  updatedAt: string;
  createdBy: string;
  isPublic: boolean;
  version: number;
  model?: string;
  category?: string;
};

export type CreatePromptTemplateParams = {
  name: string;
  description: string;
  content: string;
  variables: PromptVariable[];
  tags?: string[];
  isPublic?: boolean;
  model?: string;
  category?: string;
};

export type UpdatePromptTemplateParams = Partial<Omit<PromptTemplate, 'id' | 'createdAt' | 'createdBy'>>;

export type ExecutePromptParams = {
  templateId: string;
  variables: Record<string, any>;
  model?: string;
};

export type ExecutePromptResult = {
  output: string;
  tokenUsage: {
    prompt: number;
    completion: number;
    total: number;
  };
  modelUsed: string;
  executionTime: number;
};

// 模拟数据 - 在实际应用中会替换为API调用
const MOCK_TEMPLATES: PromptTemplate[] = [
  {
    id: 'pt-1',
    name: '通用知识问答',
    description: '一个通用的知识问答提示模板，适合回答各种领域的问题',
    content: 'You are a helpful assistant. Answer the following question accurately based on your knowledge.\n\nQuestion: {{question}}\n\nAnswer:',
    variables: [
      {
        name: 'question',
        description: '用户提出的问题',
        required: true,
        type: 'string'
      }
    ],
    tags: ['general', 'qa', 'knowledge'],
    createdAt: '2023-04-15T10:00:00Z',
    updatedAt: '2023-04-15T10:00:00Z',
    createdBy: 'user-1',
    isPublic: true,
    version: 1,
    model: 'gpt-4',
    category: 'general'
  },
  {
    id: 'pt-2',
    name: '代码解释器',
    description: '解释代码片段的功能和结构',
    content: 'You are a programming expert. Explain the following code in simple terms, describing what it does, any important patterns or techniques used, and potential issues.\n\n```{{language}}\n{{code}}\n```\n\nExplanation:',
    variables: [
      {
        name: 'language',
        description: '编程语言',
        required: true,
        type: 'string',
        defaultValue: 'javascript'
      },
      {
        name: 'code',
        description: '需要解释的代码',
        required: true,
        type: 'string'
      }
    ],
    tags: ['programming', 'code', 'explanation'],
    createdAt: '2023-04-16T14:30:00Z',
    updatedAt: '2023-04-17T09:15:00Z',
    createdBy: 'user-1',
    isPublic: true,
    version: 2,
    model: 'gpt-4',
    category: 'development'
  },
  {
    id: 'pt-3',
    name: '情感分析',
    description: '分析文本的情感倾向',
    content: 'Analyze the sentiment of the following text. Consider the emotional tone, attitude, and any subjective information expressed. Rate the sentiment on a scale of 1-5, where 1 is very negative and 5 is very positive.\n\nText: {{text}}\n\nSentiment analysis:',
    variables: [
      {
        name: 'text',
        description: '需要分析情感的文本',
        required: true,
        type: 'string'
      }
    ],
    tags: ['nlp', 'sentiment', 'analysis'],
    createdAt: '2023-04-18T08:45:00Z',
    updatedAt: '2023-04-18T08:45:00Z',
    createdBy: 'user-2',
    isPublic: false,
    version: 1,
    model: 'gpt-3.5-turbo',
    category: 'analysis'
  },
  {
    id: 'pt-4',
    name: '营销文案生成器',
    description: '为产品或服务生成引人注目的营销文案',
    content: 'Create compelling marketing copy for the following product/service. The tone should be {{tone}} and target audience is {{audience}}.\n\nProduct/Service: {{product}}\nKey features/benefits:\n{{features}}\n\nMarketing copy:',
    variables: [
      {
        name: 'product',
        description: '产品或服务名称和简介',
        required: true,
        type: 'string'
      },
      {
        name: 'features',
        description: '产品的主要特点和优势',
        required: true,
        type: 'string'
      },
      {
        name: 'tone',
        description: '文案的语气',
        required: true,
        type: 'string',
        defaultValue: 'professional'
      },
      {
        name: 'audience',
        description: '目标受众',
        required: true,
        type: 'string'
      }
    ],
    tags: ['marketing', 'copywriting', 'business'],
    createdAt: '2023-04-20T16:20:00Z',
    updatedAt: '2023-04-22T11:10:00Z',
    createdBy: 'user-3',
    isPublic: true,
    version: 3,
    model: 'gpt-4',
    category: 'business'
  },
  {
    id: 'pt-5',
    name: '对话系统提示',
    description: '构建对话型AI助手的系统提示',
    content: 'You are {{character}}, a helpful AI assistant with expertise in {{expertise}}. Your tone is {{tone}} and you always {{behavior}}.\n\nUser: {{input}}\n\n{{character}}:',
    variables: [
      {
        name: 'character',
        description: 'AI助手的角色或名称',
        required: true,
        type: 'string',
        defaultValue: 'LumosAI'
      },
      {
        name: 'expertise',
        description: 'AI助手的专业领域',
        required: true,
        type: 'string'
      },
      {
        name: 'tone',
        description: '对话的语气',
        required: true,
        type: 'string',
        defaultValue: 'friendly and professional'
      },
      {
        name: 'behavior',
        description: 'AI助手的行为特点',
        required: true,
        type: 'string',
        defaultValue: 'provide concise and accurate information'
      },
      {
        name: 'input',
        description: '用户输入',
        required: true,
        type: 'string'
      }
    ],
    tags: ['conversation', 'system-prompt', 'chatbot'],
    createdAt: '2023-04-25T09:30:00Z',
    updatedAt: '2023-04-25T14:45:00Z',
    createdBy: 'user-1',
    isPublic: true,
    version: 2,
    model: 'gpt-4',
    category: 'dialogue'
  }
];

// 提示模板服务
export const promptsService = {
  // 获取所有提示模板
  getPromptTemplates: async (userId: string): Promise<PromptTemplate[]> => {
    // 模拟API调用
    return new Promise((resolve) => {
      setTimeout(() => {
        // 返回用户创建的和公开的模板
        const templates = MOCK_TEMPLATES.filter(
          template => template.createdBy === userId || template.isPublic
        );
        resolve(templates);
      }, 500);
    });
  },

  // 按ID获取提示模板
  getPromptTemplateById: async (id: string): Promise<PromptTemplate | null> => {
    // 模拟API调用
    return new Promise((resolve) => {
      setTimeout(() => {
        const template = MOCK_TEMPLATES.find(template => template.id === id);
        resolve(template || null);
      }, 300);
    });
  },

  // 创建新的提示模板
  createPromptTemplate: async (
    params: CreatePromptTemplateParams,
    userId: string
  ): Promise<PromptTemplate> => {
    // 模拟API调用
    return new Promise((resolve) => {
      setTimeout(() => {
        const now = new Date().toISOString();
        const newTemplate: PromptTemplate = {
          id: `pt-${MOCK_TEMPLATES.length + 1}`,
          ...params,
          tags: params.tags || [],
          isPublic: params.isPublic ?? false,
          createdAt: now,
          updatedAt: now,
          createdBy: userId,
          version: 1
        };
        
        // 在实际应用中这里会发送API请求
        // 这里只是模拟添加到本地数组
        MOCK_TEMPLATES.push(newTemplate);
        
        resolve(newTemplate);
      }, 700);
    });
  },

  // 更新提示模板
  updatePromptTemplate: async (
    id: string,
    params: UpdatePromptTemplateParams
  ): Promise<PromptTemplate> => {
    // 模拟API调用
    return new Promise((resolve, reject) => {
      setTimeout(() => {
        const templateIndex = MOCK_TEMPLATES.findIndex(template => template.id === id);
        
        if (templateIndex === -1) {
          reject(new Error(`Template with id ${id} not found`));
          return;
        }
        
        const updatedTemplate: PromptTemplate = {
          ...MOCK_TEMPLATES[templateIndex],
          ...params,
          updatedAt: new Date().toISOString(),
          version: MOCK_TEMPLATES[templateIndex].version + 1
        };
        
        // 在实际应用中这里会发送API请求
        // 这里只是模拟更新本地数组
        MOCK_TEMPLATES[templateIndex] = updatedTemplate;
        
        resolve(updatedTemplate);
      }, 700);
    });
  },

  // 删除提示模板
  deletePromptTemplate: async (id: string): Promise<boolean> => {
    // 模拟API调用
    return new Promise((resolve, reject) => {
      setTimeout(() => {
        const templateIndex = MOCK_TEMPLATES.findIndex(template => template.id === id);
        
        if (templateIndex === -1) {
          reject(new Error(`Template with id ${id} not found`));
          return;
        }
        
        // 在实际应用中这里会发送API请求
        // 这里只是模拟从本地数组中删除
        MOCK_TEMPLATES.splice(templateIndex, 1);
        
        resolve(true);
      }, 500);
    });
  },

  // 执行提示模板
  executePrompt: async (params: ExecutePromptParams): Promise<ExecutePromptResult> => {
    // 模拟API调用
    return new Promise((resolve, reject) => {
      setTimeout(() => {
        const template = MOCK_TEMPLATES.find(template => template.id === params.templateId);
        
        if (!template) {
          reject(new Error(`Template with id ${params.templateId} not found`));
          return;
        }
        
        // 替换变量
        let promptContent = template.content;
        for (const [varName, varValue] of Object.entries(params.variables)) {
          promptContent = promptContent.replace(new RegExp(`{{${varName}}}`, 'g'), String(varValue));
        }
        
        // 检查是否所有必填变量都已替换
        const missingVars = template.variables
          .filter(v => v.required && promptContent.includes(`{{${v.name}}}`))
          .map(v => v.name);
        
        if (missingVars.length > 0) {
          reject(new Error(`Missing required variables: ${missingVars.join(', ')}`));
          return;
        }
        
        // 模拟执行结果
        // 在实际应用中这里会调用LLM API
        const result: ExecutePromptResult = {
          output: `这是使用 "${template.name}" 模板生成的回复。\n\n模拟的AI输出内容会显示在这里。实际应用中，这将由LLM生成。`,
          tokenUsage: {
            prompt: Math.floor(promptContent.length / 4),
            completion: 150,
            total: Math.floor(promptContent.length / 4) + 150
          },
          modelUsed: params.model || template.model || 'gpt-4',
          executionTime: Math.random() * 2000 + 500  // 模拟500ms-2.5s的执行时间
        };
        
        resolve(result);
      }, 1500);  // 模拟较长的API调用时间
    });
  },

  // 搜索提示模板
  searchPromptTemplates: async (
    query: string,
    filters?: {
      tags?: string[];
      category?: string;
      model?: string;
      isPublic?: boolean;
    }
  ): Promise<PromptTemplate[]> => {
    // 模拟API调用
    return new Promise((resolve) => {
      setTimeout(() => {
        let results = MOCK_TEMPLATES;
        
        // 搜索逻辑
        if (query) {
          const queryLower = query.toLowerCase();
          results = results.filter(template => 
            template.name.toLowerCase().includes(queryLower) ||
            template.description.toLowerCase().includes(queryLower) ||
            template.content.toLowerCase().includes(queryLower) ||
            template.tags.some(tag => tag.toLowerCase().includes(queryLower))
          );
        }
        
        // 应用过滤器
        if (filters) {
          if (filters.tags && filters.tags.length > 0) {
            results = results.filter(template => 
              filters.tags!.some(tag => template.tags.includes(tag))
            );
          }
          
          if (filters.category) {
            results = results.filter(template => 
              template.category === filters.category
            );
          }
          
          if (filters.model) {
            results = results.filter(template => 
              template.model === filters.model
            );
          }
          
          if (filters.isPublic !== undefined) {
            results = results.filter(template => 
              template.isPublic === filters.isPublic
            );
          }
        }
        
        resolve(results);
      }, 600);
    });
  }
};

// 工具函数 - 从提示模板内容中提取变量
export const extractVariablesFromContent = (content: string): string[] => {
  const regex = /{{([^}]+)}}/g;
  const matches = Array.from(content.matchAll(regex));
  const variables = matches.map(match => match[1]);
  
  // 返回不重复的变量列表
  return [...new Set(variables)];
};

// 工具函数 - 验证提示模板
export const validatePromptTemplate = (
  template: CreatePromptTemplateParams | UpdatePromptTemplateParams
): { valid: boolean; errors: string[] } => {
  const errors: string[] = [];
  
  // 检查基本字段
  if ('name' in template && (!template.name || template.name.trim() === '')) {
    errors.push('模板名称不能为空');
  }
  
  if ('content' in template && (!template.content || template.content.trim() === '')) {
    errors.push('模板内容不能为空');
  }
  
  // 如果提供了content和variables，检查变量一致性
  if ('content' in template && 'variables' in template && template.content && template.variables) {
    const contentVars = extractVariablesFromContent(template.content);
    const definedVars = template.variables.map(v => v.name);
    
    // 检查内容中的所有变量是否已定义
    const undefinedVars = contentVars.filter(v => !definedVars.includes(v));
    if (undefinedVars.length > 0) {
      errors.push(`模板内容中使用了未定义的变量: ${undefinedVars.join(', ')}`);
    }
    
    // 检查是否有定义但未使用的变量
    const unusedVars = definedVars.filter(v => !contentVars.includes(v));
    if (unusedVars.length > 0) {
      errors.push(`定义了但未在模板内容中使用的变量: ${unusedVars.join(', ')}`);
    }
  }
  
  return {
    valid: errors.length === 0,
    errors
  };
}; 