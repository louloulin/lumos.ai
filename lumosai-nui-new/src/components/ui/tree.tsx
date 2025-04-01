import * as React from "react"
import { ChevronDown } from "lucide-react"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "../../lib/utils"

// TreeRoot component
const treeVariants = cva(
  "relative overflow-auto",
  {
    variants: {
      variant: {
        default: "",
        bordered: "border rounded-md",
        card: "bg-card text-card-foreground shadow-sm rounded-md",
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

export interface TreeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof treeVariants> {}

const Tree = React.forwardRef<HTMLDivElement, TreeProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(treeVariants({ variant, size }), className)}
        role="tree"
        {...props}
      />
    )
  }
)
Tree.displayName = "Tree"

// TreeItem component
const treeItemVariants = cva(
  "flex flex-col gap-0.5 py-1",
  {
    variants: {
      spacing: {
        default: "",
        compact: "py-0.5",
        loose: "py-2",
      }
    },
    defaultVariants: {
      spacing: "default",
    },
  }
)

export interface TreeItemProps 
  extends React.HTMLAttributes<HTMLLIElement>,
    VariantProps<typeof treeItemVariants> {
  expanded?: boolean
  disabled?: boolean
}

const TreeItem = React.forwardRef<HTMLLIElement, TreeItemProps>(
  ({ className, spacing, expanded = false, disabled = false, children, ...props }, ref) => {
    // Split children into content and items
    const childrenArray = React.Children.toArray(children)
    const treeItemToggleIndex = childrenArray.findIndex(
      (child) => React.isValidElement(child) && child.type === TreeItemToggle
    )
    const treeItemContentIndex = childrenArray.findIndex(
      (child) => React.isValidElement(child) && child.type === TreeItemContent
    )
    const treeGroupIndex = childrenArray.findIndex(
      (child) => React.isValidElement(child) && child.type === TreeGroup
    )

    const toggle = treeItemToggleIndex !== -1 ? childrenArray[treeItemToggleIndex] : null
    const content = treeItemContentIndex !== -1 ? childrenArray[treeItemContentIndex] : null
    const group = treeGroupIndex !== -1 ? childrenArray[treeGroupIndex] : null

    const [isExpanded, setIsExpanded] = React.useState(expanded)

    const handleToggle = () => {
      if (!disabled) {
        setIsExpanded(!isExpanded)
      }
    }

    return (
      <li
        ref={ref}
        className={cn(treeItemVariants({ spacing }), className)}
        role="treeitem"
        aria-expanded={group ? isExpanded : undefined}
        data-disabled={disabled || undefined}
        {...props}
      >
        <div className="flex items-center">
          {toggle ? (
            React.cloneElement(toggle as React.ReactElement, {
              expanded: isExpanded,
              disabled,
              onClick: handleToggle,
            })
          ) : group ? (
            <TreeItemToggle 
              expanded={isExpanded} 
              disabled={disabled} 
              onClick={handleToggle} 
            />
          ) : (
            <div className="w-6" /> 
          )}
          {content}
        </div>
        {group && isExpanded ? group : null}
      </li>
    )
  }
)
TreeItem.displayName = "TreeItem"

// TreeItemToggle component
export interface TreeItemToggleProps extends React.HTMLAttributes<HTMLButtonElement> {
  expanded?: boolean
  disabled?: boolean
}

const TreeItemToggle = React.forwardRef<HTMLButtonElement, TreeItemToggleProps>(
  ({ className, expanded = false, disabled = false, ...props }, ref) => {
    return (
      <button
        ref={ref}
        type="button"
        className={cn(
          "flex h-6 w-6 shrink-0 items-center justify-center rounded-sm text-muted-foreground hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50",
          className
        )}
        tabIndex={-1}
        disabled={disabled}
        aria-hidden={true}
        {...props}
      >
        <ChevronDown
          className={cn(
            "h-4 w-4 transition-transform duration-200",
            expanded ? "rotate-0" : "-rotate-90"
          )}
        />
      </button>
    )
  }
)
TreeItemToggle.displayName = "TreeItemToggle"

// TreeItemContent component
const TreeItemContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn(
      "flex-1 flex items-center py-1 text-sm cursor-default select-none truncate rounded-sm px-2 hover:bg-accent hover:text-accent-foreground data-[selected]:bg-accent data-[selected]:text-accent-foreground data-[disabled]:opacity-50 data-[disabled]:pointer-events-none",
      className
    )}
    {...props}
  />
))
TreeItemContent.displayName = "TreeItemContent"

// TreeGroup component
const TreeGroup = React.forwardRef<
  HTMLUListElement,
  React.HTMLAttributes<HTMLUListElement>
>(({ className, ...props }, ref) => (
  <ul
    ref={ref}
    role="group"
    className={cn(
      "pl-6",
      className
    )}
    {...props}
  />
))
TreeGroup.displayName = "TreeGroup"

export {
  Tree,
  TreeItem,
  TreeItemToggle,
  TreeItemContent,
  TreeGroup,
} 