import React, { useState } from 'react';
import { useAuth } from '../../services/auth';
import { LoginForm } from './LoginForm';
import { RegisterForm } from './RegisterForm';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';

interface AuthPageProps {
  onAuthSuccess?: () => void;
}

export function AuthPage({ onAuthSuccess }: AuthPageProps) {
  const { isAuthenticated } = useAuth();
  const [activeTab, setActiveTab] = useState<'login' | 'register'>('login');

  // 如果用户已认证，则不需要显示登录/注册页面
  if (isAuthenticated) {
    return null;
  }

  const handleTabChange = (value: string) => {
    setActiveTab(value as 'login' | 'register');
  };

  const handleRegisterClick = () => {
    setActiveTab('register');
  };

  const handleLoginClick = () => {
    setActiveTab('login');
  };

  return (
    <div className="flex min-h-screen flex-col items-center justify-center py-12 px-4 sm:px-6 lg:px-8 bg-background">
      <div className="w-full max-w-md space-y-8">
        <div className="text-center">
          <h1 className="text-3xl font-bold tracking-tight">LumosAI平台</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            AI开发和部署的一站式平台
          </p>
        </div>
        
        <Tabs value={activeTab} onValueChange={handleTabChange} className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="login">登录</TabsTrigger>
            <TabsTrigger value="register">注册</TabsTrigger>
          </TabsList>
          
          <TabsContent value="login" className="mt-6">
            <LoginForm 
              onSuccess={onAuthSuccess} 
              onRegisterClick={handleRegisterClick} 
            />
          </TabsContent>
          
          <TabsContent value="register" className="mt-6">
            <RegisterForm 
              onSuccess={onAuthSuccess} 
              onLoginClick={handleLoginClick} 
            />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
} 