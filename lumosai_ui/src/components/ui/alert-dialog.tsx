import React from "react";

interface AlertDialogProps {
  className?: string;
  children?: React.ReactNode;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export function AlertDialog({ 
  className, 
  children,
  open,
  onOpenChange
}: AlertDialogProps) {
  if (!open) return null;
  
  return (
    <div className={`alert-dialog-overlay`}>
      <div className={`alert-dialog ${className || ""}`}>
        {children}
      </div>
    </div>
  );
}

export function AlertDialogTrigger({ 
  className, 
  children
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <button className={`alert-dialog-trigger ${className || ""}`}>
      {children}
    </button>
  );
}

export function AlertDialogContent({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className={`alert-dialog-content ${className || ""}`}>
      {children}
    </div>
  );
}

export function AlertDialogHeader({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className={`alert-dialog-header ${className || ""}`}>
      {children}
    </div>
  );
}

export function AlertDialogFooter({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className={`alert-dialog-footer ${className || ""}`}>
      {children}
    </div>
  );
}

export function AlertDialogTitle({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <h2 className={`alert-dialog-title ${className || ""}`}>
      {children}
    </h2>
  );
}

export function AlertDialogDescription({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <p className={`alert-dialog-description ${className || ""}`}>
      {children}
    </p>
  );
}

export function AlertDialogAction({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <button className={`alert-dialog-action ${className || ""}`}>
      {children}
    </button>
  );
}

export function AlertDialogCancel({ 
  className, 
  children 
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <button className={`alert-dialog-cancel ${className || ""}`}>
      {children}
    </button>
  );
} 