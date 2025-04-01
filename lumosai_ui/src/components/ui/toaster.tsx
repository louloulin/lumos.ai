import React from "react";

interface ToasterProps {
  children?: React.ReactNode;
}

export function Toaster({ children }: ToasterProps) {
  return (
    <div className="toaster-container">
      {children}
    </div>
  );
} 