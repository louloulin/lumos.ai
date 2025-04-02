import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { CreateProjectForm } from '../CreateProjectForm';
import * as authService from '../../../services/auth';
import * as projectsService from '../../../services/projects';

// 模拟服务
vi.mock('../../../services/auth', () => ({
  useAuth: vi.fn()
}));

vi.mock('../../../services/projects', () => ({
  projectsService: {
    createProject: vi.fn()
  }
}));

describe('CreateProjectForm Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the create project form correctly', () => {
    // 模拟认证状态
    vi.mocked(authService.useAuth).mockReturnValue({
      user: { id: '1', username: 'testuser', email: 'test@example.com', role: 'user', createdAt: '' },
      isAuthenticated: true,
      isLoading: false,
      error: null,
      login: vi.fn(),
      register: vi.fn(),
      logout: vi.fn()
    });

    render(<CreateProjectForm />);

    // 验证表单元素存在
    expect(screen.getByText('创建新项目')).toBeInTheDocument();
    expect(screen.getByLabelText('项目名称')).toBeInTheDocument();
    expect(screen.getByLabelText('项目描述')).toBeInTheDocument();
    expect(screen.getByLabelText('标签')).toBeInTheDocument();
    expect(screen.getByLabelText('公开项目')).toBeInTheDocument();
    expect(screen.getByText('创建项目')).toBeInTheDocument();
    expect(screen.getByText('取消')).toBeInTheDocument();
  });

  it('handles form submission correctly', async () => {
    // 模拟用户
    const mockUser = { 
      id: '1', 
      username: 'testuser', 
      email: 'test@example.com', 
      role: 'user' as const, 
      createdAt: '2023-01-01'
    };
    
    // 模拟创建项目成功
    const mockNewProject = {
      id: 'new-project-id',
      name: 'Test Project',
      description: 'Test Description',
      createdAt: '2023-06-01T00:00:00Z',
      updatedAt: '2023-06-01T00:00:00Z',
      owner: '1',
      isPublic: false,
      status: 'draft' as const,
      tags: ['test', 'ai']
    };
    
    vi.mocked(authService.useAuth).mockReturnValue({
      user: mockUser,
      isAuthenticated: true,
      isLoading: false,
      error: null,
      login: vi.fn(),
      register: vi.fn(),
      logout: vi.fn()
    });
    
    vi.mocked(projectsService.projectsService.createProject).mockResolvedValue(mockNewProject);
    
    const onSuccess = vi.fn();
    render(<CreateProjectForm onSuccess={onSuccess} />);
    
    // 填写表单
    fireEvent.change(screen.getByLabelText('项目名称'), {
      target: { value: 'Test Project' }
    });
    
    fireEvent.change(screen.getByLabelText('项目描述'), {
      target: { value: 'Test Description' }
    });
    
    // 添加标签
    const tagInput = screen.getByLabelText('标签');
    fireEvent.change(tagInput, { target: { value: 'test' } });
    fireEvent.keyDown(tagInput, { key: 'Enter' });
    
    fireEvent.change(tagInput, { target: { value: 'ai' } });
    fireEvent.keyDown(tagInput, { key: 'Enter' });
    
    // 提交表单
    fireEvent.click(screen.getByText('创建项目'));
    
    // 验证createProject被调用
    await waitFor(() => {
      expect(projectsService.projectsService.createProject).toHaveBeenCalledWith(
        {
          name: 'Test Project',
          description: 'Test Description',
          isPublic: false,
          tags: ['test', 'ai']
        },
        '1'
      );
    });
    
    // 验证成功回调被调用
    await waitFor(() => {
      expect(onSuccess).toHaveBeenCalledWith('new-project-id');
    });
    
    // 验证成功页面显示
    await waitFor(() => {
      expect(screen.getByText('项目创建成功')).toBeInTheDocument();
    });
  });

  it('validates required fields', async () => {
    // 模拟用户
    vi.mocked(authService.useAuth).mockReturnValue({
      user: { id: '1', username: 'testuser', email: 'test@example.com', role: 'user', createdAt: '' },
      isAuthenticated: true,
      isLoading: false,
      error: null,
      login: vi.fn(),
      register: vi.fn(),
      logout: vi.fn()
    });
    
    render(<CreateProjectForm />);
    
    // 创建按钮应该被禁用（因为名称是必填的）
    const createButton = screen.getByText('创建项目');
    expect(createButton).toBeDisabled();
    
    // 输入项目名称后应该启用按钮
    fireEvent.change(screen.getByLabelText('项目名称'), {
      target: { value: 'Test Project' }
    });
    
    expect(createButton).not.toBeDisabled();
  });

  it('handles tag management correctly', () => {
    // 模拟用户
    vi.mocked(authService.useAuth).mockReturnValue({
      user: { id: '1', username: 'testuser', email: 'test@example.com', role: 'user', createdAt: '' },
      isAuthenticated: true,
      isLoading: false,
      error: null,
      login: vi.fn(),
      register: vi.fn(),
      logout: vi.fn()
    });
    
    render(<CreateProjectForm />);
    
    // 添加标签
    const tagInput = screen.getByLabelText('标签');
    fireEvent.change(tagInput, { target: { value: 'test-tag' } });
    
    // 点击添加按钮
    const addButton = screen.getByRole('button', { name: '' }); // '+'按钮没有文本，所以使用空名称
    fireEvent.click(addButton);
    
    // 验证标签已添加
    expect(screen.getByText('test-tag')).toBeInTheDocument();
    
    // 验证输入框已清空
    expect(tagInput).toHaveValue('');
    
    // 移除标签
    const removeButton = screen.getByText('test-tag').nextSibling;
    fireEvent.click(removeButton as Element);
    
    // 验证标签已移除
    expect(screen.queryByText('test-tag')).not.toBeInTheDocument();
  });

  it('displays error message when creation fails', async () => {
    // 模拟用户
    vi.mocked(authService.useAuth).mockReturnValue({
      user: { id: '1', username: 'testuser', email: 'test@example.com', role: 'user', createdAt: '' },
      isAuthenticated: true,
      isLoading: false,
      error: null,
      login: vi.fn(),
      register: vi.fn(),
      logout: vi.fn()
    });
    
    // 模拟创建项目失败
    vi.mocked(projectsService.projectsService.createProject).mockRejectedValue(
      new Error('Failed to create project')
    );
    
    render(<CreateProjectForm />);
    
    // 填写并提交表单
    fireEvent.change(screen.getByLabelText('项目名称'), {
      target: { value: 'Test Project' }
    });
    
    fireEvent.click(screen.getByText('创建项目'));
    
    // 验证错误消息显示
    await waitFor(() => {
      expect(screen.getByText('Failed to create project')).toBeInTheDocument();
    });
  });
}); 