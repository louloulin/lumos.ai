import React, { useEffect, useState } from 'react';
import { useAuth } from './services/auth';
import { AuthPage } from './components/auth/AuthPage';
import { AppLayout } from './components/layout/AppLayout';
import './App.css';

function App() {
  const { isAuthenticated, isLoading } = useAuth();
  const [initialized, setInitialized] = useState(false);

  // 确保认证状态已经初始化
  useEffect(() => {
    if (!isLoading) {
      setInitialized(true);
    }
  }, [isLoading]);

  // 处理认证成功
  const handleAuthSuccess = () => {
    console.log('Authentication successful, refreshing app');
    // 强制组件重新渲染
    setInitialized(false);
    setTimeout(() => setInitialized(true), 100);
  };

  // 显示加载状态
  if (isLoading || !initialized) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-background">
        <div className="animate-spin h-10 w-10 border-4 border-primary border-t-transparent rounded-full"></div>
      </div>
    );
  }

  // 如果用户未认证，显示登录/注册页面
  if (!isAuthenticated) {
    return <AuthPage onAuthSuccess={handleAuthSuccess} />;
  }

  // 用户已认证，显示主应用布局
  return <AppLayout />;
}

export default App;
