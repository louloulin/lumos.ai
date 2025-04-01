import React from "react";

interface SeparatorProps {
  className?: string;
  orientation?: "horizontal" | "vertical";
}

export function Separator({ className, orientation = "horizontal" }: SeparatorProps) {
  return (
    orientation === "horizontal" ? 
      <hr className={`separator-horizontal ${className || ""}`} /> :
      <div className={`separator-vertical ${className || ""}`} style={{ width: "1px", height: "100%" }} />
  );
} 