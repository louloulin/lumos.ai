import React from "react";

interface DialogProps {
  className?: string;
  children?: React.ReactNode;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export function Dialog({ 
  className, 
  children,
  open,
  onOpenChange
}: DialogProps) {
  if (!open) return null;
  
  return (
    <div className={`dialog-overlay`}>
      <div className={`dialog ${className || ""}`}>
        {children}
      </div>
    </div>
  );
}

export function DialogTrigger({ 
  className, 
  children,
  asChild
}: {
  className?: string;
  children?: React.ReactNode;
  asChild?: boolean;
}) {
  return (
    <button className={`dialog-trigger ${className || ""}`}>
      {children}
    </button>
  );
}

export function DialogContent({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className={`dialog-content ${className || ""}`}>
      {children}
    </div>
  );
}

export function DialogHeader({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className={`dialog-header ${className || ""}`}>
      {children}
    </div>
  );
}

export function DialogFooter({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className={`dialog-footer ${className || ""}`}>
      {children}
    </div>
  );
}

export function DialogTitle({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <h2 className={`dialog-title ${className || ""}`}>
      {children}
    </h2>
  );
}

export function DialogDescription({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <p className={`dialog-description ${className || ""}`}>
      {children}
    </p>
  );
}

export function DialogClose({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <button className={`dialog-close ${className || ""}`}>
      {children}
    </button>
  );
}
