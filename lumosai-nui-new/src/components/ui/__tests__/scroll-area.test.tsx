import { render, screen } from '@testing-library/react'
import { expect, test, describe } from 'vitest'
import { ScrollArea, ScrollBar } from '../scroll-area'

describe('ScrollArea', () => {
  test('renders scroll area with content', () => {
    render(
      <ScrollArea className="h-[200px] w-[350px]">
        <div style={{ height: '400px', width: '500px' }}>
          <p>Test content that should overflow and create scrollbars</p>
        </div>
      </ScrollArea>
    )
    
    const content = screen.getByText('Test content that should overflow and create scrollbars')
    expect(content).toBeInTheDocument()
    
    // Check if scroll viewport exists
    const viewport = content.closest('[data-radix-scroll-area-viewport]')
    expect(viewport).toBeInTheDocument()
  })
  
  test('renders scroll area with custom className', () => {
    render(
      <ScrollArea className="test-class">
        <div>Content</div>
      </ScrollArea>
    )
    
    const scrollArea = screen.getByText('Content').closest('[data-radix-scroll-area-root]')
    expect(scrollArea).toHaveClass('test-class')
  })
  
  test('renders ScrollBar with orientation', () => {
    render(
      <div data-testid="container">
        <ScrollBar orientation="horizontal" />
      </div>
    )
    
    const container = screen.getByTestId('container')
    const scrollbar = container.querySelector('[data-orientation="horizontal"]')
    expect(scrollbar).toBeInTheDocument()
  })
}) 