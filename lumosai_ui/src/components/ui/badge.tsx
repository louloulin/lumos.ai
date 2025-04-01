import React from "react";

interface BadgeProps {
  className?: string;
  children?: React.ReactNode;
  variant?: "default" | "secondary" | "destructive" | "outline";
}

export function Badge({ 
  className, 
  children,
  variant = "default"
}: BadgeProps) {
  const getVariantClass = () => {
    switch (variant) {
      case "secondary": return "badge-secondary";
      case "destructive": return "badge-destructive";
      case "outline": return "badge-outline";
      default: return "badge-default";
    }
  };

  return (
    <span className={`badge ${getVariantClass()} ${className || ""}`}>
      {children}
    </span>
  );
}
