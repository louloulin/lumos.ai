import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test, describe } from 'vitest'
import { 
  Tooltip, 
  TooltipContent, 
  TooltipProvider, 
  TooltipTrigger 
} from '../tooltip'

describe('Tooltip', () => {
  test('renders tooltip trigger and shows content on hover', async () => {
    const user = userEvent.setup()
    
    render(
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger>Hover me</TooltipTrigger>
          <TooltipContent>Tooltip content</TooltipContent>
        </Tooltip>
      </TooltipProvider>
    )
    
    // Check trigger is rendered
    const trigger = screen.getByText('Hover me')
    expect(trigger).toBeInTheDocument()
    
    // Hover over trigger
    await user.hover(trigger)
    
    // Check tooltip content shows
    const content = screen.getByText('Tooltip content')
    expect(content).toBeInTheDocument()
    
    // Unhover trigger
    await user.unhover(trigger)
  })
  
  test('tooltip can be customized with className', async () => {
    const user = userEvent.setup()
    
    render(
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger>Hover me</TooltipTrigger>
          <TooltipContent className="custom-class">Tooltip content</TooltipContent>
        </Tooltip>
      </TooltipProvider>
    )
    
    const trigger = screen.getByText('Hover me')
    await user.hover(trigger)
    
    const content = screen.getByText('Tooltip content')
    expect(content).toHaveClass('custom-class')
  })
}) 