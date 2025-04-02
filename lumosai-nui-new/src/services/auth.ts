import { useState, useEffect } from 'react';

// 用户类型定义
export interface User {
  id: string;
  username: string;
  email: string;
  avatar?: string;
  role: 'admin' | 'user';
  createdAt: string;
}

// 认证状态类型
export interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

// 登录凭据
export interface LoginCredentials {
  email: string;
  password: string;
}

// 注册数据
export interface RegisterData {
  username: string;
  email: string;
  password: string;
}

// 检测当前环境是否为Tauri
export const isTauri = () => {
  return typeof window !== 'undefined' && window['__TAURI__' as any] !== undefined;
};

// 模拟用户数据
const MOCK_USERS: User[] = [
  {
    id: '1',
    username: 'admin',
    email: 'admin@lumosai.com',
    role: 'admin',
    createdAt: '2023-01-01T00:00:00Z',
  },
  {
    id: '2',
    username: 'demouser',
    email: 'demo@lumosai.com',
    role: 'user',
    createdAt: '2023-02-15T00:00:00Z',
  }
];

// 模拟本地存储
const localStorageKey = 'lumosai_auth';

// 模拟API调用延迟
const simulateApiDelay = (ms = 500) => new Promise(resolve => setTimeout(resolve, ms));

/**
 * 认证服务类 - 处理身份验证相关操作
 */
class AuthService {
  // 登录方法
  async login(credentials: LoginCredentials): Promise<User> {
    await simulateApiDelay();
    
    // 在实际应用中，这里应该发送API请求到服务器进行验证
    const user = MOCK_USERS.find(u => u.email === credentials.email);
    
    if (!user) {
      throw new Error('Invalid email or password');
    }
    
    // 模拟密码验证 (实际系统中不应这样做)
    if (credentials.password !== 'password') {
      throw new Error('Invalid email or password');
    }
    
    // 存储用户信息
    this.saveUserToStorage(user);
    
    return user;
  }

  // 注册方法
  async register(data: RegisterData): Promise<User> {
    await simulateApiDelay();
    
    // 检查邮箱是否已存在
    if (MOCK_USERS.some(u => u.email === data.email)) {
      throw new Error('Email already exists');
    }
    
    // 创建新用户
    const newUser: User = {
      id: String(MOCK_USERS.length + 1),
      username: data.username,
      email: data.email,
      role: 'user',
      createdAt: new Date().toISOString()
    };
    
    // 在实际应用中，这里应该发送API请求注册用户
    
    // 保存用户状态
    this.saveUserToStorage(newUser);
    
    return newUser;
  }

  // 登出方法
  async logout(): Promise<void> {
    // 从存储中清除用户信息
    if (isTauri()) {
      // 在Tauri环境中，可以使用文件系统或Tauri特定存储进行处理
      try {
        // 如果有Tauri的API可用，可以调用相应方法
        // 这里简化处理，实际实现可能需要导入Tauri相关库
        localStorage.removeItem(localStorageKey);
      } catch (err) {
        console.error('Error clearing Tauri storage:', err);
      }
    } else {
      // Web环境
      localStorage.removeItem(localStorageKey);
    }
  }

  // 获取当前用户信息
  async getCurrentUser(): Promise<User | null> {
    await simulateApiDelay(200);
    
    try {
      const userData = this.getUserFromStorage();
      return userData ? userData : null;
    } catch (err) {
      console.error('Error getting current user:', err);
      return null;
    }
  }

  // 验证用户会话是否有效
  async verifySession(): Promise<boolean> {
    try {
      const user = await this.getCurrentUser();
      return !!user;
    } catch {
      return false;
    }
  }

  // 保存用户数据到存储
  private saveUserToStorage(user: User): void {
    if (isTauri()) {
      // Tauri环境处理
      try {
        localStorage.setItem(localStorageKey, JSON.stringify(user));
      } catch (err) {
        console.error('Error saving to Tauri storage:', err);
      }
    } else {
      // Web环境
      localStorage.setItem(localStorageKey, JSON.stringify(user));
    }
  }

  // 从存储获取用户数据
  private getUserFromStorage(): User | null {
    try {
      const userData = localStorage.getItem(localStorageKey);
      return userData ? JSON.parse(userData) : null;
    } catch (err) {
      console.error('Error reading from storage:', err);
      return null;
    }
  }
}

// 创建并导出认证服务实例
export const authService = new AuthService();

/**
 * 自定义Hook: 使用认证状态和方法
 */
export function useAuth() {
  const [authState, setAuthState] = useState<AuthState>({
    user: null,
    isAuthenticated: false,
    isLoading: true,
    error: null
  });

  // 加载初始认证状态
  useEffect(() => {
    const loadUser = async () => {
      try {
        const user = await authService.getCurrentUser();
        setAuthState({
          user,
          isAuthenticated: !!user,
          isLoading: false,
          error: null
        });
      } catch (error) {
        setAuthState({
          user: null,
          isAuthenticated: false,
          isLoading: false,
          error: error instanceof Error ? error.message : 'Authentication error'
        });
      }
    };

    loadUser();
  }, []);

  // 登录函数
  const login = async (credentials: LoginCredentials) => {
    setAuthState(prev => ({ ...prev, isLoading: true, error: null }));
    
    try {
      const user = await authService.login(credentials);
      setAuthState({
        user,
        isAuthenticated: true,
        isLoading: false,
        error: null
      });
      return user;
    } catch (error) {
      setAuthState(prev => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Login failed'
      }));
      throw error;
    }
  };

  // 注册函数
  const register = async (data: RegisterData) => {
    setAuthState(prev => ({ ...prev, isLoading: true, error: null }));
    
    try {
      const user = await authService.register(data);
      setAuthState({
        user,
        isAuthenticated: true,
        isLoading: false,
        error: null
      });
      return user;
    } catch (error) {
      setAuthState(prev => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Registration failed'
      }));
      throw error;
    }
  };

  // 登出函数
  const logout = async () => {
    setAuthState(prev => ({ ...prev, isLoading: true }));
    
    try {
      await authService.logout();
      setAuthState({
        user: null,
        isAuthenticated: false,
        isLoading: false,
        error: null
      });
    } catch (error) {
      setAuthState(prev => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Logout failed'
      }));
      throw error;
    }
  };

  return {
    ...authState,
    login,
    register,
    logout
  };
} 