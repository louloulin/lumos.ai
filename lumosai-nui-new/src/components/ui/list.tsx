import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "../../lib/utils"

// List container
const listVariants = cva(
  "text-sm",
  {
    variants: {
      variant: {
        default: "",
        separator: "[&>li]:border-b [&>li:last-child]:border-0",
        spaced: "[&>li]:mb-2 [&>li:last-child]:mb-0",
      },
      size: {
        default: "",
        sm: "text-xs",
        lg: "text-base",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

export interface ListProps
  extends React.HTMLAttributes<HTMLElement>,
    VariantProps<typeof listVariants> {
  ordered?: boolean
}

const List = React.forwardRef<HTMLElement, ListProps>(
  ({ className, variant, size, ordered = false, ...props }, ref) => {
    const Comp = ordered ? "ol" : "ul"
    return React.createElement(
      Comp,
      {
        ref,
        className: cn(listVariants({ variant, size }), className),
        ...props,
      }
    )
  }
)
List.displayName = "List"

// List item
const listItemVariants = cva(
  "relative",
  {
    variants: {
      variant: {
        default: "",
        hover: "transition-colors hover:bg-muted/50 hover:text-accent-foreground rounded-sm",
        interactive: "cursor-pointer transition-colors hover:bg-muted/50 hover:text-accent-foreground rounded-sm",
      },
      padding: {
        default: "",
        sm: "py-1 px-2",
        md: "py-2 px-3",
        lg: "py-3 px-4",
      }
    },
    defaultVariants: {
      variant: "default",
      padding: "default",
    },
  }
)

export interface ListItemProps
  extends React.HTMLAttributes<HTMLLIElement>,
    VariantProps<typeof listItemVariants> {}

const ListItem = React.forwardRef<HTMLLIElement, ListItemProps>(
  ({ className, variant, padding, ...props }, ref) => {
    return (
      <li
        ref={ref}
        className={cn(listItemVariants({ variant, padding }), className)}
        {...props}
      />
    )
  }
)
ListItem.displayName = "ListItem"

// List icon - for adding icons to list items
const ListItemIcon = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div 
    ref={ref} 
    className={cn("mr-2 inline-flex h-4 w-4 items-center justify-center", className)} 
    {...props} 
  />
))
ListItemIcon.displayName = "ListItemIcon"

// List content - for wrapping content in a list item
const ListItemContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn("flex-1", className)} {...props} />
))
ListItemContent.displayName = "ListItemContent"

export {
  List,
  ListItem,
  ListItemIcon,
  ListItemContent,
  listVariants,
  listItemVariants,
} 