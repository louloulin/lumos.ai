import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { LoginForm } from '../LoginForm';
import * as authService from '../../../services/auth';

// 模拟auth服务
vi.mock('../../../services/auth', () => ({
  useAuth: vi.fn()
}));

describe('LoginForm Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the login form correctly', () => {
    // 模拟useAuth钩子返回值
    vi.mocked(authService.useAuth).mockReturnValue({
      login: vi.fn(),
      isLoading: false,
      error: null,
      user: null,
      isAuthenticated: false,
      register: vi.fn(),
      logout: vi.fn()
    });

    const onRegisterClick = vi.fn();
    render(<LoginForm onRegisterClick={onRegisterClick} />);

    // 验证表单元素存在
    expect(screen.getByText('登录到LumosAI')).toBeInTheDocument();
    expect(screen.getByLabelText('电子邮箱')).toBeInTheDocument();
    expect(screen.getByLabelText('密码')).toBeInTheDocument();
    expect(screen.getByText('登录')).toBeInTheDocument();
    expect(screen.getByText('注册新账号')).toBeInTheDocument();
  });

  it('handles form submission', async () => {
    const mockLogin = vi.fn().mockResolvedValue({ id: '1', username: 'testuser' });
    
    // 模拟useAuth钩子返回值
    vi.mocked(authService.useAuth).mockReturnValue({
      login: mockLogin,
      isLoading: false,
      error: null,
      user: null,
      isAuthenticated: false,
      register: vi.fn(),
      logout: vi.fn()
    });

    const onSuccess = vi.fn();
    const onRegisterClick = vi.fn();
    
    render(<LoginForm onSuccess={onSuccess} onRegisterClick={onRegisterClick} />);
    
    // 填写表单
    fireEvent.change(screen.getByLabelText('电子邮箱'), {
      target: { value: 'test@example.com' }
    });
    
    fireEvent.change(screen.getByLabelText('密码'), {
      target: { value: 'password123' }
    });
    
    // 提交表单
    fireEvent.click(screen.getByText('登录'));
    
    // 验证提交
    await waitFor(() => {
      expect(mockLogin).toHaveBeenCalledWith({
        email: 'test@example.com',
        password: 'password123'
      });
    });
    
    // 如果登录成功，onSuccess应该被调用
    await waitFor(() => {
      expect(onSuccess).toHaveBeenCalled();
    });
  });

  it('displays error message when login fails', async () => {
    const mockLogin = vi.fn().mockRejectedValue(new Error('Invalid credentials'));
    
    // 模拟useAuth钩子返回值，包含错误消息
    vi.mocked(authService.useAuth).mockReturnValue({
      login: mockLogin,
      isLoading: false,
      error: 'Invalid email or password',
      user: null,
      isAuthenticated: false,
      register: vi.fn(),
      logout: vi.fn()
    });

    const onRegisterClick = vi.fn();
    
    render(<LoginForm onRegisterClick={onRegisterClick} />);
    
    // 验证错误消息显示
    expect(screen.getByText('Invalid email or password')).toBeInTheDocument();
  });

  it('shows loading state when login is in progress', () => {
    // 模拟useAuth钩子返回isLoading为true
    vi.mocked(authService.useAuth).mockReturnValue({
      login: vi.fn(),
      isLoading: true,
      error: null,
      user: null,
      isAuthenticated: false,
      register: vi.fn(),
      logout: vi.fn()
    });

    const onRegisterClick = vi.fn();
    
    render(<LoginForm onRegisterClick={onRegisterClick} />);
    
    // 验证加载状态
    expect(screen.getByText('正在登录...')).toBeInTheDocument();
    
    // 验证按钮被禁用
    const loginButton = screen.getByText('正在登录...').closest('button');
    expect(loginButton).toBeDisabled();
  });

  it('navigates to registration when register link is clicked', () => {
    // 模拟useAuth钩子返回值
    vi.mocked(authService.useAuth).mockReturnValue({
      login: vi.fn(),
      isLoading: false,
      error: null,
      user: null,
      isAuthenticated: false,
      register: vi.fn(),
      logout: vi.fn()
    });

    const onRegisterClick = vi.fn();
    
    render(<LoginForm onRegisterClick={onRegisterClick} />);
    
    // 点击注册链接
    fireEvent.click(screen.getByText('注册新账号'));
    
    // 验证导航回调被调用
    expect(onRegisterClick).toHaveBeenCalled();
  });
}); 