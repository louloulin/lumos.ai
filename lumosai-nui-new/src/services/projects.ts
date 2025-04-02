import { isTauri } from './auth';

// 项目类型
export interface Project {
  id: string;
  name: string;
  description: string;
  thumbnail?: string;
  createdAt: string;
  updatedAt: string;
  owner: string;
  isPublic: boolean;
  status: 'active' | 'archived' | 'draft';
  tags: string[];
  stats?: {
    models: number;
    agents: number;
    workflows: number;
    tools: number;
  };
}

// 新项目表单类型
export interface NewProjectForm {
  name: string;
  description: string;
  isPublic: boolean;
  tags: string[];
}

// 使用统计数据类型
export interface UsageStats {
  tokensUsed: number;
  requestCount: number;
  modelsUsed: {
    name: string;
    requests: number;
    tokens: number;
  }[];
  dailyUsage: {
    date: string;
    tokens: number;
    requests: number;
  }[];
}

// 模拟项目数据
const MOCK_PROJECTS: Project[] = [
  {
    id: '1',
    name: 'AI聊天助手',
    description: '基于大型语言模型的会话式AI助手',
    createdAt: '2023-03-15T10:30:00Z',
    updatedAt: '2023-04-10T15:45:00Z',
    owner: '1',
    isPublic: true,
    status: 'active',
    tags: ['chatbot', 'llm', 'gpt'],
    stats: {
      models: 2,
      agents: 1,
      workflows: 3,
      tools: 5
    }
  },
  {
    id: '2',
    name: '数据分析工作流',
    description: '自动化数据处理和分析工作流',
    createdAt: '2023-02-20T09:00:00Z',
    updatedAt: '2023-04-05T11:20:00Z',
    owner: '1',
    isPublic: false,
    status: 'active',
    tags: ['data', 'analytics', 'automation'],
    stats: {
      models: 1,
      agents: 0,
      workflows: 2,
      tools: 8
    }
  },
  {
    id: '3',
    name: '内容生成器',
    description: '自动化内容创建和优化工具',
    createdAt: '2023-01-10T14:25:00Z',
    updatedAt: '2023-03-28T16:40:00Z',
    owner: '2',
    isPublic: true,
    status: 'active',
    tags: ['content', 'generation', 'marketing'],
    stats: {
      models: 3,
      agents: 2,
      workflows: 1,
      tools: 4
    }
  }
];

// 模拟API调用延迟
const simulateApiDelay = (ms = 800) => new Promise(resolve => setTimeout(resolve, ms));

// 存储键
const projectsStorageKey = 'lumosai_projects';

/**
 * 项目服务类 - 处理项目相关操作
 */
class ProjectsService {
  // 获取所有项目
  async getProjects(userId?: string): Promise<Project[]> {
    await simulateApiDelay();
    
    try {
      // 获取存储的项目
      const projects = this.getProjectsFromStorage();
      
      // 如果提供了用户ID，筛选该用户的项目
      if (userId) {
        return projects.filter(project => project.owner === userId);
      }
      
      return projects;
    } catch (error) {
      console.error('Error fetching projects:', error);
      return [];
    }
  }

  // 获取项目详情
  async getProjectById(id: string): Promise<Project | null> {
    await simulateApiDelay(400);
    
    try {
      const projects = this.getProjectsFromStorage();
      return projects.find(project => project.id === id) || null;
    } catch (error) {
      console.error(`Error fetching project ${id}:`, error);
      return null;
    }
  }

  // 创建新项目
  async createProject(projectData: NewProjectForm, userId: string): Promise<Project> {
    await simulateApiDelay();
    
    try {
      const projects = this.getProjectsFromStorage();
      
      // 创建新项目对象
      const newProject: Project = {
        id: String(Date.now()), // 使用时间戳作为唯一ID
        name: projectData.name,
        description: projectData.description,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        owner: userId,
        isPublic: projectData.isPublic,
        status: 'draft',
        tags: projectData.tags,
        stats: {
          models: 0,
          agents: 0,
          workflows: 0,
          tools: 0
        }
      };
      
      // 添加到项目列表并保存
      const updatedProjects = [...projects, newProject];
      this.saveProjectsToStorage(updatedProjects);
      
      return newProject;
    } catch (error) {
      console.error('Error creating project:', error);
      throw new Error('Failed to create project');
    }
  }

  // 更新项目
  async updateProject(id: string, projectData: Partial<Project>): Promise<Project> {
    await simulateApiDelay();
    
    try {
      const projects = this.getProjectsFromStorage();
      const projectIndex = projects.findIndex(project => project.id === id);
      
      if (projectIndex === -1) {
        throw new Error('Project not found');
      }
      
      // 更新项目
      const updatedProject = {
        ...projects[projectIndex],
        ...projectData,
        updatedAt: new Date().toISOString()
      };
      
      projects[projectIndex] = updatedProject;
      this.saveProjectsToStorage(projects);
      
      return updatedProject;
    } catch (error) {
      console.error(`Error updating project ${id}:`, error);
      throw new Error('Failed to update project');
    }
  }

  // 删除项目
  async deleteProject(id: string): Promise<boolean> {
    await simulateApiDelay();
    
    try {
      let projects = this.getProjectsFromStorage();
      const initialLength = projects.length;
      
      // 过滤掉要删除的项目
      projects = projects.filter(project => project.id !== id);
      
      if (projects.length === initialLength) {
        return false; // 项目未找到
      }
      
      this.saveProjectsToStorage(projects);
      return true;
    } catch (error) {
      console.error(`Error deleting project ${id}:`, error);
      throw new Error('Failed to delete project');
    }
  }

  // 存储项目数据
  private saveProjectsToStorage(projects: Project[]): void {
    if (isTauri()) {
      // Tauri环境处理
      try {
        localStorage.setItem(projectsStorageKey, JSON.stringify(projects));
      } catch (err) {
        console.error('Error saving to Tauri storage:', err);
      }
    } else {
      // Web环境
      localStorage.setItem(projectsStorageKey, JSON.stringify(projects));
    }
  }

  // 从存储获取项目数据
  private getProjectsFromStorage(): Project[] {
    try {
      const projectsData = localStorage.getItem(projectsStorageKey);
      // 如果没有存储的项目，返回模拟数据
      if (!projectsData) {
        // 初始化存储
        this.saveProjectsToStorage(MOCK_PROJECTS);
        return MOCK_PROJECTS;
      }
      return JSON.parse(projectsData);
    } catch (err) {
      console.error('Error reading projects from storage:', err);
      return MOCK_PROJECTS;
    }
  }
}

// 创建并导出项目服务实例
export const projectsService = new ProjectsService();

// 获取使用统计数据
export async function getUsageStats(userId: string): Promise<UsageStats> {
  await simulateApiDelay();
  
  // 模拟使用统计数据
  return {
    tokensUsed: 1250000,
    requestCount: 7824,
    modelsUsed: [
      { name: 'GPT-4o', requests: 3560, tokens: 750000 },
      { name: 'Claude 3 Opus', requests: 2104, tokens: 350000 },
      { name: 'Gemini Pro', requests: 1220, tokens: 150000 },
      { name: 'Mistral', requests: 940, tokens: 80000 }
    ],
    dailyUsage: Array.from({ length: 30 }, (_, i) => {
      const date = new Date();
      date.setDate(date.getDate() - (29 - i));
      
      return {
        date: date.toISOString().split('T')[0],
        tokens: Math.floor(Math.random() * 80000) + 20000,
        requests: Math.floor(Math.random() * 400) + 100
      };
    })
  };
}

// 获取项目使用统计数据 - 为仪表盘提供数据
export async function getProjectUsage(): Promise<UsageStats[]> {
  await simulateApiDelay();
  
  // 生成过去30天的每日使用数据
  return Array.from({ length: 30 }, (_, i) => {
    const date = new Date();
    date.setDate(date.getDate() - (29 - i));
    
    return {
      date: date.toISOString().split('T')[0],
      tokens: Math.floor(Math.random() * 80000) + 20000,
      requests: Math.floor(Math.random() * 400) + 100,
      tokensUsed: Math.floor(Math.random() * 100000) + 30000,
      requestCount: Math.floor(Math.random() * 500) + 150,
      modelsUsed: [
        { name: 'GPT-4', requests: Math.floor(Math.random() * 200) + 50, tokens: Math.floor(Math.random() * 40000) + 10000 },
        { name: 'Claude', requests: Math.floor(Math.random() * 150) + 30, tokens: Math.floor(Math.random() * 30000) + 5000 }
      ],
      dailyUsage: []
    };
  });
} 