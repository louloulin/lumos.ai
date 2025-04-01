import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import {
  AreaChart,
  Area,
  BarChart,
  Bar,
  LineChart,
  Line,
  PieChart,
  Pie,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts"

import { cn } from "../../lib/utils"

// Chart container
const chartContainerVariants = cva(
  "w-full overflow-hidden",
  {
    variants: {
      variant: {
        default: "bg-background",
        card: "bg-card text-card-foreground shadow-sm rounded-md p-4",
        bordered: "border rounded-md p-4",
      },
      size: {
        sm: "h-[200px]",
        default: "h-[300px]",
        lg: "h-[400px]",
        xl: "h-[500px]",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

export interface ChartContainerProps 
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof chartContainerVariants> {}

const ChartContainer = React.forwardRef<HTMLDivElement, ChartContainerProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(chartContainerVariants({ variant, size }), className)}
        {...props}
      />
    )
  }
)
ChartContainer.displayName = "ChartContainer"

// Line Chart
export interface LineChartProps {
  data: any[]
  lines: Array<{
    dataKey: string
    stroke?: string
    fill?: string
    name?: string
  }>
  xAxisDataKey: string
  showGrid?: boolean
  showTooltip?: boolean
  showLegend?: boolean
  containerProps?: React.ComponentProps<typeof ChartContainer>
}

const LineChartComponent = ({
  data,
  lines,
  xAxisDataKey,
  showGrid = true,
  showTooltip = true,
  showLegend = true,
  containerProps,
}: LineChartProps) => {
  return (
    <ChartContainer {...containerProps}>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart
          data={data}
          margin={{
            top: 10,
            right: 30,
            left: 0,
            bottom: 0,
          }}
        >
          {showGrid && <CartesianGrid strokeDasharray="3 3" />}
          <XAxis dataKey={xAxisDataKey} />
          <YAxis />
          {showTooltip && <Tooltip />}
          {showLegend && <Legend />}
          {lines.map((line, index) => (
            <Line
              key={index}
              type="monotone"
              dataKey={line.dataKey}
              stroke={line.stroke || "#8884d8"}
              name={line.name || line.dataKey}
              activeDot={{ r: 8 }}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </ChartContainer>
  )
}

// Bar Chart
export interface BarChartProps {
  data: any[]
  bars: Array<{
    dataKey: string
    fill?: string
    name?: string
  }>
  xAxisDataKey: string
  showGrid?: boolean
  showTooltip?: boolean
  showLegend?: boolean
  containerProps?: React.ComponentProps<typeof ChartContainer>
}

const BarChartComponent = ({
  data,
  bars,
  xAxisDataKey,
  showGrid = true,
  showTooltip = true,
  showLegend = true,
  containerProps,
}: BarChartProps) => {
  return (
    <ChartContainer {...containerProps}>
      <ResponsiveContainer width="100%" height="100%">
        <BarChart
          data={data}
          margin={{
            top: 10,
            right: 30,
            left: 0,
            bottom: 0,
          }}
        >
          {showGrid && <CartesianGrid strokeDasharray="3 3" />}
          <XAxis dataKey={xAxisDataKey} />
          <YAxis />
          {showTooltip && <Tooltip />}
          {showLegend && <Legend />}
          {bars.map((bar, index) => (
            <Bar
              key={index}
              dataKey={bar.dataKey}
              fill={bar.fill || "#8884d8"}
              name={bar.name || bar.dataKey}
            />
          ))}
        </BarChart>
      </ResponsiveContainer>
    </ChartContainer>
  )
}

// Area Chart
export interface AreaChartProps {
  data: any[]
  areas: Array<{
    dataKey: string
    stroke?: string
    fill?: string
    name?: string
    stackId?: string | number
  }>
  xAxisDataKey: string
  showGrid?: boolean
  showTooltip?: boolean
  showLegend?: boolean
  containerProps?: React.ComponentProps<typeof ChartContainer>
}

const AreaChartComponent = ({
  data,
  areas,
  xAxisDataKey,
  showGrid = true,
  showTooltip = true,
  showLegend = true,
  containerProps,
}: AreaChartProps) => {
  return (
    <ChartContainer {...containerProps}>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart
          data={data}
          margin={{
            top: 10,
            right: 30,
            left: 0,
            bottom: 0,
          }}
        >
          {showGrid && <CartesianGrid strokeDasharray="3 3" />}
          <XAxis dataKey={xAxisDataKey} />
          <YAxis />
          {showTooltip && <Tooltip />}
          {showLegend && <Legend />}
          {areas.map((area, index) => (
            <Area
              key={index}
              type="monotone"
              dataKey={area.dataKey}
              stroke={area.stroke || "#8884d8"}
              fill={area.fill || "#8884d8"}
              name={area.name || area.dataKey}
              strokeWidth={2}
              stackId={area.stackId}
            />
          ))}
        </AreaChart>
      </ResponsiveContainer>
    </ChartContainer>
  )
}

// Pie Chart
export interface PieChartProps {
  data: any[]
  nameKey: string
  dataKey: string
  showTooltip?: boolean
  showLegend?: boolean
  containerProps?: React.ComponentProps<typeof ChartContainer>
}

const PieChartComponent = ({
  data,
  nameKey,
  dataKey,
  showTooltip = true,
  showLegend = true,
  containerProps,
}: PieChartProps) => {
  return (
    <ChartContainer {...containerProps}>
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          {showTooltip && <Tooltip />}
          {showLegend && <Legend />}
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            labelLine={false}
            outerRadius={80}
            fill="#8884d8"
            dataKey={dataKey}
            nameKey={nameKey}
            label
          />
        </PieChart>
      </ResponsiveContainer>
    </ChartContainer>
  )
}

export {
  ChartContainer,
  LineChartComponent as LineChart,
  BarChartComponent as BarChart,
  AreaChartComponent as AreaChart,
  PieChartComponent as PieChart,
} 