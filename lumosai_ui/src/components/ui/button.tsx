import React from "react";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  className?: string;
  children?: React.ReactNode;
  variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";
  size?: "default" | "sm" | "lg" | "icon";
  asChild?: boolean;
}

export function Button({ 
  className, 
  children,
  variant = "default",
  size = "default",
  asChild,
  ...props
}: ButtonProps) {
  const getVariantClass = () => {
    switch (variant) {
      case "destructive": return "button-destructive";
      case "outline": return "button-outline";
      case "secondary": return "button-secondary";
      case "ghost": return "button-ghost";
      case "link": return "button-link";
      default: return "button-default";
    }
  };

  const getSizeClass = () => {
    switch (size) {
      case "sm": return "button-sm";
      case "lg": return "button-lg";
      case "icon": return "button-icon";
      default: return "button-default-size";
    }
  };

  return (
    <button
      className={`button ${getVariantClass()} ${getSizeClass()} ${className || ""}`}
      {...props}
    >
      {children}
    </button>
  );
}
