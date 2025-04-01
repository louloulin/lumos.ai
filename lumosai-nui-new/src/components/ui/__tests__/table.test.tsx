import { render, screen } from '@testing-library/react'
import { expect, test, describe } from 'vitest'
import { 
  Table, 
  TableHeader, 
  TableBody, 
  TableFooter, 
  TableHead, 
  TableRow, 
  TableCell, 
  TableCaption 
} from '../table'

describe('Table', () => {
  test('renders table with all components', () => {
    render(
      <Table>
        <TableCaption>Table Caption</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead>Header 1</TableHead>
            <TableHead>Header 2</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>Cell 1</TableCell>
            <TableCell>Cell 2</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>Cell 3</TableCell>
            <TableCell>Cell 4</TableCell>
          </TableRow>
        </TableBody>
        <TableFooter>
          <TableRow>
            <TableCell>Footer 1</TableCell>
            <TableCell>Footer 2</TableCell>
          </TableRow>
        </TableFooter>
      </Table>
    )
    
    // Check table structure
    expect(screen.getByText('Table Caption')).toBeInTheDocument()
    expect(screen.getByText('Header 1')).toBeInTheDocument()
    expect(screen.getByText('Header 2')).toBeInTheDocument()
    expect(screen.getByText('Cell 1')).toBeInTheDocument()
    expect(screen.getByText('Cell 2')).toBeInTheDocument()
    expect(screen.getByText('Cell 3')).toBeInTheDocument()
    expect(screen.getByText('Cell 4')).toBeInTheDocument()
    expect(screen.getByText('Footer 1')).toBeInTheDocument()
    expect(screen.getByText('Footer 2')).toBeInTheDocument()
  })
  
  test('applies custom className to table components', () => {
    render(
      <Table className="test-table-class">
        <TableHeader className="test-header-class">
          <TableRow className="test-row-class">
            <TableHead className="test-head-class">Header</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody className="test-body-class">
          <TableRow>
            <TableCell className="test-cell-class">Cell</TableCell>
          </TableRow>
        </TableBody>
        <TableFooter className="test-footer-class">
          <TableRow>
            <TableCell>Footer</TableCell>
          </TableRow>
        </TableFooter>
        <TableCaption className="test-caption-class">Caption</TableCaption>
      </Table>
    )
    
    // Check custom classes are applied
    expect(screen.getByRole('table')).toHaveClass('test-table-class')
    
    const headerElement = screen.getByText('Header').closest('thead')
    expect(headerElement).toHaveClass('test-header-class')
    
    const rowElement = screen.getByText('Header').closest('tr')
    expect(rowElement).toHaveClass('test-row-class')
    
    const headElement = screen.getByText('Header')
    expect(headElement).toHaveClass('test-head-class')
    
    const bodyElement = screen.getByText('Cell').closest('tbody')
    expect(bodyElement).toHaveClass('test-body-class')
    
    const cellElement = screen.getByText('Cell')
    expect(cellElement).toHaveClass('test-cell-class')
    
    const footerElement = screen.getByText('Footer').closest('tfoot')
    expect(footerElement).toHaveClass('test-footer-class')
    
    const captionElement = screen.getByText('Caption')
    expect(captionElement).toHaveClass('test-caption-class')
  })
}) 