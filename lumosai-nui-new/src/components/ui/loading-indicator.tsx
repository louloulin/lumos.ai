import * as React from "react"
import { cn } from "../../lib/utils"

export interface LoadingIndicatorProps extends React.HTMLAttributes<HTMLDivElement> {
  size?: "small" | "medium" | "large"
  variant?: "default" | "primary" | "secondary"
}

export function LoadingIndicator({
  className,
  size = "medium",
  variant = "default",
  ...props
}: LoadingIndicatorProps) {
  return (
    <div
      className={cn(
        "inline-flex items-center justify-center animate-spin",
        {
          "h-4 w-4": size === "small",
          "h-6 w-6": size === "medium",
          "h-8 w-8": size === "large",
          "text-foreground": variant === "default",
          "text-primary": variant === "primary",
          "text-secondary": variant === "secondary",
        },
        className
      )}
      role="status"
      aria-label="Loading"
      {...props}
    >
      <svg
        className="h-full w-full"
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
      >
        <circle
          className="opacity-25"
          cx="12"
          cy="12"
          r="10"
          stroke="currentColor"
          strokeWidth="4"
        ></circle>
        <path
          className="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
        ></path>
      </svg>
    </div>
  )
} 