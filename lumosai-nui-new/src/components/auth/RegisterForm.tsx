import React, { useState } from 'react';
import { useAuth, RegisterData } from '../../services/auth';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '../ui/card';
import { AlertCircle, Loader2, CheckCircle } from 'lucide-react';
import { Alert, AlertDescription } from '../ui/alert';

interface RegisterFormProps {
  onSuccess?: () => void;
  onLoginClick: () => void;
}

export function RegisterForm({ onSuccess, onLoginClick }: RegisterFormProps) {
  const { register, isLoading, error } = useAuth();
  const [registrationData, setRegistrationData] = useState<RegisterData>({
    username: '',
    email: '',
    password: ''
  });
  const [confirmPassword, setConfirmPassword] = useState('');
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    
    if (name === 'confirmPassword') {
      setConfirmPassword(value);
      
      // 验证密码是否匹配
      if (value && value !== registrationData.password) {
        setPasswordError('密码不匹配');
      } else {
        setPasswordError(null);
      }
    } else {
      setRegistrationData(prev => ({ ...prev, [name]: value }));
      
      // 如果正在输入密码并且已经输入了确认密码，检查匹配
      if (name === 'password' && confirmPassword) {
        if (value !== confirmPassword) {
          setPasswordError('密码不匹配');
        } else {
          setPasswordError(null);
        }
      }
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    // 确保密码匹配
    if (registrationData.password !== confirmPassword) {
      setPasswordError('密码不匹配');
      return;
    }
    
    try {
      await register(registrationData);
      setSuccess(true);
      if (onSuccess) onSuccess();
    } catch (error) {
      // 错误已在useAuth hook中处理
      console.error('Registration failed:', error);
    }
  };

  if (success) {
    return (
      <Card className="w-full max-w-md mx-auto">
        <CardHeader>
          <CardTitle className="flex items-center">
            <CheckCircle className="mr-2 h-5 w-5 text-green-500" />
            注册成功
          </CardTitle>
          <CardDescription>
            您的账户已创建成功，现在可以开始使用LumosAI平台了。
          </CardDescription>
        </CardHeader>
        <CardContent className="text-center">
          <Button onClick={onSuccess} className="w-full mt-4">
            开始使用
          </Button>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="w-full max-w-md mx-auto">
      <CardHeader>
        <CardTitle>创建LumosAI账户</CardTitle>
        <CardDescription>
          注册一个新账户开始体验LumosAI平台
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          
          <div className="space-y-2">
            <Label htmlFor="username">用户名</Label>
            <Input
              id="username"
              name="username"
              required
              placeholder="请输入用户名"
              value={registrationData.username}
              onChange={handleChange}
              disabled={isLoading}
            />
          </div>
          
          <div className="space-y-2">
            <Label htmlFor="email">电子邮箱</Label>
            <Input
              id="email"
              name="email"
              type="email"
              autoComplete="email"
              required
              placeholder="your@email.com"
              value={registrationData.email}
              onChange={handleChange}
              disabled={isLoading}
            />
          </div>
          
          <div className="space-y-2">
            <Label htmlFor="password">密码</Label>
            <Input
              id="password"
              name="password"
              type="password"
              autoComplete="new-password"
              required
              placeholder="设置密码"
              value={registrationData.password}
              onChange={handleChange}
              disabled={isLoading}
            />
          </div>
          
          <div className="space-y-2">
            <Label htmlFor="confirmPassword">确认密码</Label>
            <Input
              id="confirmPassword"
              name="confirmPassword"
              type="password"
              autoComplete="new-password"
              required
              placeholder="再次输入密码"
              value={confirmPassword}
              onChange={handleChange}
              disabled={isLoading}
              className={passwordError ? 'border-red-500' : ''}
            />
            {passwordError && (
              <p className="text-sm text-red-500 mt-1">{passwordError}</p>
            )}
          </div>
          
          <Button 
            type="submit" 
            className="w-full" 
            disabled={isLoading || !!passwordError}
          >
            {isLoading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                注册中...
              </>
            ) : (
              '创建账户'
            )}
          </Button>
        </form>
      </CardContent>
      <CardFooter>
        <div className="text-sm text-center w-full">
          <span className="text-muted-foreground">已有账号？</span>{' '}
          <Button variant="link" className="px-0" onClick={onLoginClick}>
            登录
          </Button>
        </div>
      </CardFooter>
    </Card>
  );
} 