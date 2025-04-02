import React, { useState } from 'react';
import { useAuth, LoginCredentials } from '../../services/auth';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '../ui/card';
import { AlertCircle, Loader2 } from 'lucide-react';
import { Alert, AlertDescription } from '../ui/alert';

interface LoginFormProps {
  onSuccess?: () => void;
  onRegisterClick: () => void;
}

export function LoginForm({ onSuccess, onRegisterClick }: LoginFormProps) {
  const { login, isLoading, error } = useAuth();
  const [credentials, setCredentials] = useState<LoginCredentials>({
    email: '',
    password: ''
  });
  const [localError, setLocalError] = useState<string | null>(null);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setCredentials(prev => ({ ...prev, [name]: value }));
    // 清除之前的错误
    setLocalError(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLocalError(null);
    
    try {
      await login(credentials);
      console.log('Login successful, triggering onSuccess callback');
      // 确保在成功登录后调用onSuccess回调
      if (onSuccess) {
        // 使用setTimeout确保状态更新后再执行回调
        setTimeout(() => {
          onSuccess();
        }, 100);
      }
    } catch (error) {
      console.error('Login failed:', error);
      // 设置本地错误
      setLocalError(error instanceof Error ? error.message : '登录失败，请重试');
    }
  };

  // 显示来自useAuth的错误或本地错误
  const displayError = error || localError;

  return (
    <Card className="w-full max-w-md mx-auto">
      <CardHeader>
        <CardTitle>登录到LumosAI</CardTitle>
        <CardDescription>
          输入您的凭据以访问您的账户
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          {displayError && (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{displayError}</AlertDescription>
            </Alert>
          )}
          
          <div className="space-y-2">
            <Label htmlFor="email">电子邮箱</Label>
            <Input
              id="email"
              name="email"
              type="email"
              autoComplete="email"
              required
              placeholder="your@email.com"
              value={credentials.email}
              onChange={handleChange}
              disabled={isLoading}
            />
          </div>
          
          <div className="space-y-2">
            <div className="flex justify-between items-center">
              <Label htmlFor="password">密码</Label>
              <Button variant="link" className="px-0 h-auto" type="button">
                忘记密码？
              </Button>
            </div>
            <Input
              id="password"
              name="password"
              type="password"
              autoComplete="current-password"
              required
              placeholder="输入密码"
              value={credentials.password}
              onChange={handleChange}
              disabled={isLoading}
            />
          </div>
          
          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                正在登录...
              </>
            ) : (
              '登录'
            )}
          </Button>
        </form>
      </CardContent>
      <CardFooter className="flex flex-col">
        <div className="text-sm text-center">
          <span className="text-muted-foreground">还没有账号？</span>{' '}
          <Button variant="link" className="px-0" onClick={onRegisterClick}>
            注册新账号
          </Button>
        </div>
        <div className="text-xs text-center text-muted-foreground mt-2">
          使用演示账号 demo@lumosai.com / password 登录
        </div>
      </CardFooter>
    </Card>
  );
} 