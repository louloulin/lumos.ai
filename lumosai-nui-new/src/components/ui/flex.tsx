import * as React from "react"

import { cn } from "../../lib/utils"

export interface FlexProps extends React.HTMLAttributes<HTMLDivElement> {
  inline?: boolean
  direction?: "row" | "row-reverse" | "column" | "column-reverse"
  wrap?: "nowrap" | "wrap" | "wrap-reverse"
  justify?: "start" | "end" | "center" | "between" | "around" | "evenly"
  align?: "start" | "end" | "center" | "baseline" | "stretch"
  gap?: "none" | "xs" | "sm" | "md" | "lg" | "xl"
}

const Flex = React.forwardRef<HTMLDivElement, FlexProps>(
  ({ 
    className, 
    inline = false,
    direction = "row",
    wrap = "nowrap",
    justify = "start",
    align = "start",
    gap = "none",
    ...props 
  }, ref) => {
    const directionClasses = {
      "row": "flex-row",
      "row-reverse": "flex-row-reverse",
      "column": "flex-col",
      "column-reverse": "flex-col-reverse",
    }
    
    const wrapClasses = {
      "nowrap": "flex-nowrap",
      "wrap": "flex-wrap",
      "wrap-reverse": "flex-wrap-reverse",
    }
    
    const justifyClasses = {
      "start": "justify-start",
      "end": "justify-end",
      "center": "justify-center",
      "between": "justify-between",
      "around": "justify-around",
      "evenly": "justify-evenly",
    }
    
    const alignClasses = {
      "start": "items-start",
      "end": "items-end",
      "center": "items-center",
      "baseline": "items-baseline",
      "stretch": "items-stretch",
    }
    
    const gapClasses = {
      "none": "gap-0",
      "xs": "gap-1",
      "sm": "gap-2",
      "md": "gap-4",
      "lg": "gap-6",
      "xl": "gap-8",
    }

    return (
      <div
        ref={ref}
        className={cn(
          inline ? "inline-flex" : "flex",
          directionClasses[direction],
          wrapClasses[wrap],
          justifyClasses[justify],
          alignClasses[align],
          gapClasses[gap],
          className
        )}
        {...props}
      />
    )
  }
)
Flex.displayName = "Flex"

export { Flex } 