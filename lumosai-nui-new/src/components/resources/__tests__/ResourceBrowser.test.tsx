import React from 'react';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ResourceBrowser } from '../ResourceBrowser';
import * as resourcesService from '../../../services/resources';

// 模拟资源服务
vi.mock('../../../services/resources', () => ({
  getModels: vi.fn(),
  getTools: vi.fn(),
  getAgents: vi.fn(),
  getWorkflows: vi.fn(),
  searchResources: vi.fn(),
}));

describe('ResourceBrowser Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should display loading state initially', () => {
    // 模拟异步数据加载
    vi.mocked(resourcesService.getModels).mockResolvedValue([]);

    render(<ResourceBrowser />);
    
    // 检查是否显示加载指示器
    const loadingElement = document.querySelector('.animate-spin');
    expect(loadingElement).toBeInTheDocument();
  });

  it('should display models when data is loaded', async () => {
    // 模拟模型数据
    const mockModels = [
      {
        id: 'model-1',
        name: 'Test Model',
        description: 'A test model',
        type: 'model' as const,
        provider: 'Test Provider',
        contextWindow: 8192,
        capabilities: ['text-generation'],
        createdAt: '2023-01-01T00:00:00Z',
        updatedAt: '2023-01-01T00:00:00Z',
        tags: ['test', 'model']
      }
    ];

    // 设置模拟返回值
    vi.mocked(resourcesService.getModels).mockResolvedValue(mockModels);

    render(<ResourceBrowser />);

    // 等待加载完成并验证模型数据显示
    await waitFor(() => {
      expect(screen.getByText('Test Model')).toBeInTheDocument();
      expect(screen.getByText('A test model')).toBeInTheDocument();
      expect(screen.getByText('Test Provider')).toBeInTheDocument();
      expect(screen.getByText('8,192 tokens')).toBeInTheDocument();
    });
  });

  it('should display empty state when no models are found', async () => {
    // 模拟空模型数据
    vi.mocked(resourcesService.getModels).mockResolvedValue([]);

    render(<ResourceBrowser />);

    // 等待加载完成并验证空状态显示
    await waitFor(() => {
      expect(screen.getByText('No Models Found')).toBeInTheDocument();
      expect(screen.getByText('Get started by creating your first model')).toBeInTheDocument();
    });
  });

  it('should switch between resource types', async () => {
    // 模拟各类资源数据
    const mockModels = [
      {
        id: 'model-1',
        name: 'Test Model',
        description: 'A test model',
        type: 'model' as const,
        provider: 'Test Provider',
        contextWindow: 8192,
        capabilities: ['text-generation'],
        createdAt: '2023-01-01T00:00:00Z',
        updatedAt: '2023-01-01T00:00:00Z',
        tags: ['test', 'model']
      }
    ];

    const mockTools = [
      {
        id: 'tool-1',
        name: 'Test Tool',
        description: 'A test tool',
        type: 'tool' as const,
        category: 'API',
        inputs: [],
        outputs: [],
        implementation: 'api',
        createdAt: '2023-01-01T00:00:00Z',
        updatedAt: '2023-01-01T00:00:00Z',
        tags: ['test', 'tool']
      }
    ];

    // 设置模拟返回值
    vi.mocked(resourcesService.getModels).mockResolvedValue(mockModels);
    vi.mocked(resourcesService.getTools).mockResolvedValue(mockTools);

    render(<ResourceBrowser />);

    // 等待模型加载完成
    await waitFor(() => {
      expect(screen.getByText('Test Model')).toBeInTheDocument();
    });

    // 点击工具标签页
    fireEvent.click(screen.getByText('Tools'));

    // 验证获取工具的函数被调用
    expect(resourcesService.getTools).toHaveBeenCalled();

    // 等待工具加载完成
    await waitFor(() => {
      expect(screen.getByText('Test Tool')).toBeInTheDocument();
    });
  });

  it('should search resources', async () => {
    // 模拟搜索结果
    const searchResults = [
      {
        id: 'model-2',
        name: 'Search Result Model',
        description: 'Found by search',
        type: 'model' as const,
        provider: 'Search Provider',
        contextWindow: 4096,
        capabilities: ['search-test'],
        createdAt: '2023-01-01T00:00:00Z',
        updatedAt: '2023-01-01T00:00:00Z',
        tags: ['search', 'test']
      }
    ];

    // 初始模型数据
    vi.mocked(resourcesService.getModels).mockResolvedValue([]);
    
    // 搜索结果
    vi.mocked(resourcesService.searchResources).mockResolvedValue(searchResults);

    render(<ResourceBrowser />);

    // 等待初始加载
    await waitFor(() => {
      expect(screen.getByText('No Models Found')).toBeInTheDocument();
    });

    // 输入搜索查询
    const searchInput = screen.getByPlaceholderText('Search resources...');
    fireEvent.change(searchInput, { target: { value: 'search' } });
    
    // 提交搜索
    const searchButton = screen.getByText('Search');
    fireEvent.click(searchButton);

    // 验证搜索API被调用
    expect(resourcesService.searchResources).toHaveBeenCalledWith('search', ['model'], undefined);

    // 等待搜索结果加载
    await waitFor(() => {
      expect(screen.getByText('Search Result Model')).toBeInTheDocument();
      expect(screen.getByText('Found by search')).toBeInTheDocument();
    });
  });
}); 