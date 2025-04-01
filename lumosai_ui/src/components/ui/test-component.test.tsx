import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { vi } from 'vitest';

import KnowledgeBasePage from '@/pages/knowledge-base';
import WorkflowEditor from '@/pages/workflow-editor';
import { Button } from '@/components/ui/button';

// 模拟路由
vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
  useParams: () => ({ id: 'test-id' }),
}));

// 模拟toast
vi.mock('@/components/ui/use-toast', () => ({
  useToast: () => ({
    toast: vi.fn(),
  }),
}));

describe('Button Component', () => {
  it('renders button correctly', () => {
    render(<Button>测试按钮</Button>);
    expect(screen.getByText('测试按钮')).toBeInTheDocument();
  });

  it('calls onClick handler when clicked', () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>点击我</Button>);
    
    const button = screen.getByText('点击我');
    fireEvent.click(button);
    
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('renders different variants correctly', () => {
    const { rerender } = render(<Button variant="default">默认按钮</Button>);
    expect(screen.getByText('默认按钮')).toHaveClass('bg-primary');
    
    rerender(<Button variant="destructive">删除按钮</Button>);
    expect(screen.getByText('删除按钮')).toHaveClass('bg-destructive');
    
    rerender(<Button variant="outline">轮廓按钮</Button>);
    expect(screen.getByText('轮廓按钮')).toHaveClass('border');
  });

  it('renders disabled state correctly', () => {
    render(<Button disabled>禁用按钮</Button>);
    const button = screen.getByText('禁用按钮');
    
    expect(button).toBeDisabled();
    expect(button).toHaveClass('opacity-50');
  });
});

describe('WorkflowEditor Component', () => {
  it('renders without crashing', () => {
    render(<WorkflowEditor />);
    // 由于组件使用了模拟数据和加载状态，我们只能验证它渲染不崩溃
  });

  it('should show loading state initially', () => {
    render(<WorkflowEditor />);
    // 检查加载动画的存在
    const loadingElements = document.querySelectorAll('.animate-pulse');
    expect(loadingElements.length).toBeGreaterThan(0);
  });

  // 更多工作流编辑器测试...
});

describe('KnowledgeBasePage Component', () => {
  it('renders without crashing', () => {
    render(<KnowledgeBasePage />);
    // 由于组件使用了模拟数据和加载状态，我们只能验证它渲染不崩溃
  });

  it('should show the correct title', async () => {
    render(<KnowledgeBasePage />);
    
    // 等待数据加载完成
    await waitFor(() => {
      expect(screen.getByText('知识库管理')).toBeInTheDocument();
    });
  });

  it('should display the upload document dialog when button is clicked', async () => {
    render(<KnowledgeBasePage />);
    
    // 等待数据加载完成
    await waitFor(() => {
      const uploadButton = screen.getByText('上传文档', { selector: 'button' });
      fireEvent.click(uploadButton);
    });
    
    // 验证对话框出现
    expect(screen.getByText('上传文档到知识库，支持PDF、Word、Excel、PowerPoint等多种格式')).toBeInTheDocument();
  });

  // 更多知识库页面测试...
});

// 集成测试示例
describe('UI Integration Tests', () => {
  it('should navigate between pages', () => {
    // 在实际环境中，这个测试需要使用一个更完整的路由设置
    // 这里只是一个示例框架
    const mockNavigate = vi.fn();
    vi.mocked(mockNavigate).mockImplementation(() => {});
    
    // 这里会添加模拟路由导航的测试
  });
}); 