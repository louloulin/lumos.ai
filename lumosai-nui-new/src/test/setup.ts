import { expect, afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom/matchers';

// 在测试环境中模拟ResizeObserver
class ResizeObserverMock {
  observe() {}
  unobserve() {}
  disconnect() {}
}

// 为全局环境添加ResizeObserver
global.ResizeObserver = ResizeObserverMock;

// 扩展vitest期望值以使用@testing-library/jest-dom的匹配器
expect.extend(matchers as any);

// 每个测试完成后清理
afterEach(() => {
  cleanup();
}); 