import { test, expect } from 'bun:test';
import { hello } from '../src/index';

test('基本功能', () => {
  expect(hello('world')).toContain('Hello, world!');
});
