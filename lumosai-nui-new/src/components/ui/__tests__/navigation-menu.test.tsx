import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test, describe } from 'vitest'
import { 
  NavigationMenu,
  NavigationMenuList,
  NavigationMenuTrigger,
  NavigationMenuContent,
  NavigationMenuLink
} from '../navigation-menu'

describe('NavigationMenu', () => {
  test('renders navigation menu with trigger and content', async () => {
    const user = userEvent.setup()
    
    render(
      <NavigationMenu>
        <NavigationMenuList>
          <NavigationMenuTrigger>Getting started</NavigationMenuTrigger>
          <NavigationMenuContent>
            <div className="p-4">
              <NavigationMenuLink asChild>
                <a href="/docs">Documentation</a>
              </NavigationMenuLink>
            </div>
          </NavigationMenuContent>
        </NavigationMenuList>
      </NavigationMenu>
    )
    
    // Check trigger is rendered
    const trigger = screen.getByText('Getting started')
    expect(trigger).toBeInTheDocument()
    
    // Click on trigger to open content
    await user.click(trigger)
    
    // Check content is visible
    const link = screen.getByText('Documentation')
    expect(link).toBeInTheDocument()
    expect(link).toHaveAttribute('href', '/docs')
  })
  
  test('navigationMenuTriggerStyle applies correct styling', () => {
    render(
      <NavigationMenu>
        <NavigationMenuList>
          <NavigationMenuTrigger>Menu Item</NavigationMenuTrigger>
        </NavigationMenuList>
      </NavigationMenu>
    )
    
    const trigger = screen.getByText('Menu Item')
    expect(trigger).toHaveClass('group')
    // Check for specific class that should be applied by navigationMenuTriggerStyle
    const parent = trigger.closest('button')
    expect(parent).toHaveClass('inline-flex')
  })
}) 