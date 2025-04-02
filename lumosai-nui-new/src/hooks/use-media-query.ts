import { useState, useEffect } from 'react';

/**
 * 自定义Hook: 用于媒体查询，支持响应式布局
 * @param query 媒体查询字符串，例如 '(max-width: 768px)'
 * @returns 布尔值，表示媒体查询是否匹配
 */
export function useMediaQuery(query: string): boolean {
  // 初始化状态为当前匹配状态（如果支持matchMedia）或默认值
  const getMatches = (): boolean => {
    // 确保代码在浏览器环境中运行
    if (typeof window !== 'undefined') {
      return window.matchMedia(query).matches;
    }
    return false;
  };

  const [matches, setMatches] = useState<boolean>(getMatches);

  useEffect(() => {
    // 初始化匹配状态
    setMatches(getMatches());

    // 创建媒体查询监听器
    const mediaQuery = window.matchMedia(query);
    
    // 处理媒体查询变化
    const handleChange = () => setMatches(mediaQuery.matches);
    
    // 添加监听器
    if (mediaQuery.addEventListener) {
      // 现代浏览器
      mediaQuery.addEventListener('change', handleChange);
    } else {
      // 旧浏览器支持
      mediaQuery.addListener(handleChange);
    }

    // 清理函数
    return () => {
      if (mediaQuery.removeEventListener) {
        mediaQuery.removeEventListener('change', handleChange);
      } else {
        mediaQuery.removeListener(handleChange);
      }
    };
  }, [query]);

  return matches;
} 