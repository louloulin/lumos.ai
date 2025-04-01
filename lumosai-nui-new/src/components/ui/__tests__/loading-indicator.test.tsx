import { render, screen } from '@testing-library/react'
import { expect, test, describe } from 'vitest'
import { LoadingIndicator } from '../loading-indicator'

describe('LoadingIndicator', () => {
  test('renders loading indicator with default props', () => {
    render(<LoadingIndicator />)
    
    const indicator = screen.getByRole('status', { name: 'Loading' })
    expect(indicator).toBeInTheDocument()
    // Check default medium size
    expect(indicator.classList.contains('h-6')).toBeTruthy()
    expect(indicator.classList.contains('w-6')).toBeTruthy()
    // Check default variant
    expect(indicator.classList.contains('text-foreground')).toBeTruthy()
  })
  
  test('renders loading indicator with small size', () => {
    render(<LoadingIndicator size="small" />)
    
    const indicator = screen.getByRole('status')
    expect(indicator.classList.contains('h-4')).toBeTruthy()
    expect(indicator.classList.contains('w-4')).toBeTruthy()
  })
  
  test('renders loading indicator with large size', () => {
    render(<LoadingIndicator size="large" />)
    
    const indicator = screen.getByRole('status')
    expect(indicator.classList.contains('h-8')).toBeTruthy()
    expect(indicator.classList.contains('w-8')).toBeTruthy()
  })
  
  test('renders loading indicator with primary variant', () => {
    render(<LoadingIndicator variant="primary" />)
    
    const indicator = screen.getByRole('status')
    expect(indicator.classList.contains('text-primary')).toBeTruthy()
  })
  
  test('applies custom className', () => {
    render(<LoadingIndicator className="test-class" />)
    
    const indicator = screen.getByRole('status')
    expect(indicator.classList.contains('test-class')).toBeTruthy()
  })
}) 