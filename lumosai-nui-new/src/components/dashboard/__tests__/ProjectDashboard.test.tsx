import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ProjectDashboard } from '../ProjectDashboard';
import * as projectsService from '../../../services/projects';

// 模拟项目服务
vi.mock('../../../services/projects', () => ({
  getProjects: vi.fn(),
  getProjectUsage: vi.fn(),
}));

describe('ProjectDashboard Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should display loading state initially', () => {
    // 模拟异步数据加载
    vi.mocked(projectsService.getProjects).mockResolvedValue([]);
    vi.mocked(projectsService.getProjectUsage).mockResolvedValue([]);

    render(<ProjectDashboard />);
    
    // 检查是否显示加载指示器
    const loadingElement = document.querySelector('.animate-spin');
    expect(loadingElement).toBeInTheDocument();
  });

  it('should display projects when data is loaded', async () => {
    // 模拟项目数据
    const mockProjects = [
      { 
        id: '1', 
        name: 'Test Project 1', 
        description: 'Description 1',
        lastActive: '2 hours ago', 
        status: 'active' as const, 
        model: 'GPT-4',
        createdAt: '2023-01-01T00:00:00Z',
        updatedAt: '2023-01-01T00:00:00Z'
      },
      { 
        id: '2', 
        name: 'Test Project 2', 
        description: 'Description 2',
        lastActive: '1 day ago', 
        status: 'inactive' as const, 
        model: 'Claude 3',
        createdAt: '2023-01-01T00:00:00Z',
        updatedAt: '2023-01-01T00:00:00Z'
      }
    ];
    
    const mockUsage = [
      { date: '2023-04-01', tokens: 4000, requests: 2400, cost: 240 },
      { date: '2023-04-02', tokens: 3000, requests: 1398, cost: 139 }
    ];

    // 设置模拟返回值
    vi.mocked(projectsService.getProjects).mockResolvedValue(mockProjects);
    vi.mocked(projectsService.getProjectUsage).mockResolvedValue(mockUsage);

    render(<ProjectDashboard />);

    // 等待加载完成并验证项目数据显示
    await waitFor(() => {
      expect(screen.getByText('Test Project 1')).toBeInTheDocument();
      expect(screen.getByText('Test Project 2')).toBeInTheDocument();
    });

    // 验证统计数据正确显示
    expect(screen.getByText('2')).toBeInTheDocument(); // 总项目数
    expect(screen.getByText('1')).toBeInTheDocument(); // 活跃项目数
  });

  it('should display empty state when no projects', async () => {
    // 模拟空项目数据
    vi.mocked(projectsService.getProjects).mockResolvedValue([]);
    vi.mocked(projectsService.getProjectUsage).mockResolvedValue([]);

    render(<ProjectDashboard />);

    // 等待加载完成并验证空状态显示
    await waitFor(() => {
      expect(screen.getByText('No projects found. Create your first project to get started.')).toBeInTheDocument();
    });
  });
}); 