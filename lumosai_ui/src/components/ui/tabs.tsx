'use client';

import React from "react";

interface TabsProps {
  className?: string;
  children?: React.ReactNode;
  defaultValue?: string;
  value?: string;
  onValueChange?: (value: string) => void;
}

export function Tabs({ 
  className, 
  children,
  defaultValue,
  value,
  onValueChange
}: TabsProps) {
  return (
    <div className={`tabs ${className || ""}`}>
      {children}
    </div>
  );
}

export function TabsList({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className={`tabs-list ${className || ""}`}>
      {children}
    </div>
  );
}

export function TabsTrigger({ 
  className, 
  children,
  value
}: {
  className?: string;
  children?: React.ReactNode;
  value: string;
}) {
  return (
    <button className={`tabs-trigger ${className || ""}`} data-value={value}>
      {children}
    </button>
  );
}

export function TabsContent({ 
  className, 
  children,
  value
}: {
  className?: string;
  children?: React.ReactNode;
  value: string;
}) {
  return (
    <div className={`tabs-content ${className || ""}`} data-value={value}>
      {children}
    </div>
  );
}
