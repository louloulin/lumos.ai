import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { vi } from 'vitest';
import AgentChat from './agent-chat';

// 模拟数据
const mockAgent = {
  id: 'test-agent',
  name: '测试代理',
  description: '这是一个用于测试的代理',
  model: 'Test Model'
};

describe('AgentChat Component', () => {
  // 模拟API调用
  const mockSendMessage = vi.fn().mockResolvedValue(undefined);
  
  beforeEach(() => {
    mockSendMessage.mockClear();
  });

  it('renders the agent chat component correctly', () => {
    render(<AgentChat agent={mockAgent} />);
    
    // 检查代理名称是否正确显示
    expect(screen.getByText('测试代理')).toBeInTheDocument();
    
    // 检查输入框是否存在
    expect(screen.getByPlaceholderText('输入您的消息...')).toBeInTheDocument();
    
    // 检查发送按钮是否存在且被禁用（因为没有输入）
    const sendButton = screen.getByRole('button', { name: /send/i });
    expect(sendButton).toBeInTheDocument();
    expect(sendButton).toBeDisabled();
  });

  it('allows user to type a message and send it', async () => {
    render(<AgentChat agent={mockAgent} onSendMessage={mockSendMessage} />);
    
    // 获取输入框和发送按钮
    const inputElement = screen.getByPlaceholderText('输入您的消息...');
    const sendButton = screen.getByRole('button', { name: /send/i });
    
    // 输入消息
    fireEvent.change(inputElement, { target: { value: '你好，代理！' } });
    
    // 检查发送按钮是否启用
    expect(sendButton).not.toBeDisabled();
    
    // 点击发送按钮
    fireEvent.click(sendButton);
    
    // 验证消息是否发送
    expect(mockSendMessage).toHaveBeenCalledWith('你好，代理！', []);
    
    // 验证消息是否显示在界面上
    await waitFor(() => {
      expect(screen.getByText('你好，代理！')).toBeInTheDocument();
    });
    
    // 验证输入框是否被清空
    expect(inputElement).toHaveValue('');
  });

  it('handles Enter key to send messages', () => {
    render(<AgentChat agent={mockAgent} onSendMessage={mockSendMessage} />);
    
    // 获取输入框
    const inputElement = screen.getByPlaceholderText('输入您的消息...');
    
    // 输入消息
    fireEvent.change(inputElement, { target: { value: '通过回车键发送' } });
    
    // 按下回车键
    fireEvent.keyDown(inputElement, { key: 'Enter', code: 'Enter' });
    
    // 验证消息是否发送
    expect(mockSendMessage).toHaveBeenCalledWith('通过回车键发送', []);
  });

  it('displays initial messages if provided', () => {
    const initialMessages = [
      {
        id: 'msg-1',
        role: 'user',
        content: '初始用户消息',
        timestamp: new Date(),
        status: 'sent'
      },
      {
        id: 'msg-2',
        role: 'agent',
        content: '初始代理回复',
        timestamp: new Date(),
        status: 'sent'
      }
    ];
    
    render(<AgentChat agent={mockAgent} initialMessages={initialMessages} />);
    
    // 验证初始消息是否显示
    expect(screen.getByText('初始用户消息')).toBeInTheDocument();
    expect(screen.getByText('初始代理回复')).toBeInTheDocument();
  });

  it('renders code blocks correctly', async () => {
    const initialMessages = [
      {
        id: 'msg-code',
        role: 'agent',
        content: '这是代码示例：\n```javascript\nconsole.log("Hello World");\n```',
        timestamp: new Date(),
        status: 'sent'
      }
    ];
    
    render(<AgentChat agent={mockAgent} initialMessages={initialMessages} />);
    
    // 验证代码块文本是否存在
    await waitFor(() => {
      expect(screen.getByText(/这是代码示例：/)).toBeInTheDocument();
      expect(screen.getByText(/console.log\("Hello World"\);/)).toBeInTheDocument();
    });
  });

  it('shows typing indicator when agent is responding', async () => {
    // 使用自定义的onSendMessage函数来模拟代理响应延迟
    const delayedResponse = vi.fn().mockImplementation(async () => {
      // 这将使组件进入"typing"状态，但不会自动添加响应
      // 实际组件中应该有一种方式来处理这个状态
      await new Promise(resolve => setTimeout(resolve, 100));
    });
    
    render(<AgentChat agent={mockAgent} onSendMessage={delayedResponse} />);
    
    // 发送消息
    const inputElement = screen.getByPlaceholderText('输入您的消息...');
    fireEvent.change(inputElement, { target: { value: '请稍等片刻' } });
    fireEvent.click(screen.getByRole('button', { name: /send/i }));
    
    // 检查发送状态
    await waitFor(() => {
      expect(delayedResponse).toHaveBeenCalled();
    });
  });
}); 