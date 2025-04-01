import { render, screen, fireEvent } from '@testing-library/react'
import { expect, test, describe } from 'vitest'
import { 
  Tree, 
  TreeItem, 
  TreeItemContent, 
  TreeItemToggle, 
  TreeGroup 
} from '../tree'

describe('Tree', () => {
  test('renders basic tree correctly', () => {
    render(
      <Tree data-testid="test-tree">
        <TreeItem>
          <TreeItemContent>Root Item</TreeItemContent>
        </TreeItem>
      </Tree>
    )
    
    expect(screen.getByTestId('test-tree')).toBeInTheDocument()
    expect(screen.getByRole('tree')).toBeInTheDocument()
    expect(screen.getByRole('treeitem')).toBeInTheDocument()
    expect(screen.getByText('Root Item')).toBeInTheDocument()
  })
  
  test('renders tree with nested items and handles expansion', () => {
    render(
      <Tree>
        <TreeItem>
          <TreeItemToggle data-testid="toggle-button" />
          <TreeItemContent>Parent</TreeItemContent>
          <TreeGroup>
            <TreeItem>
              <TreeItemContent>Child 1</TreeItemContent>
            </TreeItem>
            <TreeItem>
              <TreeItemContent>Child 2</TreeItemContent>
            </TreeItem>
          </TreeGroup>
        </TreeItem>
      </Tree>
    )
    
    // Group should be hidden initially
    expect(screen.queryByText('Child 1')).not.toBeInTheDocument()
    expect(screen.queryByText('Child 2')).not.toBeInTheDocument()
    
    // Click to expand
    fireEvent.click(screen.getByTestId('toggle-button'))
    
    // Group should be visible
    expect(screen.getByText('Child 1')).toBeInTheDocument()
    expect(screen.getByText('Child 2')).toBeInTheDocument()
    
    // Check aria-expanded attribute
    expect(screen.getByRole('treeitem')).toHaveAttribute('aria-expanded', 'true')
    
    // Click to collapse
    fireEvent.click(screen.getByTestId('toggle-button'))
    
    // Group should be hidden again
    expect(screen.queryByText('Child 1')).not.toBeInTheDocument()
    expect(screen.queryByText('Child 2')).not.toBeInTheDocument()
    expect(screen.getByRole('treeitem')).toHaveAttribute('aria-expanded', 'false')
  })
  
  test('renders tree with automatically generated toggle buttons', () => {
    render(
      <Tree>
        <TreeItem data-testid="parent-item">
          <TreeItemContent>Parent with auto-toggle</TreeItemContent>
          <TreeGroup>
            <TreeItem>
              <TreeItemContent>Child</TreeItemContent>
            </TreeItem>
          </TreeGroup>
        </TreeItem>
      </Tree>
    )
    
    // Find automatically generated toggle
    const toggleButton = screen.getByRole('button')
    expect(toggleButton).toBeInTheDocument()
    
    // Child should be hidden initially
    expect(screen.queryByText('Child')).not.toBeInTheDocument()
    
    // Click to expand
    fireEvent.click(toggleButton)
    
    // Child should be visible
    expect(screen.getByText('Child')).toBeInTheDocument()
  })
  
  test('applies different variants and sizes to Tree', () => {
    render(
      <Tree variant="bordered" size="lg" data-testid="custom-tree">
        <TreeItem>
          <TreeItemContent>Item</TreeItemContent>
        </TreeItem>
      </Tree>
    )
    
    const tree = screen.getByTestId('custom-tree')
    expect(tree).toHaveClass('border')
    expect(tree).toHaveClass('rounded-md')
    expect(tree).toHaveClass('text-base')
  })
  
  test('respects disabled state on TreeItem', () => {
    render(
      <Tree>
        <TreeItem disabled data-testid="disabled-item">
          <TreeItemToggle data-testid="disabled-toggle" />
          <TreeItemContent>Disabled Item</TreeItemContent>
          <TreeGroup>
            <TreeItem>
              <TreeItemContent>Child</TreeItemContent>
            </TreeItem>
          </TreeGroup>
        </TreeItem>
      </Tree>
    )
    
    const item = screen.getByTestId('disabled-item')
    expect(item).toHaveAttribute('data-disabled', 'true')
    
    // Toggle should be disabled
    const toggle = screen.getByTestId('disabled-toggle')
    expect(toggle).toBeDisabled()
    
    // Clicking shouldn't expand
    fireEvent.click(toggle)
    expect(screen.queryByText('Child')).not.toBeInTheDocument()
  })
  
  test('applies spacing variants to TreeItem', () => {
    render(
      <Tree>
        <TreeItem spacing="compact" data-testid="compact-item">
          <TreeItemContent>Compact Item</TreeItemContent>
        </TreeItem>
        <TreeItem spacing="loose" data-testid="loose-item">
          <TreeItemContent>Loose Item</TreeItemContent>
        </TreeItem>
      </Tree>
    )
    
    expect(screen.getByTestId('compact-item')).toHaveClass('py-0.5')
    expect(screen.getByTestId('loose-item')).toHaveClass('py-2')
  })
  
  test('allows custom className on all components', () => {
    render(
      <Tree className="custom-tree">
        <TreeItem className="custom-item">
          <TreeItemToggle className="custom-toggle" />
          <TreeItemContent className="custom-content">Item</TreeItemContent>
          <TreeGroup className="custom-group">
            <TreeItem>
              <TreeItemContent>Child</TreeItemContent>
            </TreeItem>
          </TreeGroup>
        </TreeItem>
      </Tree>
    )
    
    // Expand to make group visible
    fireEvent.click(screen.getByRole('button'))
    
    expect(screen.getByRole('tree')).toHaveClass('custom-tree')
    expect(screen.getByRole('treeitem')).toHaveClass('custom-item')
    expect(screen.getByRole('button')).toHaveClass('custom-toggle')
    expect(screen.getByText('Item')).toHaveClass('custom-content')
    expect(screen.getByRole('group')).toHaveClass('custom-group')
  })
  
  test('maintains expanded state when provided explicitly', () => {
    render(
      <Tree>
        <TreeItem expanded={true} data-testid="expanded-item">
          <TreeItemContent>Expanded Item</TreeItemContent>
          <TreeGroup>
            <TreeItem>
              <TreeItemContent>Child</TreeItemContent>
            </TreeItem>
          </TreeGroup>
        </TreeItem>
      </Tree>
    )
    
    // Child should be visible immediately
    expect(screen.getByText('Child')).toBeInTheDocument()
    expect(screen.getByTestId('expanded-item')).toHaveAttribute('aria-expanded', 'true')
  })
}) 