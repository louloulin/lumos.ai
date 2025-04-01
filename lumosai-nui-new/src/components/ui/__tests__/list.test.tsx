import { render, screen } from '@testing-library/react'
import { expect, test, describe } from 'vitest'
import { 
  List, 
  ListItem, 
  ListItemIcon, 
  ListItemContent 
} from '../list'

describe('List', () => {
  test('renders basic list correctly', () => {
    render(
      <List data-testid="test-list">
        <ListItem>Item 1</ListItem>
        <ListItem>Item 2</ListItem>
        <ListItem>Item 3</ListItem>
      </List>
    )
    
    const list = screen.getByTestId('test-list')
    expect(list).toBeInTheDocument()
    expect(list.tagName).toBe('UL')
    
    const items = screen.getAllByRole('listitem')
    expect(items).toHaveLength(3)
    expect(items[0]).toHaveTextContent('Item 1')
    expect(items[1]).toHaveTextContent('Item 2')
    expect(items[2]).toHaveTextContent('Item 3')
  })
  
  test('renders ordered list when ordered prop is true', () => {
    render(
      <List ordered data-testid="test-list">
        <ListItem>Item 1</ListItem>
        <ListItem>Item 2</ListItem>
      </List>
    )
    
    const list = screen.getByTestId('test-list')
    expect(list.tagName).toBe('OL')
  })
  
  test('applies variant and size classes correctly', () => {
    render(
      <List variant="separator" size="lg" data-testid="test-list">
        <ListItem>Item 1</ListItem>
        <ListItem>Item 2</ListItem>
      </List>
    )
    
    const list = screen.getByTestId('test-list')
    expect(list).toHaveClass('text-base')
  })
  
  test('renders ListItem with correct variant and padding', () => {
    render(
      <List>
        <ListItem variant="hover" padding="md" data-testid="test-item">
          Test Item
        </ListItem>
      </List>
    )
    
    const item = screen.getByTestId('test-item')
    expect(item).toHaveClass('hover:bg-muted/50')
    expect(item).toHaveClass('py-2')
    expect(item).toHaveClass('px-3')
  })
  
  test('renders ListItemIcon and ListItemContent correctly', () => {
    render(
      <List>
        <ListItem>
          <ListItemIcon data-testid="test-icon">
            <svg width="12" height="12" viewBox="0 0 12 12" />
          </ListItemIcon>
          <ListItemContent data-testid="test-content">
            Content Text
          </ListItemContent>
        </ListItem>
      </List>
    )
    
    const icon = screen.getByTestId('test-icon')
    expect(icon).toBeInTheDocument()
    expect(icon).toHaveClass('mr-2')
    
    const content = screen.getByTestId('test-content')
    expect(content).toBeInTheDocument()
    expect(content).toHaveClass('flex-1')
    expect(content).toHaveTextContent('Content Text')
  })
  
  test('applies custom className to all components', () => {
    render(
      <List className="custom-list-class">
        <ListItem className="custom-item-class">
          <ListItemIcon className="custom-icon-class" data-testid="test-icon" />
          <ListItemContent className="custom-content-class" data-testid="test-content" />
        </ListItem>
      </List>
    )
    
    expect(screen.getByRole('list')).toHaveClass('custom-list-class')
    expect(screen.getByRole('listitem')).toHaveClass('custom-item-class')
    expect(screen.getByTestId('test-icon')).toHaveClass('custom-icon-class')
    expect(screen.getByTestId('test-content')).toHaveClass('custom-content-class')
  })
}) 