'use client';

import React from "react";

interface LabelProps extends React.LabelHTMLAttributes<HTMLLabelElement> {
  className?: string;
}

export function Label({ 
  className,
  ...props
}: LabelProps) {
  return (
    <label
      className={`label ${className || ""}`}
      {...props}
    />
  );
}
