import { render, screen } from '@testing-library/react'
import { expect, test, describe } from 'vitest'
import { 
  ChartContainer, 
  LineChart,
  BarChart,
  AreaChart,
  PieChart
} from '../charts'

const mockData = [
  { name: 'Jan', value1: 400, value2: 240 },
  { name: 'Feb', value1: 300, value2: 138 },
  { name: 'Mar', value1: 200, value2: 980 },
  { name: 'Apr', value1: 278, value2: 390 },
  { name: 'May', value1: 189, value2: 480 },
]

describe('ChartContainer', () => {
  test('renders with default classes', () => {
    render(
      <ChartContainer data-testid="chart-container">
        <div>Chart content</div>
      </ChartContainer>
    )
    
    const container = screen.getByTestId('chart-container')
    expect(container).toBeInTheDocument()
    expect(container).toHaveClass('w-full')
    expect(container).toHaveClass('overflow-hidden')
    expect(container).toHaveTextContent('Chart content')
  })
  
  test('applies variant styles', () => {
    render(
      <ChartContainer variant="card" data-testid="chart-container-card">
        <div>Card variant</div>
      </ChartContainer>
    )
    
    const container = screen.getByTestId('chart-container-card')
    expect(container).toHaveClass('bg-card')
    expect(container).toHaveClass('rounded-md')
  })
  
  test('applies size styles', () => {
    render(
      <ChartContainer size="lg" data-testid="chart-container-lg">
        <div>Large size</div>
      </ChartContainer>
    )
    
    const container = screen.getByTestId('chart-container-lg')
    expect(container).toHaveClass('h-[400px]')
  })
  
  test('applies custom className', () => {
    render(
      <ChartContainer className="custom-class" data-testid="chart-container-custom">
        <div>Custom class</div>
      </ChartContainer>
    )
    
    const container = screen.getByTestId('chart-container-custom')
    expect(container).toHaveClass('custom-class')
  })
})

describe('LineChart', () => {
  test('renders line chart with data', () => {
    render(
      <LineChart
        data={mockData}
        lines={[
          { dataKey: 'value1', stroke: '#ff0000' },
          { dataKey: 'value2', stroke: '#00ff00' }
        ]}
        xAxisDataKey="name"
        containerProps={{ "data-testid": "line-chart" } as React.HTMLAttributes<HTMLDivElement>}
      />
    )
    
    const container = screen.getByTestId('line-chart')
    expect(container).toBeInTheDocument()
    // Test that Recharts components are used (specific elements are difficult to test)
    // Mostly we're testing that the component renders without errors
  })
})

describe('BarChart', () => {
  test('renders bar chart with data', () => {
    render(
      <BarChart
        data={mockData}
        bars={[
          { dataKey: 'value1', fill: '#8884d8' },
          { dataKey: 'value2', fill: '#82ca9d' }
        ]}
        xAxisDataKey="name"
        containerProps={{ "data-testid": "bar-chart" } as React.HTMLAttributes<HTMLDivElement>}
      />
    )
    
    const container = screen.getByTestId('bar-chart')
    expect(container).toBeInTheDocument()
  })
})

describe('AreaChart', () => {
  test('renders area chart with data', () => {
    render(
      <AreaChart
        data={mockData}
        areas={[
          { dataKey: 'value1', fill: 'rgba(136, 132, 216, 0.7)' },
          { dataKey: 'value2', fill: 'rgba(130, 202, 157, 0.7)', stackId: 1 }
        ]}
        xAxisDataKey="name"
        containerProps={{ "data-testid": "area-chart" } as React.HTMLAttributes<HTMLDivElement>}
      />
    )
    
    const container = screen.getByTestId('area-chart')
    expect(container).toBeInTheDocument()
  })
})

describe('PieChart', () => {
  test('renders pie chart with data', () => {
    const pieData = [
      { name: 'Group A', value: 400 },
      { name: 'Group B', value: 300 },
      { name: 'Group C', value: 300 }
    ]
    
    render(
      <PieChart
        data={pieData}
        nameKey="name"
        dataKey="value"
        containerProps={{ "data-testid": "pie-chart" } as React.HTMLAttributes<HTMLDivElement>}
      />
    )
    
    const container = screen.getByTestId('pie-chart')
    expect(container).toBeInTheDocument()
  })
}) 