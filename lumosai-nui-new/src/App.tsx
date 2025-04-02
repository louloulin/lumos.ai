import React, { useEffect, useState } from 'react';
import { useAuth } from './services/auth';
import { AuthPage } from './components/auth/AuthPage';
import { Navigate, useNavigate } from 'react-router-dom';
import './App.css';

function App() {
  const { isAuthenticated, isLoading } = useAuth();
  const [initialized, setInitialized] = useState(false);
  const navigate = useNavigate();

  // 确保认证状态已经初始化
  useEffect(() => {
    if (!isLoading) {
      setInitialized(true);
      
      // 如果用户已认证，重定向到dashboard
      if (isAuthenticated) {
        navigate('/dashboard');
      } else {
        navigate('/auth');
      }
    }
  }, [isLoading, isAuthenticated, navigate]);

  // 显示加载状态
  if (isLoading || !initialized) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-background">
        <div className="animate-spin h-10 w-10 border-4 border-primary border-t-transparent rounded-full"></div>
      </div>
    );
  }

  // 渲染空元素，实际导航由useEffect中的navigate完成
  return null;
}

export default App;
