import React from "react";

import { cn } from '../../lib/utils';

interface TextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  className?: string;
}

export function Textarea({ 
  className,
  ...props
}: TextareaProps) {
  return (
    <textarea
      className={`textarea ${className || ""}`}
      {...props}
    />
  );
}
